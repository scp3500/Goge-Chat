import { type Ref, unref } from 'vue';
import { invoke, Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ChatSession } from '../../api/chat';
import type { PausedChunks } from './state';
import { useConfigStore } from '../config';

interface MessageState {
    activeId: Ref<string | null>;
    currentMessages: Ref<any[]>;
    isGenerating: Ref<boolean>;
    generatingSessionId: Ref<string | null>;
    pausedChunks: Ref<PausedChunks>;
    streamQueue: Ref<string[]>;
    isProcessingQueue: Ref<boolean>;
    tempGeneratedMessage: Ref<{ content: string, reasoning: string } | null>;
    isLoading: Ref<boolean>;
    useReasoning: Ref<boolean>;
    useSearch: Ref<boolean>;
    // activeSession is computed, but compatible with Ref
    activeSession: Ref<ChatSession | null>;
    isChatViewActive: Ref<boolean>;
}

interface MessageActionsDependencies {
    renameSession: (id: string, title: string) => Promise<void>;
}

export function useMessageActions(state: MessageState, deps: MessageActionsDependencies) {
    const {
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
    } = state;

    const configStore = useConfigStore();

    const processStreamQueue = () => {
        if (isProcessingQueue.value) return;
        isProcessingQueue.value = true;

        const animate = () => {
            // Stop if generation stopped and queue empty
            if (!isGenerating.value && streamQueue.value.length === 0) {
                isProcessingQueue.value = false;
                return;
            }

            if (streamQueue.value.length > 0) {
                const isCurrentSession = activeId.value === generatingSessionId.value;
                const lastMsg = currentMessages.value[currentMessages.value.length - 1];

                // ⚡️ Adaptive Speed Control
                const charsPerFrame = Math.max(1, Math.floor(streamQueue.value.length / 4));
                const chunk = streamQueue.value.splice(0, charsPerFrame).join('');

                if (isCurrentSession) {
                    if (lastMsg.content === "__LOADING__") lastMsg.content = "";
                    lastMsg.content += chunk;
                }

                if (tempGeneratedMessage.value) {
                    tempGeneratedMessage.value.content += chunk;
                }
            }

            requestAnimationFrame(animate);
        };
        requestAnimationFrame(animate);
    };

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

    const setChatViewActive = (active: boolean) => {
        isChatViewActive.value = active;
        // 如果重新激活聊天视图，应用缓存的消息块
        if (active) {
            applyPausedChunks();
        }
    };

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
                    hasReasoning: !!m.reasoningContent,
                    reasoningLen: m.reasoningContent?.length || 0
                }))
            });

            // 只在确认是当前会话时才更新消息
            if (activeId.value === sessionId) {
                let newMessages = history && history.length > 0
                    ? history.map(m => ({
                        ...m
                    }))
                    : [{ role: "system", content: "你是一个简洁专业的 AI 助手。" }];

                // 🛡️ 智能合并：如果当前正在生成消息，将正在生成的临时消息追加到历史记录后
                if (isGenerating.value && generatingSessionId.value === sessionId && tempGeneratedMessage.value) {
                    console.log(" [loadMessages] Merging active generation into history");
                    newMessages.push({
                        role: "assistant", // 确保是 assistant
                        model: configStore.settings.selectedModelId,
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

    const saveAssistantResponse = async (sessionId: string, content: string, reasoningContent: string | null, fileMetadata: string | null = null, searchMetadata: string | null = null) => {
        console.log("💾 [SAVE] === START SAVING ===");
        console.log("💾 [SAVE] Content length:", content.length);
        console.log("💾 [SAVE] Reasoning content length:", reasoningContent?.length || 0);
        console.log("💾 [SAVE] File metadata:", fileMetadata);
        console.log("💾 [SAVE] Search metadata:", searchMetadata);

        const saveParams = {
            sessionId,
            role: "assistant",
            model: configStore.settings.selectedModelId,
            content,
            reasoningContent,
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
            lastMsg.model = configStore.settings.selectedModelId;
        }
        console.log("💾 [SAVE] save_message completed");
        console.log("💾 [SAVE] === END SAVING ===");
    };

    /**
     * ⚡️ 架构重构：非流式标题生成 (Blocking Mode)
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
                await deps.renameSession(sessionId, finalTitle);
            }

        } catch (e) {
            console.error("自动总结标题失败 (请检查 Rust 后端是否实现了 generate_title):", e);
        }
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
        streamQueue.value = []; // Clear queue at start

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
            console.log("🤖 [sendMessage] Creating assistant message with model:", configStore.settings.selectedModelId);
            currentMessages.value.push({
                role: "assistant",
                model: configStore.settings.selectedModelId,
                content: '__LOADING__',
                reasoningContent: '',
                fileMetadata: null,
                searchMetadata: null
            });

            const onEvent = new Channel<string>();
            let aiFullContent = '';

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

                    // 🌊 Push to smooth queue instead of direct rendering
                    for (const char of content) {
                        streamQueue.value.push(char);
                    }

                    // Kickstart the processor if idle
                    processStreamQueue();
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

            // 获取当前预设
            const activePreset = configStore.settings.presets.find(p => p.id === configStore.settings.defaultPresetId);

            // 准备发送的消息列表（排除加载中的消息）
            let msgsToSend = currentMessages.value.slice(0, -1).map((m) => ({
                role: m.role,
                content: m.content,
                reasoningContent: m.reasoningContent,
                fileMetadata: m.fileMetadata,
                searchMetadata: m.searchMetadata
            }));

            // 注入系统提示词 (如果预设中有且不是正在生成历史)
            if (activePreset?.systemPrompt && activePreset.systemPrompt.trim()) {
                // 如果第一条不是系统提示词，或者第一条系统提示词和预设的不一样，则添加/替换
                if (msgsToSend.length > 0 && msgsToSend[0].role !== 'system') {
                    msgsToSend.unshift({
                        role: 'system',
                        content: activePreset.systemPrompt,
                        reasoningContent: null,
                        fileMetadata: null,
                        searchMetadata: null
                    });
                } else if (msgsToSend.length > 0 && msgsToSend[0].role === 'system') {
                    // 如果已经有系统提示词且内容不同，则替换（或者你可以选择追加）
                    // 这里的策略是：如果预设有系统提示词，则始终确保第一条是该预设的系统提示词
                    msgsToSend[0].content = activePreset.systemPrompt;
                }
            }

            console.log("📤 Final messages to send:", {
                count: msgsToSend.length,
                preset: activePreset?.name,
                temperature: activePreset?.temperature,
                maxTokens: activePreset?.maxTokens
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

            // 调用 AI
            try {
                await invoke("ask_ai", {
                    msg: msgsToSend,
                    onEvent,
                    temperature: activePreset?.temperature,
                    max_tokens: activePreset?.maxTokens
                });
            } finally {

                unlistenSearch();
            }

            console.log("🧠 [FINAL] AI generation completed");

            // 获取最后一条消息的深度思考内容
            const lastMsg = currentMessages.value[currentMessages.value.length - 1];

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
            // generatingSessionId.value = null; // DO NOT clear here. It breaks the "isCurrentSession" check in the queue loop for the last few chars.
            pausedChunks.value = { content: [], reasoning: [] };
            // streamQueue.value = []; // DO NOT clear queue here, let it drain naturally
        }
    };

    const stopGeneration = async () => {
        isGenerating.value = false;
        // 清空生成会话状态和缓存
        generatingSessionId.value = null;
        pausedChunks.value = { content: [], reasoning: [] };
        streamQueue.value = []; // Clear queue on stop
        try { await invoke("stop_ai_generation"); } catch (err) { console.error(err); }
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
            await sendMessage(""); // 传空字符串触发逻辑
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

    return {
        processStreamQueue,
        applyPausedChunks,
        setChatViewActive,
        loadMessages,
        sendMessage,
        stopGeneration,
        clearMessages,
        deleteMessageAction,
        editMessageAction,
        regenerateAction,
        saveAssistantResponse,
        autoSummaryTitle
    };
}
