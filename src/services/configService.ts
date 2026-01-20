import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const useConfigService = defineStore('config', () => {
    // 1. 响应式配置状态
    const settings = ref({
        fontSize: 16,
        lineRatio: 1.7,
        themeColor: '#202124',
        scrollbarWidth: 12,
        apiKey: ''
    });

    /**
     * 2. 将状态应用到全局 CSS 变量
     * 这样只要 settings 一变，全站的字号、行高、背景色瞬间同步
     */
    const applyToCss = (val: any) => {
        const root = document.documentElement;
        root.style.setProperty('--font-size-base', `${val.fontSize}px`);
        root.style.setProperty('--font-ratio', val.lineRatio.toString());
        root.style.setProperty('--bg-main', val.themeColor);
        root.style.setProperty('--scrollbar-width', `${val.scrollbarWidth}px`);
    };

    /**
     * 3. 初始化：从 Rust 端加载持久化的配置
     */
    const init = async () => {
        try {
            const saved = await invoke('load_config') as any;
            if (saved) {
                settings.value = { ...settings.value, ...saved };
                applyToCss(settings.value);
            }
        } catch (e) {
            console.error("加载配置失败:", e);
        }
    };

    /**
     * 4. 更新配置：保存到内存并同步给 Rust 存入硬盘
     */
    const updateConfig = async (newVal: any) => {
        settings.value = { ...settings.value, ...newVal };
        applyToCss(settings.value);

        // 异步推送到 Rust 端保存到 config.json，不会触发 HMR 重启
        try {
            await invoke('save_config', { config: settings.value });
        } catch (e) {
            console.error("持久化配置失败:", e);
        }
    };

    return { settings, init, updateConfig };
});