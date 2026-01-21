import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { chatApi, type ChatSession } from '../api/chat';
import { invoke, Channel } from "@tauri-apps/api/core";

export const useChatStore = defineStore('chat', () => {
    // --- 状态 (State) ---
    const historyList = ref<ChatSession[]>([]);
    const activeId = ref<string | null>(null);
    const currentMessages = ref<any[]>([]); // 当前选中的消息列表
    const isGenerating = ref(false);       // AI 是否正在生成中

    // --- 计算属性 (Getters) ---
    const activeSession = computed(() =>
        historyList.value.find(s => s.id === activeId.value) || null
    );

    // --- 会话管理 Actions ---
    const loadData = async () => {
        try {
            const sessions = await chatApi.getSessions();
            historyList.value = sessions;
            if (sessions.length > 0 && activeId.value === null) {
                activeId.value = sessions[0].id;
                // ✨ 加这一句，提前把第一个会话的消息拽回来
                await loadMessages(activeId.value);
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

    // --- 消息管理 Actions ---

    /**
     * 加载特定会话的消息记录
     */
    const loadMessages = async (sessionId: string) => {
        try {
            const history = await invoke<any[]>("get_messages", { sessionId });
            currentMessages.value = history && history.length > 0
                ? history
                : [{ role: "assistant", content: "你好！我是 GoleChat。" }];
        } catch (err) {
            console.error("获取消息失败:", err);
        }
    };

    /**
     * 核心：发送消息并处理流式响应
     */
    const sendMessage = async (text: string) => {
        if (!activeId.value || !text.trim() || isGenerating.value) return;

        const sessionId = activeId.value;
        isGenerating.value = true;

        try {
            // 1. 重置后端状态并持久化用户消息
            await invoke("reset_ai_generation");
            await invoke("save_message", { sessionId, role: "user", content: text });

            // 2. 更新 UI 列表
            currentMessages.value.push({ role: "user", content: text });
            currentMessages.value.push({ role: "assistant", content: "__LOADING__" });

            // 3. 建立流式通道
            const onEvent = new Channel<string>();
            let aiFullContent = "";

            onEvent.onmessage = (chunk) => {
                if (!isGenerating.value) return;

                const lastMsg = currentMessages.value[currentMessages.value.length - 1];
                if (lastMsg.content === "__LOADING__") {
                    lastMsg.content = ""; // 收到首个 chunk，清除加载动画
                }
                lastMsg.content += chunk;
                aiFullContent += chunk;
            };

            // 4. 请求 AI
            await invoke("ask_ai", {
                msg: currentMessages.value.slice(0, -1),
                onEvent,
            });

            // 5. 生成结束，持久化 AI 回复
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

    /**
     * 中断生成
     */
    const stopGeneration = async () => {
        isGenerating.value = false;
        try {
            await invoke("stop_ai_generation");
        } catch (err) {
            console.error("中断失败:", err);
        }
    };

    /**
     * 🩺 手术点：补全持久化逻辑
     * 逻辑：先更新内存中的响应式数据，再异步告知后端写入数据库
     */
    const updateSessionScroll = async (id: string, pos: number) => {
        const session = historyList.value.find(s => s.id === id);
        if (session) {
            // 1. 内存同步（确保 UI 实时感知）
            session.last_scroll_pos = pos;

            // 2. 物理同步（调用 Rust 后端 update_session_scroll 命令）
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
        activeSession,
        loadData,
        createSession,
        loadMessages,
        sendMessage,
        stopGeneration,
        updateSessionScroll,
    };
});