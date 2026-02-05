use crate::models::{ChatRequest, Message};
use serde_json::Value;

/// 执行非流式 AI 请求的通用方法
pub async fn call_ai_backend(
    client: &reqwest::Client,
    api_key: &str,
    base_url: &str,
    payload: &ChatRequest,
) -> Result<String, String> {
    let base = base_url.trim_end_matches('/');
    let url = if base.ends_with("/chat/completions") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{}/chat/completions", base)
    } else {
        format!("{}/v1/chat/completions", base)
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    if !status.is_success() {
        let err_body = response.text().await.unwrap_or_default();
        return Err(format!("API Error ({}): {}", status, err_body));
    }

    let json: Value = response.json().await.map_err(|e| e.to_string())?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("无法解析 AI 响应内容")?;

    Ok(content.to_string())
}

/// 发送原生 Gemini 请求的通用方法
pub async fn call_gemini_backend(
    client: &reqwest::Client,
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: Vec<Message>,
) -> Result<String, String> {
    #[derive(serde::Serialize)]
    struct GeminiPart {
        text: String,
    }
    #[derive(serde::Serialize)]
    struct GeminiContent {
        role: String,
        parts: Vec<GeminiPart>,
    }
    #[derive(serde::Serialize)]
    struct GeminiRequest {
        contents: Vec<GeminiContent>,
    }

    let contents: Vec<GeminiContent> = messages
        .into_iter()
        .map(|m| {
            let role = if m.role == "user" { "user" } else { "model" };
            GeminiContent {
                role: role.to_string(),
                parts: vec![GeminiPart { text: m.content }],
            }
        })
        .collect();

    let payload = GeminiRequest { contents };
    let url = format!(
        "{}/v1beta/models/{}:generateContent?key={}",
        base_url.trim_end_matches('/'),
        model,
        api_key
    );

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    if !status.is_success() {
        let err_text = response.text().await.unwrap_or_default();
        return Err(format!("Gemini API Error ({}): {}", status, err_text));
    }

    let json: Value = response.json().await.map_err(|e| e.to_string())?;
    let content = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or("无法解析 Gemini 响应内容")?;

    Ok(content.to_string())
}
