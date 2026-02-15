use crate::immersive_settings::{BehaviorAction, ImmersiveSettings};
use rand::Rng;

/// 会话上下文信息
#[derive(Clone, Debug)]
pub struct SessionContext {
    pub session_id: i64,
    pub contact_id: i64,
    /// AI 当前情绪状态 (可选,用于调整延迟系数)
    pub mood: Option<String>,
    /// 忙碌程度 [0.0-1.0]
    pub busy_level: Option<f32>,
    /// 对话兴趣度 [0.0-1.0]
    pub interest_level: Option<f32>,
}

/// 行为决策引擎
pub struct BehaviorEngine {
    settings: ImmersiveSettings,
}

impl BehaviorEngine {
    pub fn new(settings: ImmersiveSettings) -> Self {
        Self { settings }
    }

    /// 主决策方法:根据消息内容和上下文生成行为链
    pub fn decide(&self, message: &str, context: &SessionContext) -> Vec<BehaviorAction> {
        // 🔧 修复:即使沉浸式模式未启用或行为模拟关闭,也要返回基本的 Speak 动作
        // 这样可以保证一问一答的基本功能
        if !self.settings.enabled {
            // 沉浸式模式未启用,直接发送(无延迟,无拆分)
            println!("[行为] 沉浸模式未启用, 直接说话");
            return vec![BehaviorAction::Speak(message.to_string())];
        }

        println!(
            "🧠 [Session {} | Contact {}] 开始决策: {}",
            context.session_id, context.contact_id, message
        );

        // 1. 检查是否忽略 (已读不回或延迟决策)
        if let Some(action) = self.should_ignore(message, context) {
            return vec![action];
        }

        // 2. 检查是否触发撤回修正
        if self.should_trigger_typo() {
            return self.build_typo_correction_chain(message);
        }

        // 3. 计算延迟 (考虑角色状态)
        let delay = self.calculate_delay(message, context);

        // 4. 拆分消息
        let segments = self.segment_message(message);

        // 5. 构建普通行为链
        self.build_normal_chain(delay, segments)
    }

    /// 延迟决策后重新评估
    /// 在 DelayedDecision 完成后调用,基于角色状态决定是否回复
    pub fn decide_after_delay(
        &self,
        message: &str,
        context: &SessionContext,
    ) -> Vec<BehaviorAction> {
        println!(
            "🧠 [Session {} | Contact {}] 延迟决策重新评估: {}",
            context.session_id, context.contact_id, message
        );

        // 获取延迟后回复概率
        let reply_rate = self
            .settings
            .behaviors
            .idle_delay_config
            .as_ref()
            .map(|c| c.reply_after_delay_rate)
            .unwrap_or(0.5);

        // 根据兴趣度调整回复概率
        let mut adjusted_rate = reply_rate;
        if let Some(interest_level) = context.interest_level {
            // 兴趣度越高,回复概率越高
            adjusted_rate = (reply_rate + interest_level * 0.2).min(1.0);
            println!(
                "🤔 延迟决策: 兴趣度调整回复率 {:.2} -> {:.2}",
                reply_rate, adjusted_rate
            );
        }

        // 根据忙碌程度降低回复概率
        if let Some(busy_level) = context.busy_level {
            // 降低忙碌度的负面影响，确保即便忙碌也有较高概率回复
            adjusted_rate = (adjusted_rate - busy_level * 0.15).max(0.1);
            println!("🤔 延迟决策: 忙碌度调整回复率 {:.2}", adjusted_rate);
        }

        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < adjusted_rate {
            println!("✅ 延迟后决定回复");
            // 决定回复,生成正常行为链
            let delay = self.calculate_delay(message, context);
            let segments = self.segment_message(message);
            self.build_normal_chain(delay, segments)
        } else {
            println!("❌ 延迟后决定不回复");
            // 决定不回复
            vec![BehaviorAction::Idle]
        }
    }

    /// 判断是否应该忽略此消息 (已读不回)
    /// 返回 Some(DelayedDecision) 表示延迟后再决策
    fn should_ignore(&self, message: &str, context: &SessionContext) -> Option<BehaviorAction> {
        let mut ignore_rate = self.settings.behaviors.ignore_rate;

        // 根据兴趣度动态调整忽略率
        if let Some(interest_level) = context.interest_level {
            // 兴趣度越高,忽略率越低
            // 降低兴趣度对忽略率的影响幅度
            let interest_factor = 2.0 - (interest_level * 1.2);
            ignore_rate = (ignore_rate * interest_factor).min(1.0);
            println!(
                "📊 兴趣度调整忽略率: {:.2} -> {:.2} (interest: {:.2})",
                self.settings.behaviors.ignore_rate, ignore_rate, interest_level
            );
        }

        // 根据忙碌程度增加忽略率
        if let Some(busy_level) = context.busy_level {
            // 进一步降低忙碌度导致的忽略概率
            // busy_level: 1.0 -> +10% (最高只增加 10% 的不回概率)
            let busy_contribution = (busy_level * 0.1).min(0.1);
            ignore_rate = (ignore_rate + busy_contribution).min(1.0);
            println!(
                "📊 忙碌度调整忽略率: {:.2} (busy: {:.2})",
                ignore_rate, busy_level
            );
        }

        if ignore_rate <= 0.0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < ignore_rate {
            // 检查是否配置了延迟决策
            if let Some(ref idle_config) = self.settings.behaviors.idle_delay_config {
                let delay =
                    rng.gen_range(idle_config.delay_range_ms.0..=idle_config.delay_range_ms.1);
                return Some(BehaviorAction::DelayedDecision(delay, message.to_string()));
            } else {
                // 没有配置延迟,直接忽略
                return Some(BehaviorAction::Idle);
            }
        }
        None
    }

    /// 判断是否触发撤回修正行为
    fn should_trigger_typo(&self) -> bool {
        if let Some(ref typo_config) = self.settings.behaviors.typo_correction {
            let mut rng = rand::thread_rng();
            return rng.gen::<f32>() < typo_config.trigger_rate;
        }
        false
    }

    /// 计算回复延迟 (毫秒)
    /// 基于消息长度、配置的延迟范围和角色状态
    fn calculate_delay(&self, message: &str, context: &SessionContext) -> u32 {
        let (min, max) = self.settings.behaviors.reply_delay.unwrap_or((500, 2000));

        // 基础延迟:每字符约 15ms (模拟打字速度)
        let mut base = (message.chars().count() as f32 * 15.0) as u32;

        // 根据心情调整延迟
        if let Some(ref mood) = context.mood {
            let mood_factor = match mood.as_str() {
                "happy" => 0.8,   // 开心时回复快一点
                "neutral" => 1.0, // 中性正常速度
                "busy" => 1.5,    // 忙碌时回复慢一点
                "tired" => 1.8,   // 疲惫时回复更慢
                "annoyed" => 1.3, // 烦躁时稍慢
                _ => 1.0,
            };
            base = (base as f32 * mood_factor) as u32;
            println!("😊 心情调整延迟: mood={}, factor={:.1}x", mood, mood_factor);
        }

        // 根据忙碌程度增加延迟
        if let Some(busy_level) = context.busy_level {
            // busy_level: 0.0 -> 1.0x
            // busy_level: 0.5 -> 1.5x
            // busy_level: 1.0 -> 2.0x
            let busy_factor = 1.0 + busy_level;
            base = (base as f32 * busy_factor) as u32;
            println!(
                "⏰ 忙碌度调整延迟: busy={:.2}, factor={:.1}x",
                busy_level, busy_factor
            );
        }

        // 5. 增加随机抖动 (Jitter) ±20%
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0.8..1.2);
        base = (base as f32 * jitter) as u32;

        // 6. 限制在配置范围内
        base.clamp(min, max)
    }

    /// 智能拆分消息
    /// 按自然断点(句号、换行符等)拆分, 增加随机性以模拟人类习惯
    fn segment_message(&self, message: &str) -> Vec<String> {
        let max_segments = self.settings.behaviors.multi_segment.unwrap_or(1) as usize;

        if max_segments <= 1 {
            return vec![message.to_string()];
        }

        let mut rng = rand::thread_rng();
        // 1. 随机化本次回复的最大段数 (1 ~ max_segments)
        let actual_max = rng.gen_range(1..=max_segments);

        if actual_max <= 1 {
            return vec![message.to_string()];
        }

        let mut segments = Vec::new();
        let mut current = String::new();

        for ch in message.chars() {
            current.push(ch);

            // 定义拆分符号
            let is_delimiter = ch == '\n'
                || ch == '。'
                || ch == '.'
                || ch == '!'
                || ch == '?'
                || ch == '！'
                || ch == '？';

            if is_delimiter && segments.len() < actual_max - 1 {
                // 2. 优化：不再是逢标点就拆，而是看长度
                // 只有当前累积的片段足够长时，才考虑拆分
                let current_len = current.chars().count();

                // 动态生成拆分阈值 (默认 40 ~ 100)，增加不可预测性
                let (min_t, max_t) = self
                    .settings
                    .behaviors
                    .segmentation_threshold_range
                    .unwrap_or((40, 100));
                let split_threshold = rng.gen_range(min_t..=max_t) as usize;

                if current_len >= split_threshold {
                    // 3. 概率降低到 30%，即使长句子也可能不拆
                    if rng.gen::<f32>() < 0.3 {
                        if !current.trim().is_empty() {
                            segments.push(current.trim().to_string());
                            current.clear();
                        }
                    }
                }
            }
        }

        // 添加剩余部分
        if !current.trim().is_empty() {
            segments.push(current.trim().to_string());
        }

        if segments.is_empty() {
            vec![message.to_string()]
        } else {
            segments
        }
    }

    /// 构建普通行为链 (延迟 + 分段发送)
    fn build_normal_chain(&self, delay: u32, segments: Vec<String>) -> Vec<BehaviorAction> {
        let mut chain = Vec::new();

        // 添加初始延迟
        chain.push(BehaviorAction::Wait(delay));

        // 获取段间延迟系数范围
        let (min_f, max_f) = self.settings.behaviors.segment_delay_factor;

        // 发送每个分段
        let mut rng = rand::thread_rng();
        for (i, segment) in segments.iter().enumerate() {
            chain.push(BehaviorAction::Speak(segment.clone()));

            // 分段之间添加短暂延迟 (除了最后一个)
            if i < segments.len() - 1 {
                // 随机生成段间延迟系数
                let factor = rng.gen_range(min_f..max_f);
                let segment_delay = (delay as f32 * factor) as u32;
                chain.push(BehaviorAction::Wait(segment_delay));
            }
        }

        chain
    }

    /// 构建撤回修正行为链
    /// 流程: 发送错误版本 -> 等待 -> 撤回 -> 等待 -> 发送修正版本
    fn build_typo_correction_chain(&self, message: &str) -> Vec<BehaviorAction> {
        let typo_version = self.introduce_typo(message);
        let fix_delay = self
            .settings
            .behaviors
            .typo_correction
            .as_ref()
            .map(|c| c.fix_delay_ms)
            .unwrap_or(1500);

        vec![
            BehaviorAction::Wait(800), // 初始延迟
            BehaviorAction::Speak(typo_version),
            BehaviorAction::Wait(fix_delay), // 等待后发现"错误"
            BehaviorAction::Retract(0),      // 撤回最后一条消息 (0 表示最后一条)
            BehaviorAction::Wait(500),       // 短暂延迟
            BehaviorAction::Speak(message.to_string()), // 发送修正版本
        ]
    }

    /// 引入"错别字"或小错误
    /// 简单实现:随机替换一个字符或添加重复字符
    fn introduce_typo(&self, message: &str) -> String {
        if message.len() < 3 {
            return message.to_string();
        }

        let mut rng = rand::thread_rng();
        let chars: Vec<char> = message.chars().collect();
        let mut typo_chars = chars.clone();

        // 随机选择一个位置
        let pos = rng.gen_range(1..chars.len() - 1);

        // 50% 概率重复字符, 50% 概率替换为相似字符
        if rng.gen_bool(0.5) {
            // 重复字符
            typo_chars.insert(pos, chars[pos]);
        } else {
            // 简单替换 (这里可以扩展为更智能的相似字符映射)
            let similar_chars = ['的', '地', '得', '在', '再', '做', '作'];
            if let Some(&similar) = similar_chars.get(rng.gen_range(0..similar_chars.len())) {
                typo_chars[pos] = similar;
            }
        }

        typo_chars.into_iter().collect()
    }

    /// 获取动态主动发言参数
    /// 返回 (空闲阈值秒, 成功率, 冷却秒)
    pub fn get_proactive_parameters(&self, context: &SessionContext) -> (u32, f32, u32) {
        let default_config = crate::immersive_settings::ProactiveConfig::default();
        let config = self
            .settings
            .behaviors
            .proactive_initiation
            .as_ref()
            .unwrap_or(&default_config);

        let (threshold_min, threshold_max) = config.idle_threshold_range;
        let mut success_rate = config.success_rate;
        let (cooldown_min, cooldown_max) = config.cooldown_range;

        // Covert to f32 for calculation
        let mut t_min = threshold_min as f32;
        let mut t_max = threshold_max as f32;
        let c_min = cooldown_min as f32;
        let mut c_max = cooldown_max as f32;

        // 1. 兴趣度影响 (兴趣度高 -> 阈值减小, 成功率提高)
        if let Some(interest) = context.interest_level {
            // 阈值调整: 0.5->1.0x, 1.0->0.5x, 0.0->1.5x
            // 减缓对阈值的调整幅度
            let threshold_factor = 1.25 - (interest * 0.5);
            t_min *= threshold_factor;
            t_max *= threshold_factor;

            // 成功率调整: 0.5->+0, 1.0->+0.2, 0.0->-0.2
            success_rate = (success_rate + (interest - 0.5) * 0.4).clamp(0.0, 1.0);
        } else {
            // 默认情况下如果没有interest数据，稍微提升一点成功率，避免太冷淡
            success_rate = (success_rate + 0.1).min(1.0);
        }

        // 2. 忙碌度影响 (忙碌度高 -> 阈值增加, 成功率降低)
        if let Some(busy) = context.busy_level {
            // 阈值调整: 0.0->1.0x, 1.0->1.5x
            let busy_factor = 1.0 + (busy * 0.5);
            t_min *= busy_factor;
            t_max *= busy_factor;

            // 成功率调整: 0.0->-0, 1.0->-0.2
            success_rate = (success_rate - busy * 0.2).clamp(0.0, 1.0);
        }

        // 3. 心情影响
        if let Some(ref mood) = context.mood {
            match mood.as_str() {
                "happy" => {
                    t_min *= 0.8;
                    t_max *= 0.8;
                    success_rate = (success_rate + 0.1).min(1.0);
                }
                "annoyed" | "tired" => {
                    t_min *= 1.2;
                    t_max *= 1.2;
                    success_rate = (success_rate - 0.1).max(0.0);
                }
                "busy" => {
                    t_min *= 1.2;
                    t_max *= 1.2;
                    success_rate = (success_rate - 0.1).max(0.0);
                }
                _ => {}
            }
        }

        // 4. 在最终范围内随机选择
        let mut rng = rand::thread_rng();
        // 确保 min <= max
        if t_min > t_max {
            t_max = t_min;
        }
        if c_min > c_max {
            c_max = c_min;
        }

        let final_threshold = rng.gen_range(t_min..=t_max);
        let final_cooldown = rng.gen_range(c_min..=c_max);

        println!(
            "🎲 主动发言计算: 阈值范围 {:.0}-{:.0}s -> {:.0}s, 冷却范围 {:.0}-{:.0}s -> {:.0}s",
            t_min, t_max, final_threshold, c_min, c_max, final_cooldown
        );

        (
            final_threshold.max(10.0) as u32,
            success_rate,
            final_cooldown as u32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_delay() {
        let settings = ImmersiveSettings::default();
        let engine = BehaviorEngine::new(settings);
        let context = SessionContext {
            session_id: 1,
            contact_id: 1,
            mood: None,
            busy_level: None,
            interest_level: None,
        };

        let chain = engine.decide("你好", &context);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_message_segmentation() {
        let mut settings = ImmersiveSettings::default();
        settings.behaviors.multi_segment = Some(3);
        let engine = BehaviorEngine::new(settings);

        let segments = engine.segment_message("第一句。第二句。第三句。");
        assert!(segments.len() <= 3);
    }
}
