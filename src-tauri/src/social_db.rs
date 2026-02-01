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
    pub prompt: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialSession {
    pub id: i64,
    pub contact_id: i64,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
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
            prompt TEXT,
            model TEXT,
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
        CREATE TABLE IF NOT EXISTS social_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contact_id INTEGER NOT NULL,
            title TEXT NOT NULL DEFAULT '新对话',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (contact_id) REFERENCES contacts (id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS social_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contact_id INTEGER NOT NULL,
            session_id INTEGER,  -- Allow NULL for legacy/global messages if needed, but we should migrate
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            file_metadata TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (contact_id) REFERENCES contacts (id) ON DELETE CASCADE,
            FOREIGN KEY (session_id) REFERENCES social_sessions (id) ON DELETE CASCADE
        );
        ",
    )?;

    // ⚡️ Add Index for fast pagination
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_social_messages_contact_id_id ON social_messages (contact_id, id)",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_social_messages_session_id ON social_messages (session_id)",
        [],
    );

    // Schema Migrations - contacts
    let mut stmt = conn.prepare("PRAGMA table_info(contacts)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;

    if !columns.contains(&"prompt".to_string()) {
        let _ = conn.execute("ALTER TABLE contacts ADD COLUMN prompt TEXT", []);
    }
    if !columns.contains(&"model".to_string()) {
        let _ = conn.execute("ALTER TABLE contacts ADD COLUMN model TEXT", []);
    }
    if !columns.contains(&"provider".to_string()) {
        let _ = conn.execute("ALTER TABLE contacts ADD COLUMN provider TEXT", []);
    }

    // Schema Migrations - social_messages (Adding session_id)
    let mut stmt_msg = conn.prepare("PRAGMA table_info(social_messages)")?;
    let msg_columns: Vec<String> = stmt_msg
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;

    if !msg_columns.contains(&"session_id".to_string()) {
        println!("🔧 Migrating social_messages: adding session_id");
        let _ = conn.execute(
            "ALTER TABLE social_messages ADD COLUMN session_id INTEGER",
            [],
        );

        // 🛠️ DATA MIGRATION: Move orphan messages to a default session
        println!("🔧 Migrating existing messages to default sessions...");
        conn.execute_batch(
            "
            INSERT INTO social_sessions (contact_id, title)
            SELECT DISTINCT contact_id, '默认会话' FROM social_messages WHERE session_id IS NULL;
            
            UPDATE social_messages 
            SET session_id = (
                SELECT id FROM social_sessions 
                WHERE social_sessions.contact_id = social_messages.contact_id 
                LIMIT 1
            )
            WHERE session_id IS NULL;
        ",
        )?;
    }

    // Refresh msg_columns for subsequent migrations
    let mut stmt_msg = conn.prepare("PRAGMA table_info(social_messages)")?;
    let msg_columns: Vec<String> = stmt_msg
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;

    if !msg_columns.contains(&"file_metadata".to_string()) {
        println!("🔧 Migrating social_messages: adding file_metadata");
        let _ = conn.execute(
            "ALTER TABLE social_messages ADD COLUMN file_metadata TEXT",
            [],
        );
    }

    // Schema Migrations - profiles
    let mut stmt_prof = conn.prepare("PRAGMA table_info(profiles)")?;
    let prof_columns: Vec<String> = stmt_prof
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;

    if !prof_columns.contains(&"nickname".to_string()) {
        println!("🔧 Migrating profiles table: adding nickname");
        if let Err(e) = conn.execute(
            "ALTER TABLE profiles ADD COLUMN nickname TEXT NOT NULL DEFAULT 'GoleUser'",
            [],
        ) {
            println!("⚠️ Failed to add nickname column: {}", e);
        }
    }

    // Seed data if empty
    let group_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM groups", [], |r| r.get(0))
        .unwrap_or(0);
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

    let profile_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM profiles", [], |r| r.get(0))
        .unwrap_or(0);
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
pub async fn update_social_profile(
    state: tauri::State<'_, SocialDbState>,
    nickname: String,
    avatar: Option<String>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE profiles SET nickname = ?1, avatar = ?2 WHERE id = (SELECT id FROM profiles LIMIT 1)",
        params![nickname, avatar],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_social_contacts(
    state: tauri::State<'_, SocialDbState>,
) -> Result<Vec<Contact>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, avatar, group_id, status, prompt, model, provider FROM contacts")
        .map_err(|e| e.to_string())?;
    let contact_iter = stmt
        .query_map([], |row| {
            Ok(Contact {
                id: row.get(0)?,
                name: row.get(1)?,
                avatar: row.get(2)?,
                group_id: row.get(3)?,
                status: row.get(4)?,
                prompt: row.get(5)?,
                model: row.get(6)?,
                provider: row.get(7)?,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialMessage {
    pub id: Option<i64>,
    pub contact_id: i64,
    pub session_id: Option<i64>,
    pub role: String,
    pub content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fileMetadata")]
    #[serde(alias = "file_metadata")]
    pub file_metadata: Option<String>,

    pub created_at: Option<String>,
}

#[tauri::command]
pub async fn add_social_contact(
    state: tauri::State<'_, SocialDbState>,
    name: String,
    avatar: Option<String>,
    group_id: Option<i64>,
    prompt: Option<String>,
    model: Option<String>,
    provider: Option<String>,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO contacts (name, avatar, group_id, prompt, model, provider) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![name, avatar, group_id, prompt, model, provider],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub async fn update_social_contact(
    state: tauri::State<'_, SocialDbState>,
    id: i64,
    name: String,
    avatar: Option<String>,
    prompt: Option<String>,
    model: Option<String>,
    provider: Option<String>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE contacts SET name = ?1, avatar = ?2, prompt = ?3, model = ?4, provider = ?5 WHERE id = ?6",
        params![name, avatar, prompt, model, provider, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_social_contact(
    state: tauri::State<'_, SocialDbState>,
    id: i64,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM contacts WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM social_messages WHERE contact_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_social_messages(
    state: tauri::State<'_, SocialDbState>,
    contact_id: i64,
) -> Result<Vec<SocialMessage>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, contact_id, session_id, role, content, file_metadata, created_at FROM social_messages WHERE contact_id = ?1 ORDER BY id ASC")
        .map_err(|e| e.to_string())?;
    let msg_iter = stmt
        .query_map(params![contact_id], |row| {
            Ok(SocialMessage {
                id: row.get(0)?,
                contact_id: row.get(1)?,
                session_id: row.get(2)?,
                role: row.get(3)?,
                content: row.get(4)?,
                file_metadata: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut messages = Vec::new();
    for msg in msg_iter {
        messages.push(msg.map_err(|e| e.to_string())?);
    }
    Ok(messages)
}

#[tauri::command]
pub async fn get_recent_social_messages(
    state: tauri::State<'_, SocialDbState>,
    contact_id: i64,
    session_id: Option<i64>, // ✨ Filter by session_id (Optional for transitions)
    limit: i64,
) -> Result<Vec<SocialMessage>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    // Load the LAST N messages of a session
    let sql = if session_id.is_some() {
        "SELECT id, contact_id, session_id, role, content, file_metadata, created_at FROM social_messages WHERE contact_id = ?1 AND session_id = ?2 ORDER BY id DESC LIMIT ?3"
    } else {
        // Fallback: get all messages if no session specified (or for testing)
        "SELECT id, contact_id, session_id, role, content, file_metadata, created_at FROM social_messages WHERE contact_id = ?1 ORDER BY id DESC LIMIT ?3"
    };

    let params_list: Vec<&dyn rusqlite::ToSql> = if let Some(sid) = session_id.as_ref() {
        vec![&contact_id, sid, &limit] // Borrow from session_id
    } else {
        vec![&contact_id, &limit]
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    // Rust rusqlite params! macro is for fixed args. For dynamic, we use query_map with slice
    let msg_iter = stmt
        .query_map(rusqlite::params_from_iter(params_list), |row| {
            Ok(SocialMessage {
                id: row.get(0)?,
                contact_id: row.get(1)?,
                session_id: row.get(2)?,
                role: row.get(3)?,
                content: row.get(4)?,
                file_metadata: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut messages = Vec::new();
    for msg in msg_iter {
        messages.push(msg.map_err(|e| e.to_string())?);
    }
    messages.reverse();
    Ok(messages)
}

#[tauri::command]
pub async fn get_social_messages_paginated(
    state: tauri::State<'_, SocialDbState>,
    contact_id: i64,
    session_id: Option<i64>, // ✨ Filter by session_id
    limit: i64,
    before_id: i64,
) -> Result<Vec<SocialMessage>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let sql = if session_id.is_some() {
        "SELECT id, contact_id, session_id, role, content, file_metadata, created_at FROM social_messages WHERE contact_id = ?1 AND session_id = ?2 AND id < ?3 ORDER BY id DESC LIMIT ?4"
    } else {
        "SELECT id, contact_id, session_id, role, content, file_metadata, created_at FROM social_messages WHERE contact_id = ?1 AND id < ?3 ORDER BY id DESC LIMIT ?4"
    };

    let params_vec: Vec<&dyn rusqlite::ToSql> = if let Some(sid) = session_id.as_ref() {
        vec![&contact_id, sid, &before_id, &limit]
    } else {
        vec![&contact_id, &before_id, &limit]
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let msg_iter = stmt
        .query_map(rusqlite::params_from_iter(params_vec), |row| {
            Ok(SocialMessage {
                id: row.get(0)?,
                contact_id: row.get(1)?,
                session_id: row.get(2)?,
                role: row.get(3)?,
                content: row.get(4)?,
                file_metadata: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut messages = Vec::new();
    for msg in msg_iter {
        messages.push(msg.map_err(|e| e.to_string())?);
    }
    messages.reverse();
    Ok(messages)
}

#[tauri::command]
pub async fn save_social_message(
    state: tauri::State<'_, SocialDbState>,
    contact_id: i64,
    session_id: Option<i64>, // ✨ Added session_id
    role: String,
    content: String,
    file_metadata: Option<String>,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO social_messages (contact_id, session_id, role, content, file_metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![contact_id, session_id, role, content, file_metadata],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub async fn delete_social_message(
    state: tauri::State<'_, SocialDbState>,
    id: i64,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM social_messages WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_social_messages_after(
    state: tauri::State<'_, SocialDbState>,
    contact_id: i64,
    message_id: i64,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM social_messages WHERE contact_id = ?1 AND id >= ?2",
        params![contact_id, message_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_social_message(
    state: tauri::State<'_, SocialDbState>,
    id: i64,
    content: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE social_messages SET content = ?1 WHERE id = ?2",
        params![content, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
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
#[derive(Debug, Serialize, Deserialize)]
pub struct SocialConversation {
    pub contact: Contact,
    pub last_message: Option<String>,
    pub last_message_time: Option<String>,
    pub unread_count: i32,
}

#[tauri::command]
pub async fn get_recent_social_chats(
    state: tauri::State<'_, SocialDbState>,
) -> Result<Vec<SocialConversation>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, c.avatar, c.group_id, c.status, c.prompt, c.model, c.provider, 
                    m.content, m.created_at
             FROM contacts c
             LEFT JOIN (
                SELECT contact_id, content, created_at,
                       ROW_NUMBER() OVER (PARTITION BY contact_id ORDER BY created_at DESC) as rn
                FROM social_messages
             ) m ON c.id = m.contact_id AND m.rn = 1
             WHERE m.content IS NOT NULL
             ORDER BY m.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let chat_iter = stmt
        .query_map([], |row| {
            Ok(SocialConversation {
                contact: Contact {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    avatar: row.get(2)?,
                    group_id: row.get(3)?,
                    status: row.get(4)?,
                    prompt: row.get(5)?,
                    model: row.get(6)?,
                    provider: row.get(7)?,
                },
                last_message: row.get(8)?,
                last_message_time: row.get(9)?,
                unread_count: 0, // Placeholder
            })
        })
        .map_err(|e| e.to_string())?;

    let mut chats = Vec::new();
    for chat in chat_iter {
        chats.push(chat.map_err(|e| e.to_string())?);
    }
    Ok(chats)
}
// --- ✨ Session Management Commands ---

#[tauri::command]
pub async fn get_social_sessions(
    state: tauri::State<'_, SocialDbState>,
    contact_id: i64,
) -> Result<Vec<SocialSession>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.contact_id, s.title, s.created_at, 
                    COALESCE(MAX(m.created_at), s.updated_at) as last_activity
             FROM social_sessions s
             LEFT JOIN social_messages m ON s.id = m.session_id
             WHERE s.contact_id = ?1
             GROUP BY s.id
             ORDER BY last_activity DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![contact_id], |row| {
            Ok(SocialSession {
                id: row.get(0)?,
                contact_id: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?, // Use the calculated last_activity
            })
        })
        .map_err(|e| e.to_string())?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row.map_err(|e| e.to_string())?);
    }
    Ok(sessions)
}

#[tauri::command]
pub async fn create_social_session(
    state: tauri::State<'_, SocialDbState>,
    contact_id: i64,
    title: String,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO social_sessions (contact_id, title) VALUES (?1, ?2)",
        params![contact_id, title],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub async fn update_social_session_title(
    state: tauri::State<'_, SocialDbState>,
    id: i64,
    title: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE social_sessions SET title = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![title, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn touch_social_session(
    state: tauri::State<'_, SocialDbState>,
    id: i64,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE social_sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_social_session(
    state: tauri::State<'_, SocialDbState>,
    id: i64,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    // Cascading delete handles messages
    conn.execute("DELETE FROM social_sessions WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
