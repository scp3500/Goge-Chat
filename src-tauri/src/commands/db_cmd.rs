use tauri::State;
use crate::db::{DbState, get_sessions as db_get_sessions, create_session as db_create_session, get_folders as db_get_folders, create_folder as db_create_folder, delete_folder as db_delete_folder, rename_folder as db_rename_folder, update_folder_collapsed as db_update_folder_collapsed, move_session_to_folder as db_move_session_to_folder, delete_session as db_delete_session, update_session_title as db_update_session_title, update_session_scroll as db_update_session_scroll, get_messages as db_get_messages, save_message as db_save_message, update_sessions_order as db_update_sessions_order, update_folders_order as db_update_folders_order};
use crate::models::{Session, Message};

// 🩺 内部辅助工具：确保 ID 转换安全
fn parse_id(id: &str) -> Result<i64, String> {
    id.parse::<i64>().map_err(|_| format!("无效的 ID 格式: {}", id))
}

#[tauri::command]
pub fn get_sessions(state: State<DbState>) -> Result<Vec<Session>, String> {
    let conn = state.0.lock().unwrap();
    let chat_sessions = db_get_sessions(&conn).map_err(|e| e.to_string())?;
    let sessions = chat_sessions.into_iter().map(|cs| Session {
        id: cs.id.to_string(),
        title: cs.title,
        last_scroll_pos: cs.last_scroll_pos,
        sort_order: cs.sort_order,
        updated_at: cs.updated_at,
        folder_id: cs.folder_id.map(|id| id.to_string()),
    }).collect();
    Ok(sessions)
}

#[tauri::command]
pub fn create_session(title: String, state: State<DbState>) -> Result<String, String> {
    let conn = state.0.lock().unwrap();
    let id = db_create_session(&conn, &title).map_err(|e| e.to_string())?;
    Ok(id.to_string())
}

#[tauri::command]
pub fn update_session_scroll(id: String, pos: i32, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;
    db_update_session_scroll(&conn, numeric_id, pos).map_err(|e| e.to_string())?;
    Ok(())
}

/**
 * 🩺 修复点 1：将函数名改为 rename_session (匹配前端 invoke)
 */
#[tauri::command]
pub fn rename_session(id: String, title: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;
    db_update_session_title(&conn, numeric_id, &title).map_err(|e| e.to_string())?;
    Ok(())
}

/**
 * 🩺 修复点 2：将参数名 id 改为 session_id
 * 前端传 { sessionId }，Tauri 自动对应 Rust 的 session_id
 */
#[tauri::command]
pub fn delete_session(session_id: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&session_id)?;
    db_delete_session(&conn, numeric_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_messages(session_id: String, state: State<DbState>) -> Result<Vec<Message>, String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&session_id)?;
    let chat_messages = db_get_messages(&conn, numeric_id).map_err(|e| e.to_string())?;
    let messages: Vec<Message> = chat_messages.into_iter().map(|cm| {
        if cm.reasoning_content.is_some() {
            println!("✅ DB Load: role={}, content_len={}, reasoning_len={}", cm.role, cm.content.len(), cm.reasoning_content.as_ref().map(|r| r.len()).unwrap_or(0));
        }
        Message {
            role: cm.role,
            content: cm.content,
            reasoning_content: cm.reasoning_content,
        }
    }).collect();
    println!("📊 Total messages loaded: {}", messages.len());
    Ok(messages)
}

#[tauri::command]
pub fn save_message(session_id: String, role: String, content: String, reasoning_content: Option<String>, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&session_id)?;
    let reasoning_ref = reasoning_content.as_deref();
    db_save_message(&conn, numeric_id, &role, &content, reasoning_ref).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_folders(state: State<DbState>) -> Result<Vec<crate::models::Folder>, String> {
    let conn = state.0.lock().unwrap();
    let db_folders = db_get_folders(&conn).map_err(|e| e.to_string())?;
    let folders = db_folders.into_iter().map(|f| crate::models::Folder {
        id: f.id.to_string(),
        name: f.name,
        sort_order: f.sort_order,
        is_collapsed: f.is_collapsed,
    }).collect();
    Ok(folders)
}

#[tauri::command]
pub fn create_folder(name: String, state: State<DbState>) -> Result<String, String> {
    let conn = state.0.lock().unwrap();
    let id = db_create_folder(&conn, &name).map_err(|e| e.to_string())?;
    Ok(id.to_string())
}

#[tauri::command]
pub fn delete_folder(id: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;
    db_delete_folder(&conn, numeric_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn rename_folder(id: String, name: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;
    db_rename_folder(&conn, numeric_id, &name).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_folder_collapsed(id: String, collapsed: bool, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;
    db_update_folder_collapsed(&conn, numeric_id, collapsed).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn move_session_to_folder(session_id: String, folder_id: Option<String>, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let s_id = parse_id(&session_id)?;
    let f_id = match folder_id {
        Some(fid) => Some(parse_id(&fid)?),
        None => None,
    };
    db_move_session_to_folder(&conn, s_id, f_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_sessions_order(orders: Vec<(String, i32)>, state: State<DbState>) -> Result<(), String> {
    let mut conn = state.0.lock().unwrap();
    let parsed_orders: Vec<(i64, i32)> = orders.into_iter().map(|(id, order)| {
        let numeric_id = parse_id(&id)?;
        Ok((numeric_id, order))
    }).collect::<Result<Vec<_>, String>>()?;
    db_update_sessions_order(&mut *conn, parsed_orders).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_folders_order(orders: Vec<(String, i32)>, state: State<DbState>) -> Result<(), String> {
    let mut conn = state.0.lock().unwrap();
    let parsed_orders: Vec<(i64, i32)> = orders.into_iter().map(|(id, order)| {
        let numeric_id = parse_id(&id)?;
        Ok((numeric_id, order))
    }).collect::<Result<Vec<_>, String>>()?;
    db_update_folders_order(&mut *conn, parsed_orders).map_err(|e| e.to_string())?;
    Ok(())
}