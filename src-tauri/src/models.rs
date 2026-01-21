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
    /// ✨ 【核心修复】：数据库 ID 在传输层统一使用 String
    /// 这样可以完美对接前端 Pinia Store，并规避 JS 处理大整数的精度问题
    pub id: String, 
    
    /// 会话标题，默认为“新对话”
    pub title: String,
    
    /// 记录用户上次翻到的滚动位置（单位：像素）
    pub last_scroll_pos: i32,
    
    /// 最后活跃时间，用于侧边栏按时间倒序排列
    pub updated_at: String,
}