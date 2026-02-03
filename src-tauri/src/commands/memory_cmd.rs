use crate::memory::db::FactRecord;
use crate::memory::processor::{upsert_fact, MemoryState};
use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::RwLock;

#[command]
pub async fn get_memories(
    state: State<'_, Arc<RwLock<MemoryState>>>,
    query: Option<String>,
    mode: Option<String>,
    role_id: Option<String>,
) -> Result<Vec<FactRecord>, String> {
    let state_read = state.read().await;
    let mode = mode.unwrap_or_else(|| "Standard".to_string());
    let role_id = role_id.unwrap_or_else(|| "default".to_string());
    if let Some(q) = query {
        let vector = state_read.engine.get_vector(&q)?;
        let filter = format!("mode = '{}' AND role_id = '{}'", mode, role_id);
        let results = state_read
            .db
            .search_similar_facts(vector, 20, Some(filter))
            .await?;
        Ok(results.into_iter().map(|(f, _)| f).collect())
    } else {
        // 返回所有记忆 (或者按条件过滤，但先返回全部以便管理)
        let all = state_read.db.get_all_memories().await?;
        Ok(all)
    }
}

#[command]
pub async fn delete_memory(
    state: State<'_, Arc<RwLock<MemoryState>>>,
    id: String,
    content: Option<String>,
) -> Result<(), String> {
    let state_read = state.read().await;

    // 1. 物理删除指定 ID
    state_read.db.delete_fact(&id).await?;

    // 2. 深度清理：如果提供了内容，则把所有完全一致的副本全部删除 (防止幽灵记录)
    if let Some(c) = content {
        state_read.db.delete_fact_by_content(&c).await?;
    }

    Ok(())
}

#[command]
pub async fn update_memory(
    state: State<'_, Arc<RwLock<MemoryState>>>,
    id: String,
    content: String,
    mode: String,
    role_id: String,
) -> Result<(), String> {
    // 1. 先删除旧的
    {
        let state_read = state.read().await;
        state_read.db.delete_fact(&id).await?;
    }
    // 2. 插入新的 (内容可能相似，upsert 会处理)
    upsert_fact(state.inner().clone(), &content, &role_id, &mode, false).await?;
    Ok(())
}

#[command]
pub async fn insert_memory(
    state: State<'_, Arc<RwLock<MemoryState>>>,
    content: String,
    role_id: String,
    mode: String,
    is_instruction: Option<bool>,
) -> Result<(), String> {
    upsert_fact(
        state.inner().clone(),
        &content,
        &role_id,
        &mode,
        is_instruction.unwrap_or(false),
    )
    .await?;
    Ok(())
}

#[command]
pub async fn clear_memories(state: State<'_, Arc<RwLock<MemoryState>>>) -> Result<(), String> {
    let state_read = state.read().await;
    state_read.db.clear_memories().await?;
    Ok(())
}

#[command]
pub async fn seed_memories(state: State<'_, Arc<RwLock<MemoryState>>>) -> Result<String, String> {
    let facts = [
        // 1. 标准模式 (Standard) - 全局记忆
        // 预期：在 Standard 模式下能被检索到
        (
            "用户最喜欢的颜色是'量子蓝' (Quantum Blue)",
            "Standard",
            "global",
        ),
        (
            "用户不仅是程序员，还是一名业余的量子物理学家",
            "Standard",
            "global",
        ),
        // 2. 社交模式 (Social) - 全局记忆
        // 预期：在 Social 模式下，无论哪个角色都能检索到，但 Standard 模式无法看到
        (
            "在虚拟世界中，用户的隐藏身份是代号 '007' 的特工",
            "Social",
            "global",
        ),
        // 3. 社交模式 (Social) - 角色专属记忆 (鸡煲)
        // 预期：只有在和 '鸡煲' (jibao) 聊天时能看到，其他角色看不到
        (
            "鸡煲曾经不小心把用户的咖啡打翻在键盘上，这是我们的小秘密",
            "Social",
            "鸡煲",
        ),
        // 4. 社交模式 (Social) - 角色专属记忆 (小望)
        ("小望每次见到用户都会主动提醒注意休息视力", "Social", "小望"),
    ];

    for (content, mode, role_id) in facts {
        upsert_fact(state.inner().clone(), content, role_id, mode, false).await?;
    }

    Ok(format!("成功注入 {} 条初始记忆数据", facts.len()))
}

#[command]
pub async fn optimize_memories(state: State<'_, Arc<RwLock<MemoryState>>>) -> Result<(), String> {
    let state_read = state.read().await;
    state_read.db.optimize_table().await
}
