use crate::behavior_engine::{BehaviorEngine, SessionContext};
use crate::behavior_scheduler::MessageScheduler;
use crate::commands::config_cmd;
use crate::models::{ChatRequest, Message};
use crate::social_db::SocialDbState;
use futures_util::StreamExt;
use std::sync::Arc;
use tauri::{command, AppHandle, Manager, State};

/// 发送沉浸式社交消息
///
/// 如果沉浸式模式启用,将使用行为引擎生成行为链并异步执行
/// 否则直接保存消息
#[command]
pub async fn send_social_message_immersive(
    app: AppHandle,
    scheduler: State<'_, Arc<MessageScheduler>>,
    session_id: i64,
    contact_id: i64,
    content: String,
) -> Result<(), String> {
    // 1. 更新会话活动时间 (用于 IdleMonitor 追踪)
    scheduler.touch_session(session_id).await;

    // 2. 加载配置
    let config = config_cmd::load_config(app.clone()).await?;
    let settings = config.immersive_mode;

    // 3. 检查行为模拟是否启用 (注意: 这里只决定是否启用延迟/拆分等行为)
    // 即使关闭了行为模拟,只要在社交模式下,我们仍然要在这里处理 AI 调用
    let is_behavior_enabled = settings.enabled;
    println!("🎭 [Social] 行为模拟开启状态: {}", is_behavior_enabled);

    // --- 🚀 社交模式 (沉浸式) 处理逻辑 ---
    // 注意: 用户消息已经由前端保存,这里不再重复保存

    // 3. 🧠 状态分析集成
    let db_state: tauri::State<SocialDbState> = app.state();
    let mut session_context = SessionContext {
        session_id,
        contact_id,
        mood: None,
        busy_level: None,
        interest_level: None,
    };

    // 增加消息计数
    let message_count =
        crate::social_db::increment_message_count(db_state.clone(), contact_id, session_id)?;

    println!("📊 消息计数: {}", message_count);

    // 检查是否启用状态追踪
    if let Some(ref state_config) = settings.behaviors.character_state_config {
        if state_config.enabled {
            let analyzer = crate::character_state::StateAnalyzer::new();

            // 检查是否应该触发分析
            let should_analyze = analyzer
                .should_analyze(contact_id, session_id, state_config, &db_state)
                .await?;

            if should_analyze {
                println!("🔍 触发状态分析...");

                // 获取最近的消息历史 (在单独的作用域中,确保锁被释放)
                let messages = {
                    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
                    let mut stmt = conn
                        .prepare(
                            "SELECT role, content FROM social_messages 
                             WHERE contact_id = ?1 AND session_id = ?2 
                             ORDER BY created_at DESC LIMIT 20",
                        )
                        .map_err(|e| e.to_string())?;

                    let messages: Vec<(String, String)> = stmt
                        .query_map(rusqlite::params![contact_id, session_id], |row| {
                            Ok((row.get(0)?, row.get(1)?))
                        })
                        .map_err(|e| e.to_string())?
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| e.to_string())?;

                    messages
                };

                // 执行状态分析
                let analysis = analyzer
                    .analyze_state(contact_id, session_id, messages)
                    .await?;

                // 保存分析结果
                crate::social_db::upsert_character_state(
                    db_state.clone(),
                    contact_id,
                    session_id,
                    analysis.mood.clone(),
                    analysis.busy_level,
                    analysis.interest_level,
                )?;

                // 重置消息计数
                crate::social_db::reset_message_count(db_state.clone(), contact_id, session_id)?;

                // 更新上下文
                session_context.mood = Some(analysis.mood);
                session_context.busy_level = Some(analysis.busy_level);
                session_context.interest_level = Some(analysis.interest_level);

                println!("✅ 状态分析完成并保存");
            } else {
                // 从数据库加载现有状态
                if let Some(state) =
                    crate::social_db::get_character_state(db_state.clone(), contact_id, session_id)?
                {
                    session_context.mood =
                        state.get("mood").and_then(|v| v.as_str()).map(String::from);
                    session_context.busy_level = state
                        .get("busyLevel")
                        .and_then(|v| v.as_f64())
                        .map(|f| f as f32);
                    session_context.interest_level = state
                        .get("interestLevel")
                        .and_then(|v| v.as_f64())
                        .map(|f| f as f32);
                    println!(
                        "📦 从缓存加载状态: mood={:?}, busy={:?}, interest={:?}",
                        session_context.mood,
                        session_context.busy_level,
                        session_context.interest_level
                    );
                }
            }
        }
    }

    // 5. 🤖 调用 AI 获取回答 (内部流式收集)
    println!("🤖 [Social] 正在请求 AI 响应...");

    // A. 获取对话历史 (20条)
    let history = {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT role, content FROM social_messages 
                 WHERE session_id = ?1 
                 ORDER BY created_at DESC LIMIT 21", // 包含刚刚保存的那条
            )
            .map_err(|e| e.to_string())?;

        let messages: Vec<Message> = stmt
            .query_map(rusqlite::params![session_id], |row| {
                Ok(Message {
                    id: None,
                    model: None,
                    role: row.get(0)?,
                    content: row.get(1)?,
                    reasoning_content: None,
                    file_metadata: None,
                    search_metadata: None,
                    provider: None,
                    mode: None,
                    role_id: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut history = messages;
        history.reverse();
        history
    };

    // B. 获取配置
    let provider_id = config.default_provider_id.clone();
    let model = config.selected_model_id.clone();
    let providers = config.providers.as_array().ok_or("无法读取提供商列表")?;
    let provider_config = providers
        .iter()
        .find(|p| p["id"].as_str() == Some(&provider_id))
        .ok_or(format!("找不到提供商配置: {}", provider_id))?;

    let api_key = provider_config["apiKey"].as_str().unwrap_or_default();
    let base_url = provider_config["baseUrl"].as_str().unwrap_or_default();

    // C. 执行 AI 调用 (内部流式处理)
    let client = app.state::<reqwest::Client>();
    let ai_response = if provider_id == "gemini" {
        crate::ai_utils::call_gemini_backend(&client, api_key, base_url, &model, history).await?
    } else {
        // OpenAI 兼容流式处理 (虽然后端有 call_ai_backend, 但为了满足用户"内部流式收集"的需求, 这里手动处理)
        let payload = ChatRequest {
            model: model.clone(),
            messages: history,
            stream: true,
            temperature: Some(0.8),
            max_tokens: Some(1024),
        };

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
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("AI 网络请求失败: {}", e))?;

        if !response.status().is_success() {
            let err_body = response.text().await.unwrap_or_default();
            return Err(format!("AI API 错误: {}", err_body));
        }

        let mut full_content = String::new();
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| e.to_string())?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(newline_idx) = buffer.find('\n') {
                let line = buffer.drain(..=newline_idx).collect::<String>();
                let line = line.trim();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                            full_content.push_str(content);
                            // 这里可以发送中间推理过程(如果有的话), 但目前我们只收集内容
                        }
                    }
                }
            }
        }
        full_content
    };

    println!(
        "✅ AI 响应收集完成: {}...",
        ai_response.chars().take(20).collect::<String>()
    );

    // 6. 使用行为引擎生成行为链 (针对 AI 的回答)
    let engine = BehaviorEngine::new(settings.clone());
    let chain = engine.decide(&ai_response, &session_context);

    // 7. 异步执行行为链
    scheduler
        .execute_behavior_chain(
            session_id,
            contact_id,
            chain,
            session_context,
            settings,
            app.clone(),
        )
        .await?;

    Ok(())
}

/// 取消指定会话的所有待执行行为
#[command]
pub async fn cancel_immersive_behaviors(
    scheduler: State<'_, Arc<MessageScheduler>>,
    session_id: i64,
) -> Result<(), String> {
    scheduler.cancel_session_behaviors(session_id).await;
    Ok(())
}

/// 取消所有活跃的沉浸式行为
#[command]
pub async fn cancel_all_immersive_behaviors(
    scheduler: State<'_, Arc<MessageScheduler>>,
) -> Result<(), String> {
    scheduler.cancel_all_behaviors().await;
    Ok(())
}
