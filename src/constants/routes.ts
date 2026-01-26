// 路由常量定义

/**
 * 应用路由路径
 */
export const ROUTES = {
    // 主页面
    HOME: '/',
    CHAT: '/chat',
    
    // 设置页面
    SETTINGS: '/settings',
    SETTINGS_MODELS: '/settings/models',
    SETTINGS_APPEARANCE: '/settings/appearance',
    SETTINGS_ADVANCED: '/settings/advanced',
    SETTINGS_ABOUT: '/settings/about',
    
    // 其他页面
    HISTORY: '/history',
    FOLDERS: '/folders',
    SEARCH: '/search',
    
    // 外部链接
    DOCUMENTATION: 'https://docs.golechat.com',
    GITHUB: 'https://github.com/yourusername/golechat',
    FEEDBACK: 'https://feedback.golechat.com',
} as const;

/**
 * 路由名称映射
 */
export const ROUTE_NAMES = {
    [ROUTES.HOME]: '首页',
    [ROUTES.CHAT]: '聊天',
    [ROUTES.SETTINGS]: '设置',
    [ROUTES.SETTINGS_MODELS]: '模型设置',
    [ROUTES.SETTINGS_APPEARANCE]: '外观设置',
    [ROUTES.SETTINGS_ADVANCED]: '高级设置',
    [ROUTES.SETTINGS_ABOUT]: '关于',
    [ROUTES.HISTORY]: '历史记录',
    [ROUTES.FOLDERS]: '文件夹',
    [ROUTES.SEARCH]: '搜索',
} as const;

/**
 * 路由元信息
 */
export interface RouteMeta {
    title: string;
    requiresAuth?: boolean;
    showInNav?: boolean;
    icon?: string;
}

/**
 * 路由配置
 */
export const ROUTE_CONFIG: Record<string, RouteMeta> = {
    [ROUTES.HOME]: {
        title: '首页',
        showInNav: true,
        icon: 'home',
    },
    [ROUTES.CHAT]: {
        title: '聊天',
        showInNav: true,
        icon: 'message',
    },
    [ROUTES.SETTINGS]: {
        title: '设置',
        showInNav: true,
        icon: 'settings',
    },
    [ROUTES.HISTORY]: {
        title: '历史记录',
        showInNav: true,
        icon: 'history',
    },
    [ROUTES.FOLDERS]: {
        title: '文件夹',
        showInNav: true,
        icon: 'folder',
    },
    [ROUTES.SEARCH]: {
        title: '搜索',
        showInNav: true,
        icon: 'search',
    },
};

/**
 * 获取路由标题
 */
export function getRouteTitle(path: string): string {
    return ROUTE_CONFIG[path]?.title || ROUTE_NAMES[path as keyof typeof ROUTE_NAMES] || '未知页面';
}

/**
 * 导航项列表
 */
export const NAV_ITEMS = Object.entries(ROUTE_CONFIG)
    .filter(([_, config]) => config.showInNav)
    .map(([path, config]) => ({
        path,
        title: config.title,
        icon: config.icon,
    }));