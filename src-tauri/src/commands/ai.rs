use crate::commands::config_cmd;
use crate::models::{ChatRequest, Message};
use futures_util::StreamExt;
use serde_json::json;
use std::sync::atomic::Ordering;
use tauri::{ipc::Channel, AppHandle, Emitter, State, Window};

#[tauri::command]
pub async fn ask_ai(
    app: AppHandle,
    state: State<'_, crate::GoleState>,
    msg: Vec<Message>,
    on_event: Channel<String>,
    window: Window,
) -> Result<(), String> {
    // 1. 加载配置
    let config = config_cmd::load_config(app).await?;

    // 2. 确定当前使用的模型和提供商
    let selected_model = config.selected_model_id.clone();
    let selected_provider_id = config.default_provider_id.clone();

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

    let client = reqwest::Client::new();
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

    let payload = ChatRequest {
        model: model.to_string(),
        messages: clean_msgs,
        stream: true,
    };

    let url = if base_url.ends_with("/chat/completions") {
        base_url.clone()
    } else {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        if state.stop_flag.load(Ordering::Relaxed) {
            println!("⚡ [后端信号] 用户打断了生成");
            break;
        }

        let chunk = chunk.map_err(|e| e.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline_idx) = buffer.find('\n') {
            let line = buffer.drain(..=newline_idx).collect::<String>();
            let line = line.trim();

            if line == "data: [DONE]" {
                return Ok(());
            }

            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    let choice = &json["choices"][0];
                    let delta = &choice["delta"];

                    if let Some(content) = delta["content"].as_str() {
                        on_event
                            .send(format!("c:{}", content))
                            .map_err(|e| e.to_string())?;
                    }

                    if let Some(reasoning) = delta["reasoning_content"].as_str() {
                        on_event
                            .send(format!("r:{}", reasoning))
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    println!("✅ AI 生成任务已彻底释放");
    Ok(())
}
