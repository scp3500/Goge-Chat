use serde::{Deserialize, Serialize};

/// 消息条目结构：支持流式输出和历史记录渲染
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
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
pub struct Session {
    /// 数据库自增 ID
    pub id: i64,
    /// 会话标题，默认为“新对话”
    pub title: String,
    /// 【新增】记录用户上次翻到的滚动位置（单位：像素）
    /// 使用 i32 足够存储数百万条对话的高度，且在前端 DOM 对应 scrollTop
    pub last_scroll_pos: i32,
    /// 【建议新增】最后活跃时间，用于侧边栏按时间倒序排列
    pub updated_at: String,
}