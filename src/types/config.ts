// 模型功能特性
export interface ModelFeatures {
    vision?: boolean;     // 视觉识别
    web?: boolean;        // 联网搜索
    reasoning?: boolean;  // 深度思考/推理
    tools?: boolean;      // 工具调用/函数调用
    image?: boolean;      // 图像生成
}

// 具体模型配置
export interface ModelConfig {
    id: string;           // 模型标识（API 使用的 ID）
    name: string;         // 显示名称
    group?: string;       // 分组名称（如 "Gemini 2.5", "Gemini 3"）
    features?: ModelFeatures;
}

// 模型详细信息
export interface ModelInfo {
    id: string;      // 对应供应商的原始 id，如 'gemini-1.5-pro'
    name: string;    // 显示名称，如 'Gemini 1.5 Pro'
    group?: string;  // 分组名称，如 'Gemini 1.5'
    features?: ModelFeature[]; // 模型特性
}

// 模型特性类型
export type ModelFeature = 'vision' | 'search' | 'reasoning' | 'tools' | 'web';

// 模型提供商配置
export interface ModelProviderConfig {
    id: string;
    name: string;
    icon: string;
    enabled: boolean;
    apiKey: string;
    baseUrl?: string;
    models: (string | ModelInfo)[]; // 兼容旧版本的字符串数组
    defaultModel?: string;
    temperature?: number;
    maxTokens?: number;
    customParams?: Record<string, any>;
    disableUrlSuffix?: boolean;
    isCustom?: boolean;
    lastTestModelId?: string;
}

// 提示词库项
export interface PromptLibraryItem {
    id: string;
    name: string;
    icon: string;
    content: string;
    description: string;
}

// 模型预设配置
export interface ModelPreset {
    id: string;
    name: string;
    temperature: number;
    maxTokens: number;
    systemPrompt: string;
    isDefault?: boolean;
    // 高级配置可以放在这里
    topP?: number;
    presencePenalty?: number;
    frequencyPenalty?: number;
}


// 应用设置类型
export interface AppSettings {
    // 外观设置
    fontSize: number;
    lineRatio: number;
    themeColor: string;
    theme: 'light' | 'dark';
    darkThemeId: string; // 用户选择的深色主题 ID
    lightThemeId: string; // 用户选择的浅色主题 ID
    scrollbarWidth: number;

    // AI 模型设置
    providers: ModelProviderConfig[];
    defaultProviderId: string;
    selectedModelId: string;
    globalModelId: string;    // 新增：全局默认模型
    useReasoning: boolean;
    useSearch: boolean;
    searchProvider: string;

    // 预设设置
    presets: ModelPreset[];
    defaultPresetId: string;
    globalPresetId: string;   // 新增：全局默认预设

    // 预置提示词库
    promptLibrary: PromptLibraryItem[];
    defaultSystemPrompt: string;

    // 聊天体验设置
    enableStream: boolean;      // 是否开启流式传输
    enableBubble: boolean;      // 是否开启气泡模式

    // 用户头像设置
    showUserAvatar: boolean;    // 是否显示用户头像
    userAvatarPath: string;     // 用户头像本地路径

    // 搜索设置
    searchInstanceUrl: string;

    defaultSearchProvider: string;

    // 兼容旧版本的 apiKey（将逐步废弃）
    apiKey?: string;

    // Chat Mode Configuration
    chatMode: ChatModeConfig;

    // UI Density / Scaling
    globalScale: number;

    // Avatar Adjustments
    userAvatarSize: number;
    userAvatarBorderRadius: number;
    userAvatarOffsetX: number;
    userAvatarOffsetY: number;

    // 🅰️ Global Typography
    fontFamilyEnglish: string;
    fontFamilyChinese: string;
}

export interface ChatModeConfig {
    enabled: boolean;
    dayThemeId: string;
    nightThemeId: string;
    enableStream: boolean;      // Override global stream setting
    enableLoadingBar: boolean; // Show/Hide "Thinking..." or progress bar
    showSocialClock: boolean; // Show/Hide real-time clock in social sidebar
}

// 默认的模型提供商配置
export const DEFAULT_PROVIDERS: ModelProviderConfig[] = [
    {
        id: 'deepseek',
        name: 'DeepSeek',
        icon: 'deepseek',
        enabled: true,
        apiKey: '',
        baseUrl: 'https://api.deepseek.com',
        models: ['deepseek-chat', 'deepseek-reasoner'],
        defaultModel: 'deepseek-chat',
        temperature: 1.0,
        maxTokens: 8192
    },
    {
        id: 'openai',
        name: 'OpenAI',
        icon: 'openai',
        enabled: false,
        apiKey: '',
        baseUrl: 'https://api.openai.com/v1',
        models: ['gpt-4', 'gpt-4-turbo', 'gpt-3.5-turbo'],
        defaultModel: 'gpt-4-turbo',
        temperature: 0.7,
        maxTokens: 4096
    },
    {
        id: 'claude',
        name: 'Claude',
        icon: 'anthropic',
        enabled: false,
        apiKey: '',
        baseUrl: 'https://api.anthropic.com',
        models: ['claude-3-opus', 'claude-3-sonnet', 'claude-3-haiku'],
        defaultModel: 'claude-3-sonnet',
        temperature: 0.7,
        maxTokens: 4096
    },
    {
        id: 'gemini',
        name: 'Gemini 3',
        icon: 'google',
        enabled: false,
        apiKey: '',
        baseUrl: 'https://generativelanguage.googleapis.com',
        models: [
            'gemini-3-pro-preview',
            'gemini-3-flash-preview',
            'gemini-3-pro-image-preview'
        ],
        defaultModel: 'gemini-3-pro-preview',
        temperature: 0.7,
        maxTokens: 4096
    },
    {
        id: 'ollama',
        name: 'Ollama',
        icon: 'ollama',
        enabled: false,
        apiKey: '',
        baseUrl: 'http://localhost:11434',
        models: ['llama2', 'mistral', 'codellama'],
        defaultModel: 'llama2',
        temperature: 0.7,
        maxTokens: 2048
    },
    {
        id: 'qwen',
        name: 'Qwen',
        icon: 'qwen',
        enabled: false,
        apiKey: '',
        baseUrl: 'https://dashscope.aliyuncs.com/api',
        models: ['qwen-turbo', 'qwen-plus', 'qwen-max'],
        defaultModel: 'qwen-turbo',
        temperature: 0.7,
        maxTokens: 4096
    },
    {
        id: 'siliconflow',
        name: 'SiliconFlow',
        icon: 'siliconflow',
        enabled: false,
        apiKey: '',
        baseUrl: 'https://api.siliconflow.cn',
        models: [
            'deepseek-ai/DeepSeek-V3.2'
        ],
        defaultModel: 'deepseek-ai/DeepSeek-V3.2',
        temperature: 0.7,
        maxTokens: 4096
    }
];

// 提供一组默认值，防止空指针
export const DEFAULT_SETTINGS: AppSettings = {
    fontSize: 16,
    lineRatio: 1.7,
    themeColor: 'var(--theme-preset-1)',
    theme: 'dark',
    darkThemeId: 'dark', // 默认深色主题
    lightThemeId: 'light', // 默认浅色主题
    scrollbarWidth: 12,
    providers: DEFAULT_PROVIDERS,
    defaultProviderId: 'deepseek',
    selectedModelId: 'deepseek-chat',
    globalModelId: 'deepseek-chat',
    useReasoning: false,
    useSearch: false,
    searchProvider: 'all',
    searchInstanceUrl: 'https://searx.be',
    defaultSearchProvider: 'all',
    presets: [
        {
            id: 'default_preset',
            name: '默认配置',
            temperature: 0.7,
            maxTokens: 4096,
            systemPrompt: '',
            isDefault: true
        }
    ],
    defaultPresetId: 'default_preset',
    globalPresetId: 'default_preset',
    promptLibrary: [], // 将在 store init 时根据内置常量初始化
    defaultSystemPrompt: "你是一个简洁专业的 AI 助手。请提供准确、客观、且有见地的回答。",

    enableStream: true,
    enableBubble: false,
    showUserAvatar: false,
    userAvatarPath: "",

    apiKey: '', // 兼容旧版本
    chatMode: {
        enabled: false,
        dayThemeId: 'wechat',
        nightThemeId: 'dark_plus',
        enableStream: false,
        enableLoadingBar: false,
        showSocialClock: false
    },
    globalScale: 1.0,
    userAvatarSize: 36,
    userAvatarBorderRadius: 6,
    userAvatarOffsetX: 0,
    userAvatarOffsetY: 0,

    // 🅰️ Global Typography
    fontFamilyEnglish: "", // e.g. "Segoe UI", "Inter"
    fontFamilyChinese: "", // e.g. "Microsoft YaHei", "PingFang SC"
};


// 设置分类
export type SettingsCategory = 'profile' | 'models' | 'presets' | 'prompts' | 'appearance' | 'chatmode' | 'advanced' | 'about';

// 配置类别（向后兼容别名）
export type ConfigCategory = SettingsCategory;

// 主题配置
export interface ThemeConfig {
    id: string;
    name: string;
    color: string;
    isDark: boolean;
}

// 验证结果
export interface ValidationResult {
    valid: boolean;
    errors: Record<string, string>;
    warnings?: Record<string, string>;
}

// 设置验证规则
export interface ValidationRule<T = any> {
    validate: (value: T) => string | null;
    message?: string;
}

// 表单字段错误
export interface FieldError {
    field: string;
    message: string;
    type: 'error' | 'warning' | 'info';
}

// 消息错误元数据
export interface ErrorMetadata {
    message: string;
    type: 'timeout' | 'quota' | 'server' | 'network' | 'unknown';
    code?: string;
    details?: string;
}