use crate::commands::config_cmd;
use crate::memory::processor::{get_relevant_context, MemoryState};
use crate::models::{ChatRequest, Message};
use futures_util::StreamExt;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{ipc::Channel, AppHandle, Emitter, State};
use tokio::sync::RwLock;

#[tauri::command]
pub async fn ask_ai(
    app: AppHandle,
    state: State<'_, crate::GoleState>,
    memory_state: State<'_, Arc<RwLock<MemoryState>>>,
    msg: Vec<Message>,
    on_event: Channel<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
    // 🟢 新增：允许前端显式传入当前绘画的 provider 和 model
    explicit_provider_id: Option<String>,
    explicit_model_id: Option<String>,
    client: State<'_, reqwest::Client>,
) -> Result<(), String> {
    // --- 🚀 核心优化：并行执行预处理任务 ---
    let start_total = std::time::Instant::now(); // ⏱️ 开始计时
    let config = config_cmd::load_config(app.clone()).await?;

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

    // --- 🚀 核心优化：并行执行[搜索]和[记忆]任务 ---
    let messages_for_search = messages.clone();
    let search_instance_url = config.search_instance_url.clone();

    // 提取记忆检索参数
    let last_user_msg = messages.iter().rev().find(|m| m.role == "user");
    let query = last_user_msg.map(|m| m.content.clone()).unwrap_or_default();
    let mode = last_user_msg
        .and_then(|m| m.mode.as_deref())
        .unwrap_or("Standard")
        .to_string();
    let role_id = last_user_msg
        .and_then(|m| m.role_id.as_deref())
        .unwrap_or("default")
        .to_string();

    // 创建并发任务
    let memory_state_inner = memory_state.inner().clone();
    let enable_rag = config.enable_rag; // 🚀 检查全局 RAG 开关

    let app_for_memory = app.clone(); // ✨ 为内存任务克隆 AppHandle
    let memory_task = async move {
        if enable_rag {
            get_relevant_context_parallel(app_for_memory, memory_state_inner, query, mode, role_id)
                .await
        } else {
            Ok(None)
        }
    };

    let search_task = handle_search_parallel(app.clone(), messages_for_search, search_instance_url);

    // 并行等待
    let (search_res, memory_res): (Result<Vec<Message>, String>, Result<Option<String>, String>) =
        tokio::join!(search_task, memory_task);

    let pre_processing_time = start_total.elapsed();
    println!(
        "⏱️ [性能-分析] 前处理阶段(搜索/记忆/配置)耗时: {}ms",
        pre_processing_time.as_millis()
    );

    // 处理搜索结果
    let mut clean_msgs = search_res?;

    // 处理记忆结果并注入
    if let Ok(Some(context)) = memory_res {
        if let Some(sys_msg) = clean_msgs.iter_mut().find(|m| m.role == "system") {
            sys_msg.content = format!("{}\n\n{}", context, sys_msg.content);
        } else {
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
    }

    let temperature =
        temperature.or_else(|| provider_config["temperature"].as_f64().map(|f| f as f32));
    let max_tokens = max_tokens.or_else(|| provider_config["maxTokens"].as_u64().map(|u| u as u32));

    // --- 🧹 极致优化：在发送给 AI 之前抹除所有逻辑标记 ---
    for m in clean_msgs.iter_mut() {
        if m.role == "user" {
            // 剔除 [REASON]
            m.content = m.content.replace("[REASON]", "");
            // 剔除 [SEARCH] (支持带参数的格式 [SEARCH:provider])
            if m.content.contains("[SEARCH") {
                // 使用简单的正则或字符串处理移除 [SEARCH...]
                let mut start = 0;
                while let Some(s_idx) = m.content[start..].find("[SEARCH") {
                    let absolute_start = start + s_idx;
                    if let Some(e_idx) = m.content[absolute_start..].find(']') {
                        m.content
                            .replace_range(absolute_start..=absolute_start + e_idx, "");
                        // 替换后字符串变短，从当前位置继续找
                        start = absolute_start;
                    } else {
                        break;
                    }
                }
            }
            // 最终修剪一下首尾空白
            m.content = m.content.trim().to_string();
        }
    }

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
            start_total,
            pre_processing_time,
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
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let err_body = response.text().await.unwrap_or_default();
        return Err(format!("API Error: {}", err_body));
    }

    if !stream.unwrap_or(true) {
        // --- 🛑 非流式响应处理 ---
        let content =
            crate::ai_utils::call_ai_backend(&client, &api_key, &base_url, &payload).await?;

        on_event
            .send(format!("c:{}", content))
            .map_err(|e| e.to_string())?;

        return Ok(());
    }

    // --- 🌊 流式响应处理 (⚡️ 极致优化：20ms 微合批减少 IPC 频率) ---
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut ttft_logged = false;

    let mut pending_content = String::new();
    let mut pending_reasoning = String::new();
    let mut last_emit = std::time::Instant::now();
    let mut emit_count = 0; // 🚀 前几个字不合批，立即发送以获得最快体感速度

    while let Some(chunk) = stream.next().await {
        if state.stop_flag.load(Ordering::Relaxed) {
            break;
        }

        let chunk = chunk.map_err(|e| e.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline_idx) = buffer.find('\n') {
            let line = buffer.drain(..=newline_idx).collect::<String>();
            let line = line.trim();

            if line.is_empty() || line == "data: [DONE]" {
                if line == "data: [DONE]" {
                    // 彻底结束前清空缓存
                    if !pending_content.is_empty() {
                        let _ = on_event.send(format!("c:{}", pending_content));
                    }
                    if !pending_reasoning.is_empty() {
                        let _ = on_event.send(format!("r:{}", pending_reasoning));
                    }
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
                                let network_ttft = start_total.elapsed().as_millis() as i64
                                    - pre_processing_time.as_millis() as i64;
                                println!(
                                    "⏱️ [性能] 首字总响应: {}ms | 网络等待: {}ms",
                                    start_total.elapsed().as_millis(),
                                    network_ttft
                                );
                                ttft_logged = true;
                            }
                            pending_content.push_str(content);
                        }

                        if let Some(reasoning) = delta["reasoning_content"]
                            .as_str()
                            .or_else(|| delta["reasoning"].as_str())
                            .or_else(|| delta["thought"].as_str())
                        {
                            pending_reasoning.push_str(reasoning);
                        }

                        // ⏱️ 判定：前 5 次下发立即执行 (保证极速 TTFT)，后续切换到 20ms 周期或 100 字符缓冲区
                        if emit_count < 5
                            || last_emit.elapsed().as_millis() >= 20
                            || pending_content.len() > 100
                        {
                            if !pending_content.is_empty() {
                                let _ = on_event.send(format!("c:{}", pending_content));
                                pending_content.clear();
                                emit_count += 1;
                            }
                            if !pending_reasoning.is_empty() {
                                let _ = on_event.send(format!("r:{}", pending_reasoning));
                                pending_reasoning.clear();
                            }
                            last_emit = std::time::Instant::now();
                        }
                    } else if let Some(err) = json["error"].as_object() {
                        return Err(format!("Stream Error: {:?}", err));
                    }
                }
            }
        }
    }

    // 扫尾
    if !pending_content.is_empty() {
        let _ = on_event.send(format!("c:{}", pending_content));
    }
    if !pending_reasoning.is_empty() {
        let _ = on_event.send(format!("r:{}", pending_reasoning));
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
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "systemInstruction")]
    system_instruction: Option<GeminiContent>,
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
    start_total: std::time::Instant,
    pre_processing_time: std::time::Duration,
) -> Result<(), String> {
    if !stream {
        // --- 🛑 非流式处理 ---
        let content = crate::ai_utils::call_gemini_backend(
            client,
            &api_key,
            &base_url,
            &model,
            messages.clone(),
        )
        .await?;

        on_event
            .send(format!("c:{}", content))
            .map_err(|e| e.to_string())?;

        return Ok(());
    }

    // --- 🌊 流式处理 ---
    // 1. 转换消息格式
    let mut system_instruction = None;
    let mut contents = Vec::new();

    for m in messages {
        if m.role == "system" {
            system_instruction = Some(GeminiContent {
                role: None,
                parts: vec![GeminiPart {
                    text: Some(m.content),
                }],
            });
        } else {
            let role = if m.role == "user" { "user" } else { "model" };
            contents.push(GeminiContent {
                role: Some(role.to_string()),
                parts: vec![GeminiPart {
                    text: Some(m.content),
                }],
            });
        }
    }

    let payload = GeminiRequest {
        contents,
        system_instruction,
    };

    // 2. 构造 URL (更加鲁棒的判断)
    let mode = "streamGenerateContent";
    let base = base_url.trim_end_matches('/');

    let url = if base.contains("/models/") {
        // 如果用户提供了完整路径，只补全 key
        format!("{}?key={}", base, api_key)
    } else {
        // 智能补全版本和路径
        let version = if base.contains("/v1") { "" } else { "/v1beta" };
        format!(
            "{}{}/models/{}:{}?key={}",
            base, version, model, mode, api_key
        )
    };

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
        return Err(format!("Gemini API 错误 (状态码 {}): {}", status, err_text));
    }

    // --- 🌊 流式处理 ---
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut ttft_logged = false;
    let mut pending_content = String::new();
    let mut last_emit = std::time::Instant::now();
    let mut emit_count = 0;

    while let Some(chunk) = stream.next().await {
        if state.stop_flag.load(Ordering::Relaxed) {
            break;
        }

        let chunk = chunk.map_err(|e| e.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Gemini 的 stream 数据是一个包含多个 JSON 对象的数组，格式大致为 [ {...}, {...} ]
        // 这里尝试解析完整的 JSON 对象块
        while let Some(start_idx) = buffer.find('{') {
            if state.stop_flag.load(Ordering::Relaxed) {
                break;
            }
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
                                    // 🟢 监测：从用户输入到流式输出首字的性能耗时 (Gemini)
                                    let network_ttft = start_total.elapsed().as_millis() as i64
                                        - pre_processing_time.as_millis() as i64;
                                    println!(
                                        "⏱️ [性能] 首字总响应 (Gemini): {}ms | 网络等待: {}ms",
                                        start_total.elapsed().as_millis(),
                                        network_ttft
                                    );
                                    ttft_logged = true;
                                }
                                pending_content.push_str(text);

                                // ⏱️ Gemini 同样采用：前 5 次极速响应，后续合批策略
                                if emit_count < 5
                                    || last_emit.elapsed().as_millis() >= 20
                                    || pending_content.len() > 100
                                {
                                    if !pending_content.is_empty() {
                                        on_event
                                            .send(format!("c:{}", pending_content))
                                            .map_err(|e| e.to_string())?;
                                        pending_content.clear();
                                        emit_count += 1;
                                    }
                                    last_emit = std::time::Instant::now();
                                }
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

#[tauri::command]
pub async fn discover_models_raw(
    url: String,
    api_key: Option<String>,
    headers_map: Option<std::collections::HashMap<String, String>>,
    client: State<'_, reqwest::Client>,
) -> Result<Value, String> {
    let mut request = client.get(&url);

    if let Some(key) = api_key {
        if !key.is_empty() {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
    }

    if let Some(h) = headers_map {
        for (k, v) in h {
            request = request.header(k, v);
        }
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let err_text = response.text().await.unwrap_or_default();
        return Err(format!("API Error ({}): {}", status, err_text));
    }

    let data = response
        .json::<Value>()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;
    Ok(data)
}

// --- 🚀 助手函数：并行处理搜索逻辑 ---
async fn handle_search_parallel(
    app: AppHandle,
    messages: Vec<Message>,
    search_instance_url: String,
) -> Result<Vec<Message>, String> {
    let mut clean_msgs = messages.clone();

    // 检查最后一条消息是否有 [SEARCH]
    if let Some(m) = clean_msgs.last_mut() {
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

            // 发送搜索开始事件
            let _ = app.emit(
                "search-status",
                json!({ "status": "searching", "query": original_query }),
            );

            match crate::commands::search::perform_search(
                &search_instance_url,
                &original_query,
                &provider,
            )
            .await
            {
                Ok(results) => {
                    // 发送搜索结果事件
                    let _ = app.emit(
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
                    let _ = app.emit("search-status", json!({ "status": "error", "message": e }));
                }
            }
        }
    }

    Ok(clean_msgs)
}

// --- 🚀 助手函数：并行处理记忆检索逻辑 ---
async fn get_relevant_context_parallel(
    app: AppHandle,
    memory_state: Arc<RwLock<MemoryState>>,
    query: String,
    mode: String,
    role_id: String,
) -> Result<Option<String>, String> {
    if query.is_empty() {
        return Ok(None);
    }

    // 发送记忆检索开始事件
    let _ = app.emit(
        "memory-status",
        json!({ "status": "searching", "query": query }),
    );
    let start_time = std::time::Instant::now();

    // 执行记忆检索
    let context = get_relevant_context(memory_state, &query, &mode, &role_id).await?;

    let duration = start_time.elapsed().as_millis();

    if !context.is_empty() {
        // 发送记忆检索完成事件
        let _ = app.emit(
            "memory-status",
            json!({ "status": "done", "duration": duration, "has_context": true }),
        );
        Ok(Some(context))
    } else {
        // 发送记忆检索完成事件 (无结果)
        let _ = app.emit(
            "memory-status",
            json!({ "status": "done", "duration": duration, "has_context": false }),
        );
        Ok(None)
    }
}

// --- 🚀 连接预热：在用户输入时提前建立连接 ---
#[tauri::command]
pub async fn prewarm_connection(
    base_url: String,
    client: State<'_, reqwest::Client>,
) -> Result<(), String> {
    let start = std::time::Instant::now();

    // 构造一个最小的健康检查请求（通常 /v1/models 端点不需要鉴权）
    let url = if base_url.contains("generativelanguage.googleapis.com") {
        // Gemini 使用不同的端点
        format!("{}/v1beta/models", base_url.trim_end_matches('/'))
    } else {
        // OpenAI 兼容端点
        let base = base_url.trim_end_matches('/');
        if base.ends_with("/v1") {
            format!("{}/models", base)
        } else {
            format!("{}/v1/models", base)
        }
    };

    // 发起预热请求（不关心结果，只为建立连接）
    let _ = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    let elapsed = start.elapsed().as_millis();
    println!(
        "🔥 [PREWARM] Connection to {} established in {}ms",
        base_url, elapsed
    );

    Ok(())
}
