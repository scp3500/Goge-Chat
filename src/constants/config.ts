// 配置常量

/**
 * 应用默认配置
 */
export const DEFAULT_CONFIG = {
    // 外观配置
    APPEARANCE: {
        fontSize: 16,
        lineRatio: 1.7,
        themeColor: '#1E1F20',
        scrollbarWidth: 12,
        themeMode: 'dark' as 'light' | 'dark' | 'auto',
        language: 'zh-CN',
        animationEnabled: true,
        reduceMotion: false,
    },
    
    // 聊天配置
    CHAT: {
        useReasoning: false,
        autoScroll: true,
        showTimestamps: true,
        markdownEnabled: true,
        codeHighlighting: true,
        messageBubbles: true,
        enterToSend: true,
        shiftEnterToNewline: true,
        maxMessageLength: 10000,
        maxHistoryLength: 1000,
    },
    
    // 模型配置
    MODELS: {
        defaultProvider: 'deepseek',
        defaultModel: 'deepseek-chat',
        temperature: 0.7,
        maxTokens: 2048,
        topP: 0.9,
        frequencyPenalty: 0,
        presencePenalty: 0,
        streamEnabled: true,
    },
    
    // 提供商配置
    PROVIDERS: {
        deepseek: {
            enabled: true,
            apiKey: '',
            baseUrl: 'https://api.deepseek.com',
            models: ['deepseek-chat', 'deepseek-coder'],
        },
        openai: {
            enabled: false,
            apiKey: '',
            baseUrl: 'https://api.openai.com/v1',
            models: ['gpt-4', 'gpt-3.5-turbo'],
        },
        claude: {
            enabled: false,
            apiKey: '',
            baseUrl: 'https://api.anthropic.com',
            models: ['claude-3-opus', 'claude-3-sonnet'],
        },
        gemini: {
            enabled: false,
            apiKey: '',
            baseUrl: 'https://generativelanguage.googleapis.com',
            models: ['gemini-pro', 'gemini-ultra'],
        },
    },
    
    // 高级配置
    ADVANCED: {
        autoSave: true,
        autoSaveInterval: 30000, // 30秒
        autoBackup: false,
        autoBackupInterval: 86400000, // 24小时
        logLevel: 'info' as 'debug' | 'info' | 'warn' | 'error',
        enableAnalytics: false,
        enableCrashReporting: false,
        enableTelemetry: false,
    },
    
    // 存储配置
    STORAGE: {
        maxStorageSize: 1024 * 1024 * 100, // 100MB
        compressionEnabled: true,
        encryptionEnabled: false,
        backupLocation: '',
    },
} as const;

/**
 * 主题颜色
 */
export const THEME_COLORS = [
    { id: 'dark-gray', name: '深灰', color: '#1E1F20' },
    { id: 'charcoal', name: '炭黑', color: '#1b1b1f' },
    { id: 'midnight', name: '午夜', color: '#0f0f10' },
    { id: 'slate', name: '石板', color: '#2c2c32' },
    { id: 'navy', name: '海军蓝', color: '#1a365d' },
    { id: 'forest', name: '森林绿', color: '#1a472a' },
    { id: 'burgundy', name: '勃艮第红', color: '#5c2d42' },
    { id: 'purple', name: '深紫', color: '#4c1d95' },
] as const;

/**
 * 字体大小选项
 */
export const FONT_SIZES = [
    { value: 12, label: '极小' },
    { value: 14, label: '小' },
    { value: 16, label: '中' },
    { value: 18, label: '大' },
    { value: 20, label: '极大' },
] as const;

/**
 * 行高比例选项
 */
export const LINE_HEIGHT_RATIOS = [
    { value: 1.2, label: '紧凑' },
    { value: 1.5, label: '标准' },
    { value: 1.7, label: '宽松' },
    { value: 2.0, label: '非常宽松' },
] as const;

/**
 * 滚动条宽度选项
 */
export const SCROLLBAR_WIDTHS = [
    { value: 8, label: '细' },
    { value: 12, label: '中' },
    { value: 16, label: '粗' },
] as const;

/**
 * 温度选项
 */
export const TEMPERATURE_OPTIONS = [
    { value: 0.1, label: '严谨 (0.1)' },
    { value: 0.3, label: '保守 (0.3)' },
    { value: 0.7, label: '平衡 (0.7)' },
    { value: 1.0, label: '创意 (1.0)' },
    { value: 1.5, label: '非常创意 (1.5)' },
] as const;

/**
 * 最大令牌数选项
 */
export const MAX_TOKENS_OPTIONS = [
    { value: 512, label: '短 (512)' },
    { value: 1024, label: '中 (1024)' },
    { value: 2048, label: '长 (2048)' },
    { value: 4096, label: '非常长 (4096)' },
    { value: 8192, label: '超长 (8192)' },
] as const;

/**
 * 语言选项
 */
export const LANGUAGE_OPTIONS = [
    { value: 'zh-CN', label: '简体中文' },
    { value: 'zh-TW', label: '繁体中文' },
    { value: 'en-US', label: 'English' },
    { value: 'ja-JP', label: '日本語' },
    { value: 'ko-KR', label: '한국어' },
] as const;

/**
 * 日志级别选项
 */
export const LOG_LEVEL_OPTIONS = [
    { value: 'debug', label: '调试' },
    { value: 'info', label: '信息' },
    { value: 'warn', label: '警告' },
    { value: 'error', label: '错误' },
] as const;

/**
 * 自动保存间隔选项（毫秒）
 */
export const AUTO_SAVE_INTERVALS = [
    { value: 10000, label: '10秒' },
    { value: 30000, label: '30秒' },
    { value: 60000, label: '1分钟' },
    { value: 300000, label: '5分钟' },
    { value: 600000, label: '10分钟' },
] as const;

/**
 * 自动备份间隔选项（毫秒）
 */
export const AUTO_BACKUP_INTERVALS = [
    { value: 3600000, label: '1小时' },
    { value: 86400000, label: '24小时' },
    { value: 604800000, label: '7天' },
    { value: 2592000000, label: '30天' },
] as const;

/**
 * 配置键名常量
 */
export const CONFIG_KEYS = {
    // 存储键名
    STORAGE_PREFIX: 'golechat:',
    SETTINGS_KEY: 'golechat:settings',
    SESSIONS_KEY: 'golechat:sessions',
    FOLDERS_KEY: 'golechat:folders',
    MESSAGES_KEY: 'golechat:messages',
    
    // 本地存储键名
    LOCAL_SETTINGS: 'app_settings',
    LOCAL_API_KEY: 'api_key',
    LOCAL_THEME: 'app_theme',
    LOCAL_LANGUAGE: 'app_language',
    
    // Cookie 键名
    COOKIE_SESSION: 'session_id',
    COOKIE_USER: 'user_id',
    
    // 索引键名
    INDEX_SESSIONS_BY_FOLDER: 'sessions_by_folder',
    INDEX_MESSAGES_BY_SESSION: 'messages_by_session',
    INDEX_FOLDERS_BY_ORDER: 'folders_by_order',
} as const;