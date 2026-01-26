use serde::{Deserialize, Serialize};

/// 消息条目结构：支持流式输出和历史记录渲染
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]  // ✅ 添加这一行
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

/// AI 请求封装：完美对接 OpenAI/DeepSeek 格式协议
#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
}

/// 会话元数据：项目的灵魂支柱
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]  // ✅ 添加这一行（保持一致性）
pub struct Session {
    pub id: String, 
    pub title: String,
    pub last_scroll_pos: i32,
    pub sort_order: i32,
    pub updated_at: String,
    pub folder_id: Option<String>,
}

/// 文件夹结构
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]  // ✅ 添加这一行（保持一致性）
pub struct Folder {
    pub id: String,
    pub name: String,
    pub sort_order: i32,
    pub is_collapsed: bool,
}