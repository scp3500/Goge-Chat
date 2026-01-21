// src/api/chat.ts
import { invoke } from '@tauri-apps/api/core';

// 建议简单定义一下 Session 类型，解决 unshift 报错
export interface ChatSession {
    id: string;
    title: string;
    last_scroll_pos?: number;
}

export const chatApi = {
    /** 获取所有会话 */
    getSessions: () => invoke<ChatSession[]>('get_sessions'),

    /** 创建新会话 */
    createSession: (title: string) => invoke<string>('create_session', { title }),

    /** 删除会话 */
    deleteSession: (id: string) => invoke('delete_session', { id }),

    /** 重命名 */
    renameSession: (id: string, title: string) => invoke('update_session_title', { id, title }),
};