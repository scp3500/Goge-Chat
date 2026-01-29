/// <reference path="../../vite-env.d.ts" />
// 导入所有供应商图标（作为原始字符串）
import deepseekIconRaw from './deepseek.svg?raw';
import openaiIconRaw from './openai.svg?raw';
import anthropicIconRaw from './anthropic.svg?raw';
import qwenIconRaw from './qwen.svg?raw';
import ollamaIconRaw from './ollama.svg?raw';
import geminiIconRaw from './gemini.svg?raw';

// 默认图标（用于自定义供应商）
const DEFAULT_ICON = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>';

// 导出图标映射
export const PROVIDER_ICONS = {
    deepseek: deepseekIconRaw,
    google: geminiIconRaw,
    openai: openaiIconRaw,
    anthropic: anthropicIconRaw,
    qwen: qwenIconRaw,
    ollama: ollamaIconRaw,
    gemini: geminiIconRaw,
    default: DEFAULT_ICON, // 默认图标
} as const;

export type ProviderIconKey = keyof typeof PROVIDER_ICONS;

// 获取供应商图标，如果是 URL 字符串则直接返回，否则从映射中获取
export function getProviderIcon(icon: string | ProviderIconKey): string {
    // 如果是完整的 URL 或 data URI，直接返回
    if (icon.startsWith('http') || icon.startsWith('data:') || icon.startsWith('/')) {
        return icon;
    }
    // 否则从 PROVIDER_ICONS 中查找，找不到就用默认图标
    return PROVIDER_ICONS[icon as ProviderIconKey] || PROVIDER_ICONS.default;
}
