// Live2D logic moved to standalone project

mod ai_utils;
mod behavior_engine;
mod behavior_scheduler;
mod character_state;
mod commands;
mod db;
mod immersive_settings;
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
use tauri::{Emitter, State};

// ✨ 【新增导入】：用于 HTTP 请求
// (Message, Client, etc. moved to title_commands.rs)

// ✨ 【新增状态】：定义全局中断标志位
pub struct GoleState {
    pub stop_flag: Arc<AtomicBool>,
}

// 🎯 [点击穿透] 共享坐标状态 - 改为支持多个区域
#[derive(serde::Deserialize, Clone, Debug)]
pub struct InteractionRegion {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

pub struct PassthroughState(pub Arc<Mutex<PassthroughData>>);

pub struct PassthroughData {
    pub regions: Vec<InteractionRegion>,
    pub active: bool,
}

// ✨ 【新增指令 1】：强制变红灯
#[tauri::command]
async fn stop_ai_generation(state: State<'_, GoleState>) -> Result<(), String> {
    state.stop_flag.store(true, Ordering::Relaxed);
    Ok(())
}

// ✨ 【新增指令 2】：重置为绿灯
#[tauri::command]
async fn reset_ai_generation(state: State<'_, GoleState>) -> Result<(), String> {
    state.stop_flag.store(false, Ordering::Relaxed);
    Ok(())
}

// ✨ 【新增指令 3】：设置窗口点击穿透
#[tauri::command]
async fn set_window_ignore_cursor_events(
    window: tauri::Window,
    ignore: bool,
    passthrough_state: State<'_, PassthroughState>,
) -> Result<(), String> {
    // 同时更新 active 状态，方便后台线程决定是否继续检测
    if let Ok(mut data) = passthrough_state.0.lock() {
        data.active = ignore;
    }

    window
        .set_ignore_cursor_events(ignore)
        .map_err(|e| format!("Failed to set ignore cursor events: {}", e))?;
    Ok(())
}

// ✨ 【新增指令 4】：更新监控区域 (支持多个)
#[tauri::command]
async fn start_passthrough_monitor(
    regions: Vec<InteractionRegion>,
    passthrough_state: State<'_, PassthroughState>,
) -> Result<(), String> {
    if let Ok(mut data) = passthrough_state.0.lock() {
        data.regions = regions;
        data.active = true;
    }
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

            // --- 🎯 点击穿透监控初始化 (支持多区域渲染项目) ---
            let passthrough_store = Arc::new(Mutex::new(PassthroughData {
                regions: Vec::new(),
                active: false,
            }));
            let ps_clone = passthrough_store.clone();
            app.manage(PassthroughState(passthrough_store));

            let main_window = app.get_webview_window("main").ok_or("找不到主窗口")?;
            tauri::async_runtime::spawn(async move {
                let mut last_ignore = false;
                loop {
                    // 检查窗口是否仍然有效
                    if let Err(_) = main_window.is_visible() {
                        break;
                    }

                    let (active, regions) = {
                        let r = ps_clone.lock().unwrap();
                        (r.active, r.regions.clone())
                    };

                    if active {
                        if let Ok(pos) = main_window.cursor_position() {
                            let sf = main_window.scale_factor().unwrap_or(1.0);

                            // 广播全局坐标给前端 (逻辑坐标)
                            let _ = main_window.emit(
                                "global-mouse-move",
                                serde_json::json!({
                                    "x": pos.x / sf,
                                    "y": pos.y / sf
                                }),
                            );

                            let mut in_any_area = false;
                            for reg in regions {
                                let px = reg.x * sf;
                                let py = reg.y * sf;
                                let pw = reg.w * sf;
                                let ph = reg.h * sf;
                                if pos.x >= px
                                    && pos.x <= px + pw
                                    && pos.y >= py
                                    && pos.y <= py + ph
                                {
                                    in_any_area = true;
                                    break;
                                }
                            }

                            let should_ignore = !in_any_area;
                            if should_ignore != last_ignore {
                                let _ = main_window.set_ignore_cursor_events(should_ignore);
                                last_ignore = should_ignore;
                            }
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(4)).await;
                }
            });

            // --- Config State Setup (Cache) ---
            let initial_config = tauri::async_runtime::block_on(async {
                commands::config_cmd::load_config_internal(app_handle).await
            })?;
            let config_state = commands::config_cmd::ConfigState(Arc::new(
                tokio::sync::RwLock::new(initial_config),
            ));
            app.manage(config_state);

            // --- HTTP Client Setup ---
            app.manage(reqwest::Client::new());

            // --- Alice Memory Engine Setup (Lazy Loaded) ---
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

            // --- Immersive Mode Scheduler Setup ---
            let scheduler = Arc::new(behavior_scheduler::MessageScheduler::new());
            scheduler.start_idle_monitor(app_handle.clone());
            app.manage(scheduler);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 配置管理
            commands::config_cmd::load_config,
            commands::config_cmd::save_config,
            // AI 交互
            commands::ai::ask_ai,
            commands::ai::discover_models_raw,
            stop_ai_generation,
            reset_ai_generation,
            set_window_ignore_cursor_events,
            start_passthrough_monitor,
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
            commands::asr_cmd::transcribe_pcm,
            commands::tts_cmd::generate_tts,
            commands::tts_cmd::next_tts_request_id,
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
            // 🎭 Immersive Mode Commands
            commands::immersive_cmd::send_social_message_immersive,
            commands::immersive_cmd::cancel_immersive_behaviors,
            commands::immersive_cmd::cancel_all_immersive_behaviors,
            // 🧠 Character State Commands
            social_db::get_character_state,
            social_db::upsert_character_state,
            social_db::increment_message_count,
            social_db::is_state_cache_valid,
            social_db::reset_message_count,
            social_db::cleanup_duplicate_messages,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 运行异常");
}
