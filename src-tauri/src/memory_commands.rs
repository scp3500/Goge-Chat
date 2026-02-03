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
