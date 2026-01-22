// src-tauri/src/lib.rs

mod models;
mod db;
mod commands;

use rusqlite::Connection;
use std::sync::Mutex;
use crate::db::DbState;
use tauri::Manager;

// ✨ 【新增导入】：用于多线程安全的红绿灯标志位
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::State;

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
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle();
            let app_dir = app_handle.path().app_data_dir().expect("无法获取 C 盘数据目录");
            
            if !app_dir.exists() {
                std::fs::create_dir_all(&app_dir).expect("无法在 C 盘创建数据目录");
            }

            let db_path = app_dir.join("alice_data.db");
            let conn = Connection::open(&db_path).expect("无法初始化数据库连接");
            db::init_db(&conn).expect("数据库初始化或升级失败");

            // ✨ 【状态管理】：注入数据库连接
            app.manage(DbState(Mutex::new(conn)));

            // ✨ 【核心新增】：注入物理中断状态锁
            app.manage(GoleState {
                stop_flag: Arc::new(AtomicBool::new(false)),
            });

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

            // 数据库 CRUD 指令
            commands::db_cmd::get_sessions,
            commands::db_cmd::create_session,
            commands::db_cmd::delete_session,
            commands::db_cmd::get_messages,
            commands::db_cmd::save_message,
            // 🩺 关键手术点：将 update_session_title 改为 rename_session
            commands::db_cmd::rename_session, 
            commands::db_cmd::update_session_scroll 
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 运行异常");
}