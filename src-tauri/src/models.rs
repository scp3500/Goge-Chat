use serde::{Deserialize, Serialize};

/// 消息条目结构：支持流式输出和历史记录渲染
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    // DeepSeek 官方 API 暂无特殊的 reasoning 开关，通常通过模型名区分
    // 但我们可以预留扩展位
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

    /// 排序顺序
    pub sort_order: i32,
    
    /// 最后活跃时间，用于侧边栏按时间倒序排列
    pub updated_at: String,

    /// 归属文件夹 ID
    pub folder_id: Option<String>,
}

/// 文件夹结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub sort_order: i32,
    pub is_collapsed: bool,
}