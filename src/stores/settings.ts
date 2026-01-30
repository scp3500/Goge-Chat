// src/stores/settings.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { SettingsCategory } from '../types/config';

/**
 * 设置界面 UI 状态管理
 * 负责管理设置面板的显示状态、当前选中的分类和提供商等
 */
export const useSettingsStore = defineStore('settings', () => {
    // ========== 状态 ==========

    const savedCategory = localStorage.getItem('activeCategory') as SettingsCategory | null;
    const activeCategory = ref<SettingsCategory>(savedCategory || 'models');

    const activeProviderId = ref<string>(localStorage.getItem('activeProviderId') || 'deepseek');
    const activePresetId = ref<string>(localStorage.getItem('activePresetId') || 'default_preset');


    /** 设置面板是否打开 */
    const isModalOpen = ref(false);

    /** 是否有未保存的更改 */
    const hasUnsavedChanges = ref(false);

    /** 搜索关键词（用于设置项搜索） */
    const searchQuery = ref('');

    // ========== 方法 ==========

    /**
     * 打开设置界面
     * @param category 可选，指定打开的分类
     */
    const openSettings = (category?: SettingsCategory) => {
        if (category) {
            activeCategory.value = category;
            localStorage.setItem('activeCategory', category);
        }
        isModalOpen.value = true;
    };

    /**
     * 关闭设置界面
     * 清理未保存更改标志
     */
    const closeSettings = () => {
        isModalOpen.value = false;
        hasUnsavedChanges.value = false;
        searchQuery.value = '';
    };

    /**
     * 切换设置分类
     */
    const setCategory = (category: SettingsCategory) => {
        activeCategory.value = category;
        localStorage.setItem('activeCategory', category);
    };

    /**
     * 设置当前选中的提供商
     */
    const setActiveProvider = (providerId: string) => {
        activeProviderId.value = providerId;
        localStorage.setItem('activeProviderId', providerId);
    };

    /**
     * 设置当前选中的预设
     */
    const setActivePreset = (presetId: string) => {
        activePresetId.value = presetId;
        localStorage.setItem('activePresetId', presetId);
    };

    const markAsChanged = () => {
        hasUnsavedChanges.value = true;
    };

    /**
     * 清除未保存更改标志
     */
    const clearChangedFlag = () => {
        hasUnsavedChanges.value = false;
    };

    return {
        // 状态
        activeCategory,
        activeProviderId,
        activePresetId,
        isModalOpen,
        hasUnsavedChanges,
        searchQuery,

        // 方法
        openSettings,
        closeSettings,
        setCategory,
        setActiveProvider,
        setActivePreset,
        markAsChanged,
        clearChangedFlag
    };

});
