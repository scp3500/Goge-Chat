use rusqlite::{Connection, Result, params};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;

// 状态管理容器
pub struct DbState(pub Mutex<Connection>);

// --- 数据模型定义 ---

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: i64,
    pub title: String,
    pub last_scroll_pos: i32,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Option<i64>,
    pub session_id: i64,
    pub role: String,
    pub content: String,
    pub created_at: Option<String>,
}

// --- 数据库初始化与迁移 ---

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            last_scroll_pos INTEGER DEFAULT 0,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE CASCADE
        );
    ")?;

    let mut stmt = conn.prepare("PRAGMA table_info(sessions)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get(1))?
        .collect::<Result<Vec<_>>>()?;

    if !columns.contains(&"last_scroll_pos".to_string()) {
        conn.execute("ALTER TABLE sessions ADD COLUMN last_scroll_pos INTEGER DEFAULT 0", [])?;
    }

    if !columns.contains(&"updated_at".to_string()) {
        conn.execute("ALTER TABLE sessions ADD COLUMN updated_at DATETIME", [])?;
        conn.execute("UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL", [])?;
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

pub fn get_sessions(conn: &Connection) -> Result<Vec<ChatSession>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, last_scroll_pos, updated_at FROM sessions ORDER BY updated_at DESC"
    )?;

    let session_iter = stmt.query_map([], |row| {
        Ok(ChatSession {
            id: row.get(0)?,
            title: row.get(1)?,
            last_scroll_pos: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;

    let mut sessions = Vec::new();
    for session in session_iter {
        sessions.push(session?);
    }
    Ok(sessions)
}

pub fn create_session(conn: &Connection, title: &str) -> Result<i64> {
    conn.execute("INSERT INTO sessions (title) VALUES (?1)", params![title])?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_session(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn update_session_title(conn: &Connection, id: i64, title: &str) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET title = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![title, id]
    )?;
    Ok(())
}

pub fn update_session_scroll(conn: &Connection, id: i64, pos: i32) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET last_scroll_pos = ?1 WHERE id = ?2",
        params![pos, id]
    )?;
    Ok(())
}

// --- 消息管理逻辑 ---

pub fn get_messages(conn: &Connection, session_id: i64) -> Result<Vec<ChatMessage>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, content, created_at FROM messages WHERE session_id = ?1 ORDER BY id ASC"
    )?;

    let msg_iter = stmt.query_map(params![session_id], |row| {
        Ok(ChatMessage {
            id: Some(row.get(0)?),
            session_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            created_at: Some(row.get(4)?),
        })
    })?;

    let mut messages = Vec::new();
    for msg in msg_iter {
        messages.push(msg?);
    }
    Ok(messages)
}

pub fn save_message(conn: &Connection, session_id: i64, role: &str, content: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO messages (session_id, role, content) VALUES (?1, ?2, ?3)",
        params![session_id, role, content],
    )?;
    conn.execute(
        "UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![session_id]
    )?;
    Ok(())
}