// Live2D logic moved to standalone project

mod commands;
mod db;
mod memory;
mod memory_commands;
mod models;
mod social_db;
mod title_commands;

use crate::db::DbState;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

// ✨ 【新增导入】：用于多线程安全的红绿灯标志位
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::State;

// ✨ 【新增导入】：用于 HTTP 请求
// (Message, Client, etc. moved to title_commands.rs)

// ✨ 【新增状态】：定义全局中断标志位
pub struct GoleState {
    pub stop_flag: Arc<AtomicBool>,
}

// ✨ 【新增指令 1】：强制变红灯
#[tauri::command]
async fn stop_ai_generation(state: State<'_, GoleState>) -> Result<(), String> {
    state.stop_flag.store(true, Ordering::Relaxed);
    println!("🛑 后端已收到中断信号，红灯亮起");
    Ok(())
}

// ✨ 【新增指令 2】：重置为绿灯
#[tauri::command]
async fn reset_ai_generation(state: State<'_, GoleState>) -> Result<(), String> {
    state.stop_flag.store(false, Ordering::Relaxed);
    println!("🟢 状态已重置，绿灯亮起");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_handle = app.handle();

            // --- 1. 定位“便携式”数据目录 (当前可执行文件同级目录下的 data) ---
            let exe_path =
                std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let exe_dir = exe_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let app_data_dir = exe_dir.join("data");

            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir).expect("无法创建数据目录");
            }

            let target_db_path = app_data_dir.join("goge.db");
            let target_social_db_path = app_data_dir.join("gole_social.db");

            // --- 2. 数据库搬迁逻辑已移除，强制使用 D 盘便携目录 ---

            let conn = Connection::open(&target_db_path).expect("无法初始化数据库连接");
            db::init_db(&conn).expect("数据库初始化或升级失败");
            app.manage(DbState(Mutex::new(conn)));

            let social_conn =
                Connection::open(&target_social_db_path).expect("无法初始化社交数据库连接");
            social_db::init_social_db(&social_conn).expect("社交数据库初始化失败");
            app.manage(social_db::SocialDbState(Mutex::new(social_conn)));

            app.manage(GoleState {
                stop_flag: Arc::new(AtomicBool::new(false)),
            });

            // --- HTTP Client Setup ---
            app.manage(reqwest::Client::new());

            // --- Alice Memory Engine Setup ---
            let memory_state = memory::processor::MemoryState::new(app_handle)?;
            // 确保表存在 (1536 是 BGE-Small 的维度，如果是其它模型请调整)
            // 实际上 bge-small-zh-v1.5 的维度是 512
            let memory_state = Arc::new(tokio::sync::RwLock::new(memory_state));
            let ms_clone = memory_state.clone();
            tauri::async_runtime::block_on(async move {
                let ms = ms_clone.read().await;
                let _ = ms.db.ensure_table(512).await;
            });
            app.manage(memory_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 配置管理
            commands::config_cmd::load_config,
            commands::config_cmd::save_config,
            // AI 交互
            commands::ai::ask_ai,
            stop_ai_generation,
            reset_ai_generation,
            title_commands::generate_title,
            // 数据库 CRUD 指令
            commands::db_cmd::get_sessions,
            commands::db_cmd::create_session,
            commands::db_cmd::delete_session,
            commands::db_cmd::clear_messages,
            commands::db_cmd::delete_message,
            commands::db_cmd::update_message,
            commands::db_cmd::delete_messages_after,
            commands::db_cmd::get_messages,
            commands::db_cmd::save_message,
            commands::db_cmd::rename_session,
            commands::db_cmd::update_session_scroll,
            commands::db_cmd::update_sessions_order,
            commands::db_cmd::get_folders,
            commands::db_cmd::create_folder,
            commands::db_cmd::delete_folder,
            commands::db_cmd::rename_folder,
            commands::db_cmd::move_session_to_folder,
            commands::db_cmd::update_folder_collapsed,
            commands::db_cmd::update_folders_order,
            commands::db_cmd::update_session_config,
            // 文件指令
            commands::file_cmd::open_file,
            commands::file_cmd::read_file_text_content,
            commands::file_cmd::read_file_base64,
            commands::file_cmd::upload_user_avatar,
            // 社交数据库指令
            social_db::get_social_profile,
            social_db::get_social_contacts,
            social_db::get_social_groups,
            social_db::get_social_setting,
            social_db::set_social_setting,
            social_db::add_social_contact,
            social_db::update_social_contact,
            social_db::delete_social_contact,
            social_db::delete_social_message,
            social_db::delete_social_messages_after,
            social_db::update_social_message,
            social_db::get_social_messages,
            social_db::get_recent_social_messages,
            social_db::get_social_messages_paginated,
            social_db::save_social_message,
            social_db::get_recent_social_chats,
            social_db::update_social_profile,
            // ✨ Session Commands
            social_db::get_social_sessions,
            social_db::create_social_session,
            social_db::update_social_session_title,
            social_db::touch_social_session,
            social_db::delete_social_session,
            // 🧠 Memory Commands
            commands::memory_cmd::get_memories,
            commands::memory_cmd::insert_memory,
            commands::memory_cmd::clear_memories,
            commands::memory_cmd::delete_memory,
            commands::memory_cmd::update_memory,
            commands::memory_cmd::seed_memories,
            commands::memory_cmd::optimize_memories,
            memory_commands::trigger_fact_sync,
            memory_commands::diagnose_database,
            memory_commands::force_cleanup_database,
            memory_commands::rebuild_database,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 运行异常");
}
