// src/composables/useProviderConfig.ts
import { computed, ref } from 'vue';
import { useConfigStore } from '../stores/config';
import { useSettingsStore } from '../stores/settings';
import { ModelProviderConfig } from '../types/config';

/**
 * 模型提供商配置专用逻辑
 * 处理提供商的选择、配置更新、启用/禁用等操作
 */
export function useProviderConfig() {
    const configStore = useConfigStore();
    const settingsStore = useSettingsStore();

    /** 当前选中的提供商配置 */
    const currentProvider = computed(() =>
        configStore.getProvider(settingsStore.activeProviderId)
    );

    /** 当前提供商名称 */
    const currentProviderName = computed(() =>
        currentProvider.value?.name || '未选择'
    );

    /** 所有提供商列表 */
    const allProviders = computed(() => configStore.settings.providers);

    /** 已启用的提供商列表 */
    const enabledProviders = computed(() => configStore.enabledProviders);

    /** 当前提供商是否已启用 */
    const isCurrentProviderEnabled = computed(() =>
        currentProvider.value?.enabled || false
    );

    /**
     * 切换提供商启用状态
     */
    const toggleProvider = async (providerId: string) => {
        try {
            await configStore.toggleProvider(providerId);
        } catch (error) {
            console.error('切换提供商状态失败:', error);
            throw error;
        }
    };

    /**
     * 更新当前提供商的配置
     */
    const updateCurrentProvider = async (config: Partial<ModelProviderConfig>) => {
        if (!currentProvider.value) {
            throw new Error('没有选中的提供商');
        }

        try {
            await configStore.updateProvider(currentProvider.value.id, config);
            settingsStore.markAsChanged();
        } catch (error) {
            console.error('更新提供商配置失败:', error);
            throw error;
        }
    };

    /**
     * 更新指定提供商的配置
     */
    const updateProvider = async (providerId: string, config: Partial<ModelProviderConfig>) => {
        try {
            await configStore.updateProvider(providerId, config);
            settingsStore.markAsChanged();
        } catch (error) {
            console.error('更新提供商配置失败:', error);
            throw error;
        }
    };

    /**
     * 选择提供商（切换当前选中）
     */
    const selectProvider = (providerId: string) => {
        settingsStore.setActiveProvider(providerId);
    };

    /**
     * 设置为默认提供商
     */
    const setAsDefault = async (providerId: string) => {
        try {
            await configStore.setDefaultProvider(providerId);
            settingsStore.markAsChanged();
        } catch (error) {
            console.error('设置默认提供商失败:', error);
            throw error;
        }
    };

    /**
     * 检查提供商是否为默认
     */
    const isDefaultProvider = (providerId: string) => {
        return configStore.settings.defaultProviderId === providerId;
    };

    /**
     * 验证提供商配置
     */
    const validateProviderConfig = (config: Partial<ModelProviderConfig>) => {
        const errors: string[] = [];

        if (config.apiKey !== undefined && config.apiKey.trim() === '') {
            errors.push('API Key 不能为空');
        }

        if (config.baseUrl !== undefined) {
            try {
                new URL(config.baseUrl);
            } catch {
                errors.push('Base URL 格式无效');
            }
        }

        if (config.temperature !== undefined && (config.temperature < 0 || config.temperature > 2)) {
            errors.push('Temperature 必须在 0-2 之间');
        }

        if (config.maxTokens !== undefined && config.maxTokens <= 0) {
            errors.push('Max Tokens 必须大于 0');
        }

        return {
            valid: errors.length === 0,
            errors
        };
    };

    return {
        // 计算属性
        currentProvider,
        currentProviderName,
        allProviders,
        enabledProviders,
        isCurrentProviderEnabled,

        // 方法
        toggleProvider,
        updateCurrentProvider,
        updateProvider,
        selectProvider,
        setAsDefault,
        isDefaultProvider,
        validateProviderConfig
    };
}
