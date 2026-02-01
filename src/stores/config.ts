// src/stores/config.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { configCommands, fileCommands } from '../tauri/commands';
import { AppSettings, DEFAULT_SETTINGS, ModelProviderConfig, ModelPreset, PromptLibraryItem, ModelInfo } from '../types/config';
import { PREBUILT_PROMPTS } from '../constants/prompts';

export const useConfigStore = defineStore('config', () => {
    // ========== 状态 ==========
    const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });
    const isLoading = ref(false);
    const lastError = ref<string | null>(null);
    const userAvatarUrl = ref<string>(''); // Loaded base64 avatar for display
    let lastLoadedPath = ''; // Prevent redundant loads

    // Import lazily to avoid circular issues or just at top
    // But we can't import inside defineStore easily if it is a module import
    // Let's add import at top of file

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

        // 🛡️ 强力修复：彻底移除可能存在的内联硬编码背景色，确保 CSS 变量生效
        root.style.removeProperty('--bg-main');

        // 应用全局缩放 (UI 密度)
        // 注意：zoom 在现代浏览器中表现良好，但某些布局可能需要更精细的处理
        const scale = val.globalScale || 1.0;
        (root.style as any).zoom = scale.toString();

        // 文字大小补偿计算：fontSize / scale 确保文字物理大小在不同缩放下保持一致
        const compensatedFontSize = val.fontSize / scale;
        root.style.setProperty('--font-size-base', `${compensatedFontSize}px`);

        root.style.setProperty('--font-ratio', val.lineRatio.toString());
        root.style.setProperty('--scrollbar-width', `${val.scrollbarWidth}px`);

        // 头像调整
        root.style.setProperty('--user-avatar-size', `${val.userAvatarSize || 36}px`);
        root.style.setProperty('--user-avatar-radius', `${val.userAvatarBorderRadius || 6}px`);
        root.style.setProperty('--user-avatar-offset-x', `${val.userAvatarOffsetX || 0}px`);
        root.style.setProperty('--user-avatar-offset-x', `${val.userAvatarOffsetX || 0}px`);
        root.style.setProperty('--user-avatar-offset-y', `${val.userAvatarOffsetY || 0}px`);

        // Load avatar if path exists and url is empty
        if (val.userAvatarPath && !userAvatarUrl.value) {
            loadUserAvatar();
        }

        // 应用主题属性
        // 根据当前模式（light/dark）获取用户预设的具体主题 ID
        const activeThemeId = val.theme === 'light' ? (val.lightThemeId || 'light') : (val.darkThemeId || 'dark');
        root.setAttribute('data-theme', activeThemeId);

        // 同时切换类名以便排查
        if (val.theme === 'light') {
            root.classList.add('light-mode');
            root.classList.remove('app-dark'); // Ensure app-dark is removed
        } else {
            root.classList.remove('light-mode');
            root.classList.add('app-dark'); // ✅ Add app-dark for SocialChatContainer overrides
        }
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

                // 🛡️ Hotfix: 检测并修复被污染的 defaultSystemPrompt
                // 如果用户之前受到 bug 影响，导致全局默认提示词被错设为 "提示词创作" (Prompt Singularity)，则自动重置回默认助手
                let fixedDefaultPrompt = saved.defaultSystemPrompt;
                if (fixedDefaultPrompt && fixedDefaultPrompt.includes("Role: Prompt Singularity")) {
                    console.warn("[ConfigStore] Detected polluted defaultSystemPrompt, resetting to default.");
                    fixedDefaultPrompt = DEFAULT_SETTINGS.defaultSystemPrompt;
                }

                // 合并配置，确保新增字段有默认值
                settings.value = {
                    ...DEFAULT_SETTINGS,
                    ...saved,
                    defaultSystemPrompt: fixedDefaultPrompt || saved.defaultSystemPrompt || DEFAULT_SETTINGS.defaultSystemPrompt,
                    // 确保 providers 数组完整（处理新增的提供商）
                    providers: mergeProviders(saved.providers || [], DEFAULT_SETTINGS.providers),
                    // 确保 presets 数组完整
                    presets: mergePresets(saved.presets || [], DEFAULT_SETTINGS.presets),
                    // 确保 promptLibrary 完整
                    promptLibrary: mergePromptLibrary(saved.promptLibrary || [], PREBUILT_PROMPTS),
                    // 确保 chatMode 完整 (Deep Merge)
                    chatMode: {
                        ...DEFAULT_SETTINGS.chatMode,
                        ...(saved.chatMode || {})
                    }
                };

                console.log('[ConfigStore INIT] After merge, final order:',
                    settings.value.providers.map(p => p.id).join(','));

                // Initialize avatar
                if (settings.value.userAvatarPath) {
                    await loadUserAvatar();
                }

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
        // 🛡️ 修复：先对 savedProviders 进行去重，防止配置文件中出现重复的 provider block
        const uniqueSavedProviders = savedProviders.filter((provider, index, self) =>
            index === self.findIndex((p) => p.id === provider.id)
        );

        const merged = uniqueSavedProviders.map(saved => {
            const defaultProv = defaultProviders.find(p => p.id === saved.id);
            if (!defaultProv) {
                // 如果是用户自定义提供商也需要处理模型规格化
                return {
                    ...saved,
                    models: normalizeModels(saved.models || [])
                };
            }

            // 合并模型列表，并基于 ID 进行去重，确保新增的默认模型能出现，同时保留用户自定义配置
            const modelMap = new Map<string, string | ModelInfo>();

            // 先放进默认模型
            if (defaultProv.models) {
                defaultProv.models.forEach(m => {
                    const id = typeof m === 'string' ? m : m.id;
                    modelMap.set(id, m);
                });
            }

            // 再用用户保存的模型覆盖（用户保存的具有更高优先级）
            if (saved.models) {
                saved.models.forEach(m => {
                    const id = typeof m === 'string' ? m : m.id;
                    modelMap.set(id, m);
                });
            }

            const allModels = normalizeModels(Array.from(modelMap.values()));

            return {
                ...defaultProv, // 使用最新的默认值（如 id, name, icon, baseUrl, models 等）
                ...saved,       // 覆盖用户的个性化配置（enabled, apiKey, temperature, maxTokens 等）
                name: defaultProv.name, // 强制使用最新的内置名称
                icon: defaultProv.icon, // 强制使用最新的内置图标
                baseUrl: defaultProv.baseUrl, // 强制使用最新的内置 API 地址
                models: allModels // 使用规格化和合并后的模型列表
            };
        });

        // 2. 添加全新的（默认配置中有但已保存配置中没有）提供商
        for (const defaultProvider of defaultProviders) {
            if (!merged.find(p => p.id === defaultProvider.id)) {
                merged.push({
                    ...defaultProvider,
                    models: normalizeModels(defaultProvider.models || [])
                });
            }
        }

        return merged;
    };

    /**
     * 规格化模型列表：将字符串数组转换为 ModelInfo 对象数组
     */
    const normalizeModels = (models: (string | ModelInfo)[]): ModelInfo[] => {
        return models.map(m => {
            if (typeof m === 'string') {
                // 尝试根据名称推断特性和分组
                const modelId = m;
                let group = '';
                const features: any[] = [];

                if (modelId.toLowerCase().includes('vision') || modelId.toLowerCase().includes('-v')) features.push('vision');
                if (modelId.toLowerCase().includes('reasoner') || modelId.toLowerCase().includes('reason')) features.push('reasoning');

                // Gemini 分组逻辑
                if (modelId.startsWith('gemini-1.5')) group = 'Gemini 1.5';
                else if (modelId.startsWith('gemini-2.0')) group = 'Gemini 2.0';
                else if (modelId.startsWith('gemini-exp')) group = 'Experimental';

                return {
                    id: modelId,
                    name: modelId,
                    group: group,
                    features: features
                };
            }
            return m;
        });
    };

    /**
     * 合并已保存的预设和默认预设
     * 确保默认预设始终存在，并保留用户的修改
     */
    const mergePresets = (
        savedPresets: ModelPreset[],
        defaultPresets: ModelPreset[]
    ): ModelPreset[] => {
        const merged = [...(savedPresets || [])];

        // 确保默认预设存在
        for (const def of defaultPresets) {
            if (!merged.find(p => p.id === def.id)) {
                merged.push({ ...def });
            }
        }

        return merged;
    };

    /**
     * 合并已保存的提示词库和内置提示词库
     */
    const mergePromptLibrary = (
        saved: PromptLibraryItem[],
        builtin: PromptLibraryItem[]
    ): PromptLibraryItem[] => {
        // 1. Start with saved items (preserves order)
        let merged = [...(saved || [])];

        // 2. Sync built-in content updates (e.g. edited .md files)
        merged = merged.map(item => {
            const freshBuiltin = builtin.find(b => b.id === item.id);
            if (freshBuiltin) {
                // If it's a built-in item, force update the content/metadata from file
                // keeping the user's "id" position in the list
                return {
                    ...item,
                    ...freshBuiltin // Overwrite with fresh data from .md
                };
            }
            return item; // Keep custom user items as is
        });

        // 3. Add new built-in items that aren't in saved list yet
        for (const fresh of builtin) {
            if (!merged.find(m => m.id === fresh.id)) {
                merged.push({ ...fresh });
            }
        }

        // 4. Filter malformed
        return merged.filter(item => item && item.id);
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

            // 🛡️ [隔离修复]：在持久化到 config.json 时，剔除与“当前活跃会话”相关的 transient 状态
            // 这确保了不同会话/窗口不会竞争同一个全局配置文件中的 active 模型/预设
            const {
                selectedModelId,
                defaultPresetId,
                ...persistentSettings
            } = settings.value;

            await configCommands.saveConfig(persistentSettings as AppSettings);
            lastError.value = null;

            // 🔄 Reactive Avatar Loading: Reload if path changed
            if (newPartialSettings.userAvatarPath !== undefined && newPartialSettings.userAvatarPath !== lastLoadedPath) {
                await loadUserAvatar();
            }
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

    // ========== 预设管理 ==========

    /** 获取指定预设 */
    const getPreset = (id: string) =>
        settings.value.presets?.find(p => p.id === id);

    /** 更新预设内容 */
    const updatePreset = async (presetId: string, config: Partial<ModelPreset>) => {
        const presetIndex = settings.value.presets.findIndex(p => p.id === presetId);
        if (presetIndex === -1) return;

        const updatedPresets = JSON.parse(JSON.stringify(settings.value.presets));
        updatedPresets[presetIndex] = {
            ...updatedPresets[presetIndex],
            ...config
        };

        await updateConfig({ presets: updatedPresets });
    };

    /** 添加新预设 */
    const addPreset = async (name: string) => {
        const id = `preset_${Date.now()}`;
        const newPreset: ModelPreset = {
            id,
            name,
            temperature: 0.7,
            maxTokens: 4096,
            systemPrompt: ''
        };

        const newPresets = [...(settings.value.presets || []), newPreset];
        await updateConfig({ presets: newPresets });
        return id;
    };

    /** 删除预设 */
    const removePreset = async (presetId: string) => {
        const newPresets = settings.value.presets.filter(p => p.id !== presetId);

        let newDefaultId = settings.value.defaultPresetId;
        if (settings.value.defaultPresetId === presetId) {
            newDefaultId = newPresets.length > 0 ? newPresets[0].id : '';
        }

        await updateConfig({
            presets: newPresets,
            defaultPresetId: newDefaultId
        });
    };

    /** 处理预置排序 */
    const handlePresetsReorder = async (newSimplePresets: { id: string }[]) => {
        try {
            const originalPresets = [...settings.value.presets];
            const newOrder: ModelPreset[] = newSimplePresets
                .map(simple => originalPresets.find(p => p.id === simple.id))
                .filter((p): p is ModelPreset => Boolean(p));

            await updateConfig({ presets: JSON.parse(JSON.stringify(newOrder)) });
        } catch (error) {
            console.error("[ConfigStore PRESETS_REORDER] FAILED:", error);
            throw error;
        }
    };

    // ========== 提示词库管理 ==========

    /** 更新提示词库项 */
    const updatePrompt = async (id: string, config: Partial<PromptLibraryItem>) => {
        const index = settings.value.promptLibrary.findIndex(p => p.id === id);
        if (index === -1) return;

        const updated = JSON.parse(JSON.stringify(settings.value.promptLibrary));
        updated[index] = { ...updated[index], ...config };
        await updateConfig({ promptLibrary: updated });
    };

    /** 添加提示词库项 */
    const addPrompt = async (item: Omit<PromptLibraryItem, 'id'>) => {
        const id = `prompt_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;
        const newItem: PromptLibraryItem = { ...item, id };
        const updated = [...settings.value.promptLibrary, newItem];
        await updateConfig({ promptLibrary: updated });
        return id;
    };

    /** 删除提示词库项 */
    const removePrompt = async (id: string) => {
        const updated = settings.value.promptLibrary.filter(p => p.id !== id);
        await updateConfig({ promptLibrary: updated });
    };

    /** 提示词库管理排序 */
    const handlePromptsReorder = async (newSimplePrompts: { id: string }[]) => {
        const originalPrompts = [...settings.value.promptLibrary];
        const newOrder = newSimplePrompts
            .map(simple => originalPrompts.find(p => p.id === simple.id))
            .filter((p): p is PromptLibraryItem => Boolean(p));
        await updateConfig({ promptLibrary: JSON.parse(JSON.stringify(newOrder)) });
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

    // ========== 头像加载 ==========
    const loadUserAvatar = async () => {
        const path = settings.value.userAvatarPath;
        if (!path) {
            userAvatarUrl.value = '';
            return;
        }

        // If it's already a data URL or http URL, just use it
        if (path.startsWith('data:') || path.startsWith('http')) {
            userAvatarUrl.value = path;
            lastLoadedPath = path;
            return;
        }

        try {
            // Read file content using robust Rust command
            const base64 = await fileCommands.readFileBase64(path);

            // Guess mime type
            const mimeType = path.toLowerCase().endsWith('.png') ? 'image/png' :
                path.toLowerCase().endsWith('.gif') ? 'image/gif' :
                    path.toLowerCase().endsWith('.webp') ? 'image/webp' :
                        'image/jpeg';

            userAvatarUrl.value = `data:${mimeType};base64,${base64}`;
            lastLoadedPath = path;
        } catch (e) {
            console.error('Failed to load avatar:', e);
            userAvatarUrl.value = '';
        }
    };

    /**
     * 上传用户头像
     */
    const uploadAvatar = async (filePath: string) => {
        try {
            const savedPath = await fileCommands.uploadUserAvatar(filePath);

            // Update settings
            await updateConfig({ userAvatarPath: savedPath });

            // Reload avatar for display
            await loadUserAvatar();

            return savedPath;
        } catch (e) {
            console.error("上传头像失败:", e);
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
        resetToDefaults,
        uploadAvatar,
        userAvatarUrl, // Export state
        loadUserAvatar,

        // 预设管理
        getPreset,
        updatePreset,
        addPreset,
        removePreset,
        handlePresetsReorder,

        // 提示词库管理
        updatePrompt,
        addPrompt,
        removePrompt,
        handlePromptsReorder
    };
});