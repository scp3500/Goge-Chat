// src/api/chat.ts
import { invoke } from '@tauri-apps/api/core';

// 建议简单定义一下 Session 类型，解决 unshift 报错
export interface ChatSession {
    id: string;
    title: string;
    last_scroll_pos?: number;
    sort_order?: number;
    folder_id?: string | null;
}

export const chatApi = {
    /** 获取所有会话 */
    getSessions: () => invoke<ChatSession[]>('get_sessions'),

    /** 创建新会话 */
    createSession: (title: string) => invoke<string>('create_session', { title }),

    /** 删除会话 */
    deleteSession: (id: string) => invoke('delete_session', { id }),

    /** 重命名 */
    renameSession: (id: string, title: string) => invoke('rename_session', { id, title }),

    /** 更新排序 */
    updateSessionsOrder: (orders: [string, number][]) => invoke('update_sessions_order', { orders }),

    /** 更新文件夹排序 */
    updateFoldersOrder: (orders: [string, number][]) => invoke('update_folders_order', { orders }),
};