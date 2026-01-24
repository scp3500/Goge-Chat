import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { chatApi, type ChatSession } from '../api/chat';
import { invoke, Channel } from "@tauri-apps/api/core";

export interface Folder {
    id: string;
    name: string;
    sort_order: number;
    is_collapsed: boolean;
}

export const useChatStore = defineStore('chat', () => {
    // --- 状态 (State) ---
    const historyList = ref<ChatSession[]>([]);
    const folders = ref<Folder[]>([]);
    const activeId = ref<string | null>(null);
    const currentMessages = ref<any[]>([]);
    const isGenerating = ref(false);
    const useReasoning = ref(false); // 是否使用推理功能
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
            const [sessions, folderList] = await Promise.all([
                chatApi.getSessions(),
                invoke<Folder[]>("get_folders")
            ]);
            historyList.value = sessions;
            folders.value = folderList;
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
                last_scroll_pos: 0,
                sort_order: 0
            });
            activeId.value = newId;
            // 初始化对话时，不再使用欢迎语，改为系统指令
            currentMessages.value = [{ role: "system", content: "你是一个简洁专业的 AI 助手。" }];
        } catch (e) {
            console.error("创建失败", e);
        }
    };

    const deleteSession = async (sessionId: string) => {
        try {
            // 1. 调用 Rust 后端删除数据库记录
            await invoke("delete_session", { sessionId });

            // 2. 更新本地 UI 列表
            historyList.value = historyList.value.filter(s => s.id !== sessionId);

            // 3. 自动切换逻辑
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

    const renameSession = async (id: string, newTitle: string) => {
        try {
            // 1. 同步内存状态
            const session = historyList.value.find(s => s.id === id);
            if (session) {
                session.title = newTitle;
            }
            // 2. 同步数据库
            await invoke("rename_session", { id, title: newTitle });

            // 3. 强制触发响应式更新
            historyList.value = [...historyList.value];
        } catch (e) {
            console.error("重命名失败:", e);
        }
    };

    // --- 文件夹管理 Actions ---

    const createFolder = async (name: string) => {
        try {
            const id = await invoke<string>("create_folder", { name });
            folders.value.push({ id, name, sort_order: 0, is_collapsed: false });
        } catch (e) {
            console.error("创建文件夹失败:", e);
        }
    };

    const deleteFolder = async (id: string) => {
        try {
            await invoke("delete_folder", { id });
            folders.value = folders.value.filter(f => f.id !== id);
            // 更新本地 session，去掉它们的 folder_id
            historyList.value.forEach(s => {
                if (s.folder_id === id) s.folder_id = null;
            });
        } catch (e) {
            console.error("删除文件夹失败:", e);
        }
    };

    const renameFolder = async (id: string, name: string) => {
        try {
            await invoke("rename_folder", { id, name });
            const folder = folders.value.find(f => f.id === id);
            if (folder) folder.name = name;
        } catch (e) {
            console.error("重命名文件夹失败:", e);
        }
    };

    const moveSessionToFolder = async (sessionId: string, folderId: string | null) => {
        try {
            await invoke("move_session_to_folder", { sessionId, folderId });
            const session = historyList.value.find(s => s.id === sessionId);
            if (session) session.folder_id = folderId;
        } catch (e) {
            console.error("移动会话失败:", e);
        }
    };

    const toggleFolder = async (id: string) => {
        const folder = folders.value.find(f => f.id === id);
        if (folder) {
            folder.is_collapsed = !folder.is_collapsed;
            try {
                await invoke("update_folder_collapsed", { id, collapsed: folder.is_collapsed });
            } catch (e) {
                console.error("更新文件夹折叠状态失败:", e);
            }
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
                    ? history.map(m => ({
                        ...m,
                        reasoning_content: m.reasoning_content || ""
                    }))
                    : [{ role: "system", content: "你是一个简洁专业的 AI 助手。" }];
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

            currentMessages.value.push({
                role: "user",
                content: text
            });
            currentMessages.value.push({
                role: "assistant",
                content: "__LOADING__",
                reasoning_content: ""
            });

            const onEvent = new Channel<string>();
            let aiFullContent = "";
            let aiFullReasoning = "";

            onEvent.onmessage = (data) => {
                if (!isGenerating.value) return;
                const lastMsg = currentMessages.value[currentMessages.value.length - 1];

                // 这里保持流式解析，因为需要实时给用户展示打字机效果
                const type = data.substring(0, 2);
                const content = data.substring(2);

                if (type === "c:") {
                    if (lastMsg.content === "__LOADING__") {
                        lastMsg.content = "";
                    }
                    lastMsg.content += content;
                    aiFullContent += content;
                } else if (type === "r:") {
                    if (!lastMsg.reasoning_content) lastMsg.reasoning_content = "";
                    lastMsg.reasoning_content += content;
                    aiFullReasoning += content;
                }
            };

            const msgsToSend = currentMessages.value.slice(0, -1).map((m, idx) => {
                const cleanMsg = {
                    role: m.role,
                    content: m.content,
                    reasoning_content: m.reasoning_content || undefined
                };
                if (idx === currentMessages.value.length - 2 && useReasoning.value) {
                    cleanMsg.content = `[REASON]${m.content}`;
                }
                return cleanMsg;
            });

            await invoke("ask_ai", {
                msg: msgsToSend,
                onEvent,
            });

            if (aiFullContent.trim().length > 0 || aiFullReasoning.trim().length > 0) {
                await invoke("save_message", {
                    sessionId,
                    role: "assistant",
                    content: aiFullContent,
                    reasoning_content: aiFullReasoning || null,
                });

                // 检查是否需要自动总结标题
                const msgCount = currentMessages.value.filter(m => m.content !== "__LOADING__").length;
                if (msgCount >= 5 && activeSession.value?.title === "新对话") {
                    autoSummaryTitle(sessionId);
                }
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

    /**
     * ⚡️ 架构重构：非流式标题生成 (Blocking Mode)
     * 彻底解决 "c:c:标题" 这种协议头污染问题
     * * 注意：这需要 Rust 后端实现 `generate_title` 命令！
     */
    const autoSummaryTitle = async (sessionId: string) => {
        try {
            // 1. 构造 Prompt
            const prompt = "请总结以上对话的标题(5-8字)。直接返回标题文字，不要代码，不要标点符号。";

            const filteredMsgs = currentMessages.value.filter(m => m.content !== "__LOADING__");
            // 取前几轮对话 + prompt
            const summaryMsgs = [
                ...filteredMsgs.slice(1, 5),
                { role: "user", content: prompt }
            ];

            console.log("=== [Blocking] 请求后端生成标题 ===");

            // 2. ⚡️ 核心改动：使用 generate_title，不再使用 Channel 流式接收
            // 这是一个异步等待过程，前端会等待后端完全生成好字符串后一次性返回
            const rawTitle = await invoke<string>("generate_title", {
                msg: summaryMsgs
            });

            console.log("✨ 后端返回原始标题:", rawTitle);

            // 3. 简单的长度截断
            let finalTitle = rawTitle.trim();
            if (finalTitle.length > 10) {
                finalTitle = finalTitle.substring(0, 10);
            }

            // 5. 应用更新
            if (finalTitle && finalTitle.length > 0 && finalTitle !== "新对话") {
                await renameSession(sessionId, finalTitle);
            }

        } catch (e) {
            console.error("自动总结标题失败 (请检查 Rust 后端是否实现了 generate_title):", e);
        }
    };

    const reorderSessions = async (newList: ChatSession[]) => {
        historyList.value = newList;
        const orders: [string, number][] = newList.map((s, index) => [s.id, index]);
        try {
            await chatApi.updateSessionsOrder(orders);
        } catch (e) {
            console.error("更新排序失败:", e);
        }
    };

    const reorderFolders = async (newList: Folder[]) => {
        folders.value = newList;
        const orders: [string, number][] = newList.map((f, index) => [f.id, index]);
        try {
            await invoke("update_folders_order", { orders });
        } catch (e) {
            console.error("更新文件夹排序失败:", e);
        }
    };

    return {
        historyList,
        folders,
        activeId,
        currentMessages,
        isGenerating,
        isLoading,
        useReasoning,
        activeSession,
        loadData,
        switchSession,
        createSession,
        deleteSession,
        renameSession,
        createFolder,
        deleteFolder,
        renameFolder,
        moveSessionToFolder,
        loadMessages,
        sendMessage,
        stopGeneration,
        updateSessionScroll,
        reorderSessions,
        reorderFolders,
        toggleFolder,
    };
});