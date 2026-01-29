
import { type ChatSession } from '../../api/chat';

export interface Folder {
    id: string;
    name: string;
    sort_order: number;
    is_collapsed: boolean;
}

export interface Message {
    id?: number;
    role: string;
    content: string;
    reasoningContent?: string | null;
    fileMetadata?: string | null;
    searchMetadata?: string | null;
    searchStatus?: 'searching' | 'done' | 'error';
    searchQuery?: string;
}

export interface PausedChunks {
    content: string[];
    reasoning: string[];
}
