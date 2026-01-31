import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import { chatApi, type ChatSession } from '../api/chat';
import { invoke } from "@tauri-apps/api/core";
import { useConfigStore } from './config';
import type { Folder, PausedChunks } from './chat/state';
import { useFolderActions } from './chat/folders';
import { useSessionActions } from './chat/sessions';
import { useMessageActions } from './chat/messages';

// Re-export types for consumers
export type { Folder, ChatSession };

export const useChatStore = defineStore('chat', () => {
    // --- 状态 (State) ---
    const historyList = ref<ChatSession[]>([]);
    const folders = ref<Folder[]>([]);
    const activeId = ref<string | null>(null);
    const activeSocialContactId = ref<number | null>(null); // 👥 Social Mode Active Contact Persistence
    const currentMessages = ref<any[]>([]);
    const isGenerating = ref(false);
    const isLoading = ref(false);

    // 📜 Scroll Position Persistence
    const sessionScrollPositions = ref<Record<string, number>>({});

    // Load initial scroll positions from localStorage
    try {
        const savedScroll = localStorage.getItem('session_scroll_positions');
        if (savedScroll) {
            sessionScrollPositions.value = JSON.parse(savedScroll);
        }
    } catch (e) {
        console.error("Failed to load scroll positions:", e);
    }

    // Persist scroll positions on change
    watch(sessionScrollPositions, (newVal) => {
        localStorage.setItem('session_scroll_positions', JSON.stringify(newVal));
    }, { deep: true });

    const updateSessionScroll = (sessionId: string, position: number) => {
        if (!sessionId) return;
        sessionScrollPositions.value[sessionId] = position;
    };

    const getSessionScroll = (sessionId: string) => {
        return sessionScrollPositions.value[sessionId] || 0;
    };

    // 👥 Load Social Contact Persistence
    try {
        const savedSocialId = localStorage.getItem('active_social_contact_id');
        if (savedSocialId) {
            activeSocialContactId.value = parseInt(savedSocialId, 10);
            console.log("📍 [PERSISTENCE] Loaded active social contact:", activeSocialContactId.value);
        }
    } catch (e) {
        console.error("Failed to load active social contact:", e);
    }

    // Persist social contact changes
    watch(activeSocialContactId, (newId) => {
        if (newId) {
            localStorage.setItem('active_social_contact_id', newId.toString());
        } else {
            localStorage.removeItem('active_social_contact_id');
        }
    });

    const updateSocialContactId = (id: number | null) => {
        activeSocialContactId.value = id;
    };

    // --- 暂停/恢复相关状态 ---
    const generatingSessionId = ref<string | null>(null);  // 记录正在生成消息的会话 ID
    const pausedChunks = ref<PausedChunks>({ content: [], reasoning: [] });  // 暂停期间的消息块
    const isChatViewActive = ref(true);  // 追踪聊天视图是否激活（用于区分设置界面）

    // 临时保存正在生成的完整消息（用于在会话切换时恢复）
    const tempGeneratedMessage = ref<{ content: string, reasoning: string } | null>(null);

    // 🏄‍♂️ Smooth Streaming Queue State
    const streamQueue = ref<string[]>([]);
    const isProcessingQueue = ref(false);

    // 使用 config store 中的推理设置
    const configStore = useConfigStore();

    // --- 计算属性 (Getters) ---
    const activeSession = computed(() =>
        historyList.value.find(s => s.id === activeId.value) || null
    );

    const useReasoning = computed({
        get: () => configStore.settings.useReasoning,
        set: (value: boolean) => {
            console.log("🧠 useReasoning changed:", value);
            configStore.updateConfig({ useReasoning: value });
        }
    });

    const useSearch = computed({
        get: () => configStore.settings.useSearch,
        set: (value: boolean) => {
            console.log("🔍 useSearch changed:", value);
            configStore.updateConfig({ useSearch: value });
        }
    });

    const searchProvider = computed({
        get: () => configStore.settings.searchProvider,
        set: (value: string) => {
            console.log("🔍 searchProvider changed:", value);
            configStore.updateConfig({ searchProvider: value });
        }
    });

    // --- Composition ---

    // 1. Folders - completely independent
    const folderActions = useFolderActions(folders, historyList);

    // 2. Forward declarations for circular dependencies
    // MessageActions needs renameSession (from SessionActions)
    let _renameSession: ((id: string, title: string) => Promise<void>) | null = null;
    const renameSessionProxy = async (id: string, title: string) => {
        if (_renameSession) await _renameSession(id, title);
        else console.warn("renameSession not initialized yet");
    };

    // 3. Initialize Message Actions
    const messageState = {
        activeId,
        currentMessages,
        isGenerating,
        generatingSessionId,
        pausedChunks,
        streamQueue,
        isProcessingQueue,
        tempGeneratedMessage,
        isLoading,
        useReasoning,
        useSearch,
        activeSession,
        isChatViewActive
    };

    const messageActions = useMessageActions(messageState, {
        renameSession: renameSessionProxy
    });

    // 4. Define switchSession (now messageActions is available)
    const switchSession = async (sessionId: string) => {
        if (activeId.value === sessionId) return;

        activeId.value = sessionId;
        await messageActions.loadMessages(sessionId);
    };

    // 5. Initialize Session Actions (needs switchSession)
    const sessionActions = useSessionActions(
        historyList,
        activeId,
        currentMessages,
        switchSession
    );

    // 6. Link back the dependency
    _renameSession = sessionActions.renameSession;

    // --- Persistence ---
    // 监听 activeId 变化并持久化
    // 这样无论是点击切换、新建会话还是删除后自动切换，都会保存
    watch(activeId, (newVal) => {
        console.log("📍 [PERSISTENCE] activeId changed to:", newVal);
        if (newVal) {
            localStorage.setItem('latest_active_session_id', newVal);
            console.log("📍 [PERSISTENCE] Saved to localStorage:", newVal);
        } else {
            localStorage.removeItem('latest_active_session_id');
            console.log("📍 [PERSISTENCE] Removed from localStorage");
        }
    });

    // --- Root Level Actions (like loadData) ---
    const loadData = async () => {
        try {
            const [sessions, folderList] = await Promise.all([
                chatApi.getSessions(),
                invoke<Folder[]>("get_folders")
            ]);

            console.log("📂 [LOAD] Got sessions:", sessions.length);
            console.log("📂 [LOAD] Session IDs:", sessions.map(s => s.id).join(', '));

            historyList.value = sessions;
            folders.value = folderList;

            if (sessions.length > 0 && activeId.value === null) {
                // 优先尝试恢复上次打开的会话
                const lastId = localStorage.getItem('latest_active_session_id');
                console.log("📂 [LOAD] Last saved session ID:", lastId);

                const lastSessionExists = sessions.some(s => s.id === lastId);
                console.log("📂 [LOAD] Last session exists?", lastSessionExists);

                if (lastId && lastSessionExists) {
                    console.log("📂 [LOAD] Restoring last session:", lastId);
                    await switchSession(lastId);
                } else {
                    // 默认打开第一个
                    console.log("📂 [LOAD] Opening first session:", sessions[0].id);
                    await switchSession(sessions[0].id);
                }
            } else {
                console.log("📂 [LOAD] Skip: sessions.length =", sessions.length, ", activeId =", activeId.value);
            }
        } catch (e) {
            console.error("DB加载失败", e);
        }
    };

    return {
        // State
        historyList,
        folders,
        activeId,
        currentMessages,
        isGenerating,
        generatingSessionId,
        isChatViewActive,
        isLoading,
        activeSession,
        activeSocialContactId, // 👥 Exposed State
        sessionScrollPositions, // 📜 Exposed State

        // Config proxy
        useReasoning,
        useSearch,
        searchProvider,

        // Root Actions
        loadData,
        switchSession,
        // Session Actions
        ...sessionActions,

        // Message Actions
        ...messageActions,

        // 📜 Exposed Action (Placed AFTER spread to ensure our local version takes precedence)
        updateSessionScroll,
        getSessionScroll,
        updateSocialContactId, // 👥 Exposed Action
    };
});