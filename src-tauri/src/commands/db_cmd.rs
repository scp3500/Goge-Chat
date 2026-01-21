use tauri::State;
use rusqlite::params;
use crate::db::DbState;
use crate::models::{Session, Message};

// 🩺 内部辅助工具：确保 ID 转换安全
fn parse_id(id: &str) -> Result<i64, String> {
    id.parse::<i64>().map_err(|_| format!("无效的 ID 格式: {}", id))
}

#[tauri::command]
pub fn get_sessions(state: State<DbState>) -> Result<Vec<Session>, String> {
    let conn = state.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, title, last_scroll_pos, updated_at FROM sessions ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let iter = stmt.query_map([], |row| {
        // ✨ 【关键修复点】：通过显式类型标注 let id_num: i64 让编译器闭嘴
        let id_num: i64 = row.get(0)?; 
        
        Ok(Session {
            id: id_num.to_string(), // 将 i64 转为 String 赋给模型
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
pub fn create_session(title: String, state: State<DbState>) -> Result<String, String> {
    let conn = state.0.lock().unwrap();
    conn.execute(
        "INSERT INTO sessions (title, last_scroll_pos) VALUES (?, 0)",
        params![title],
    ).map_err(|e| e.to_string())?;
    
    // 返回 String 类型的 ID
    Ok(conn.last_insert_rowid().to_string())
}

#[tauri::command]
pub fn update_session_scroll(id: String, pos: i32, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;

    conn.execute(
        "UPDATE sessions SET last_scroll_pos = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        params![pos, numeric_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_session_title(id: String, title: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;

    conn.execute(
        "UPDATE sessions SET title = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        params![title, numeric_id]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_session(id: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;

    conn.execute("DELETE FROM messages WHERE session_id = ?", params![numeric_id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM sessions WHERE id = ?", params![numeric_id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_messages(session_id: String, state: State<DbState>) -> Result<Vec<Message>, String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&session_id)?;

    let mut stmt = conn.prepare("SELECT role, content FROM messages WHERE session_id = ? ORDER BY id ASC").map_err(|e| e.to_string())?;
    let iter = stmt.query_map([numeric_id], |row| Ok(Message { role: row.get(0)?, content: row.get(1)? })).map_err(|e| e.to_string())?;
    
    let mut m = Vec::new();
    for res in iter { m.push(res.map_err(|e| e.to_string())?); }
    Ok(m)
}

#[tauri::command]
pub fn save_message(session_id: String, role: String, content: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&session_id)?;

    conn.execute("INSERT INTO messages (session_id, role, content) VALUES (?, ?, ?)", params![numeric_id, role, content]).map_err(|e| e.to_string())?;
    conn.execute("UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?", params![numeric_id]).map_err(|e| e.to_string())?;
    Ok(())
}