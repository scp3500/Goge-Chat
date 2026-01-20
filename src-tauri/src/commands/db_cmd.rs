use tauri::State;
use rusqlite::params;
use crate::db::DbState;
use crate::models::{Session, Message};

#[tauri::command]
pub fn get_sessions(state: State<DbState>) -> Result<Vec<Session>, String> {
    let conn = state.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, title, last_scroll_pos, updated_at FROM sessions ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let iter = stmt.query_map([], |row| {
        Ok(Session {
            id: row.get(0)?,
            title: row.get(1)?,
            last_scroll_pos: row.get(2)?,
            updated_at: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut s = Vec::new();
    for res in iter {
        s.push(res.map_err(|e| e.to_string())?);
    }
    Ok(s)
}

#[tauri::command]
pub fn create_session(title: String, state: State<DbState>) -> Result<i64, String> {
    let conn = state.0.lock().unwrap();
    conn.execute(
        "INSERT INTO sessions (title, last_scroll_pos) VALUES (?, 0)",
        params![title],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_session_scroll(id: i64, pos: i32, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    conn.execute(
        "UPDATE sessions SET last_scroll_pos = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        params![pos, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_session_title(id: i64, title: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    conn.execute(
        "UPDATE sessions SET title = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        params![title, id]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// ✨ 【核心修复】：必须先删除关联消息，再删除会话
#[tauri::command]
pub fn delete_session(id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    // 1. 物理删除所有属于该会话的消息
    conn.execute("DELETE FROM messages WHERE session_id = ?", params![id]).map_err(|e| e.to_string())?;
    // 2. 删除会话记录
    conn.execute("DELETE FROM sessions WHERE id = ?", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_messages(session_id: i64, state: State<DbState>) -> Result<Vec<Message>, String> {
    let conn = state.0.lock().unwrap();
    let mut stmt = conn.prepare("SELECT role, content FROM messages WHERE session_id = ? ORDER BY id ASC").map_err(|e| e.to_string())?;
    let iter = stmt.query_map([session_id], |row| Ok(Message { role: row.get(0)?, content: row.get(1)? })).map_err(|e| e.to_string())?;
    let mut m = Vec::new();
    for res in iter { m.push(res.map_err(|e| e.to_string())?); }
    Ok(m)
}

#[tauri::command]
pub fn save_message(session_id: i64, role: String, content: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    conn.execute("INSERT INTO messages (session_id, role, content) VALUES (?, ?, ?)", params![session_id, role, content]).map_err(|e| e.to_string())?;
    conn.execute("UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?", params![session_id]).map_err(|e| e.to_string())?;
    Ok(())
}