// src/stores/config.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { configCommands } from '../tauri/commands';
import { AppSettings, DEFAULT_SETTINGS, ModelProviderConfig } from '../types/config';

export const useConfigStore = defineStore('config', () => {
    // ========== 状态 ==========
    const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });
    const isLoading = ref(false);
    const lastError = ref<string | null>(null);

    // ========== 计算属性 ==========

    /** 已启用的提供商列表 */
    const enabledProviders = computed(() =>
        settings.value.providers.filter(p => p.enabled)
    );

    /** 当前选中的默认提供商 */
    const currentProvider = computed(() =>
        settings.value.providers.find(p => p.id === settings.value.defaultProviderId)
    );

    /** 获取指定提供商配置 */
    const getProvider = (id: string) =>
        settings.value.providers.find(p => p.id === id);

    // ========== CSS 应用 ==========

    /**
     * 将配置应用到全局 CSS 变量
     * 实现视觉样式的实时响应
     */
    const applyToCss = (val: AppSettings) => {
        const root = document.documentElement;
        root.style.setProperty('--font-size-base', `${val.fontSize}px`);
        root.style.setProperty('--font-ratio', val.lineRatio.toString());
        root.style.setProperty('--bg-main', val.themeColor);
        root.style.setProperty('--scrollbar-width', `${val.scrollbarWidth}px`);
    };

    // ========== 初始化 ==========

    /**
     * 从 Tauri 后端加载持久化配置
     * 合并默认值，处理版本迁移
     */
    const init = async () => {
        isLoading.value = true;
        lastError.value = null;

        try {
            const saved = await configCommands.loadConfig();

            if (saved) {
                // 合并配置，确保新增字段有默认值
                settings.value = {
                    ...DEFAULT_SETTINGS,
                    ...saved,
                    // 确保 providers 数组完整（处理新增的提供商）
                    providers: mergeProviders(saved.providers || [], DEFAULT_SETTINGS.providers)
                };

                applyToCss(settings.value);
            }
        } catch (e) {
            console.error("加载配置失败:", e);
            lastError.value = e instanceof Error ? e.message : String(e);
        } finally {
            isLoading.value = false;
        }
    };

    /**
     * 合并已保存的提供商配置和默认配置
     * 确保新增的提供商也能出现在列表中
     */
    const mergeProviders = (
        savedProviders: ModelProviderConfig[],
        defaultProviders: ModelProviderConfig[]
    ): ModelProviderConfig[] => {
        const merged = [...savedProviders];

        // 添加缺失的默认提供商
        for (const defaultProvider of defaultProviders) {
            if (!merged.find(p => p.id === defaultProvider.id)) {
                merged.push({ ...defaultProvider });
            }
        }

        return merged;
    };

    // ========== 配置更新 ==========

    /**
     * 更新部分配置
     * 自动应用到 CSS 并持久化到后端
     */
    const updateConfig = async (newPartialSettings: Partial<AppSettings>) => {
        const oldSettings = { ...settings.value };

        try {
            settings.value = { ...settings.value, ...newPartialSettings };
            applyToCss(settings.value);

            await configCommands.saveConfig(settings.value);
            lastError.value = null;
        } catch (e) {
            console.error("持久化配置失败:", e);
            lastError.value = e instanceof Error ? e.message : String(e);

            // 回滚
            settings.value = oldSettings;
            applyToCss(oldSettings);

            throw e;
        }
    };

    /**
     * 更新指定提供商的配置
     */
    const updateProvider = async (providerId: string, config: Partial<ModelProviderConfig>) => {
        const providerIndex = settings.value.providers.findIndex(p => p.id === providerId);

        if (providerIndex === -1) {
            throw new Error(`Provider ${providerId} not found`);
        }

        const updatedProviders = [...settings.value.providers];
        updatedProviders[providerIndex] = {
            ...updatedProviders[providerIndex],
            ...config
        };

        await updateConfig({ providers: updatedProviders });
    };

    /**
     * 切换提供商启用状态
     */
    const toggleProvider = async (providerId: string) => {
        const provider = getProvider(providerId);
        if (provider) {
            await updateProvider(providerId, { enabled: !provider.enabled });
        }
    };

    /**
     * 设置默认提供商
     */
    const setDefaultProvider = async (providerId: string) => {
        const provider = getProvider(providerId);
        if (!provider) {
            throw new Error(`Provider ${providerId} not found`);
        }

        await updateConfig({ defaultProviderId: providerId });
    };

    /**
     * 重置为默认配置
     */
    const resetToDefaults = async () => {
        settings.value = { ...DEFAULT_SETTINGS };
        applyToCss(settings.value);

        try {
            await configCommands.saveConfig(settings.value);
            lastError.value = null;
        } catch (e) {
            console.error("重置配置失败:", e);
            lastError.value = e instanceof Error ? e.message : String(e);
            throw e;
        }
    };

    // ========== 向后兼容 ==========

    /**
     * 获取兼容性 API Key（旧版本使用）
     * 优先返回当前提供商的 apiKey
     */
    const getCompatApiKey = computed(() => {
        return currentProvider.value?.apiKey || settings.value.apiKey || '';
    });

    return {
        // 状态
        settings,
        isLoading,
        lastError,

        // 计算属性
        enabledProviders,
        currentProvider,
        getCompatApiKey,

        // 方法
        getProvider,
        init,
        updateConfig,
        updateProvider,
        toggleProvider,
        setDefaultProvider,
        resetToDefaults
    };
});