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

    // 2. 【安全校验】
    if config.api_key.trim().is_empty() {
        return Err("API Key 未配置，请前往设置页面填写".to_string());
    }

    let api_key = config.api_key;
    let base_url = "https://api.deepseek.com/chat/completions";
    let model = "deepseek-chat";

    let client = Client::new();

    let request_body = TitleChatRequest {
        model: model.to_string(),
        messages: msg,
        stream: false, // 🔥 关键：关闭流式
    };

    // 发送请求
    let response = client
        .post(base_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("API 报错: {}", error_text));
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
