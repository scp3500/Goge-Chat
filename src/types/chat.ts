// 聊天会话类型
export interface ChatSession {
    id: string;
    title: string;
    last_scroll_pos?: number;
    sort_order?: number;
    folder_id?: string | null;
    preset_id?: string | null;
    model_id?: string | null;
    system_prompt?: string | null;
}

// 文件夹类型
export interface Folder {
    id: string;
    name: string;
    sort_order: number;
    is_collapsed: boolean;
}

// 消息类型
export interface ChatMessage {
    id?: number;
    model?: string;
    role: 'user' | 'assistant' | 'system';
    content: string;
    reasoningContent?: string | null;
    fileMetadata?: string | null;
}

// AI 提供者类型
export interface AIProvider {
    id: string;
    name: string;
    icon: string;
    status: 'on' | 'off';
}

// 发送给 AI 的消息格式
export interface AIMessage {
    role: string;
    content: string;
    reasoningContent?: string | null;
    fileMetadata?: string | null;
}