use crate::immersive_settings::BehaviorAction;
use crate::social_db::SocialDbState;
use rusqlite;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};
use tokio_util::sync::CancellationToken;

/// 会话活动追踪信息
#[derive(Clone)]
struct SessionActivity {
    last_interaction: Instant,
    last_proactive: Option<Instant>,
}

/// 消息调度器 - 管理异步行为链的执行
pub struct MessageScheduler {
    /// 活跃任务的取消令牌 (session_id -> CancellationToken)
    active_tasks: Arc<RwLock<HashMap<i64, CancellationToken>>>,
    /// 会话活动追踪 (session_id -> SessionActivity)
    session_activities: Arc<RwLock<HashMap<i64, SessionActivity>>>,
    /// IdleMonitor 取消令牌
    idle_monitor_token: CancellationToken,
}

impl MessageScheduler {
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            session_activities: Arc::new(RwLock::new(HashMap::new())),
            idle_monitor_token: CancellationToken::new(),
        }
    }

    /// 执行行为链
    pub async fn execute_behavior_chain(
        &self,
        session_id: i64,
        contact_id: i64,
        chain: Vec<BehaviorAction>,
        context: crate::behavior_engine::SessionContext,
        settings: crate::immersive_settings::ImmersiveSettings,
        app: AppHandle,
        window: WebviewWindow,
    ) -> Result<(), String> {
        // 1. 更新会话活动时间
        self.touch_session(session_id).await;

        // 2. 取消该会话的现有任务
        self.cancel_session_behaviors(session_id).await;

        // 3. 创建新的取消令牌
        let token = CancellationToken::new();
        {
            let mut tasks = self.active_tasks.write().await;
            tasks.insert(session_id, token.clone());
        }

        // 4. 异步执行行为链
        let active_tasks = self.active_tasks.clone();
        tauri::async_runtime::spawn(async move {
            let result = Box::pin(Self::execute_chain_internal(
                session_id,
                contact_id,
                chain,
                context,
                settings,
                token.clone(),
                app,
                window,
            ))
            .await;

            // 5. 执行完成后清理
            {
                let mut tasks = active_tasks.write().await;
                tasks.remove(&session_id);
            }

            if let Err(e) = result {
                eprintln!("❌ 行为链执行失败: {}", e);
            }
        });

        Ok(())
    }

    /// 内部执行逻辑 (可被取消)
    async fn execute_chain_internal(
        session_id: i64,
        contact_id: i64,
        chain: Vec<BehaviorAction>,
        context: crate::behavior_engine::SessionContext,
        settings: crate::immersive_settings::ImmersiveSettings,
        token: CancellationToken,
        app: AppHandle,
        window: WebviewWindow,
    ) -> Result<(), String> {
        let mut last_message_id: Option<i64> = None;

        for action in chain {
            // 检查是否被取消
            if token.is_cancelled() {
                println!("🛑 行为链被取消 (session_id: {})", session_id);
                // 清除打字状态
                let _ = window.emit(
                    "typing-status",
                    serde_json::json!({
                        "contactId": contact_id,
                        "isTyping": false
                    }),
                );
                return Ok(());
            }

            match action {
                BehaviorAction::Wait(ms) => {
                    // 显示"正在输入"
                    let _ = window.emit(
                        "typing-status",
                        serde_json::json!({
                            "contactId": contact_id,
                            "isTyping": true
                        }),
                    );

                    // 可中断的等待
                    tokio::select! {
                        _ = sleep(Duration::from_millis(ms as u64)) => {},
                        _ = token.cancelled() => {
                            let _ = window.emit(
                                "typing-status",
                                serde_json::json!({
                                    "contactId": contact_id,
                                    "isTyping": false
                                }),
                            );
                            return Ok(());
                        }
                    }
                }

                BehaviorAction::Speak(content) => {
                    // 清除打字状态
                    let _ = window.emit(
                        "typing-status",
                        serde_json::json!({
                            "contactId": contact_id,
                            "isTyping": false
                        }),
                    );

                    // 保存消息到数据库
                    let db_state = app.state::<SocialDbState>();
                    let conn = db_state
                        .0
                        .lock()
                        .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;

                    let message_id = conn
                        .query_row(
                            "INSERT INTO social_messages (contact_id, session_id, role, content, created_at)
                             VALUES (?1, ?2, 'assistant', ?3, datetime('now'))
                             RETURNING id",
                            rusqlite::params![contact_id, session_id, content],
                            |row| row.get::<_, i64>(0),
                        )
                        .map_err(|e| e.to_string())?;

                    last_message_id = Some(message_id);

                    // 发送消息事件到前端
                    let _ = window.emit(
                        "new-social-message",
                        serde_json::json!({
                            "messageId": message_id,
                            "contactId": contact_id,
                            "sessionId": session_id,
                            "role": "assistant",
                            "content": content,
                            "createdAt": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
                        }),
                    );

                    println!("💬 发送消息: {} (id: {})", content, message_id);
                }

                BehaviorAction::Retract(msg_id) => {
                    // 0 表示撤回最后一条消息
                    let target_id = if msg_id == 0 {
                        last_message_id.ok_or("没有可撤回的消息")?
                    } else {
                        msg_id
                    };

                    // 从数据库删除消息
                    let db_state = app.state::<SocialDbState>();
                    let conn = db_state
                        .0
                        .lock()
                        .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;

                    conn.execute(
                        "DELETE FROM social_messages WHERE id = ?1",
                        rusqlite::params![target_id],
                    )
                    .map_err(|e| e.to_string())?;

                    // 发送撤回事件到前端
                    let _ = window.emit(
                        "message-retracted",
                        serde_json::json!({
                            "messageId": target_id,
                            "contactId": contact_id,
                        }),
                    );

                    println!("🔙 撤回消息 (id: {})", target_id);

                    // 清除 last_message_id
                    if Some(target_id) == last_message_id {
                        last_message_id = None;
                    }
                }

                BehaviorAction::Idle => {
                    // 已读不回 - 什么都不做
                    println!("👀 已读不回 (session_id: {})", session_id);
                }

                BehaviorAction::DelayedDecision(delay_ms, original_message) => {
                    // 延迟后重新决策
                    println!("⏳ 延迟决策: {}ms (session_id: {})", delay_ms, session_id);

                    // 可中断的等待
                    tokio::select! {
                        _ = sleep(Duration::from_millis(delay_ms as u64)) => {
                            println!("⏰ 延迟决策时间到 (session_id: {})", session_id);

                            // 重新获取角色状态并在延迟后重新决策
                            let engine = crate::behavior_engine::BehaviorEngine::new(settings.clone());
                            let sub_chain = engine.decide_after_delay(&original_message, &context);

                            // 递归执行新的行为链 (注意: 这里依然使用当前的 token)
                            Box::pin(Self::execute_chain_internal(
                                session_id,
                                contact_id,
                                sub_chain,
                                context.clone(),
                                settings.clone(),
                                token.clone(),
                                app.clone(),
                                window.clone(),
                            )).await?;
                        }
                        _ = token.cancelled() => {
                            println!("🛑 延迟决策被取消 (session_id: {})", session_id);
                            return Ok(());
                        }
                    }
                }
            }
        }

        // 确保最后清除打字状态
        let _ = window.emit(
            "typing-status",
            serde_json::json!({
                "contactId": contact_id,
                "isTyping": false
            }),
        );

        Ok(())
    }

    /// 取消指定会话的所有行为
    pub async fn cancel_session_behaviors(&self, session_id: i64) {
        let mut tasks = self.active_tasks.write().await;
        if let Some(token) = tasks.remove(&session_id) {
            token.cancel();
            println!("🛑 取消会话 {} 的行为链", session_id);
        }
    }

    /// 取消所有活跃的行为
    pub async fn cancel_all_behaviors(&self) {
        let mut tasks = self.active_tasks.write().await;
        for (session_id, token) in tasks.drain() {
            token.cancel();
            println!("🛑 取消会话 {} 的行为链", session_id);
        }
    }

    // ========== IdleMonitor 功能 ==========

    /// 更新会话的最后交互时间
    pub async fn touch_session(&self, session_id: i64) {
        let mut activities = self.session_activities.write().await;

        // 先读取旧的 last_proactive 值
        let last_proactive = activities.get(&session_id).and_then(|a| a.last_proactive);

        // 然后插入新的活动记录
        activities.insert(
            session_id,
            SessionActivity {
                last_interaction: Instant::now(),
                last_proactive,
            },
        );
    }

    /// 启动 IdleMonitor 后台任务
    pub fn start_idle_monitor(&self, app: AppHandle) {
        let activities = self.session_activities.clone();
        let token = self.idle_monitor_token.clone();

        tauri::async_runtime::spawn(async move {
            loop {
                // 每 60 秒检查一次
                tokio::select! {
                    _ = sleep(Duration::from_secs(60)) => {
                        // 执行空闲检查
                        Self::check_idle_sessions(app.clone(), activities.clone()).await;
                    }
                    _ = token.cancelled() => {
                        println!("🛑 IdleMonitor 已停止");
                        break;
                    }
                }
            }
        });

        println!("🚀 IdleMonitor 已启动");
    }

    /// 检查空闲会话并触发主动消息
    async fn check_idle_sessions(
        app: AppHandle,
        activities: Arc<RwLock<HashMap<i64, SessionActivity>>>,
    ) {
        // 1. 加载全局配置
        let config = match crate::commands::config_cmd::load_config(app.clone()).await {
            Ok(c) => c,
            Err(_) => return,
        };
        let settings = config.immersive_mode;

        // 如果沉浸式模式或主动发言未启用,直接返回
        if !settings.enabled || settings.behaviors.proactive_initiation.is_none() {
            return;
        }

        let now = Instant::now();
        let mut sessions_to_trigger = Vec::new();

        {
            let activities_guard = activities.read().await;
            let db_state = app.state::<SocialDbState>();

            println!(
                "🔍 [IdleMonitor] 检查空闲会话，当前追踪 {} 个会话",
                activities_guard.len()
            );

            for (session_id, activity) in activities_guard.iter() {
                let session_id = *session_id;
                let idle_duration = now.duration_since(activity.last_interaction);

                // 获取联系人 ID (从活动记录中通过 session_id 无法直接获取, 除非我们也存储它)
                // 这里我们假设 session_id 可以用来查询数据库获取上下文

                // 2. 获取该会话的角色状态
                let context = {
                    // 查询数据库获取 contact_id
                    let conn = match db_state.0.lock() {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    let contact_id = conn
                        .query_row(
                            "SELECT contact_id FROM social_sessions WHERE id = ?1",
                            rusqlite::params![session_id],
                            |row| row.get::<_, i64>(0),
                        )
                        .ok();

                    if let Some(contact_id) = contact_id {
                        // 加载状态
                        drop(conn); // 释放锁
                        let state_val = crate::social_db::get_character_state(
                            db_state.clone(),
                            contact_id,
                            session_id,
                        )
                        .ok()
                        .flatten();

                        let mut ctx = crate::behavior_engine::SessionContext {
                            session_id,
                            contact_id,
                            mood: None,
                            busy_level: None,
                            interest_level: None,
                        };

                        if let Some(s) = state_val {
                            ctx.mood = s.get("mood").and_then(|v| v.as_str()).map(String::from);
                            ctx.busy_level = s
                                .get("busyLevel")
                                .and_then(|v| v.as_f64())
                                .map(|f| f as f32);
                            ctx.interest_level = s
                                .get("interestLevel")
                                .and_then(|v| v.as_f64())
                                .map(|f| f as f32);
                        }
                        Some(ctx)
                    } else {
                        None
                    }
                };

                let context = match context {
                    Some(ctx) => ctx,
                    None => continue,
                };

                // 3. 计算动态参数
                let engine = crate::behavior_engine::BehaviorEngine::new(settings.clone());
                let (idle_threshold, success_rate, cooldown) =
                    engine.get_proactive_parameters(&context);

                println!(
                    "📊 [IdleMonitor] 会话 {} 空闲 {}秒 (阈值: {}分钟, 成功率: {:.1}%, 冷却: {}分钟)",
                    session_id,
                    idle_duration.as_secs(),
                    idle_threshold,
                    success_rate * 100.0,
                    cooldown
                );

                // 检查是否超过空闲阈值
                if idle_duration.as_secs() >= (idle_threshold as u64 * 60) {
                    // 检查冷却时间
                    let is_cooled_down = if let Some(last_proactive) = activity.last_proactive {
                        now.duration_since(last_proactive).as_secs() >= (cooldown as u64 * 60)
                    } else {
                        true
                    };

                    if is_cooled_down {
                        // 4. 随机判定
                        let mut rng = rand::thread_rng();
                        use rand::Rng;
                        let roll = rng.gen::<f32>();
                        println!(
                            "🎲 [IdleMonitor] 会话 {} 随机判定: {:.3} < {:.3} = {}",
                            session_id,
                            roll,
                            success_rate,
                            roll < success_rate
                        );
                        if roll < success_rate {
                            sessions_to_trigger.push((session_id, context.contact_id, context));
                        }
                    } else {
                        println!("❄️ [IdleMonitor] 会话 {} 仍在冷却中", session_id);
                    }
                } else {
                    println!("⏱️ [IdleMonitor] 会话 {} 未达到空闲阈值", session_id);
                }
            }
        }

        // 5. 触发主动消息
        for (session_id, _contact_id, context) in sessions_to_trigger {
            let mut activities_guard = activities.write().await;
            if let Some(activity) = activities_guard.get_mut(&session_id) {
                println!("💬 会话 {} 触发动态主动消息 (Fetching AI...)", session_id);
                activity.last_proactive = Some(now);

                // 获取 AI 响应并执行行为链
                let app_clone = app.clone();
                let settings_clone = settings.clone();
                let context_clone = context.clone();

                tauri::async_runtime::spawn(async move {
                    // A. 获取对话历史 (最后 5 条)
                    let history = {
                        let db_state = app_clone.state::<SocialDbState>();
                        let conn = match db_state.0.lock() {
                            Ok(c) => c,
                            Err(_) => return,
                        };
                        let mut stmt = match conn.prepare(
                            "SELECT role, content FROM social_messages 
                             WHERE session_id = ?1 
                             ORDER BY created_at DESC LIMIT 5",
                        ) {
                            Ok(s) => s,
                            Err(_) => return,
                        };

                        let history: Vec<crate::models::Message> = stmt
                            .query_map(rusqlite::params![session_id], |row| {
                                Ok(crate::models::Message {
                                    id: None,
                                    model: None,
                                    role: row.get::<_, String>(0)?,
                                    content: row.get::<_, String>(1)?,
                                    reasoning_content: None,
                                    file_metadata: None,
                                    search_metadata: None,
                                    provider: None,
                                    mode: None,
                                    role_id: None,
                                })
                            })
                            .and_then(|iter| iter.collect())
                            .unwrap_or_default();

                        let mut history = history;
                        history.reverse();
                        history
                    };

                    // B. 加载提示词模板
                    let prompt_template = match crate::character_state::StateAnalyzer::load_prompt_template("proactive_message.txt") {
                        Ok(p) => p,
                        Err(_) => "你正处于单人沉浸式聊天模式。由于对方长时间没说话，请根据当前气氛主动开启一个小话题或关怀。".to_string(),
                    };

                    // C. 构建系统提示词
                    let mood_str = context_clone.mood.as_deref().unwrap_or("neutral");
                    let busy_str = context_clone
                        .busy_level
                        .map(|f| format!("{:.1}", f))
                        .unwrap_or("0.5".to_string());
                    let interest_str = context_clone
                        .interest_level
                        .map(|f| format!("{:.1}", f))
                        .unwrap_or("0.5".to_string());

                    let system_content = format!(
                        "{}\n\n当前角色状态:\n- 心情: {}\n- 忙碌度: {}\n- 兴趣度: {}\n\n请直接输出你想说的内容，保持角色人设，语气自然。",
                        prompt_template, mood_str, busy_str, interest_str
                    );

                    let mut full_messages = vec![crate::models::Message {
                        id: None,
                        model: None,
                        role: "system".to_string(),
                        content: system_content,
                        reasoning_content: None,
                        file_metadata: None,
                        search_metadata: None,
                        provider: None,
                        mode: None,
                        role_id: None,
                    }];
                    full_messages.extend(history);

                    // D. 调用 AI
                    let client = app_clone.state::<reqwest::Client>();

                    // 加载 AI 配置
                    let config =
                        match crate::commands::config_cmd::load_config(app_clone.clone()).await {
                            Ok(c) => c,
                            Err(_) => return,
                        };

                    let provider_id = config.default_provider_id;
                    let model = config.selected_model_id;

                    let providers = config.providers.as_array().unwrap();
                    let provider_config = providers
                        .iter()
                        .find(|p| p["id"].as_str() == Some(&provider_id))
                        .unwrap();
                    let api_key = provider_config["apiKey"].as_str().unwrap_or_default();
                    let base_url = provider_config["baseUrl"].as_str().unwrap_or_default();

                    let ai_response = if provider_id == "gemini" {
                        crate::ai_utils::call_gemini_backend(
                            &client,
                            api_key,
                            base_url,
                            &model,
                            full_messages,
                        )
                        .await
                    } else {
                        let payload = crate::models::ChatRequest {
                            model: model.clone(),
                            messages: full_messages,
                            stream: false,
                            temperature: Some(0.8),
                            max_tokens: Some(512),
                        };
                        crate::ai_utils::call_ai_backend(&client, api_key, base_url, &payload).await
                    };

                    if let Ok(content) = ai_response {
                        // E. 生成行为链并执行
                        let engine =
                            crate::behavior_engine::BehaviorEngine::new(settings_clone.clone());
                        let chain = engine.decide(&content, &context_clone);

                        // 获取主窗口执行
                        if let Some(window) = app_clone.get_webview_window("main") {
                            let scheduler = app_clone.state::<Arc<MessageScheduler>>();
                            let _ = scheduler
                                .execute_behavior_chain(
                                    session_id,
                                    context_clone.contact_id,
                                    chain,
                                    context_clone,
                                    settings_clone,
                                    app_clone.clone(),
                                    window,
                                )
                                .await;
                        }
                    }
                });
            }
        }
    }

    /// 停止 IdleMonitor
    pub fn stop_idle_monitor(&self) {
        self.idle_monitor_token.cancel();
    }
}

impl Drop for MessageScheduler {
    fn drop(&mut self) {
        // 停止 IdleMonitor 后台任务
        self.stop_idle_monitor();
        println!("🛑 MessageScheduler 已销毁");
    }
}

impl Default for MessageScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = MessageScheduler::new();
        assert_eq!(scheduler.active_tasks.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_cancellation() {
        let scheduler = MessageScheduler::new();
        let token = CancellationToken::new();

        {
            let mut tasks = scheduler.active_tasks.write().await;
            tasks.insert(1, token.clone());
        }

        scheduler.cancel_session_behaviors(1).await;
        assert!(token.is_cancelled());
    }
}
