// Tauri 事件封装
import { listen, emit, type Event, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * 事件名称枚举
 */
export enum AppEvents {
    // 应用事件
    APP_READY = 'app:ready',
    APP_ERROR = 'app:error',
    
    // 聊天事件
    CHAT_SESSION_CREATED = 'chat:session-created',
    CHAT_SESSION_DELETED = 'chat:session-deleted',
    CHAT_SESSION_UPDATED = 'chat:session-updated',
    
    // 消息事件
    MESSAGE_RECEIVED = 'message:received',
    MESSAGE_SENT = 'message:sent',
    
    // AI 事件
    AI_GENERATION_STARTED = 'ai:generation-started',
    AI_GENERATION_STOPPED = 'ai:generation-stopped',
    AI_GENERATION_COMPLETED = 'ai:generation-completed',
    
    // 配置事件
    CONFIG_UPDATED = 'config:updated',
    
    // 数据库事件
    DB_BACKUP_COMPLETED = 'db:backup-completed',
    DB_RESTORE_COMPLETED = 'db:restore-completed',
}

/**
 * 事件监听器类型
 */
export type EventListener<T = any> = (event: Event<T>) => void;

/**
 * 事件系统封装
 */
export class EventSystem {
    private listeners: Map<string, UnlistenFn[]> = new Map();

    /**
     * 监听事件
     * @param event 事件名称
     * @param handler 事件处理函数
     */
    async on<T = any>(event: AppEvents | string, handler: EventListener<T>): Promise<void> {
        const unlisten = await listen<T>(event, handler);
        
        if (!this.listeners.has(event)) {
            this.listeners.set(event, []);
        }
        this.listeners.get(event)!.push(unlisten);
    }

    /**
     * 一次性监听事件
     * @param event 事件名称
     * @param handler 事件处理函数
     */
    async once<T = any>(event: AppEvents | string, handler: EventListener<T>): Promise<void> {
        const wrappedHandler: EventListener<T> = (ev) => {
            handler(ev);
            this.off(event, wrappedHandler);
        };
        await this.on(event, wrappedHandler);
    }

    /**
     * 取消监听事件
     * @param event 事件名称
     * @param handler 可选的事件处理函数，如果不提供则取消该事件的所有监听
     */
    off(event: AppEvents | string, handler?: EventListener): void {
        const unlisteners = this.listeners.get(event);
        if (!unlisteners) return;

        if (handler) {
            // 由于 Tauri 的 listen 返回的 UnlistenFn 不包含 handler 信息，
            // 我们无法精确匹配，所以这里简化处理：移除所有监听
            console.warn('精确移除事件监听器在 Tauri 中不支持，将移除该事件的所有监听');
        }

        // 移除所有监听
        unlisteners.forEach(unlisten => unlisten());
        this.listeners.delete(event);
    }

    /**
     * 发射事件
     * @param event 事件名称
     * @param payload 事件载荷
     */
    async emit<T = any>(event: AppEvents | string, payload?: T): Promise<void> {
        await emit(event, payload);
    }

    /**
     * 清除所有事件监听
     */
    clearAll(): void {
        this.listeners.forEach((unlisteners, event) => {
            unlisteners.forEach(unlisten => unlisten());
        });
        this.listeners.clear();
    }
}

/**
 * 全局事件系统实例
 */
export const eventSystem = new EventSystem();

/**
 * 快捷函数：监听事件
 */
export function onEvent<T = any>(event: AppEvents | string, handler: EventListener<T>): Promise<void> {
    return eventSystem.on(event, handler);
}

/**
 * 快捷函数：发射事件
 */
export function emitEvent<T = any>(event: AppEvents | string, payload?: T): Promise<void> {
    return eventSystem.emit(event, payload);
}

/**
 * 快捷函数：一次性监听
 */
export function onceEvent<T = any>(event: AppEvents | string, handler: EventListener<T>): Promise<void> {
    return eventSystem.once(event, handler);
}

/**
 * 快捷函数：取消监听
 */
export function offEvent(event: AppEvents | string, handler?: EventListener): void {
    return eventSystem.off(event, handler);
}