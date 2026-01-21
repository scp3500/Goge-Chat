// src/api/config/index.ts
import { invoke } from '@tauri-apps/api/core';
// 确保这里的路径与上面的文件名完全一致
import { AppSettings } from './types';

export const configApi = {
    /**
     * 从 Rust 加载配置
     */
    async load(): Promise<AppSettings | null> {
        try {
            // 这里的 'load_config' 必须对应 Rust 端的 #[tauri::command]
            return await invoke<AppSettings>('load_config');
        } catch (e) {
            console.error("API Error [load_config]:", e);
            return null;
        }
    },

    /**
     * 保存配置到 Rust
     */
    async save(config: AppSettings): Promise<void> {
        try {
            await invoke('save_config', { config });
        } catch (e) {
            console.error("API Error [save_config]:", e);
            throw e; // 抛出错误让 Store 处理
        }
    }
};