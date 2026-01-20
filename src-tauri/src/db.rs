use rusqlite::{Connection, Result};
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);

/// 数据库初始化：地基工程
pub fn init_db(conn: &Connection) -> Result<()> {
    // 1. 创建基础表结构（如果是新用户，这里会直接成功）
    // 注意：CREATE TABLE 时使用 CURRENT_TIMESTAMP 是合法的
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
            FOREIGN KEY (session_id) REFERENCES sessions (id)
        );
    ")?;

    // 2. 动态检测并修复旧表字段
    let mut stmt = conn.prepare("PRAGMA table_info(sessions)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get(1))?
        .collect::<Result<Vec<_>>>()?;

    // --- 修复 last_scroll_pos ---
    if !columns.contains(&"last_scroll_pos".to_string()) {
        // 使用 0 作为常量默认值，这是允许的
        conn.execute("ALTER TABLE sessions ADD COLUMN last_scroll_pos INTEGER DEFAULT 0", [])?;
        println!("Migration: Added last_scroll_pos to sessions");
    }

    // --- 修复 updated_at (关键修复点) ---
    if !columns.contains(&"updated_at".to_string()) {
        // 第一步：只添加列，不设置动态默认值
        conn.execute("ALTER TABLE sessions ADD COLUMN updated_at DATETIME", [])?;
        
        // 第二步：手动初始化旧数据的时间戳
        conn.execute("UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL", [])?;
        println!("Migration: Added updated_at to sessions and initialized data");
    }

    Ok(())
}