use rusqlite::{Connection, Result, params};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;

// 状态管理容器
pub struct DbState(pub Mutex<Connection>);

// --- 数据模型定义 ---

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: i64,
    pub folder_id: Option<i64>,
    pub title: String,
    pub last_scroll_pos: i32,
    pub sort_order: i32,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub sort_order: i32,
    pub is_collapsed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Option<i64>,
    pub session_id: i64,
    pub role: String,
    pub content: String,
    pub reasoning_content: Option<String>,
    pub created_at: Option<String>,
}

// --- 数据库初始化与迁移 ---

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sort_order INTEGER DEFAULT 0,
            is_collapsed BOOLEAN DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id INTEGER,
            title TEXT NOT NULL,
            last_scroll_pos INTEGER DEFAULT 0,
            sort_order INTEGER DEFAULT 0,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (folder_id) REFERENCES folders (id) ON DELETE SET NULL
        );
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            reasoning_content TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE CASCADE
        );
    ")?;

    let mut stmt = conn.prepare("PRAGMA table_info(sessions)")?;
    let columns_sessions: Vec<String> = stmt.query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;

    if !columns_sessions.contains(&"last_scroll_pos".to_string()) {
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN last_scroll_pos INTEGER DEFAULT 0", []);
    }

    if !columns_sessions.contains(&"updated_at".to_string()) {
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN updated_at DATETIME", []);
        let _ = conn.execute("UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL", []);
    }

    if !columns_sessions.contains(&"sort_order".to_string()) {
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN sort_order INTEGER DEFAULT 0", []);
    }

    // folder_id 可能已经在 CREATE TABLE 语句中创建了，也可能是以前的老数据库需要迁移
    // 但为了保险，我们只在它确实不存在时才 ALTER TABLE
    if !columns_sessions.contains(&"folder_id".to_string()) {
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN folder_id INTEGER", []);
    }

    let mut stmt = conn.prepare("PRAGMA table_info(folders)")?;
    let columns_folders: Vec<String> = stmt.query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;
    if !columns_folders.contains(&"is_collapsed".to_string()) {
        let _ = conn.execute("ALTER TABLE folders ADD COLUMN is_collapsed BOOLEAN DEFAULT 0", []);
    }

    let mut stmt = conn.prepare("PRAGMA table_info(messages)")?;
    let columns_messages: Vec<String> = stmt.query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;

    if !columns_messages.contains(&"reasoning_content".to_string()) {
        let _ = conn.execute("ALTER TABLE messages ADD COLUMN reasoning_content TEXT", []);
    }

    Ok(())
}

// --- 会话管理逻辑 ---

/**
 * 🩺 导师说明：
 * 在 Rust 项目中，如果函数是 pub 的但依然报 unused 警告，
 * 说明在当前二进制包中确实没有任何地方调用它。
 * 你可以在模块顶部加上 #![allow(dead_code)] 来全局消除，
 * 但更好的做法是在 db_cmd.rs 中开始引用它们。
 */

pub(crate) fn get_sessions(conn: &Connection) -> Result<Vec<ChatSession>> {
    let mut stmt = conn.prepare(
        "SELECT id, folder_id, title, last_scroll_pos, sort_order, updated_at FROM sessions ORDER BY sort_order ASC, updated_at DESC"
    )?;

    let session_iter = stmt.query_map([], |row| {
        Ok(ChatSession {
            id: row.get(0)?,
            folder_id: row.get(1)?,
            title: row.get(2)?,
            last_scroll_pos: row.get(3)?,
            sort_order: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;

    let mut sessions = Vec::new();
    for session in session_iter {
        sessions.push(session?);
    }
    Ok(sessions)
}

pub(crate) fn create_session(conn: &Connection, title: &str) -> Result<i64> {
    conn.execute("INSERT INTO sessions (title, last_scroll_pos) VALUES (?1, 0)", params![title])?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn get_folders(conn: &Connection) -> Result<Vec<Folder>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, is_collapsed FROM folders ORDER BY sort_order ASC, created_at DESC"
    )?;

    let folder_iter = stmt.query_map([], |row| {
        Ok(Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            is_collapsed: row.get(3)?,
        })
    })?;

    let mut folders = Vec::new();
    for folder in folder_iter {
        folders.push(folder?);
    }
    Ok(folders)
}

pub(crate) fn create_folder(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute("INSERT INTO folders (name) VALUES (?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn delete_folder(conn: &Connection, id: i64) -> Result<()> {
    // sessions 表有 ON DELETE SET NULL，所以直接删
    conn.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
    Ok(())
}

pub(crate) fn rename_folder(conn: &Connection, id: i64, name: &str) -> Result<()> {
    conn.execute("UPDATE folders SET name = ?1 WHERE id = ?2", params![name, id])?;
    Ok(())
}

pub(crate) fn update_folder_collapsed(conn: &Connection, id: i64, collapsed: bool) -> Result<()> {
    conn.execute("UPDATE folders SET is_collapsed = ?1 WHERE id = ?2", params![collapsed, id])?;
    Ok(())
}

pub(crate) fn move_session_to_folder(conn: &Connection, session_id: i64, folder_id: Option<i64>) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET folder_id = ?1 WHERE id = ?2",
        params![folder_id, session_id]
    )?;
    Ok(())
}

pub(crate) fn delete_session(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM messages WHERE session_id = ?1", params![id])?;
    conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
    Ok(())
}

pub(crate) fn update_session_title(conn: &Connection, id: i64, title: &str) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET title = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![title, id]
    )?;
    Ok(())
}

pub(crate) fn update_session_scroll(conn: &Connection, id: i64, pos: i32) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET last_scroll_pos = ?1 WHERE id = ?2",
        params![pos, id]
    )?;
    Ok(())
}

// --- 消息管理逻辑 ---

pub(crate) fn get_messages(conn: &Connection, session_id: i64) -> Result<Vec<ChatMessage>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, content, reasoning_content, created_at FROM messages WHERE session_id = ?1 ORDER BY id ASC"
    )?;

    let msg_iter = stmt.query_map(params![session_id], |row| {
        Ok(ChatMessage {
            id: Some(row.get(0)?),
            session_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            reasoning_content: row.get(4)?,
            created_at: Some(row.get(5)?),
        })
    })?;

    let mut messages = Vec::new();
    for msg in msg_iter {
        messages.push(msg?);
    }
    Ok(messages)
}

pub(crate) fn save_message(conn: &Connection, session_id: i64, role: &str, content: &str, reasoning_content: Option<&str>) -> Result<()> {
    println!("💾 [DEBUG] Saving message: role={}, content_len={}, reasoning_len={}", role, content.len(), reasoning_content.map(|r| r.len()).unwrap_or(0));
    conn.execute(
        "INSERT INTO messages (session_id, role, content, reasoning_content) VALUES (?1, ?2, ?3, ?4)",
        params![session_id, role, content, reasoning_content],
    )?;
    conn.execute(
        "UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![session_id]
    )?;
    Ok(())
}

pub(crate) fn update_sessions_order(conn: &mut Connection, orders: Vec<(i64, i32)>) -> Result<()> {
    let tx = conn.transaction()?;
    for (id, order) in orders {
        tx.execute(
            "UPDATE sessions SET sort_order = ? WHERE id = ?",
            params![order, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub(crate) fn update_folders_order(conn: &mut Connection, orders: Vec<(i64, i32)>) -> Result<()> {
    let tx = conn.transaction()?;
    for (id, order) in orders {
        tx.execute(
            "UPDATE folders SET sort_order = ? WHERE id = ?",
            params![order, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}