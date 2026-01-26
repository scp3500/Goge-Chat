// Tauri 命令参数和返回类型

// 会话相关命令
export interface CreateSessionParams {
    title: string;
}

export interface RenameSessionParams {
    id: string;
    title: string;
}

export interface DeleteSessionParams {
    sessionId: string;
}

export interface UpdateSessionScrollParams {
    id: string;
    pos: number;
}

// 文件夹相关命令
export interface CreateFolderParams {
    name: string;
}

export interface RenameFolderParams {
    id: string;
    name: string;
}

export interface DeleteFolderParams {
    id: string;
}

export interface UpdateFolderCollapsedParams {
    id: string;
    collapsed: boolean;
}

export interface MoveSessionToFolderParams {
    sessionId: string;
    folderId: string | null;
}

// 消息相关命令
export interface SaveMessageParams {
    sessionId: string;
    role: string;
    content: string;
    reasoningContent?: string | null;
}

export interface GetMessagesParams {
    sessionId: string;
}

// AI 相关命令
export interface AskAIParams {
    msg: Array<{
        role: string;
        content: string;
        reasoningContent?: string | null;
    }>;
    onEvent: any; // Channel 类型
}

export interface GenerateTitleParams {
    msg: Array<{
        role: string;
        content: string;
        reasoningContent?: string | null;
    }>;
}

// 排序相关命令
export interface UpdateSessionsOrderParams {
    orders: [string, number][];
}

export interface UpdateFoldersOrderParams {
    orders: [string, number][];
}