use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub struct SocialDbState(pub Mutex<Connection>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub id: i64,
    pub name: String,
    pub avatar: Option<String>,
    pub group_id: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub nickname: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
}

pub fn init_social_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sort_order INTEGER DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS contacts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            avatar TEXT,
            group_id INTEGER,
            status TEXT,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE SET NULL
        );
        CREATE TABLE IF NOT EXISTS profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nickname TEXT NOT NULL,
            avatar TEXT,
            bio TEXT
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );
        ",
    )?;

    // Seed data if empty
    let group_count: i64 = conn.query_row("SELECT COUNT(*) FROM groups", [], |r| r.get(0))?;
    if group_count == 0 {
        conn.execute(
            "INSERT INTO groups (name, sort_order) VALUES ('我的好友', 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO groups (name, sort_order) VALUES ('工作专区', 2)",
            [],
        )?;
    }

    let profile_count: i64 = conn.query_row("SELECT COUNT(*) FROM profiles", [], |r| r.get(0))?;
    if profile_count == 0 {
        conn.execute(
            "INSERT INTO profiles (nickname, bio) VALUES ('GoleUser', '探索沉浸式社交新体验')",
            [],
        )?;
    }

    Ok(())
}

// Basic CRUD Commands (to be expanded)
#[tauri::command]
pub async fn get_social_profile(state: tauri::State<'_, SocialDbState>) -> Result<Profile, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, nickname, avatar, bio FROM profiles LIMIT 1")
        .map_err(|e| e.to_string())?;
    let profile = stmt
        .query_row([], |row| {
            Ok(Profile {
                id: row.get(0)?,
                nickname: row.get(1)?,
                avatar: row.get(2)?,
                bio: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(profile)
}

#[tauri::command]
pub async fn get_social_contacts(
    state: tauri::State<'_, SocialDbState>,
) -> Result<Vec<Contact>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, avatar, group_id, status FROM contacts")
        .map_err(|e| e.to_string())?;
    let contact_iter = stmt
        .query_map([], |row| {
            Ok(Contact {
                id: row.get(0)?,
                name: row.get(1)?,
                avatar: row.get(2)?,
                group_id: row.get(3)?,
                status: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut contacts = Vec::new();
    for contact in contact_iter {
        contacts.push(contact.map_err(|e| e.to_string())?);
    }
    Ok(contacts)
}

#[tauri::command]
pub async fn get_social_groups(
    state: tauri::State<'_, SocialDbState>,
) -> Result<Vec<Group>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, sort_order FROM groups ORDER BY sort_order ASC")
        .map_err(|e| e.to_string())?;
    let group_iter = stmt
        .query_map([], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut groups = Vec::new();
    for group in group_iter {
        groups.push(group.map_err(|e| e.to_string())?);
    }
    Ok(groups)
}

#[tauri::command]
pub async fn set_social_setting(
    state: tauri::State<'_, SocialDbState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_social_setting(
    state: tauri::State<'_, SocialDbState>,
    key: String,
) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| e.to_string())?;
    let value: Option<String> = stmt
        .query_row(params![key], |row| row.get::<_, String>(0))
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(value)
}
