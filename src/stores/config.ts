// src/stores/config.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
// 注意这里：../ 代表跳出 stores 文件夹，进入 api/config
import { configApi } from '../api/config';
import { AppSettings, DEFAULT_SETTINGS } from '../types/config';

export const useConfigStore = defineStore('config', () => {
    const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });

    const applyToCss = (val: AppSettings) => {
        const root = document.documentElement;
        root.style.setProperty('--font-size-base', `${val.fontSize}px`);
        root.style.setProperty('--font-ratio', val.lineRatio.toString());
        root.style.setProperty('--bg-main', val.themeColor);
        root.style.setProperty('--scrollbar-width', `${val.scrollbarWidth}px`);
    };

    const init = async () => {
        try {
            const saved = await configApi.load();
            if (saved) {
                settings.value = { ...DEFAULT_SETTINGS, ...saved };
                applyToCss(settings.value);
            }
        } catch (e) {
            console.error("加载配置失败:", e);
        }
    };

    const updateConfig = async (newPartialSettings: Partial<AppSettings>) => {
        settings.value = { ...settings.value, ...newPartialSettings };
        applyToCss(settings.value);
        try {
            await configApi.save(settings.value);
        } catch (e) {
            console.error("持久化配置失败:", e);
        }
    };

    return { settings, init, updateConfig };
});