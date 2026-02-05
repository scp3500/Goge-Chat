use crate::memory::db::{FactRecord, LanceDbManager};
use crate::memory::embed::EmbeddingEngine;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct MemoryState {
    pub engine: EmbeddingEngine,
    pub db: LanceDbManager,
}

impl MemoryState {
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let engine = EmbeddingEngine::new(app_handle)?;
        let db = LanceDbManager::new(app_handle)?;
        Ok(Self { engine, db })
    }
}

pub async fn upsert_fact(
    state: Arc<RwLock<MemoryState>>,
    content: &str,
    role_id: &str,
    mode: &str,
    is_instruction: bool,
) -> Result<(), String> {
    let start_total = Instant::now();
    let state_read = state.read().await;

    // 1. 向量化
    let start_vec = Instant::now();
    let doc_vector = state_read.engine.get_vector(content)?;
    let duration_vec = start_vec.elapsed();

    // 2. 去重搜索
    let start_search = Instant::now();
    let filter = format!("(mode = '{}' AND role_id = '{}')", mode, role_id);
    let results = state_read
        .db
        .search_similar_facts(doc_vector.clone(), 20, Some(filter))
        .await?;
    let duration_search = start_search.elapsed();

    // 3. 收集需要删除的 ID (批量操作优化)
    let start_cleanup = Instant::now();
    let ids_to_delete: Vec<String> = results
        .into_iter()
        .filter_map(|(old_fact, distance)| {
            let similarity = 1.0 - (distance / 2.0);
            if old_fact.content == content || similarity > 0.85 {
                Some(old_fact.id)
            } else {
                None
            }
        })
        .collect();

    // 批量删除 (只调用一次 optimize_table)
    if !ids_to_delete.is_empty() {
        state_read.db.delete_facts_batch(&ids_to_delete).await?;
    }
    let duration_cleanup = start_cleanup.elapsed();

    // 4. 插入新记录
    let start_insert = Instant::now();
    let fact = FactRecord {
        id: Uuid::new_v4().to_string(),
        content: content.to_string(),
        mode: mode.to_string(),
        role_id: role_id.to_string(),
        metadata: json!({
            "is_instruction": is_instruction,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        })
        .to_string(),
    };
    state_read.db.insert_fact(doc_vector, fact).await?;
    let duration_insert = start_insert.elapsed();

    let total_duration = start_total.elapsed();
    println!(
        "⏱️ [性能] upsert_fact 总耗时: {:?} | 向量化: {:?} | 搜索: {:?} | 清理: {:?} | 插入: {:?}",
        total_duration, duration_vec, duration_search, duration_cleanup, duration_insert
    );

    Ok(())
}

pub async fn get_relevant_context(
    state: Arc<RwLock<MemoryState>>,
    query: &str,
    mode: &str,
    role_id: &str,
) -> Result<String, String> {
    if query.chars().count() < 3 {
        return Ok("".to_string());
    }

    let start_total = Instant::now();
    let state_read = state.read().await;

    let start_vec = Instant::now();
    let query_with_prefix = format!("为查询编写一个表征：{}", query);
    let vector = state_read.engine.get_vector(&query_with_prefix)?;
    let duration_vec = start_vec.elapsed();

    // 🛡️ 维度一：物理隔绝 (Memory Isolation)
    let filter = if mode == "Social" {
        format!(
            "(mode = 'Social' AND role_id = 'global') OR (mode = 'Social' AND role_id = '{}') OR metadata LIKE '%\"is_instruction\":true%'",
            role_id
        )
    } else {
        "mode = 'Standard' AND role_id = 'global'".to_string()
    };

    let start_search = Instant::now();
    let results = state_read
        .db
        .search_similar_facts(vector, 10, Some(filter))
        .await?;
    let duration_search = start_search.elapsed();

    let total_duration = start_total.elapsed();
    if total_duration.as_millis() > 500 {
        println!(
            "⏱️ [性能] get_relevant_context 耗时较长: {:?} | 向量化: {:?} | 搜索: {:?}",
            total_duration, duration_vec, duration_search
        );
    }

    if results.is_empty() {
        return Ok("".to_string());
    }

    /* 移除详细候选列表打印以清理控制台 */

    let mut results = results;
    // 按距离（相似度）排序
    results.sort_by(|(_, dist_a), (_, dist_b)| {
        dist_a
            .partial_cmp(dist_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut context = String::from("\n[已知背景信息]\n");
    let mut injected_count = 0;

    // 🧠 距离阈值 (Threshold):
    // 1.4 (30%) 太宽松会导致“巧克力”匹配到“游戏”。
    // 建议设为 1.3 左右，既能保证一定的联想能力，又能过滤掉明显无关的噪音。
    let distance_threshold = 1.3;

    for (fact, distance) in results.into_iter().take(5) {
        if distance > distance_threshold {
            continue;
        }

        context.push_str(&format!("- {}\n", fact.content));
        injected_count += 1;
    }

    if injected_count == 0 {
        return Ok("".to_string());
    }

    if injected_count > 0 {
        println!("🧠 [记忆] 成功为 AI 注入 {} 条关联上下文", injected_count);
    }
    Ok(context)
}

pub async fn extract_and_store_facts(
    app_handle: &AppHandle,
    state: Arc<RwLock<MemoryState>>,
    session_id: i64,
    role_id: &str,
    mode: &str,
) -> Result<(), String> {
    let app_handle = app_handle.clone();
    let state_clone = state.clone();
    let role_id = role_id.to_string();
    let mode = mode.to_string();

    // 🧠 维度二：静默合成机制 (Automated Extraction)
    // 异步复盘 (Async Task)：利用 Rust 的 tauri::async_runtime::spawn
    tauri::async_runtime::spawn(async move {
        // println!("🧠 [记忆] 启动异步事实提取任务 (Session: {})", session_id);
        // println!("🧠 [记忆] 提取模式: {}, 角色: {}", mode, role_id);

        // 1. 安全校验：验证该 session_id 确实属于该 contact_id (role_id)
        // 🛡️ 维度三：身份交叉校验
        let messages_str = {
            if mode == "Social" {
                let social_db = app_handle.state::<crate::social_db::SocialDbState>();
                let conn = social_db.0.lock().unwrap();

                // 首先确认 session 归属
                let belongs_to: Option<i64> = conn
                    .query_row(
                        "SELECT contact_id FROM social_sessions WHERE id = ?1",
                        rusqlite::params![session_id],
                        |row| row.get(0),
                    )
                    .ok();

                if belongs_to.map(|id| id.to_string()) != Some(role_id.clone()) {
                    println!(
                        "❌ [记忆安全] 拦截：Session {} 不属于角色 {}，拒绝同步",
                        session_id, role_id
                    );
                    return;
                }

                let mut stmt = conn.prepare("SELECT role, content FROM social_messages WHERE session_id = ?1 ORDER BY id DESC LIMIT 10").unwrap();
                let rows = stmt
                    .query_map(rusqlite::params![session_id], |row| {
                        let r: String = row.get(0)?;
                        let c: String = row.get(1)?;
                        let role_tag = if r == "user" {
                            "【用户】"
                        } else {
                            "【AI助手】"
                        };
                        Ok(format!("{}: {}", role_tag, c))
                    })
                    .unwrap();
                let mut msgs: Vec<String> = rows
                    .filter_map(|r: rusqlite::Result<String>| r.ok())
                    .collect();
                msgs.reverse();
                msgs.join("\n")
            } else {
                let db = app_handle.state::<crate::db::DbState>();
                let conn = db.0.lock().unwrap();
                let mut stmt = conn.prepare("SELECT role, content FROM messages WHERE session_id = ?1 ORDER BY id DESC LIMIT 10").unwrap();
                let rows = stmt
                    .query_map(rusqlite::params![session_id], |row| {
                        let r: String = row.get(0)?;
                        let c: String = row.get(1)?;
                        let role_tag = if r == "user" {
                            "【用户】"
                        } else {
                            "【AI助手】"
                        };
                        Ok(format!("{}: {}", role_tag, c))
                    })
                    .unwrap();
                let mut msgs: Vec<String> = rows
                    .filter_map(|r: rusqlite::Result<String>| r.ok())
                    .collect();
                msgs.reverse();
                msgs.join("\n")
            }
        };

        if messages_str.is_empty() {
            println!("🧠 [记忆] 对话记录为空，跳过提取");
            return;
        }

        println!(
            "🧠 [记忆] 正在构造 Prompt 请求 AI 提取事实 ({} 字符)...",
            messages_str.len()
        );

        // 2. 准备 Prompt - 极简版
        let prompt = format!(
            "请分析对话并提取关于【用户】的持久事实（如偏好、身份、经历）。\n\
             要求：\n\
             1. 仅限用户：严禁将AI的猜测、建议或提问当作用户事实。\n\
             2. 严禁幻觉：只记录用户明确陈述的信息。\n\
             3. 简洁：每行一条事实，最多2条，若无则回“无”。\n\
             \n\
             对话：\n\
             {}\n\
             \n\
             事实：",
            messages_str
        );

        // 3. 调用 AI (复用 generate_title 的逻辑，但为内部调用)
        let messages = vec![crate::models::Message {
            id: None,
            model: None,
            role: "user".to_string(),
            content: prompt,
            reasoning_content: None,
            file_metadata: None,
            search_metadata: None,
            provider: None,
            mode: None,
            role_id: None,
        }];

        let start_llm = Instant::now();
        // 调用 generate_title (内部调用，不需要 command 标记)
        let client = app_handle.state::<reqwest::Client>();
        match crate::title_commands::generate_title_internal(app_handle.clone(), messages, &client)
            .await
        {
            Ok(facts_str) => {
                let duration_llm = start_llm.elapsed();
                println!("⏱️ [性能] AI 事实提取耗时: {:?}", duration_llm);

                if facts_str == "无" || facts_str.is_empty() {
                    println!("🧠 [记忆] AI 回复：本次会话未发现新事实");
                    return;
                }

                println!(
                    "🧠 [记忆] AI 返回了潜在事实: \n--- AI START ---\n{}\n--- AI END ---",
                    facts_str
                );

                let facts: Vec<&str> = facts_str.split('\n').collect();
                // 🛡️ 核心限额：每次复盘绝不记录超过 2 条事实
                for content in facts.into_iter().take(2) {
                    let content = content.trim();
                    if content.is_empty() || content == "无" {
                        continue;
                    }

                    println!("🧠 [记忆] 提取到新事实: {}", content);
                    // 存储新事实 (Upsert 逻辑本身包含冲突检测/相似度清理)
                    match upsert_fact(state_clone.clone(), content, &role_id, &mode, false).await {
                        Ok(_) => {}
                        Err(e) => println!("❌ [记忆] upsert_fact 失败: {}", e),
                    }
                }
                // println!("⏱️ [性能] 异步提取任务总用时: {:?}", start_task.elapsed());
            }
            Err(e) => {
                println!("❌ [记忆] 事实提取失败: {}", e);
            }
        }
    });

    Ok(())
}
