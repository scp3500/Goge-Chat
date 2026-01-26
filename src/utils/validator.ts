// 验证工具函数

/**
 * 验证电子邮件格式
 * @param email 电子邮件地址
 */
export function isValidEmail(email: string): boolean {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
}

/**
 * 验证URL格式
 * @param url URL地址
 */
export function isValidUrl(url: string): boolean {
    try {
        new URL(url);
        return true;
    } catch {
        return false;
    }
}

/**
 * 验证API密钥格式（基本检查）
 * @param apiKey API密钥
 */
export function isValidApiKey(apiKey: string): boolean {
    return apiKey.trim().length > 10;
}

/**
 * 验证非空字符串
 * @param value 字符串值
 */
export function isNonEmptyString(value: string): boolean {
    return typeof value === 'string' && value.trim().length > 0;
}

/**
 * 验证数字范围
 * @param value 数值
 * @param min 最小值
 * @param max 最大值
 */
export function isNumberInRange(value: number, min: number, max: number): boolean {
    return typeof value === 'number' && !isNaN(value) && value >= min && value <= max;
}

/**
 * 验证对象是否包含必需字段
 * @param obj 要验证的对象
 * @param requiredFields 必需字段数组
 */
export function hasRequiredFields(obj: Record<string, any>, requiredFields: string[]): boolean {
    if (!obj || typeof obj !== 'object') return false;
    
    for (const field of requiredFields) {
        if (!(field in obj) || obj[field] === undefined || obj[field] === null) {
            return false;
        }
    }
    return true;
}

/**
 * 验证会话ID格式
 * @param sessionId 会话ID
 */
export function isValidSessionId(sessionId: string): boolean {
    return typeof sessionId === 'string' && sessionId.length === 8 && /^[a-zA-Z0-9]+$/.test(sessionId);
}

/**
 * 验证消息内容
 * @param content 消息内容
 */
export function isValidMessageContent(content: string): boolean {
    return typeof content === 'string' && content.trim().length > 0 && content.length <= 10000;
}

/**
 * 深度思考内容验证
 * @param reasoningContent 深度思考内容
 */
export function isValidReasoningContent(reasoningContent: string | null | undefined): boolean {
    if (reasoningContent === null || reasoningContent === undefined) return true;
    return typeof reasoningContent === 'string' && reasoningContent.length <= 50000;
}