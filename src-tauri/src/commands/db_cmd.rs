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
        .prepare("SELECT id, title, last_scroll_pos, sort_order, updated_at, folder_id FROM sessions ORDER BY sort_order ASC, updated_at DESC")
        .map_err(|e| e.to_string())?;

    let iter = stmt.query_map([], |row| {
        let id_num: i64 = row.get(0)?;
        let folder_id: Option<i64> = row.get(5)?;
        Ok(Session {
            id: id_num.to_string(),
            title: row.get(1)?,
            last_scroll_pos: row.get(2)?,
            sort_order: row.get(3)?,
            updated_at: row.get(4)?,
            folder_id: folder_id.map(|id| id.to_string()),
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

/**
 * 🩺 修复点 1：将函数名改为 rename_session (匹配前端 invoke)
 */
#[tauri::command]
pub fn rename_session(id: String, title: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;
    conn.execute(
        "UPDATE sessions SET title = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        params![title, numeric_id]
    ).map_err(|e| e.to_string())?;
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
    conn.execute("DELETE FROM messages WHERE session_id = ?", params![numeric_id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM sessions WHERE id = ?", params![numeric_id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_messages(session_id: String, state: State<DbState>) -> Result<Vec<Message>, String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&session_id)?;
    let mut stmt = conn.prepare("SELECT role, content, reasoning_content FROM messages WHERE session_id = ? ORDER BY id ASC").map_err(|e| e.to_string())?;
    let iter = stmt.query_map([numeric_id], |row| {
        let role: String = row.get(0)?;
        let content: String = row.get(1)?;
        let reasoning_content: Option<String> = row.get(2)?;
        
        if reasoning_content.is_some() {
            println!("✅ DB Load: role={}, content_len={}, reasoning_len={}", role, content.len(), reasoning_content.as_ref().map(|r| r.len()).unwrap_or(0));
        }
        
        Ok(Message {
            role,
            content,
            reasoning_content,
        })
    }).map_err(|e| e.to_string())?;
    let mut m = Vec::new();
    for res in iter { m.push(res.map_err(|e| e.to_string())?); }
    println!("📊 Total messages loaded: {}", m.len());
    Ok(m)
}

#[tauri::command]
pub fn save_message(session_id: String, role: String, content: String, reasoning_content: Option<String>, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&session_id)?;
    conn.execute(
        "INSERT INTO messages (session_id, role, content, reasoning_content) VALUES (?, ?, ?, ?)",
        params![numeric_id, role, content, reasoning_content]
    ).map_err(|e| e.to_string())?;
    conn.execute("UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?", params![numeric_id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_folders(state: State<DbState>) -> Result<Vec<crate::models::Folder>, String> {
    let conn = state.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name, sort_order, is_collapsed FROM folders ORDER BY sort_order ASC, created_at DESC")
        .map_err(|e| e.to_string())?;

    let iter = stmt.query_map([], |row| {
        let id_num: i64 = row.get(0)?;
        Ok(crate::models::Folder {
            id: id_num.to_string(),
            name: row.get(1)?,
            sort_order: row.get(2)?,
            is_collapsed: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut folders = Vec::new();
    for res in iter {
        folders.push(res.map_err(|e| e.to_string())?);
    }
    Ok(folders)
}

#[tauri::command]
pub fn create_folder(name: String, state: State<DbState>) -> Result<String, String> {
    let conn = state.0.lock().unwrap();
    conn.execute(
        "INSERT INTO folders (name) VALUES (?)",
        params![name],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid().to_string())
}

#[tauri::command]
pub fn delete_folder(id: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;
    conn.execute("DELETE FROM folders WHERE id = ?", params![numeric_id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn rename_folder(id: String, name: String, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;
    conn.execute("UPDATE folders SET name = ? WHERE id = ?", params![name, numeric_id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_folder_collapsed(id: String, collapsed: bool, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let numeric_id = parse_id(&id)?;
    conn.execute("UPDATE folders SET is_collapsed = ?1 WHERE id = ?2", params![collapsed, numeric_id]).map_err(|e| e.to_string())?;
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
    conn.execute(
        "UPDATE sessions SET folder_id = ? WHERE id = ?",
        params![f_id, s_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_sessions_order(orders: Vec<(String, i32)>, state: State<DbState>) -> Result<(), String> {
    let mut conn = state.0.lock().unwrap();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    for (id, order) in orders {
        let numeric_id = parse_id(&id)?;
        tx.execute(
            "UPDATE sessions SET sort_order = ? WHERE id = ?",
            params![order, numeric_id],
        ).map_err(|e| e.to_string())?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_folders_order(orders: Vec<(String, i32)>, state: State<DbState>) -> Result<(), String> {
    let mut conn = state.0.lock().unwrap();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    for (id, order) in orders {
        let numeric_id = parse_id(&id)?;
        tx.execute(
            "UPDATE folders SET sort_order = ? WHERE id = ?",
            params![order, numeric_id],
        ).map_err(|e| e.to_string())?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}