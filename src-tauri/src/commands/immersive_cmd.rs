use crate::behavior_engine::{BehaviorEngine, SessionContext};
use crate::behavior_scheduler::MessageScheduler;
use crate::commands::config_cmd;
use crate::social_db::SocialDbState;
use std::sync::Arc;
use tauri::{command, AppHandle, Manager, State, WebviewWindow};

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
    window: WebviewWindow,
) -> Result<(), String> {
    // 1. 更新会话活动时间 (用于 IdleMonitor 追踪)
    scheduler.touch_session(session_id).await;

    // 2. 加载配置
    let config = config_cmd::load_config(app.clone()).await?;
    let settings = config.immersive_mode;

    // 3. 检查沉浸式模式是否启用
    if !settings.enabled {
        // 直接保存消息 (传统模式)
        let db_state: tauri::State<SocialDbState> = app.state();
        {
            let conn = db_state
                .0
                .lock()
                .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;

            conn.execute(
                "INSERT INTO social_messages (contact_id, session_id, role, content, created_at)
                 VALUES (?1, ?2, 'assistant', ?3, datetime('now'))",
                rusqlite::params![contact_id, session_id, content],
            )
            .map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

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

    // 5. 使用行为引擎生成行为链
    let engine = BehaviorEngine::new(settings.clone());
    let chain = engine.decide(&content, &session_context);

    // 6. 异步执行行为链
    scheduler
        .execute_behavior_chain(
            session_id,
            contact_id,
            chain,
            session_context,
            settings,
            app.clone(),
            window,
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
