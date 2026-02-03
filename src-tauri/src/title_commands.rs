use crate::commands::config_cmd;
use crate::models::Message;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

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

#[tauri::command]
pub async fn generate_title(
    app: AppHandle,
    msg: Vec<Message>,
    explicit_provider_id: Option<String>,
    explicit_model_id: Option<String>,
    client: State<'_, reqwest::Client>,
) -> Result<String, String> {
    generate_title_internal_with_params(app, msg, explicit_provider_id, explicit_model_id, &client)
        .await
}

pub async fn generate_title_internal(
    app: AppHandle,
    msg: Vec<Message>,
    client: &reqwest::Client,
) -> Result<String, String> {
    generate_title_internal_with_params(app, msg, None, None, client).await
}

async fn generate_title_internal_with_params(
    app: AppHandle,
    msg: Vec<Message>,
    explicit_provider_id: Option<String>,
    explicit_model_id: Option<String>,
    client: &reqwest::Client,
) -> Result<String, String> {
    let start_total = std::time::Instant::now();
    println!("🦀 Rust 后端: 正在请求 AI 处理任务 (接口复用)...");

    // 1. 【动态读取】加载配置
    let config = config_cmd::load_config(app).await?;

    // 2. 【安全校验】获取当前选中的提供商和模型
    let selected_provider_id = explicit_provider_id.unwrap_or(config.default_provider_id.clone());
    let selected_model_id = explicit_model_id.unwrap_or(config.selected_model_id.clone());

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

    // --- Gemini Native 支持 ---
    if selected_provider_id == "gemini" {
        let res =
            handle_gemini_title_native(api_key, base_url_raw, selected_model_id, msg, client).await;
        let duration = start_total.elapsed();
        println!("⏱️ [性能] AI 任务处理总耗时 (Gemini): {:?}", duration);
        return res;
    }

    // 格式化 URL
    let base_url = if base_url_raw.ends_with("/chat/completions") {
        base_url_raw.clone()
    } else {
        format!("{}/chat/completions", base_url_raw.trim_end_matches('/'))
    };

    let request_body = TitleChatRequest {
        model: selected_model_id,
        messages: msg,
        stream: false,
    };

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

    let api_res: APIResponse = response
        .json()
        .await
        .map_err(|e| format!("JSON 解析失败: {}", e))?;
    let raw_content = api_res
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "新任务".to_string());
    let clean_result = raw_content.trim().to_string();

    let duration = start_total.elapsed();
    println!("⏱️ [性能] AI 任务处理总耗时: {:?}", duration);

    Ok(clean_result)
}

async fn handle_gemini_title_native(
    api_key: String,
    base_url: String,
    model: String,
    messages: Vec<Message>,
    client: &reqwest::Client,
) -> Result<String, String> {
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
