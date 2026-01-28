// src/composables/useSettingsValidation.ts
import { ValidationResult, ValidationRule } from '../types/config';

/**
 * 设置验证逻辑
 * 提供各种配置项的验证规则和通用验证方法
 */
export function useSettingsValidation() {
    /**
     * 验证 API Key
     */
    const validateApiKey = (key: string): string | null => {
        if (!key || key.trim() === '') {
            return 'API Key 不能为空';
        }
        if (key.length < 10) {
            return 'API Key 长度不足（至少10个字符）';
        }
        if (!/^[a-zA-Z0-9_-]+$/.test(key)) {
            return 'API Key 包含无效字符';
        }
        return null;
    };

    /**
     * 验证 URL
     */
    const validateUrl = (url: string): string | null => {
        if (!url || url.trim() === '') {
            return 'URL 不能为空';
        }

        try {
            const parsed = new URL(url);

            if (!['http:', 'https:'].includes(parsed.protocol)) {
                return 'URL 必须使用 HTTP 或 HTTPS 协议';
            }

            return null;
        } catch {
            return '无效的 URL 格式';
        }
    };

    /**
     * 验证数字范围
     */
    const validateNumberRange = (
        value: number,
        min: number,
        max: number,
        fieldName: string = '数值'
    ): string | null => {
        if (isNaN(value)) {
            return `${fieldName} 必须是有效数字`;
        }
        if (value < min || value > max) {
            return `${fieldName} 必须在 ${min} 到 ${max} 之间`;
        }
        return null;
    };

    /**
     * 验证字体大小
     */
    const validateFontSize = (size: number): string | null => {
        return validateNumberRange(size, 10, 24, '字体大小');
    };

    /**
     * 验证行高比例
     */
    const validateLineRatio = (ratio: number): string | null => {
        return validateNumberRange(ratio, 1.0, 3.0, '行高比例');
    };

    /**
     * 验证滚动条宽度
     */
    const validateScrollbarWidth = (width: number): string | null => {
        return validateNumberRange(width, 4, 20, '滚动条宽度');
    };

    /**
     * 验证颜色值（支持 hex 格式）
     */
    const validateColor = (color: string): string | null => {
        if (!color || color.trim() === '') {
            return '颜色值不能为空';
        }

        // 验证 hex 格式
        if (!/^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$/.test(color)) {
            return '颜色必须是有效的十六进制格式（例如：#1E1F20）';
        }

        return null;
    };

    /**
     * 验证温度参数
     */
    const validateTemperature = (temp: number): string | null => {
        return validateNumberRange(temp, 0, 2, '温度参数');
    };

    /**
     * 验证最大 Token 数
     */
    const validateMaxTokens = (tokens: number): string | null => {
        if (tokens <= 0) {
            return '最大 Token 数必须大于 0';
        }
        if (tokens > 100000) {
            return '最大 Token 数不能超过 100000';
        }
        return null;
    };

    /**
     * 通用验证方法
     * 根据提供的规则验证整个设置对象
     */
    const validateSettings = (
        settings: Record<string, any>,
        rules: Record<string, ValidationRule>
    ): ValidationResult => {
        const errors: Record<string, string> = {};
        const warnings: Record<string, string> = {};

        for (const [field, rule] of Object.entries(rules)) {
            const value = settings[field];
            const error = rule.validate(value);

            if (error) {
                errors[field] = error;
            }
        }

        return {
            valid: Object.keys(errors).length === 0,
            errors,
            warnings
        };
    };

    /**
     * 批量验证多个字段
     */
    const validateFields = (
        fields: Array<{ name: string; value: any; validator: (value: any) => string | null }>
    ): ValidationResult => {
        const errors: Record<string, string> = {};

        for (const field of fields) {
            const error = field.validator(field.value);
            if (error) {
                errors[field.name] = error;
            }
        }

        return {
            valid: Object.keys(errors).length === 0,
            errors
        };
    };

    return {
        // 单个字段验证
        validateApiKey,
        validateUrl,
        validateNumberRange,
        validateFontSize,
        validateLineRatio,
        validateScrollbarWidth,
        validateColor,
        validateTemperature,
        validateMaxTokens,

        // 批量验证
        validateSettings,
        validateFields
    };
}
