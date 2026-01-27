// Tauri 命令封装
import { invoke, Channel } from '@tauri-apps/api/core';
import type {
    ChatSession,
    Folder,
    ChatMessage
} from '../types/chat';
import type {
    SaveMessageParams,
    AskAIParams,
    GenerateTitleParams
} from '../types/tauri';

/**
 * 会话相关命令
 */
export const sessionCommands = {
    /** 获取所有会话 */
    getSessions: () => invoke<ChatSession[]>('get_sessions'),

    /** 创建新会话 */
    createSession: (title: string) => invoke<string>('create_session', { title }),

    /** 删除会话 */
    deleteSession: (sessionId: string) => invoke<void>('delete_session', { sessionId }),

    /** 重命名会话 */
    renameSession: (id: string, title: string) => invoke<void>('rename_session', { id, title }),

    /** 更新会话滚动位置 */
    updateSessionScroll: (id: string, pos: number) => invoke<void>('update_session_scroll', { id, pos }),

    /** 更新会话排序 */
    updateSessionsOrder: (orders: [string, number][]) => invoke<void>('update_sessions_order', { orders }),
};

/**
 * 文件夹相关命令
 */
export const folderCommands = {
    /** 获取所有文件夹 */
    getFolders: () => invoke<Folder[]>('get_folders'),

    /** 创建文件夹 */
    createFolder: (name: string) => invoke<string>('create_folder', { name }),

    /** 删除文件夹 */
    deleteFolder: (id: string) => invoke<void>('delete_folder', { id }),

    /** 重命名文件夹 */
    renameFolder: (id: string, name: string) => invoke<void>('rename_folder', { id, name }),

    /** 更新文件夹折叠状态 */
    updateFolderCollapsed: (id: string, collapsed: boolean) => invoke<void>('update_folder_collapsed', { id, collapsed }),

    /** 移动会话到文件夹 */
    moveSessionToFolder: (sessionId: string, folderId: string | null) => invoke<void>('move_session_to_folder', { sessionId, folderId }),

    /** 更新文件夹排序 */
    updateFoldersOrder: (orders: [string, number][]) => invoke<void>('update_folders_order', { orders }),
};

/**
 * 消息相关命令
 */
export const messageCommands = {
    /** 获取会话消息 */
    getMessages: (sessionId: string) => invoke<ChatMessage[]>('get_messages', { sessionId }),

    /** 保存消息 */
    saveMessage: (params: SaveMessageParams) => invoke<void>('save_message', params as unknown as Record<string, unknown>),

    /** 保存助手回复(带深度思考和搜索元数据) */
    saveAssistantResponse: (sessionId: string, content: string, reasoningContent: string | null, fileMetadata: string | null = null, searchMetadata: string | null = null) =>
        invoke<number>('save_message', {
            sessionId,
            role: 'assistant',
            content,
            reasoningContent,
            fileMetadata,
            searchMetadata
        }),
};

/**
 * AI 相关命令
 */
export const aiCommands = {
    /** 向 AI 提问 */
    askAI: (msg: AskAIParams['msg'], onEvent: Channel<string>) =>
        invoke<void>('ask_ai', { msg, onEvent }),

    /** 停止 AI 生成 */
    stopAIGeneration: () => invoke<void>('stop_ai_generation'),

    /** 重置 AI 生成状态 */
    resetAIGeneration: () => invoke<void>('reset_ai_generation'),

    /** 生成标题 */
    generateTitle: (msg: GenerateTitleParams['msg']) => invoke<string>('generate_title', { msg }),
};

/**
 * 配置相关命令
 */
export const configCommands = {
    /** 保存配置 */
    saveConfig: (config: any) => invoke<void>('save_config', { config }),

    /** 加载配置 */
    loadConfig: () => invoke<any>('load_config'),
};

/**
 * 数据库相关命令
 */
export const dbCommands = {
    /** 初始化数据库 */
    initDb: () => invoke<void>('init_db'),

    /** 备份数据库 */
    backupDb: (path: string) => invoke<void>('backup_db', { path }),

    /** 恢复数据库 */
    restoreDb: (path: string) => invoke<void>('restore_db', { path }),
};

/**
 * 所有命令的聚合对象
 */
export const tauriCommands = {
    ...sessionCommands,
    ...folderCommands,
    ...messageCommands,
    ...aiCommands,
    ...configCommands,
    ...dbCommands,
};

/**
 * 创建事件通道
 */
export function createEventChannel<T = string>(): Channel<T> {
    return new Channel<T>();
}

/**
 * 通用调用包装器，提供错误处理
 */
export async function safeInvoke<T>(command: string, args?: any): Promise<T | null> {
    try {
        return await invoke<T>(command, args);
    } catch (error) {
        console.error(`Tauri command "${command}" failed:`, error);
        return null;
    }
}