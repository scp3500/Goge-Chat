/**
 * 默认系统提示词
 * 当用户未设置提示词时，将自动使用此提示词。
 */
export const DEFAULT_SYSTEM_PROMPT = "你是一个简洁专业的 AI 助手。请提供准确、客观、且有见地的回答。";

/**
 * 预制提示词库
 */
export interface PrebuiltPrompt {
    id: string;
    name: string;
    icon: string;
    content: string;
    description: string;
}

import prebuiltPromptsData from '../assets/prompts/prompts-list';

export const PREBUILT_PROMPTS: PrebuiltPrompt[] = prebuiltPromptsData;
