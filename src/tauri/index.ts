// Tauri 模块导出
export * from './commands';
export * from './events';

// 重新导出 Tauri API 核心功能
export { invoke, Channel, type InvokeArgs } from '@tauri-apps/api/core';
export { listen, emit, type Event, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * Tauri 工具函数
 */

/**
 * 检查是否在 Tauri 环境中运行
 */
export function isTauriEnvironment(): boolean {
    return typeof window !== 'undefined' && '__TAURI__' in window;
}

/**
 * 安全调用 Tauri 命令，提供回退方案
 * @param command 命令名称
 * @param args 命令参数
 * @param fallback 回退值（当不在 Tauri 环境中时返回）
 */
export async function safeTauriInvoke<T>(
    command: string,
    args?: any,
    fallback?: T
): Promise<T | null> {
    if (!isTauriEnvironment()) {
        console.warn(`Not in Tauri environment, command "${command}" will not be executed`);
        return fallback ?? null;
    }

    try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<T>(command, args);
    } catch (error) {
        console.error(`Tauri command "${command}" failed:`, error);
        return fallback ?? null;
    }
}

/**
 * 初始化 Tauri 模块
 */
export async function initTauri(): Promise<boolean> {
    if (!isTauriEnvironment()) {
        console.warn('Not in Tauri environment, skipping Tauri initialization');
        return false;
    }

    try {
        // 可以在这里添加初始化逻辑，比如检查后端连接等
        console.log('Tauri environment initialized');
        return true;
    } catch (error) {
        console.error('Failed to initialize Tauri:', error);
        return false;
    }
}