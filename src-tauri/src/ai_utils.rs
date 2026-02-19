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

/// 发送原生 Gemini 请求的通用方法 (非流式)
pub async fn call_gemini_backend(
    client: &reqwest::Client,
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: Vec<Message>,
) -> Result<String, String> {
    let mut full_content = String::new();
    call_gemini_streaming(client, api_key, base_url, model, messages, |chunk| {
        full_content.push_str(&chunk);
    })
    .await?;
    Ok(full_content)
}

/// 发送原生 Gemini 请求的通用方法 (流式)
pub async fn call_gemini_streaming<F>(
    client: &reqwest::Client,
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: Vec<Message>,
    mut on_chunk: F,
) -> Result<(), String>
where
    F: FnMut(String),
{
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
        "{}/v1beta/models/{}:streamGenerateContent?key={}",
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

    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(start_idx) = buffer.find('{') {
            let mut depth = 0;
            let mut end_idx = None;
            let bytes = buffer.as_bytes();

            for i in start_idx..bytes.len() {
                if bytes[i] == b'{' {
                    depth += 1;
                } else if bytes[i] == b'}' {
                    if depth > 0 {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = Some(i);
                            break;
                        }
                    }
                }
            }

            if let Some(end) = end_idx {
                let json_str = &buffer[start_idx..=end];
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(candidates) = json["candidates"].as_array() {
                        if let Some(candidate) = candidates.get(0) {
                            if let Some(parts) = candidate["content"]["parts"].as_array() {
                                for part in parts {
                                    if let Some(text) = part["text"].as_str() {
                                        on_chunk(text.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                // 🛠️ 完善：确保 drain 掉 end 之前的所有内容 (包括可能存在的 [ 或 , 等 JSON 数组分隔符)
                buffer.drain(..=end);
            } else {
                break;
            }
        }
    }

    Ok(())
}
