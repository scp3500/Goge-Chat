// 常量模块导出
export * from './icons';

/**
 * 应用信息常量
 */
export const APP_INFO = {
    name: 'GoleChat',
    version: '0.1.0',
    description: '一个简洁高效的 AI 聊天客户端',
    author: 'GoleChat Team',
    repository: 'https://github.com/scp3500/golechat',
    license: 'MIT',
    copyright: `© ${new Date().getFullYear()} GoleChat Team`,
} as const;

/**
 * 性能常量
 */
export const PERFORMANCE = {
    // 性能限制
    MAX_SESSIONS: 1000,
    MAX_MESSAGES_PER_SESSION: 10000,
    MAX_FOLDERS: 100,
    MAX_MESSAGE_LENGTH: 100000,

    // 缓存设置
    CACHE_TTL: 5 * 60 * 1000, // 5分钟
    DEBOUNCE_DELAY: 300, // 300毫秒
    THROTTLE_DELAY: 100, // 100毫秒

    // 加载超时
    LOAD_TIMEOUT: 10000, // 10秒
    REQUEST_TIMEOUT: 30000, // 30秒
    CONNECTION_TIMEOUT: 5000, // 5秒
} as const;