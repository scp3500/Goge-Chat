use serde::{Deserialize, Serialize};

/// 沉浸式模式核心配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImmersiveSettings {
    pub enabled: bool,
    pub behaviors: BehaviorToggles,
}

/// 行为开关和参数配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BehaviorToggles {
    /// 回复延迟范围 (min_ms, max_ms)
    #[serde(rename = "replyDelay")]
    pub reply_delay: Option<(u32, u32)>,

    /// 最大消息拆分段数
    #[serde(rename = "multiSegment")]
    pub multi_segment: Option<u8>,

    /// 段间延迟系数 (相对于主延迟的百分比, 0.0-1.0)
    #[serde(rename = "segmentDelayFactor")]
    pub segment_delay_factor: f32,

    /// 已读不回概率 [0.0-1.0]
    #[serde(rename = "ignoreRate")]
    pub ignore_rate: f32,

    /// 已读不回延迟配置
    #[serde(rename = "idleDelayConfig")]
    pub idle_delay_config: Option<IdleDelayConfig>,

    /// 撤回修正配置
    #[serde(rename = "typoCorrection")]
    pub typo_correction: Option<TypoConfig>,

    /// 主动发言配置
    #[serde(rename = "proactiveInitiation")]
    pub proactive_initiation: Option<ProactiveConfig>,

    /// 角色状态追踪配置
    #[serde(rename = "characterStateConfig")]
    pub character_state_config: Option<CharacterStateConfig>,

    /// 打字状态抖动
    #[serde(rename = "typingJitter")]
    pub typing_jitter: bool,
}

/// 撤回修正行为配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TypoConfig {
    /// 触发概率 [0.0-1.0]
    #[serde(rename = "triggerRate")]
    pub trigger_rate: f32,

    /// 撤回后重发延迟 (毫秒)
    #[serde(rename = "fixDelayMs")]
    pub fix_delay_ms: u32,
}

/// 主动发言配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProactiveConfig {
    /// 空闲阈值 (分钟)
    #[serde(rename = "idleThresholdMin")]
    pub idle_threshold_min: u32,
    /// 成功概率 [0.0-1.0]
    #[serde(rename = "successRate")]
    pub success_rate: f32,
    /// 冷却时长 (分钟)
    #[serde(rename = "cooldownMin")]
    pub cooldown_min: u32,
}

impl Default for ProactiveConfig {
    fn default() -> Self {
        Self {
            idle_threshold_min: 10,
            success_rate: 0.3,
            cooldown_min: 30,
        }
    }
}

/// 已读不回延迟配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct IdleDelayConfig {
    /// 延迟范围 (min_ms, max_ms)
    #[serde(rename = "delayRangeMs")]
    pub delay_range_ms: (u32, u32),

    /// 延迟后回复的概率 [0.0-1.0]
    #[serde(rename = "replyAfterDelayRate")]
    pub reply_after_delay_rate: f32,
}

/// 角色状态追踪配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterStateConfig {
    /// 启用状态追踪
    pub enabled: bool,
    /// 状态分析频率 (每N条消息触发一次)
    #[serde(rename = "analysisFrequency")]
    pub analysis_frequency: u32,
    /// 状态缓存时长 (分钟)
    #[serde(rename = "cacheDurationMin")]
    pub cache_duration_min: u32,
    /// 是否在主动消息触发时执行状态分析
    #[serde(rename = "analysisOnProactive")]
    pub analysis_on_proactive: bool,
    /// 心情对延迟的影响权重 [0.0-1.0]
    #[serde(rename = "moodWeight")]
    pub mood_weight: f32,
    /// 忙碌对已读不回的影响权重 [0.0-1.0]
    #[serde(rename = "busyWeight")]
    pub busy_weight: f32,
}

/// 行为原子 - 可组合的基础行为单元
#[derive(Debug, Clone)]
pub enum BehaviorAction {
    /// 等待指定毫秒数 (模拟"正在输入")
    Wait(u32),

    /// 发送文本消息
    Speak(String),

    /// 撤回指定消息 ID (0 表示最后一条)
    Retract(i64),

    /// 无动作 (已读不回)
    Idle,

    /// 延迟后重新决策 (延迟毫秒数, 原始消息内容)
    DelayedDecision(u32, String),
}

impl Default for ImmersiveSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            behaviors: BehaviorToggles::default(),
        }
    }
}

impl Default for BehaviorToggles {
    fn default() -> Self {
        Self {
            reply_delay: Some((800, 3000)),
            multi_segment: Some(3),
            segment_delay_factor: 0.3,
            ignore_rate: 0.0,
            idle_delay_config: Some(IdleDelayConfig {
                delay_range_ms: (60000, 300000), // 1-5分钟
                reply_after_delay_rate: 0.5,
            }),
            typo_correction: Some(TypoConfig {
                trigger_rate: 0.02,
                fix_delay_ms: 1500,
            }),
            proactive_initiation: Some(ProactiveConfig {
                idle_threshold_min: 10,
                success_rate: 0.3,
                cooldown_min: 30,
            }),
            character_state_config: Some(CharacterStateConfig {
                enabled: false, // 默认关闭,避免意外API调用
                analysis_frequency: 20,
                cache_duration_min: 30,
                analysis_on_proactive: true,
                mood_weight: 0.3,
                busy_weight: 0.5,
            }),
            typing_jitter: true,
        }
    }
}
