use crate::commands::config_cmd;
use crate::memory::processor::{get_relevant_context, MemoryState};
use crate::models::{ChatRequest, Message};
use futures_util::StreamExt;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{ipc::Channel, AppHandle, Emitter, State, Window};
use tokio::sync::RwLock;

#[tauri::command]
pub async fn ask_ai(
    app: AppHandle,
    state: State<'_, crate::GoleState>,
    memory_state: State<'_, Arc<RwLock<MemoryState>>>,
    msg: Vec<Message>,
    on_event: Channel<String>,
    window: Window,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
    // 🟢 新增：允许前端显式传入当前绘画的 provider 和 model
    explicit_provider_id: Option<String>,
    explicit_model_id: Option<String>,
    client: State<'_, reqwest::Client>,
) -> Result<(), String> {
    // 1. 加载配置
    let config = config_cmd::load_config(app).await?;

    // 2. 确定当前使用的模型和提供商
    // 优先使用显式传入的参数，如果没有（旧版前端），则回退到全局配置
    let selected_model = explicit_model_id.unwrap_or(config.selected_model_id.clone());
    let selected_provider_id = explicit_provider_id.unwrap_or(config.default_provider_id.clone());

    // 从 providers 数组中找到当前选中的提供商配置
    let providers = config
        .providers
        .as_array()
        .ok_or("配置错误：无法读取提供商列表")?;
    let provider_config = providers
        .iter()
        .find(|p| p["id"].as_str() == Some(&selected_provider_id))
        .ok_or(format!("找不到提供商配置: {}", selected_provider_id))?;

    let api_key = provider_config["apiKey"].as_str().unwrap_or("").to_string();
    let base_url = provider_config["baseUrl"]
        .as_str()
        .unwrap_or("https://api.deepseek.com")
        .to_string();

    if api_key.trim().is_empty() {
        return Err(format!(
            "{} 的 API Key 未配置，请前往设置页面填写",
            provider_config["name"].as_str().unwrap_or("该提供商")
        ));
    }

    let messages = msg;

    // 检查是否需要强制使用推理 (如果用户手动输入了 [REASON] 标记)
    let has_reason_tag = messages
        .iter()
        .any(|m| m.role == "user" && m.content.contains("[REASON]"));

    let model = if has_reason_tag {
        // 如果有标记且是 DeepSeek，切换到 reasoner
        if selected_provider_id == "deepseek" {
            "deepseek-reasoner".to_string()
        } else {
            selected_model
        }
    } else {
        selected_model
    };

    // 预处理消息
    let mut clean_msgs = messages.clone();

    if let Some(m) = clean_msgs.last_mut() {
        if m.role == "user" && m.content.contains("[REASON]") {
            m.content = m.content.replace("[REASON]", "");
        }
        if m.role == "user" && m.content.contains("[SEARCH]") {
            let (original_query, provider) = if m.content.contains("[SEARCH:") {
                let start = m.content.find("[SEARCH:").unwrap();
                let end = m.content[start..].find(']').unwrap() + start;
                let provider = &m.content[start + 8..end];
                let clean = m.content.replace(&m.content[start..=end], "");
                (clean, provider.to_string())
            } else {
                (m.content.replace("[SEARCH]", ""), "all".to_string())
            };

            println!("🔍 正在执行网络搜索 (源: {}): {}", provider, original_query);

            // 发送搜索开始事件
            let _ = window.emit(
                "search-status",
                json!({ "status": "searching", "query": original_query }),
            );

            match crate::commands::search::perform_search(
                &config.search_instance_url,
                &original_query,
                &provider,
            )
            .await
            {
                Ok(results) => {
                    println!("✅ 搜索成功，获取到 {} 条结果", results.len());

                    // 发送搜索结果事件
                    let _ = window.emit(
                        "search-status",
                        json!({ "status": "done", "results": results }),
                    );

                    let mut context = String::from("【联网搜索参考资料】\n");
                    for (i, res) in results.iter().enumerate() {
                        context.push_str(&format!(
                            "{}. {}\n   链接: {}\n   内容: {}\n\n",
                            i + 1,
                            res.title,
                            res.url,
                            res.snippet
                        ));
                    }

                    m.content = format!(
                        "用户原始问题: {}\n\n{}\n请分析以上搜索结果，结合你的知识，为用户提供准确且最新的回答。",
                        original_query, context
                    );
                }
                Err(e) => {
                    println!("❌ 搜索失败: {}", e);
                    let _ =
                        window.emit("search-status", json!({ "status": "error", "message": e }));
                }
            }
        }
    }

    // --- 🧠 Alice Memory Integration ---
    if let Some(last_user_msg) = messages.iter().rev().find(|m| m.role == "user") {
        let mode = last_user_msg.mode.as_deref().unwrap_or("Standard");
        let role_id = last_user_msg.role_id.as_deref().unwrap_or("default");

        let query = last_user_msg.content.clone();

        // 发送记忆检索开始事件
        let _ = window.emit(
            "memory-status",
            json!({ "status": "searching", "query": query }),
        );
        let start_time = std::time::Instant::now();

        let context =
            get_relevant_context(memory_state.inner().clone(), &query, mode, role_id).await?;

        let duration = start_time.elapsed().as_millis();

        if !context.is_empty() {
            println!(
                "🧠 [记忆注入] 模式: {}, 角色: {}, 耗时: {}ms (注入 {} 字符)",
                mode,
                role_id,
                duration,
                context.len()
            );

            // 发送记忆检索完成事件
            let _ = window.emit(
                "memory-status",
                json!({ "status": "done", "duration": duration, "has_context": true }),
            );
            // 找到系统提示词并注入
            if let Some(sys_msg) = clean_msgs.iter_mut().find(|m| m.role == "system") {
                sys_msg.content = format!("{}\n\n{}", context, sys_msg.content);
            } else {
                // 如果没有系统提示词，在最前面插入一个
                clean_msgs.insert(
                    0,
                    Message {
                        id: None,
                        model: None,
                        role: "system".to_string(),
                        content: context,
                        reasoning_content: None,
                        file_metadata: None,
                        search_metadata: None,
                        provider: None,
                        mode: None,
                        role_id: None,
                    },
                );
            }
        } else {
            // 发送记忆检索完成事件 (无结果)
            let _ = window.emit(
                "memory-status",
                json!({ "status": "done", "duration": duration, "has_context": false }),
            );
        }
    }

    let temperature =
        temperature.or_else(|| provider_config["temperature"].as_f64().map(|f| f as f32));
    let max_tokens = max_tokens.or_else(|| provider_config["maxTokens"].as_u64().map(|u| u as u32));

    // --- ⬇️ Google Gemini Native 支持 ⬇️ ---
    if selected_provider_id == "gemini" {
        return handle_gemini_native(
            api_key,
            base_url,
            model,
            clean_msgs,
            state,
            on_event,
            stream.unwrap_or(true),
            &client,
        )
        .await;
    }
    // --- ⬆️ Google Gemini Native 支持 ⬆️ ---

    let payload = ChatRequest {
        model: model.to_string(),
        messages: clean_msgs,
        stream: stream.unwrap_or(true),
        temperature,
        max_tokens,
    };

    let disable_url_suffix = provider_config["disableUrlSuffix"]
        .as_bool()
        .unwrap_or(false);

    let url = if disable_url_suffix {
        base_url.clone()
    } else {
        let base = base_url.trim_end_matches('/');
        if base.ends_with("/chat/completions") {
            base.to_string()
        } else if base.ends_with("/v1") {
            format!("{}/chat/completions", base)
        } else {
            // 🛡️ 修复：如果不包含 v1，自动补全 /v1/chat/completions，与前端测试保持一致
            // 这解决了类似 https://api.ohmygpt.com 这种 BaseURL 导致的测试通过但对话失败的问题
            format!("{}/v1/chat/completions", base)
        }
    };

    // println!("🔗 最终对话请求地址: {}", url);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            println!("❌ 请求失败: {}", e);
            e.to_string()
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let err_body = response.text().await.unwrap_or_default();
        println!("❌ API 返回错误 ({}): {}", status, err_body);
        return Err(format!("API Error: {}", err_body));
    }

    if !stream.unwrap_or(true) {
        // --- 🛑 非流式响应处理 ---
        let json: Value = response.json().await.map_err(|e| e.to_string())?;
        // println!("📩 收到非流式响应: {:?}", json); // 移除冗余
        let choice = &json["choices"][0];
        let message = &choice["message"];

        if let Some(content) = message["content"].as_str() {
            on_event
                .send(format!("c:{}", content))
                .map_err(|e| e.to_string())?;
        }

        if let Some(reasoning) = message["reasoning_content"].as_str() {
            on_event
                .send(format!("r:{}", reasoning))
                .map_err(|e| e.to_string())?;
        }

        return Ok(());
    }

    // --- 🌊 流式响应处理 ---
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut ttft_logged = false;
    let start_gen = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        if state.stop_flag.load(Ordering::Relaxed) {
            println!("⚡ [后端信号] 用户打断了生成");
            break;
        }

        let chunk = chunk.map_err(|e| e.to_string())?;
        let chunk_str = String::from_utf8_lossy(&chunk);
        buffer.push_str(&chunk_str);

        while let Some(newline_idx) = buffer.find('\n') {
            let line = buffer.drain(..=newline_idx).collect::<String>();
            let line = line.trim();

            if line.is_empty() || line == "data: [DONE]" {
                if line == "data: [DONE]" {
                    return Ok(());
                }
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(choices) = json["choices"].as_array() {
                        if choices.is_empty() {
                            continue;
                        }
                        let choice = &choices[0];
                        let delta = &choice["delta"];

                        if let Some(content) = delta["content"].as_str() {
                            if !ttft_logged {
                                println!("⏱️ [性能] AI 响应 TTFT: {:?}", start_gen.elapsed());
                                ttft_logged = true;
                            }
                            on_event
                                .send(format!("c:{}", content))
                                .map_err(|e| e.to_string())?;
                        }

                        if let Some(reasoning) = delta["reasoning_content"].as_str() {
                            on_event
                                .send(format!("r:{}", reasoning))
                                .map_err(|e| e.to_string())?;
                        }
                    } else if let Some(err) = json["error"].as_object() {
                        println!("❌ 流中发现错误: {:?}", err);
                        return Err(format!("Stream Error: {:?}", err));
                    }
                }
            }
        }
    }

    // println!("✅ AI 生成任务已彻底释放");
    Ok(())
}

// --- ⬇️ Gemini Native 相关结构和实现 ⬇️ ---

#[derive(Serialize)]
struct GeminiPart {
    text: Option<String>,
}

#[derive(Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

async fn handle_gemini_native(
    api_key: String,
    base_url: String,
    model: String,
    messages: Vec<Message>,
    state: State<'_, crate::GoleState>,
    on_event: Channel<String>,
    stream: bool,
    client: &reqwest::Client,
) -> Result<(), String> {
    // 1. 转换消息格式
    let contents: Vec<GeminiContent> = messages
        .into_iter()
        .map(|m| {
            let role = if m.role == "user" { "user" } else { "model" };
            GeminiContent {
                role: role.to_string(),
                parts: vec![GeminiPart {
                    text: Some(m.content),
                }],
            }
        })
        .collect();

    let payload = GeminiRequest { contents };

    // 2. 构造 URL
    let mode = if stream {
        "streamGenerateContent"
    } else {
        "generateContent"
    };

    let url = format!(
        "{}/v1beta/models/{}:{}?key={}",
        base_url.trim_end_matches('/'),
        model,
        mode,
        api_key
    );

    println!("🚀 [Native Gemini] 请求地址 (stream: {}): {}", stream, url);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Gemini 网络请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let err_text = response.text().await.unwrap_or_default();
        // The original instruction had a syntactically incorrect line here.
        // Assuming the intent was to return the error.
        return Err(format!("Gemini API 错误 (状态码 {}): {}", status, err_text));
    }

    if !stream {
        // --- 🛑 非流式处理 ---
        let json: Value = response.json().await.map_err(|e| e.to_string())?;
        if let Some(candidates) = json["candidates"].as_array() {
            if let Some(candidate) = candidates.first() {
                if let Some(parts) = candidate["content"]["parts"].as_array() {
                    for part in parts {
                        if let Some(text) = part["text"].as_str() {
                            on_event
                                .send(format!("c:{}", text))
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
            }
        }
        return Ok(());
    }

    // --- 🌊 流式处理 ---
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut ttft_logged = false;
    let start_gen = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        if state.stop_flag.load(Ordering::Relaxed) {
            break;
        }

        let chunk = chunk.map_err(|e| e.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Gemini 的 stream 数据是一个包含多个 JSON 对象的数组，格式大致为 [ {...}, {...} ]
        // 这里尝试解析完整的 JSON 对象块
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
                if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                    // 解析 candidates[0].content.parts[0].text
                    if let Some(parts) = json["candidates"][0]["content"]["parts"].as_array() {
                        for part in parts {
                            if let Some(text) = part["text"].as_str() {
                                if !ttft_logged {
                                    println!(
                                        "⏱️ [性能] AI (Gemini) 响应 TTFT: {:?}",
                                        start_gen.elapsed()
                                    );
                                    ttft_logged = true;
                                }
                                on_event
                                    .send(format!("c:{}", text))
                                    .map_err(|e| e.to_string())?;
                            }
                        }
                    }
                }
                buffer.drain(..=end);
            } else {
                break; // 等待更多数据
            }
        }
    }

    Ok(())
}
