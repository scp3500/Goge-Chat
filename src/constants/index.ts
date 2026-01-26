// 常量模块导出
export * from './icons';
export * from './routes';
export * from './messages';
export * from './config';

/**
 * 应用信息常量
 */
export const APP_INFO = {
    name: 'GoleChat',
    version: '0.1.0',
    description: '一个简洁高效的 AI 聊天客户端',
    author: 'GoleChat Team',
    repository: 'https://github.com/yourusername/golechat',
    license: 'MIT',
    copyright: `© ${new Date().getFullYear()} GoleChat Team`,
} as const;

/**
 * 应用功能常量
 */
export const APP_FEATURES = {
    // 支持的功能
    SUPPORTED_FEATURES: [
        '多会话管理',
        '文件夹组织',
        '消息历史记录',
        'Markdown 渲染',
        '代码高亮',
        '主题切换',
        '多语言支持',
        '本地存储',
        '数据备份恢复',
        '快捷键支持',
    ] as const,
    
    // 计划中的功能
    PLANNED_FEATURES: [
        '插件系统',
        '云端同步',
        '团队协作',
        'API 市场',
        '自定义工作流',
        '语音输入',
        '图像识别',
    ] as const,
    
    // 技术栈
    TECH_STACK: {
        frontend: ['Vue 3', 'TypeScript', 'Pinia', 'Vite'],
        backend: ['Rust', 'Tauri', 'SQLite'],
        styling: ['CSS3', 'CSS Variables', 'Flexbox/Grid'],
        tools: ['ESLint', 'Prettier', 'Git'],
    } as const,
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

/**
 * 安全常量
 */
export const SECURITY = {
    // 加密设置
    ENCRYPTION_ALGORITHM: 'AES-GCM',
    KEY_DERIVATION_ITERATIONS: 100000,
    SALT_LENGTH: 16,
    IV_LENGTH: 12,
    
    // 验证设置
    MIN_PASSWORD_LENGTH: 8,
    MAX_PASSWORD_LENGTH: 128,
    SESSION_TIMEOUT: 30 * 60 * 1000, // 30分钟
    MAX_LOGIN_ATTEMPTS: 5,
    
    // 隐私设置
    DATA_RETENTION_DAYS: 90,
    AUTO_DELETE_INACTIVE_DAYS: 30,
    ANONYMIZE_DATA: true,
} as const;

/**
 * 快捷键常量
 */
export const SHORTCUTS = {
    // 全局快捷键
    NEW_CHAT: ['Ctrl', 'N'],
    NEW_WINDOW: ['Ctrl', 'Shift', 'N'],
    SEARCH: ['Ctrl', 'K'],
    SETTINGS: ['Ctrl', ','],
    
    // 聊天快捷键
    SEND_MESSAGE: ['Enter'],
    NEW_LINE: ['Shift', 'Enter'],
    STOP_GENERATION: ['Ctrl', '.'],
    CLEAR_INPUT: ['Escape'],
    
    // 编辑快捷键
    COPY: ['Ctrl', 'C'],
    PASTE: ['Ctrl', 'V'],
    CUT: ['Ctrl', 'X'],
    UNDO: ['Ctrl', 'Z'],
    REDO: ['Ctrl', 'Y'],
    
    // 导航快捷键
    PREVIOUS_SESSION: ['Ctrl', 'ArrowUp'],
    NEXT_SESSION: ['Ctrl', 'ArrowDown'],
    GO_HOME: ['Ctrl', 'H'],
    GO_BACK: ['Alt', 'ArrowLeft'],
    GO_FORWARD: ['Alt', 'ArrowRight'],
    
    // 视图快捷键
    TOGGLE_SIDEBAR: ['Ctrl', 'B'],
    TOGGLE_FULLSCREEN: ['F11'],
    ZOOM_IN: ['Ctrl', '+'],
    ZOOM_OUT: ['Ctrl', '-'],
    RESET_ZOOM: ['Ctrl', '0'],
} as const;