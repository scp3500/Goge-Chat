import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { chatApi, type ChatSession } from '../api/chat';
import { invoke, Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useConfigStore } from './config';

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
    const isLoading = ref(false);
    const useSearch = ref(false);
    const searchProvider = ref('all');

    // --- 暂停/恢复相关状态 ---
    const generatingSessionId = ref<string | null>(null);  // 记录正在生成消息的会话 ID
    const pausedChunks = ref<{ content: string[], reasoning: string[] }>({ content: [], reasoning: [] });  // 暂停期间的消息块
    const isChatViewActive = ref(true);  // 追踪聊天视图是否激活（用于区分设置界面）

    // 临时保存正在生成的完整消息（用于在会话切换时恢复）
    const tempGeneratedMessage = ref<{ content: string, reasoning: string } | null>(null);

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

    // --- 会话管理 Actions ---

    const switchSession = async (sessionId: string) => {
        if (activeId.value === sessionId) return;

        activeId.value = sessionId;
        await loadMessages(sessionId);
    };

    /**
     * 应用缓存的消息块（用于从设置界面返回聊天界面时）
     */
    const applyPausedChunks = () => {
        // 只在有缓存且仍在生成时才应用
        if (!generatingSessionId.value || !isGenerating.value) {
            return;
        }

        if (generatingSessionId.value === activeId.value && pausedChunks.value.content.length > 0) {
            const lastMsg = currentMessages.value[currentMessages.value.length - 1];
            // 确保最后一条消息存在且确实是 assistant 消息
            if (lastMsg && lastMsg.role === 'assistant' && !lastMsg.id) {
                // 应用缓存的内容消息块
                for (const chunk of pausedChunks.value.content) {
                    lastMsg.content += chunk;
                }
                // 应用缓存的推理消息块
                for (const chunk of pausedChunks.value.reasoning) {
                    if (!lastMsg.reasoningContent) {
                        lastMsg.reasoningContent = "";
                    }
                    lastMsg.reasoningContent += chunk;
                }
                // 清空缓存
                pausedChunks.value = { content: [], reasoning: [] };
            }
        }
    };

    /**
     * 设置聊天视图激活状态
     */
    const setChatViewActive = (active: boolean) => {
        isChatViewActive.value = active;
        // 如果重新激活聊天视图，应用缓存的消息块
        if (active) {
            applyPausedChunks();
        }
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

    const clearMessages = async (sessionId: string) => {
        try {
            // 1. 调用 Rust 后端清空消息
            await invoke("clear_messages", { sessionId });

            // 2. 如果是当前会话，清空本地显示
            if (activeId.value === sessionId) {
                currentMessages.value = [{ role: "system", content: "你是一个简洁专业的 AI 助手。" }];
            }
        } catch (e) {
            console.error("清空消息失败:", e);
        }
    };

    const deleteMessageAction = async (messageId: number | undefined, index: number) => {
        try {
            if (messageId) {
                await invoke("delete_message", { messageId });
            }
            currentMessages.value.splice(index, 1);
        } catch (e) {
            console.error("删除消息失败:", e);
        }
    };

    const editMessageAction = async (messageId: number | undefined, index: number, newContent: string) => {
        try {
            if (!activeId.value) return;

            // 1. 更新数据库中该条消息的内容
            if (messageId) {
                await invoke("update_message", { messageId, content: newContent });

                // 2. 删除该条消息之后的所有消息
                await invoke("delete_messages_after", {
                    sessionId: activeId.value,
                    messageId: messageId
                });
            }

            // 3. 更新本地状态：修改内容并截断列表
            currentMessages.value[index].content = newContent;
            currentMessages.value = currentMessages.value.slice(0, index + 1);

            // 4. 重新触发 AI 回答
            await sendMessage(""); // 传空字符串触发逻辑 (需要修改 sendMessage 适应此情况)
        } catch (e) {
            console.error("编辑消息失败:", e);
        }
    };

    const regenerateAction = async (index: number) => {
        try {
            if (!activeId.value) return;

            // 1. 如果当前点击的是 assistant 消息，先删除它
            const msg = currentMessages.value[index];
            if (msg.role === 'assistant' && msg.id) {
                await invoke("delete_message", { messageId: msg.id });
                currentMessages.value.splice(index, 1);
            }

            // 2. 重新触发 AI 回答 (基于最后一条 user 消息)
            await sendMessage("");
        } catch (e) {
            console.error("重新生成失败:", e);
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
            // 🚩 新建文件夹默认置顶 (unshift) 且默认折叠 (is_collapsed: true)
            folders.value.unshift({ id, name, sort_order: 0, is_collapsed: true });

            // 同步折叠状态到数据库
            try {
                await invoke("update_folder_collapsed", { id, collapsed: true });
            } catch (err) {
                console.error("同步文件夹折叠状态失败:", err);
            }
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
        // 🔧 修复：只在真正需要时清空，避免在加载过程中显示空白
        isLoading.value = true;
        try {
            const history = await invoke<any[]>("get_messages", { sessionId });
            console.log("📥 Frontend received messages:", {
                count: history?.length || 0,
                messages: history?.map(m => ({
                    role: m.role,
                    contentLen: m.content.length,
                    hasReasoning: !!m.reasoningContent,  // ✅ 改为 camelCase
                    reasoningLen: m.reasoningContent?.length || 0  // ✅ 改为 camelCase
                }))
            });

            // 打印助手消息的深度思考内容详情
            if (history) {
                history.forEach((m, i) => {
                    if (m.role === "assistant" && m.reasoningContent) {  // ✅ 改为 camelCase
                        console.log(`📥 Assistant message ${i} reasoning content length:`, m.reasoningContent.length);
                        console.log(`📥 Assistant message ${i} reasoning preview:`, m.reasoningContent.substring(0, 100) + "...");
                    }
                });
            }

            // 只在确认是当前会话时才更新消息
            if (activeId.value === sessionId) {
                let newMessages = history && history.length > 0
                    ? history.map(m => ({
                        ...m
                    }))
                    : [{ role: "system", content: "你是一个简洁专业的 AI 助手。" }];

                // 🛡️ 智能合并：如果当前正在生成消息，将正在生成的临时消息追加到历史记录后
                if (isGenerating.value && generatingSessionId.value === sessionId && tempGeneratedMessage.value) {
                    console.log("� [loadMessages] Merging active generation into history");
                    newMessages.push({
                        role: "assistant", // 确保是 assistant
                        content: tempGeneratedMessage.value.content || "",
                        reasoningContent: tempGeneratedMessage.value.reasoning || "",
                        // id 为空表示未保存
                    });
                }

                // 原子性更新
                currentMessages.value = newMessages;
            }
        } catch (err) {
            console.error("获取消息失败:", err);
        } finally {
            isLoading.value = false;
        }
    };

    /**
     * 保存助手回复到数据库
     * @param sessionId 会话ID
     * @param content 助手回复内容
     * @param reasoningContent 深度思考内容 ✅ 改为 camelCase
     */
    const saveAssistantResponse = async (sessionId: string, content: string, reasoningContent: string | null, fileMetadata: string | null = null, searchMetadata: string | null = null) => {  // ✅ 参数改为 camelCase
        console.log("💾 [SAVE] === START SAVING ===");
        console.log("💾 [SAVE] Content length:", content.length);
        console.log("💾 [SAVE] Reasoning content length:", reasoningContent?.length || 0);  // ✅ 改为 camelCase
        console.log("💾 [SAVE] File metadata:", fileMetadata);
        console.log("💾 [SAVE] Search metadata:", searchMetadata);

        const saveParams = {
            sessionId,
            role: "assistant",
            content,
            reasoningContent,  // ✅ 改为 camelCase
            fileMetadata,
            searchMetadata
        };

        console.log("💾 [SAVE] saveParams:", JSON.stringify(saveParams, null, 2));
        console.log("💾 [SAVE] Invoking save_message...");
        const msgId = await invoke<number>("save_message", saveParams);

        // 更新本地消息的 ID
        const lastMsg = currentMessages.value[currentMessages.value.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.id = msgId;
        }
        console.log("💾 [SAVE] save_message completed");
        console.log("💾 [SAVE] === END SAVING ===");
    };

    const sendMessage = async (text: string, fileMetadata: string | null = null, provider: string = 'all') => {
        // 如果 text 为空，则表示“基于当前历史重新生成”，此时要求必须有历史消息
        const isRegeneratingFromHistory = text.trim() === "" && currentMessages.value.length > 0;

        if (!activeId.value || isGenerating.value) return;
        if (!isRegeneratingFromHistory && !text.trim()) return;

        const sessionId = activeId.value;
        isGenerating.value = true;

        // 设置正在生成消息的会话 ID 并清空之前的缓存
        generatingSessionId.value = sessionId;
        pausedChunks.value = { content: [], reasoning: [] };

        try {
            await invoke("reset_ai_generation");

            if (!isRegeneratingFromHistory) {
                const msgId = await invoke<number>("save_message", {
                    sessionId,
                    role: "user",
                    content: text,
                    reasoningContent: null,
                    fileMetadata: fileMetadata,
                    searchMetadata: null
                });

                // 添加到当前消息列表
                currentMessages.value.push({
                    id: msgId,
                    role: "user",
                    content: text,
                    reasoningContent: null,
                    fileMetadata: fileMetadata,
                    searchMetadata: null
                });
            }

            // 添加加载中的助手消息
            currentMessages.value.push({
                role: "assistant",
                content: '',
                reasoningContent: '',
                fileMetadata: null,
                searchMetadata: null
            });

            const onEvent = new Channel<string>();
            let aiFullContent = '';
            let reasoningChunkCount = 0;

            // 监听搜索状态事件
            const unlistenSearch = await listen('search-status', (event: any) => {
                const payload = event.payload;
                const lastMsg = currentMessages.value[currentMessages.value.length - 1];

                if (payload.status === 'searching') {
                    lastMsg.searchStatus = 'searching';
                    lastMsg.searchQuery = payload.query;
                } else if (payload.status === 'done') {
                    lastMsg.searchStatus = 'done';
                    lastMsg.searchMetadata = JSON.stringify(payload.results);
                } else if (payload.status === 'error') {
                    lastMsg.searchStatus = 'error';
                }
            });

            onEvent.onmessage = (data) => {
                if (!isGenerating.value) return;

                // 只要是当前会话就更新（不管视图是否隐藏）
                const isCurrentSession = activeId.value === generatingSessionId.value;
                const lastMsg = currentMessages.value[currentMessages.value.length - 1];

                // 处理内容流
                if (data.startsWith("c:")) {
                    const content = data.substring(2);
                    aiFullContent += content;

                    // 同步更新 tempGeneratedMessage
                    if (tempGeneratedMessage.value) {
                        tempGeneratedMessage.value.content += content;
                    }

                    if (isCurrentSession) {
                        if (lastMsg.content === "__LOADING__") lastMsg.content = "";
                        lastMsg.content += content;
                    }
                }
                // 处理推理流
                else if (data.startsWith("r:")) {
                    const content = data.substring(2);

                    // 同步更新 tempGeneratedMessage
                    if (tempGeneratedMessage.value) {
                        tempGeneratedMessage.value.reasoning += content;
                    }

                    if (isCurrentSession) {
                        if (!lastMsg.reasoningContent) lastMsg.reasoningContent = "";
                        lastMsg.reasoningContent += content;
                    }
                } else if (data.startsWith("data: ")) {
                    console.log(`🧠 [DEBUG] Raw data event: ${data.substring(0, 50)}...`);
                } else {
                    console.log(`🧠 [DEBUG] Unknown event prefix: ${data.substring(0, 10)}`);
                }
            };

            // 准备发送的消息列表（排除加载中的消息）
            const msgsToSend = currentMessages.value.slice(0, -1).map((m) => ({
                role: m.role,
                content: m.content,
                reasoningContent: m.reasoningContent,
                fileMetadata: m.fileMetadata,
                searchMetadata: m.searchMetadata // Include searchMetadata
            }));

            console.log("📤 Messages to send before reasoning:", {
                count: msgsToSend.length,
                useReasoning: useReasoning.value,
                messages: msgsToSend.map(m => ({
                    role: m.role,
                    contentLen: m.content.length,
                    hasReasoning: !!m.reasoningContent  // ✅ 改为 camelCase
                }))
            });

            // 如果启用推理，在最后一条用户消息前添加标记
            if (useReasoning.value || useSearch.value) {
                for (let i = msgsToSend.length - 1; i >= 0; i--) {
                    if (msgsToSend[i].role === "user") {
                        if (useReasoning.value) msgsToSend[i].content = `[REASON]${msgsToSend[i].content}`;
                        if (useSearch.value) {
                            const tag = provider === 'all' ? '[SEARCH]' : `[SEARCH:${provider}]`;
                            msgsToSend[i].content = `${tag}${msgsToSend[i].content}`;
                        }
                        break;
                    }
                }
            }

            console.log("📤 Messages to send after reasoning:", {
                count: msgsToSend.length,
                messages: msgsToSend.map(m => ({
                    role: m.role,
                    contentLen: m.content.length,
                    hasReason: m.content.startsWith('[REASON]'),
                    hasReasoning: !!m.reasoningContent  // ✅ 改为 camelCase
                }))
            });

            // 调用 AI
            try {
                await invoke("ask_ai", {
                    msg: msgsToSend,
                    onEvent,
                });
            } finally {
                unlistenSearch();
            }

            console.log("🧠 [FINAL] AI generation completed");
            console.log("🧠 [FINAL] aiFullContent length:", aiFullContent.length);
            console.log("🧠 [FINAL] reasoningChunkCount:", reasoningChunkCount);

            // 获取最后一条消息的深度思考内容
            const lastMsg = currentMessages.value[currentMessages.value.length - 1];
            console.log("🧠 [DEBUG] Last message object:", {
                role: lastMsg.role,
                contentLength: lastMsg.content.length,
                hasReasoningContent: !!lastMsg.reasoningContent,  // ✅ 改为 camelCase
                reasoningContentLength: lastMsg.reasoningContent?.length || 0,  // ✅ 改为 camelCase
                hasSearchMetadata: !!lastMsg.searchMetadata,
                searchMetadataLength: lastMsg.searchMetadata?.length || 0
            });

            const finalReasoningContent = lastMsg.reasoningContent || null;
            const finalSearchMetadata = lastMsg.searchMetadata || null;

            // 保存助手回复 - 只在生成完成后保存完整的 AI 回复
            await saveAssistantResponse(sessionId, aiFullContent, finalReasoningContent, null, finalSearchMetadata);

            // 自动总结标题
            const msgCount = currentMessages.value.filter(m => m.content !== "__LOADING__").length;
            if (msgCount >= 5 && activeSession.value?.title === "新对话") {
                autoSummaryTitle(sessionId);
            }
            console.log("💾 [SAVE] === END SAVING ===");
        } catch (error) {
            console.error("对话失败:", error);
        } finally {
            isGenerating.value = false;
            // 清空生成会话状态和缓存
            generatingSessionId.value = null;
            pausedChunks.value = { content: [], reasoning: [] };
        }
    };

    const stopGeneration = async () => {
        isGenerating.value = false;
        // 清空生成会话状态和缓存
        generatingSessionId.value = null;
        pausedChunks.value = { content: [], reasoning: [] };
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
            const prompt = "请总结以上对话的标题(8-10字)。直接返回标题文字，不要代码，不要标点符号。";

            const filteredMsgs = currentMessages.value.filter(m => m.content !== "__LOADING__");
            // 取前几轮对话 + prompt
            const summaryMsgs = [
                ...filteredMsgs.slice(1, 5),
                { role: "user", content: prompt }
            ];

            console.log("=== [Blocking] 请求后端生成标题 ===");

            // 2. ⚡️ 核心改动:使用 generate_title，不再使用 Channel 流式接收
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
        // 使用 map 确保我们只取必要的字段，并且维持传入的物理顺序
        historyList.value = [...newList];
        const orders: [string, number][] = historyList.value.map((s, index) => [s.id, index]);
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
            await chatApi.updateFoldersOrder(orders);
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
        generatingSessionId,
        isChatViewActive,
        isLoading,
        useReasoning,
        useSearch,
        searchProvider,
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
        clearMessages,
        deleteMessageAction,
        editMessageAction,
        regenerateAction,
        stopGeneration,
        updateSessionScroll,
        reorderSessions,
        reorderFolders,
        toggleFolder,
        applyPausedChunks,
        setChatViewActive,
    };
});