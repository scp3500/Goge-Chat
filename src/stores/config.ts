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
                console.log('[ConfigStore INIT] Loaded from backend, raw providers order:',
                    Array.isArray(saved.providers) ? (saved.providers as any[]).map((p: any) => p.id).join(',') : 'N/A');

                // 合并配置，确保新增字段有默认值
                settings.value = {
                    ...DEFAULT_SETTINGS,
                    ...saved,
                    // 确保 providers 数组完整（处理新增的提供商）
                    providers: mergeProviders(saved.providers || [], DEFAULT_SETTINGS.providers)
                };

                console.log('[ConfigStore INIT] After merge, final order:',
                    settings.value.providers.map(p => p.id).join(','));
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
        // 1. 以已保存的提供商为基础，保持其顺序
        const merged = savedProviders.map(saved => {
            const defaultProv = defaultProviders.find(p => p.id === saved.id);
            if (!defaultProv) return saved;

            // 合并模型列表，确保新增的默认模型能出现
            const allModels = [...new Set([...(saved.models || []), ...(defaultProv.models || [])])];

            return {
                ...defaultProv, // 使用最新的默认值（如 id, name, icon, baseUrl, models 等）
                ...saved,       // 覆盖用户的个性化配置（enabled, apiKey, temperature, maxTokens 等）
                name: defaultProv.name, // 强制使用最新的内置名称（如 "Gemini 3"）
                icon: defaultProv.icon, // 强制使用最新的内置图标
                baseUrl: defaultProv.baseUrl, // 强制使用最新的内置 API 地址
                models: allModels // 使用合并后的模型列表
            };
        });

        // 2. 添加全新的（默认配置中有但已保存配置中没有）提供商
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
            console.warn(`[ConfigStore] Provider ${providerId} not found, cannot update.`);
            return;
        }

        const updatedProviders = JSON.parse(JSON.stringify(settings.value.providers));
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
     * 处理提供商重新排序事件
     * @param newSimpleProviders 包含 id 的简化提供商列表，表示新的顺序
     */
    const handleReorder = async (newSimpleProviders: { id: string }[]) => {
        console.log('[ConfigStore REORDER] ========== START ==========');
        console.log('[ConfigStore REORDER] Received new order from UI:', newSimpleProviders.map(p => p.id).join(','));

        try {
            const originalProviders = [...settings.value.providers];
            console.log('[ConfigStore REORDER] Current order in store:', originalProviders.map(p => p.id).join(','));

            // 根据简化列表的顺序，从完整配置中找到对应的提供商
            const newOrder: ModelProviderConfig[] = newSimpleProviders
                .map(simple => originalProviders.find(p => p.id === simple.id))
                .filter((p): p is ModelProviderConfig => Boolean(p)); // 过滤掉未找到的并进行类型断言

            if (newOrder.length !== originalProviders.length) {
                console.warn("[ConfigStore REORDER] Length mismatch! Expected:", originalProviders.length, "Got:", newOrder.length);
            }

            console.log('[ConfigStore REORDER] Calling updateProvidersOrder with:', newOrder.map(p => p.id).join(','));
            await updateProvidersOrder(newOrder);
            console.log('[ConfigStore REORDER] ========== COMPLETE ==========');
        } catch (error) {
            console.error("[ConfigStore REORDER] FAILED:", error);
            lastError.value = error instanceof Error ? error.message : String(error);
            throw error;
        }
    };

    /**
     * 更新提供商顺序
     */
    const updateProvidersOrder = async (newProviders: ModelProviderConfig[]) => {
        console.log('[ConfigStore UPDATE_ORDER] Received:', newProviders.map(p => p.id).join(','));
        // 深拷贝确保响应式引用被切断，防止 Pinia/Vue 同步冲突
        const cleanProviders = JSON.parse(JSON.stringify(newProviders));
        console.log('[ConfigStore UPDATE_ORDER] After deep clone:', (cleanProviders as any[]).map((p: any) => p.id).join(','));
        console.log('[ConfigStore UPDATE_ORDER] Calling updateConfig...');
        await updateConfig({ providers: cleanProviders });
        console.log('[ConfigStore UPDATE_ORDER] updateConfig completed');
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
     * 添加自定义提供商
     */
    const addCustomProvider = async () => {
        const id = `custom_${Date.now()}_${Math.random().toString(36).substr(2, 5)}`;
        const newProvider: ModelProviderConfig = {
            id,
            name: 'Custom Provider',
            icon: '🔌',
            enabled: true,
            apiKey: '',
            baseUrl: 'https://api.openai.com/v1',
            models: ['gpt-3.5-turbo', 'gpt-4'],
            isCustom: true
        };

        const newProviders = [...settings.value.providers, newProvider];
        await updateConfig({
            providers: newProviders,
            defaultProviderId: id // 自动切换到新创建的提供商
        });
        return id;
    };

    /**
     * 删除提供商
     */
    const removeProvider = async (providerId: string) => {
        const newProviders = settings.value.providers.filter(p => p.id !== providerId);

        // 如果删除的是当前选中的提供商，切换到第一个可用的
        let newDefaultId = settings.value.defaultProviderId;
        if (settings.value.defaultProviderId === providerId) {
            newDefaultId = newProviders.length > 0 ? newProviders[0].id : '';
        }

        await updateConfig({
            providers: newProviders,
            defaultProviderId: newDefaultId
        });
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
        updateProvidersOrder,
        toggleProvider,
        setDefaultProvider,
        addCustomProvider,
        removeProvider,
        handleReorder,
        resetToDefaults
    };
});