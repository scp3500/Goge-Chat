// src/composables/useSettings.ts
import { computed } from 'vue';
import { useConfigStore } from '../stores/config';
import { useSettingsStore } from '../stores/settings';

/**
 * 设置面板通用逻辑复用
 * 提供统一的设置相关状态和方法访问
 */
export function useSettings() {
    const configStore = useConfigStore();
    const settingsStore = useSettingsStore();

    /** 当前分类的标题 */
    const currentCategoryTitle = computed(() => {
        const titles: Record<string, string> = {
            models: '模型配置',
            appearance: '外观设置',
            advanced: '高级设置',
            about: '关于'
        };
        return titles[settingsStore.activeCategory] || '设置';
    });

    /** 当前分类的描述 */
    const currentCategoryDescription = computed(() => {
        const descriptions: Record<string, string> = {
            models: '配置AI模型提供商和相关参数',
            appearance: '自定义界面外观和显示效果',
            advanced: '高级功能和实验性特性',
            about: '应用信息和许可证'
        };
        return descriptions[settingsStore.activeCategory] || '';
    });

    /** 是否正在保存配置 */
    const isSaving = computed(() => configStore.isLoading);

    /** 配置错误信息 */
    const configError = computed(() => configStore.lastError);

    /**
     * 打开设置到指定分类
     */
    const openCategory = (category: 'models' | 'appearance' | 'advanced' | 'about') => {
        settingsStore.openSettings(category);
    };

    return {
        // Stores
        configStore,
        settingsStore,

        // 计算属性
        currentCategoryTitle,
        currentCategoryDescription,
        isSaving,
        configError,

        // 方法
        openCategory
    };
}
