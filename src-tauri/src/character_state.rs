use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

/// 角色状态数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    pub contact_id: i64,
    pub session_id: i64,
    /// 心情状态: "happy", "neutral", "busy", "tired", "annoyed"
    pub mood: String,
    /// 忙碌程度 [0.0-1.0]
    pub busy_level: f32,
    /// 对话兴趣度 [0.0-1.0]
    pub interest_level: f32,
    /// 消息计数器(用于频率控制)
    pub message_count: u32,
    /// 上次分析时间
    pub last_analyzed: String,
}

impl Default for CharacterState {
    fn default() -> Self {
        Self {
            contact_id: 0,
            session_id: 0,
            mood: "neutral".to_string(),
            busy_level: 0.5,
            interest_level: 0.5,
            message_count: 0,
            last_analyzed: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// LLM分析返回的状态结构
#[derive(Debug, Deserialize)]
pub struct StateAnalysisResult {
    pub mood: String,
    pub busy_level: f32,
    pub interest_level: f32,
    #[allow(dead_code)]
    pub reasoning: Option<String>,
}

/// 状态分析器
pub struct StateAnalyzer {
    // 将来会添加LLM客户端引用
}

impl StateAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    /// 检查是否应该触发状态分析
    /// 返回 true 表示应该分析，false 表示跳过
    pub async fn should_analyze(
        &self,
        contact_id: i64,
        session_id: i64,
        config: &crate::immersive_settings::CharacterStateConfig,
        db_state: &crate::social_db::SocialDbState,
    ) -> Result<bool, String> {
        // 1. 检查缓存是否有效
        let cache_valid = {
            let conn = db_state.0.lock().map_err(|e| e.to_string())?;

            let result = conn
                .query_row(
                    "SELECT 1 FROM character_states 
                     WHERE contact_id = ?1 AND session_id = ?2
                     AND datetime(last_analyzed, '+' || ?3 || ' minutes') > datetime('now')",
                    rusqlite::params![contact_id, session_id, config.cache_duration_min],
                    |_| Ok(true),
                )
                .optional()
                .map_err(|e| e.to_string())?
                .unwrap_or(false);

            result
        }; // conn 在这里被drop

        if cache_valid {
            println!("📦 状态缓存有效，跳过分析");
            return Ok(false);
        }

        // 2. 检查消息计数
        let message_count = {
            let conn = db_state.0.lock().map_err(|e| e.to_string())?;

            let count = conn
                .query_row(
                    "SELECT message_count FROM character_states 
                     WHERE contact_id = ?1 AND session_id = ?2",
                    rusqlite::params![contact_id, session_id],
                    |row| row.get::<_, u32>(0),
                )
                .optional()
                .map_err(|e| e.to_string())?
                .unwrap_or(0);

            count
        }; // conn 在这里被drop

        if message_count < config.analysis_frequency {
            println!(
                "📊 消息计数 {} < 分析频率 {}, 跳过分析",
                message_count, config.analysis_frequency
            );
            return Ok(false);
        }

        println!("✅ 触发状态分析 (消息计数: {})", message_count);
        Ok(true)
    }

    /// 分析角色状态
    /// TODO: 实现真正的LLM调用逻辑
    pub async fn analyze_state(
        &self,
        contact_id: i64,
        session_id: i64,
        recent_messages: Vec<(String, String)>, // (role, content)
    ) -> Result<StateAnalysisResult, String> {
        // 1. 加载 state_analysis.txt 提示词模板
        let _prompt = match Self::load_prompt_template("state_analysis.txt") {
            Ok(p) => p,
            Err(e) => {
                println!("⚠️ 无法加载状态分析模板: {}, 使用默认逻辑", e);
                "".to_string()
            }
        };

        // TODO: 2. 构建对话历史字符串
        // TODO: 3. 调用 LLM API
        // TODO: 4. 解析 JSON 响应

        // 占位实现: 基于消息数量和内容做简单推断
        let total_messages = recent_messages.len();

        let mood = if total_messages > 10 {
            "busy".to_string()
        } else if total_messages > 5 {
            "neutral".to_string()
        } else {
            "happy".to_string()
        };

        let busy_level = (total_messages as f32 / 20.0).min(1.0);
        let interest_level = 0.5; // 默认中等兴趣

        // 使用 CharacterState 结构体记录 (消除 never constructed 警告)
        let _state = CharacterState {
            contact_id,
            session_id,
            mood: mood.clone(),
            busy_level,
            interest_level,
            message_count: total_messages as u32,
            last_analyzed: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        println!(
            "🧠 [Contact {}] 状态分析完成: mood={}, busy={:.2}, interest={:.2}",
            contact_id, mood, busy_level, interest_level
        );

        Ok(StateAnalysisResult {
            mood,
            busy_level,
            interest_level,
            reasoning: Some(format!("基于{}条消息的简单推断", total_messages)),
        })
    }

    /// 加载提示词模板
    pub fn load_prompt_template(template_name: &str) -> Result<String, String> {
        // 在开发环境中, 从 src-tauri 所在的层级查找 assets/prompts
        // 在打包环境中, 这部分需要适配资源路径
        use std::fs;
        let paths = vec![
            format!("../src/assets/prompts/{}", template_name),
            format!("src/assets/prompts/{}", template_name),
            format!("./assets/prompts/{}", template_name),
        ];

        for path in paths {
            if let Ok(content) = fs::read_to_string(&path) {
                return Ok(content);
            }
        }

        Err(format!("模板 {} 在已知路径中未找到", template_name))
    }
}

impl Default for StateAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
