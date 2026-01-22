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
    const isLoading = ref(false);

    // --- 计算属性 (Getters) ---
    const activeSession = computed(() =>
        historyList.value.find(s => s.id === activeId.value) || null
    );

    // --- 会话管理 Actions ---

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
            currentMessages.value = [{ role: "assistant", content: "你好！我是 GoleChat。" }];
        } catch (e) {
            console.error("创建失败", e);
        }
    };

    /**
     * 🩺 手术点 1：补全删除逻辑
     */
    const deleteSession = async (sessionId: string) => {
        try {
            // 1. 调用 Rust 后端删除数据库记录
            await invoke("delete_session", { sessionId });

            // 2. 更新本地 UI 列表
            historyList.value = historyList.value.filter(s => s.id !== sessionId);

            // 3. 自动切换逻辑：如果删掉的是当前对话
            if (activeId.value === sessionId) {
                if (historyList.value.length > 0) {
                    await switchSession(historyList.value[0].id);
                } else {
                    activeId.value = null;
                    currentMessages.value = [];
                }
            }
        } catch (e) {
            console.error("删除会话失败:", e);
        }
    };

    /**
     * 🩺 手术点 2：补全重命名逻辑
     */
    const renameSession = async (id: string, newTitle: string) => {
        try {
            // 1. 同步内存状态
            const session = historyList.value.find(s => s.id === id);
            if (session) {
                session.title = newTitle;
            }
            // 2. 同步数据库
            await invoke("rename_session", { id, title: newTitle });
        } catch (e) {
            console.error("重命名失败:", e);
        }
    };

    // --- 消息管理 Actions ---

    const loadMessages = async (sessionId: string) => {
        currentMessages.value = [];
        isLoading.value = true;
        try {
            const history = await invoke<any[]>("get_messages", { sessionId });
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
        isLoading,
        activeSession,
        loadData,
        switchSession,
        createSession,
        deleteSession, // ✨ 必须暴露
        renameSession, // ✨ 必须暴露
        loadMessages,
        sendMessage,
        stopGeneration,
        updateSessionScroll,
    };
});