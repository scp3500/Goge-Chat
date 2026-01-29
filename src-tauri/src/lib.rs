// src-tauri/src/lib.rs

mod commands;
mod db;
mod models; // I'll rename the file to avoid conflict or just use it as a module

use crate::db::DbState;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

// ✨ 【新增导入】：用于多线程安全的红绿灯标志位
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::State;

// ✨ 【新增导入】：用于 HTTP 请求
use crate::models::Message;
use reqwest::Client;
use serde::{Deserialize, Serialize}; // 假设 Message 在 models 模块中定义

// ✨ 【新增状态】：定义全局中断标志位
pub struct GoleState {
    pub stop_flag: Arc<AtomicBool>,
}

// --- 辅助结构体：用于 generate_title 的 API 请求与响应 ---
#[derive(Serialize)]
struct TitleChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Deserialize)]
struct APIResponse {
    choices: Vec<APIChoice>,
}

#[derive(Deserialize)]
struct APIChoice {
    message: APIMessage,
}

#[derive(Deserialize)]
struct APIMessage {
    content: String,
}

// ✨ 【新增指令 1】：强制变红灯
#[tauri::command]
async fn stop_ai_generation(state: State<'_, GoleState>) -> Result<(), String> {
    state.stop_flag.store(true, Ordering::Relaxed);
    println!("🛑 后端已收到中断信号，红灯亮起");
    Ok(())
}

// ✨ 【新增指令 2】：重置为绿灯
#[tauri::command]
async fn reset_ai_generation(state: State<'_, GoleState>) -> Result<(), String> {
    state.stop_flag.store(false, Ordering::Relaxed);
    println!("🟢 状态已重置，绿灯亮起");
    Ok(())
}

// ✨ 【核心新增指令 3】：源头生成标题 (Blocking Mode)
// 彻底解决流式传输带来的协议头污染问题
#[tauri::command]
async fn generate_title(app: tauri::AppHandle, msg: Vec<Message>) -> Result<String, String> {
    println!("🦀 Rust 后端: 正在请求 AI 生成标题 (非流式)...");

    // 1. 【动态读取】加载配置
    let config = commands::config_cmd::load_config(app).await?;

    // 2. 【安全校验】获取当前选中的提供商和模型
    let selected_provider_id = config.default_provider_id.clone();
    let selected_model_id = config.selected_model_id.clone();

    // 从 providers 数组中找到当前选中的提供商配置
    let providers = config
        .providers
        .as_array()
        .ok_or("配置错误: 无法读取提供商列表")?;
    let provider_config = providers
        .iter()
        .find(|p| p["id"].as_str() == Some(&selected_provider_id))
        .ok_or(format!("找不到提供商配置: {}", selected_provider_id))?;

    let api_key = provider_config["apiKey"].as_str().unwrap_or("").to_string();
    let base_url_raw = provider_config["baseUrl"]
        .as_str()
        .unwrap_or("https://api.deepseek.com")
        .to_string();

    if api_key.trim().is_empty() {
        return Err(format!(
            "{} 的 API Key 未配置，请前往设置页面填写",
            provider_config["name"].as_str().unwrap_or("该提供商")
        ));
    }

    // --- ⬇️ Google Gemini Native 支持 ⬇️ ---
    if selected_provider_id == "gemini" {
        return handle_gemini_title_native(api_key, base_url_raw, selected_model_id, msg).await;
    }
    // --- ⬆️ Google Gemini Native 支持 ⬆️ ---

    // 格式化 URL
    let base_url = if base_url_raw.ends_with("/chat/completions") {
        base_url_raw.clone()
    } else if selected_provider_id == "gemini" && !base_url_raw.contains("v1beta/openai") {
        // ✨ 【核心修复】：Gemini 的 OpenAI 兼容地址需要包含 v1beta/openai
        format!(
            "{}/v1beta/openai/chat/completions",
            base_url_raw.trim_end_matches('/')
        )
    } else {
        format!("{}/chat/completions", base_url_raw.trim_end_matches('/'))
    };

    println!("🔗 最终标题生成请求地址: {}", base_url);

    let client = Client::new();

    let request_body = TitleChatRequest {
        model: selected_model_id,
        messages: msg,
        stream: false, // 🔥 关键：关闭流式
    };

    // 发送请求
    let response = client
        .post(&base_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取错误详情".to_string());
        return Err(format!("API 状态码 {}: {}", status, error_text));
    }

    // 解析 JSON
    let api_res: APIResponse = response
        .json()
        .await
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    // 提取内容
    let raw_content = api_res
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "新对话".to_string());

    // 🧹 Rust 级基础清洗 (去掉换行和前后空格)
    let clean_title = raw_content.replace("\n", "").trim().to_string();

    println!("✨ 后端生成标题完成: {}", clean_title);
    Ok(clean_title)
}

async fn handle_gemini_title_native(
    api_key: String,
    base_url: String,
    model: String,
    messages: Vec<Message>,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    // 1. 转换消息格式 (非流式：generateContent)
    let contents: Vec<serde_json::Value> = messages
        .into_iter()
        .map(|m| {
            let role = if m.role == "user" { "user" } else { "model" };
            serde_json::json!({
                "role": role,
                "parts": [{ "text": m.content }]
            })
        })
        .collect();

    let url = format!(
        "{}/v1beta/models/{}:generateContent?key={}",
        base_url.trim_end_matches('/'),
        model,
        api_key
    );

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "contents": contents }))
        .send()
        .await
        .map_err(|e| format!("Gemini 网络请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let err_text = response.text().await.unwrap_or_default();
        return Err(format!("Gemini API 错误 (状态码 {}): {}", status, err_text));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let raw_title = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("新对话")
        .to_string();

    let clean_title = raw_title.replace("\n", "").trim().to_string();
    Ok(clean_title)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle();
            let app_dir = app_handle
                .path()
                .app_data_dir()
                .expect("无法获取 C 盘数据目录");

            if !app_dir.exists() {
                std::fs::create_dir_all(&app_dir).expect("无法在 C 盘创建数据目录");
            }

            let db_path = app_dir.join("alice_data.db");
            let conn = Connection::open(&db_path).expect("无法初始化数据库连接");
            db::init_db(&conn).expect("数据库初始化或升级失败");

            // ✨ 【状态管理】：注入数据库连接
            app.manage(DbState(Mutex::new(conn)));

            // ✨ 【核心新增】：注入物理中断状态锁
            app.manage(GoleState {
                stop_flag: Arc::new(AtomicBool::new(false)),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 配置管理
            commands::config_cmd::load_config,
            commands::config_cmd::save_config,
            // AI 交互
            commands::ai::ask_ai,
            stop_ai_generation,
            reset_ai_generation,
            generate_title, // 👈 记得在这里注册！
            // 数据库 CRUD 指令
            commands::db_cmd::get_sessions,
            commands::db_cmd::create_session,
            commands::db_cmd::delete_session,
            commands::db_cmd::clear_messages,
            commands::db_cmd::delete_message,
            commands::db_cmd::update_message,
            commands::db_cmd::delete_messages_after,
            commands::db_cmd::get_messages,
            commands::db_cmd::save_message,
            commands::db_cmd::rename_session,
            commands::db_cmd::update_session_scroll,
            commands::db_cmd::update_sessions_order,
            commands::db_cmd::get_folders,
            commands::db_cmd::create_folder,
            commands::db_cmd::delete_folder,
            commands::db_cmd::rename_folder,
            commands::db_cmd::move_session_to_folder,
            commands::db_cmd::update_folder_collapsed,
            commands::db_cmd::update_folders_order,
            // 文件指令
            commands::file_cmd::open_file,
            commands::file_cmd::read_file_text_content,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 运行异常");
}
