import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { chatApi, type ChatSession } from '../api/chat';
import { invoke, Channel } from "@tauri-apps/api/core";

export const useChatStore = defineStore('chat', () => {
    // --- 状态 (State) ---
    const historyList = ref<ChatSession[]>([]);
    const activeId = ref<string | null>(null);
    const currentMessages = ref<any[]>([]);
    const isGenerating = ref(false);
    const isLoading = ref(false); // ✨ 新增：标记是否正在从数据库加载历史记录

    // --- 计算属性 (Getters) ---
    const activeSession = computed(() =>
        historyList.value.find(s => s.id === activeId.value) || null
    );

    // --- 会话管理 Actions ---

    /**
     * 手术点 1：统一切换逻辑
     * 侧边栏点击时请直接调用这个方法，而不是直接修改 activeId
     */
    const switchSession = async (sessionId: string) => {
        if (activeId.value === sessionId) return;

        activeId.value = sessionId;
        await loadMessages(sessionId);
    };

    const loadData = async () => {
        try {
            const sessions = await chatApi.getSessions();
            historyList.value = sessions;
            if (sessions.length > 0 && activeId.value === null) {
                // 默认加载第一个
                await switchSession(sessions[0].id);
            }
        } catch (e) {
            console.error("DB加载失败", e);
        }
    };

    const createSession = async () => {
        try {
            const newId = await chatApi.createSession("新对话");
            historyList.value.unshift({
                id: newId,
                title: "新对话",
                last_scroll_pos: 0
            });
            activeId.value = newId;
            // 创建新会话，直接初始化
            currentMessages.value = [{ role: "assistant", content: "你好！我是 GoleChat。" }];
        } catch (e) {
            console.error("创建失败", e);
        }
    };

    // --- 消息管理 Actions ---

    /**
     * 手术点 2：优化加载逻辑，解决跳变
     */
    const loadMessages = async (sessionId: string) => {
        // 1. 立即同步清空，解决“残留旧对话”导致的跳变
        currentMessages.value = [];
        isLoading.value = true;

        try {
            // 2. 向 Rust 请求数据
            const history = await invoke<any[]>("get_messages", { sessionId });

            // 3. 只有当用户还没切走时，才更新数据（防止竞态）
            if (activeId.value === sessionId) {
                currentMessages.value = history && history.length > 0
                    ? history
                    : [{ role: "assistant", content: "你好！我是 GoleChat。" }];
            }
        } catch (err) {
            console.error("获取消息失败:", err);
        } finally {
            isLoading.value = false;
        }
    };

    // --- 消息流处理 (保持原样) ---
    const sendMessage = async (text: string) => {
        if (!activeId.value || !text.trim() || isGenerating.value) return;

        const sessionId = activeId.value;
        isGenerating.value = true;

        try {
            await invoke("reset_ai_generation");
            await invoke("save_message", { sessionId, role: "user", content: text });

            currentMessages.value.push({ role: "user", content: text });
            currentMessages.value.push({ role: "assistant", content: "__LOADING__" });

            const onEvent = new Channel<string>();
            let aiFullContent = "";

            onEvent.onmessage = (chunk) => {
                if (!isGenerating.value) return;
                const lastMsg = currentMessages.value[currentMessages.value.length - 1];
                if (lastMsg.content === "__LOADING__") {
                    lastMsg.content = "";
                }
                lastMsg.content += chunk;
                aiFullContent += chunk;
            };

            await invoke("ask_ai", {
                msg: currentMessages.value.slice(0, -1),
                onEvent,
            });

            if (aiFullContent.trim().length > 0) {
                await invoke("save_message", {
                    sessionId,
                    role: "assistant",
                    content: aiFullContent,
                });
            }
        } catch (error) {
            console.error("对话失败:", error);
        } finally {
            isGenerating.value = false;
        }
    };

    const stopGeneration = async () => {
        isGenerating.value = false;
        try { await invoke("stop_ai_generation"); } catch (err) { console.error(err); }
    };

    const updateSessionScroll = async (id: string, pos: number) => {
        const session = historyList.value.find(s => s.id === id);
        if (session) {
            session.last_scroll_pos = pos;
            try {
                await invoke('update_session_scroll', { id, pos });
            } catch (err) {
                console.error("滚动位置持久化失败:", err);
            }
        }
    };

    return {
        historyList,
        activeId,
        currentMessages,
        isGenerating,
        isLoading, // 暴露给 UI 展现加载态
        activeSession,
        loadData,
        switchSession, // 暴露新的切换方法
        createSession,
        loadMessages,
        sendMessage,
        stopGeneration,
        updateSessionScroll,
    };
});