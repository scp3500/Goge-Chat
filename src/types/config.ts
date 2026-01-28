// 模型提供商配置
export interface ModelProviderConfig {
    id: string;
    name: string;
    icon: string;
    enabled: boolean;
    apiKey: string;
    baseUrl?: string;
    models: string[];
    defaultModel?: string;
    temperature?: number;
    maxTokens?: number;
    customParams?: Record<string, any>;
}

// 应用设置类型
export interface AppSettings {
    // 外观设置
    fontSize: number;
    lineRatio: number;
    themeColor: string;
    scrollbarWidth: number;

    // AI 模型设置
    providers: ModelProviderConfig[];
    defaultProviderId: string;
    selectedModelId: string;
    useReasoning: boolean;

    // 搜索设置
    searchInstanceUrl: string;
    defaultSearchProvider: string;

    // 兼容旧版本的 apiKey（将逐步废弃）
    apiKey?: string;
}

// 默认的模型提供商配置
export const DEFAULT_PROVIDERS: ModelProviderConfig[] = [
    {
        id: 'deepseek',
        name: 'DeepSeek',
        icon: '🐋',
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
        icon: '🤖',
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
        icon: '🦜',
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
        name: 'Gemini',
        icon: '💎',
        enabled: false,
        apiKey: '',
        baseUrl: 'https://generativelanguage.googleapis.com',
        models: ['gemini-pro', 'gemini-pro-vision'],
        defaultModel: 'gemini-pro',
        temperature: 0.7,
        maxTokens: 4096
    },
    {
        id: 'ollama',
        name: 'Ollama',
        icon: '🦙',
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
        icon: '🐑',
        enabled: false,
        apiKey: '',
        baseUrl: 'https://dashscope.aliyuncs.com/api',
        models: ['qwen-turbo', 'qwen-plus', 'qwen-max'],
        defaultModel: 'qwen-turbo',
        temperature: 0.7,
        maxTokens: 4096
    }
];

// 提供一组默认值，防止空指针
export const DEFAULT_SETTINGS: AppSettings = {
    fontSize: 16,
    lineRatio: 1.7,
    themeColor: '#1E1F20',
    scrollbarWidth: 12,
    providers: DEFAULT_PROVIDERS,
    defaultProviderId: 'deepseek',
    selectedModelId: 'deepseek-chat',
    useReasoning: false,
    searchInstanceUrl: 'https://searx.be',
    defaultSearchProvider: 'all',
    apiKey: '' // 兼容旧版本
};

// 设置分类
export type SettingsCategory = 'models' | 'appearance' | 'advanced' | 'about';

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