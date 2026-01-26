// 应用设置类型
export interface AppSettings {
    fontSize: number;
    lineRatio: number;
    themeColor: string;
    scrollbarWidth: number;
    apiKey: string;
    useReasoning: boolean;
}

// 提供一组默认值，防止空指针
export const DEFAULT_SETTINGS: AppSettings = {
    fontSize: 16,
    lineRatio: 1.7,
    themeColor: '#1E1F20',
    scrollbarWidth: 12,
    apiKey: '',
    useReasoning: false
};

// 配置类别
export type ConfigCategory = 'models' | 'appearance' | 'advanced' | 'about';

// 主题配置
export interface ThemeConfig {
    id: string;
    name: string;
    color: string;
    isDark: boolean;
}