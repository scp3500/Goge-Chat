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
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
        parts: Vec<GeminiPart>,
    }
    #[derive(serde::Serialize)]
    struct GeminiRequest {
        contents: Vec<GeminiContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "systemInstruction")]
        system_instruction: Option<GeminiContent>,
    }

    let mut system_instruction = None;
    let mut contents = Vec::new();

    for m in messages {
        if m.role == "system" {
            system_instruction = Some(GeminiContent {
                role: None,
                parts: vec![GeminiPart { text: m.content }],
            });
        } else {
            let role = if m.role == "user" { "user" } else { "model" };
            contents.push(GeminiContent {
                role: Some(role.to_string()),
                parts: vec![GeminiPart { text: m.content }],
            });
        }
    }

    let payload = GeminiRequest {
        contents,
        system_instruction,
    };
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

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let content = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or("无法解析 Gemini 响应内容")?;

    Ok(content.to_string())
}
