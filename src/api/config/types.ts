export interface AppSettings {
    fontSize: number;
    lineRatio: number;
    themeColor: string;
    scrollbarWidth: number;
    apiKey: string;
}

// 提供一组默认值，防止空指针
export const DEFAULT_SETTINGS: AppSettings = {
    fontSize: 16,
    lineRatio: 1.7,
    themeColor: '#202124',
    scrollbarWidth: 12,
    apiKey: ''
};