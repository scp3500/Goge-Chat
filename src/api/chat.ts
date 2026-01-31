// src/api/chat.ts
import { invoke } from '@tauri-apps/api/core';
import type { ChatSession } from '../types/chat';

// 重新导出 ChatSession 类型以保持向后兼容性
export type { ChatSession };

export const chatApi = {
    /** 获取所有会话 */
    getSessions: () => invoke<ChatSession[]>('get_sessions'),

    /** 创建新会话 */
    createSession: (title: string, presetId?: string, modelId?: string, systemPrompt?: string) =>
        invoke<string>('create_session', { title, presetId, modelId, systemPrompt }),

    /** 删除会话 */
    deleteSession: (id: string) => invoke('delete_session', { id }),

    /** 重命名 */
    renameSession: (id: string, title: string) => invoke('rename_session', { id, title }),

    /** 更新排序 */
    updateSessionsOrder: (orders: [string, number][]) => invoke('update_sessions_order', { orders }),

    /** 更新文件夹排序 */
    updateFoldersOrder: (orders: [string, number][]) => invoke('update_folders_order', { orders }),
};