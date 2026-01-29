use serde::{Deserialize, Serialize};

/// 消息条目结构：完全兼容蛇形和驼峰
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub role: String,
    pub content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "reasoningContent")] // ← 输出用驼峰
    #[serde(alias = "reasoning_content")] // ← 输入兼容蛇形
    #[serde(alias = "reasoningContent")] // ← 输入兼容驼峰（明确声明）
    pub reasoning_content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fileMetadata")]
    #[serde(alias = "file_metadata")]
    pub file_metadata: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "searchMetadata")]
    #[serde(alias = "search_metadata")]
    pub search_metadata: Option<String>,
}

/// AI 请求封装
#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "max_tokens")]
    pub max_tokens: Option<u32>,
}

/// 会话元数据：完全兼容蛇形和驼峰
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    #[serde(alias = "id")]
    pub id: String,

    #[serde(alias = "title")]
    pub title: String,

    #[serde(rename = "last_scroll_pos")] // ← 输出用蛇形
    #[serde(alias = "lastScrollPos")] // ← 输入兼容驼峰
    #[serde(alias = "last_scroll_pos")] // ← 输入兼容蛇形（明确声明）
    pub last_scroll_pos: i32,

    #[serde(rename = "sort_order")]
    #[serde(alias = "sortOrder")]
    #[serde(alias = "sort_order")]
    pub sort_order: i32,

    #[serde(rename = "updated_at")]
    #[serde(alias = "updatedAt")]
    #[serde(alias = "updated_at")]
    pub updated_at: String,

    #[serde(rename = "folder_id")]
    #[serde(alias = "folderId")]
    #[serde(alias = "folder_id")]
    pub folder_id: Option<String>,
}

/// 文件夹结构：完全兼容蛇形和驼峰
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Folder {
    #[serde(alias = "id")]
    pub id: String,

    #[serde(alias = "name")]
    pub name: String,

    #[serde(rename = "sort_order")] // ← 输出用蛇形
    #[serde(alias = "sortOrder")] // ← 输入兼容驼峰
    #[serde(alias = "sort_order")] // ← 输入兼容蛇形（明确声明）
    pub sort_order: i32,

    #[serde(rename = "is_collapsed")]
    #[serde(alias = "isCollapsed")]
    #[serde(alias = "is_collapsed")]
    pub is_collapsed: bool,
}
