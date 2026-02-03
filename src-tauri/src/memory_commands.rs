use crate::memory::processor::{extract_and_store_facts, MemoryState};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::RwLock;

#[tauri::command]
pub async fn trigger_fact_sync(
    app: AppHandle,
    memory_state: State<'_, Arc<RwLock<MemoryState>>>,
    session_id: i64,
    role_id: String,
    mode: String,
) -> Result<(), String> {
    println!(
        "📡 [指令] 收到 trigger_fact_sync | session_id: {}, role: {}, mode: {}",
        session_id, role_id, mode
    );
    extract_and_store_facts(
        &app,
        memory_state.inner().clone(),
        session_id,
        &role_id,
        &mode,
    )
    .await
}

#[tauri::command]
pub async fn diagnose_database(
    memory_state: State<'_, Arc<RwLock<MemoryState>>>,
) -> Result<crate::memory::db::DatabaseDiagnostic, String> {
    let ms = memory_state.read().await;
    ms.db.get_diagnostic().await
}

#[tauri::command]
pub async fn force_cleanup_database(
    memory_state: State<'_, Arc<RwLock<MemoryState>>>,
) -> Result<String, String> {
    println!("🧹 [数据库] 开始强制清理...");
    let (before_count, after_count) = {
        let ms = memory_state.read().await;
        let before = ms.db.get_all_memories().await?.len();

        // 执行 3 轮压缩确保彻底
        for i in 0..3 {
            println!("  第 {} 轮压缩...", i + 1);
            ms.db.optimize_table().await?;
        }

        let after = ms.db.get_all_memories().await?.len();
        (before, after)
    };

    Ok(format!(
        "✅ 清理完成！\n原有记录: {} 条\n剩余记录: {} 条",
        before_count, after_count
    ))
}

#[tauri::command]
pub async fn rebuild_database(
    memory_state: State<'_, Arc<RwLock<MemoryState>>>,
    confirmation_code: String,
) -> Result<String, String> {
    if confirmation_code != "REBUILD" {
        return Err("确认码错误，操作已取消".to_string());
    }

    let ms = memory_state.read().await;

    // 1. 获取当前数量用于反馈
    let backup_count = ms.db.get_all_memories().await?.len();
    println!("📦 尝试重建数据库 (当前包含 {} 条记录)", backup_count);

    // 2. 执行清空 (clear_memories 内部会 drop table 并 ensure_table)
    ms.db.clear_memories().await?;

    Ok(format!(
        "🔄 数据库已重建（已清空原有 {} 条记录）",
        backup_count
    ))
}
