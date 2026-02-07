import { type Ref, unref, watch } from 'vue';
import { invoke, Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ChatSession } from '../../api/chat';
import type { PausedChunks } from './state';
import { useConfigStore } from '../config';
import { DEFAULT_SYSTEM_PROMPT } from '../../constants/prompts';
import { Logger } from '../../utils/logger';

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
    let isInternalSync = false;

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

                // ⚡️ 优化：更激进的批量处理策略
                // 当队列较长时,一次性处理更多字符,减少渲染次数
                let charsPerFrame;
                if (streamQueue.value.length > 100) {
                    charsPerFrame = Math.min(50, Math.floor(streamQueue.value.length / 2)); // 大量积压时快速消化
                } else if (streamQueue.value.length > 20) {
                    charsPerFrame = Math.min(20, Math.floor(streamQueue.value.length / 3)); // 中等积压
                } else {
                    charsPerFrame = Math.max(3, Math.floor(streamQueue.value.length / 2)); // 少量时保持流畅感
                }

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
        // 🔧 修复：立即清空当前消息，确保淡入淡出动画有一个干净的起点
        currentMessages.value = [];
        isLoading.value = true;
        try {
            const history = await invoke<any[]>("get_messages", { sessionId });
            /*
            console.log("📥 Frontend received messages:", {
                count: history?.length || 0,
                messages: history?.map(m => ({
                    role: m.role,
                    contentLen: m.content.length,
                    hasReasoning: !!m.reasoningContent,
                    reasoningLen: m.reasoningContent?.length || 0
                }))
            });
            */

            // 只在确认是当前会话时才更新消息
            if (activeId.value === sessionId) {
                let newMessages = history && history.length > 0
                    ? history.map(m => ({
                        ...m,
                        providerId: m.provider // 🟢 Fix: Map DB provider field to frontend providerId
                    }))
                    : [];

                // 🛡️ 智能合并：如果当前正在生成消息，将正在生成的临时消息追加到历史记录后
                if (isGenerating.value && generatingSessionId.value === sessionId && tempGeneratedMessage.value) {
                    // console.log(" [loadMessages] Merging active generation into history");
                    newMessages.push({
                        role: "assistant", // 确保是 assistant
                        model: configStore.settings.selectedModelId,
                        providerId: configStore.settings.defaultProviderId, // 🟢 Fix
                        content: tempGeneratedMessage.value.content || "",
                        reasoningContent: tempGeneratedMessage.value.reasoning || "",
                        // id 为空表示未保存
                    });
                }

                // 原子性更新
                currentMessages.value = newMessages;

                // 🔄 同步会话配置到全局状态
                const session = activeSession.value;
                if (session) {
                    isInternalSync = true;
                    // 如果会话有特定配置，则使用；否则回滚到全局默认值
                    configStore.settings.defaultPresetId = session.preset_id || configStore.settings.globalPresetId;

                    const targetModelId = session.model_id || configStore.settings.globalModelId;
                    configStore.settings.selectedModelId = targetModelId;

                    // 🟢 Fix: auto-detect provider based on model ID
                    // Many users (and the code) forget to save/sync the provider ID, leading to "DeepSeek" default.
                    // We reverse-lookup the provider that owns this model.
                    if (targetModelId) {
                        const allProviders = configStore.settings.providers || [];
                        const ownerProvider = allProviders.find(p =>
                            p.models?.some((m: any) => {
                                const mId = typeof m === 'string' ? m : m.id;
                                return mId === targetModelId;
                            })
                        );

                        if (ownerProvider) {
                            console.log(`[loadMessages] Auto-detected provider for model ${targetModelId}:`, ownerProvider.id);
                            configStore.settings.defaultProviderId = ownerProvider.id;
                        }
                    }

                    // 🟢 Fix: Do NOT overwrite global defaultSystemPrompt with session specific prompt.
                    // The global setting should only be changed by the user in Settings.
                    // configStore.settings.defaultSystemPrompt = session.system_prompt || configStore.settings.defaultSystemPrompt;
                    setTimeout(() => { isInternalSync = false; }, 0);
                }
            }
        } catch (err) {
            console.error("获取消息失败:", err);
        } finally {
            isLoading.value = false;
        }
    };

    const saveAssistantResponse = async (sessionId: string, content: string, reasoningContent: string | null, fileMetadata: string | null = null, searchMetadata: string | null = null, explicitModelId?: string, explicitProviderId?: string) => {
        /*
        console.log("💾 [SAVE] === START SAVING ===");
        console.log("💾 [SAVE] Content length:", content.length);
        console.log("💾 [SAVE] Reasoning content length:", reasoningContent?.length || 0);
        console.log("💾 [SAVE] File metadata:", fileMetadata);
        console.log("💾 [SAVE] Search metadata:", searchMetadata);
        */

        const targetModel = explicitModelId || configStore.settings.selectedModelId;
        const targetProvider = explicitProviderId || configStore.settings.defaultProviderId;

        const saveParams = {
            sessionId,
            role: "assistant",
            model: targetModel,
            provider: targetProvider, // 🟢 Fix: Pass provider to backend
            content,
            reasoningContent,
            fileMetadata,
            searchMetadata
        };

        // console.log("💾 [SAVE] saveParams:", JSON.stringify(saveParams, null, 2));
        // console.log("💾 [SAVE] Invoking save_message...");
        const msgId = await invoke<number>("save_message", saveParams);

        // 更新本地消息的 ID
        const lastMsg = currentMessages.value[currentMessages.value.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.id = msgId;
            lastMsg.model = targetModel;
            lastMsg.providerId = targetProvider;
        }
        // console.log("💾 [SAVE] save_message completed");
        // console.log("💾 [SAVE] === END SAVING ===");
    };

    /**
     * ⚡️ 架构重构：非流式标题生成 (Blocking Mode)
     */
    const autoSummaryTitle = async (sessionId: string) => {
        try {
            console.log(`[Title] 开始为会话 ${sessionId} 生成标题...`);
            const prompt = "请总结以上对话的标题(8-10字)。直接返回标题文字，不要代码，不要标点符号。";

            const filteredMsgs = currentMessages.value.filter(m => m.content && m.content !== "__LOADING__");
            if (filteredMsgs.length < 2) {
                console.log("[Title] 消息太少，跳过总结");
                return;
            }

            // 确保包含用户消息。如果第一条是 system，则从第 1 条开始取；否则从第 0 条开始。
            const startIdx = filteredMsgs[0]?.role === 'system' ? 1 : 0;
            const summaryMsgs = [
                ...filteredMsgs.slice(startIdx, startIdx + 4).map(m => ({
                    role: m.role,
                    content: m.content
                })),
                { role: "user", content: prompt }
            ];

            const rawTitle = await invoke<string>("generate_title", {
                msg: summaryMsgs,
                explicitProviderId: configStore.settings.defaultProviderId,
                explicitModelId: configStore.settings.selectedModelId
            });

            console.log("[Title] 后端返回原始标题:", rawTitle);

            // 3. 清理标题（去除引号、换行、末尾标点）
            let finalTitle = rawTitle.trim()
                .replace(/^["'“”«「]|["'“”»」]$/g, "")
                .replace(/[。！!？?]$/, "")
                .trim();

            if (finalTitle.length > 15) {
                finalTitle = finalTitle.substring(0, 15);
            }

            // 5. 应用更新
            const currentSession = activeSession.value;
            const oldTitle = currentSession?.title || "";

            if (finalTitle && finalTitle.length > 0 && finalTitle !== oldTitle && !["新对话", "默认会话", "New Chat"].includes(finalTitle)) {
                console.log(`[Title] 标题变更: "${oldTitle}" -> "${finalTitle}"`);
                await deps.renameSession(sessionId, finalTitle);
            } else {
                console.log("[Title] 标题无变化或 AI 返回了默认值，跳过更新");
            }

        } catch (e) {
            console.error("自动总结标题失败:", e);
        }
    };

    const sendMessage = async (text: string, fileMetadata: string | null = null, provider: string = 'all', mentions: any[] = []) => {
        // 如果 text 为空，则表示“基于当前历史重新生成”，此时要求必须有历史消息
        const isRegeneratingFromHistory = text.trim() === "" && currentMessages.value.length > 0;

        if (!activeId.value || isGenerating.value) return;
        if (!isRegeneratingFromHistory && !text.trim()) return;

        const startTime = Date.now();
        Logger.stage('Chat Sequence Started');

        const sessionId = activeId.value;
        const currentMode = configStore.settings.chatMode?.enabled ? "Social" : "Standard";

        isGenerating.value = true;

        // 设置正在生成消息的会话 ID 并清空之前的缓存
        generatingSessionId.value = sessionId;
        pausedChunks.value = { content: [], reasoning: [] };
        streamQueue.value = []; // Clear queue at start

        const isStreamEnabled = configStore.settings.chatMode?.enabled
            ? configStore.settings.chatMode.enableStream
            : configStore.settings.enableStream;

        try {
            // await invoke("reset_ai_generation"); // Moved inside loop

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

            // --- 确定要调用的模型列表 ---
            let modelsToCall = mentions && mentions.length > 0
                ? mentions
                : [{ id: configStore.settings.selectedModelId, providerId: configStore.settings.defaultProviderId }];

            // 🛡️ Safety filter
            modelsToCall = modelsToCall.filter(m => m && m.id);

            if (modelsToCall.length === 0) {
                modelsToCall = [{ id: configStore.settings.selectedModelId, providerId: configStore.settings.defaultProviderId }];
            }

            Logger.info(`Models to call (Count ${mentions?.length || 0}): ${modelsToCall.map(m => m.id).join(', ')}`);

            for (const modelInfo of modelsToCall) {
                // 🔄 Reset generation state before each model call
                await invoke("reset_ai_generation");

                const currentModelId = modelInfo.id;
                const currentProviderId = modelInfo.providerId;

                // 添加加载中的助手消息
                currentMessages.value.push({
                    role: "assistant",
                    model: currentModelId,
                    providerId: currentProviderId,
                    content: '__LOADING__',
                    reasoningContent: '',
                    fileMetadata: null,
                    searchMetadata: null
                });

                const onEvent = new Channel<string>();
                let aiFullContent = '';
                let ttft = 0; // Time to first token
                let searchStartTime = 0;
                let memoryStartTime = 0;

                // 监听搜索状态事件
                const unlistenSearch = await listen('search-status', (event: any) => {
                    const payload = event.payload;
                    const lastMsg = currentMessages.value[currentMessages.value.length - 1];

                    if (payload.status === 'searching') {
                        searchStartTime = Date.now();
                        Logger.info(`Searching: ${payload.query}`);
                        lastMsg.searchStatus = 'searching';
                        lastMsg.searchQuery = payload.query;
                    } else if (payload.status === 'done') {
                        const searchDuration = Date.now() - searchStartTime;
                        Logger.success('Search completed', searchDuration, { results: payload.results?.length });
                        lastMsg.searchStatus = 'done';
                        lastMsg.searchMetadata = JSON.stringify(payload.results);
                    } else if (payload.status === 'error') {
                        Logger.error('Search failed', payload.message);
                        lastMsg.searchStatus = 'error';
                    }
                });

                // 监听记忆检索事件
                const unlistenMemory = await listen('memory-status', (event: any) => {
                    const payload = event.payload;
                    if (payload.status === 'searching') {
                        memoryStartTime = Date.now();
                        Logger.info('Retrieving memory context...');
                    } else if (payload.status === 'done') {
                        const memoryDuration = Date.now() - memoryStartTime;
                        Logger.success('Memory retrieval completed', memoryDuration, { hasContext: payload.has_context });
                    }
                });

                onEvent.onmessage = (data) => {
                    if (!isGenerating.value) return;

                    if (ttft === 0 && (data.startsWith("c:") || data.startsWith("r:"))) {
                        ttft = Date.now() - startTime;
                        Logger.timing('Time to First Token (TTFT)', ttft);
                    }

                    // 只要是当前会话就更新（不管视图是否隐藏）
                    const isCurrentSession = activeId.value === generatingSessionId.value;
                    const lastMsg = currentMessages.value[currentMessages.value.length - 1];

                    // 处理内容流
                    if (data.startsWith("c:")) {
                        const content = data.substring(2);
                        aiFullContent += content;

                        // 🌊 Streaming Control
                        if (isStreamEnabled) {
                            streamQueue.value.push(...content.split(''));
                            if (!isProcessingQueue.value) {
                                processStreamQueue();
                            }
                        } else {
                            if (isCurrentSession && lastMsg) {
                                if (lastMsg.content === "__LOADING__") lastMsg.content = "";
                                lastMsg.content += content;
                            }
                        }
                    }
                    // 处理推理流
                    else if (data.startsWith("r:")) {
                        const content = data.substring(2);
                        if (isCurrentSession) {
                            if (!lastMsg.reasoningContent) lastMsg.reasoningContent = "";
                            lastMsg.reasoningContent += content;
                        }
                    }
                };

                // 准备发送的消息列表（排除当前的加载中消息）
                let msgsToSend = currentMessages.value.slice(0, -1).map((m) => ({
                    role: m.role,
                    content: m.content,
                    reasoningContent: m.reasoningContent,
                    fileMetadata: m.fileMetadata,
                    searchMetadata: m.searchMetadata,
                    mode: currentMode,
                    roleId: "default"
                }));

                // 获取并注入系统提示词
                const activePreset = configStore.settings.presets.find(p => p.id === configStore.settings.defaultPresetId);
                let presetPrompt = activePreset?.id === 'default_preset' ? "" : activePreset?.systemPrompt;
                const finalSystemPrompt = presetPrompt || configStore.settings.defaultSystemPrompt || DEFAULT_SYSTEM_PROMPT;

                if (msgsToSend.length > 0 && msgsToSend[0].role !== 'system') {
                    msgsToSend.unshift({
                        role: 'system',
                        content: finalSystemPrompt,
                        reasoningContent: null,
                        fileMetadata: null,
                        searchMetadata: null,
                        mode: currentMode,
                        roleId: "default"
                    });
                } else if (msgsToSend.length > 0 && msgsToSend[0].role === 'system') {
                    msgsToSend[0].content = finalSystemPrompt;
                }

                // 注入会话特定的系统提示词
                const sessionSpecificPrompt = activeSession.value?.system_prompt;
                if (sessionSpecificPrompt) {
                    if (msgsToSend[0].role === 'system') msgsToSend[0].content = sessionSpecificPrompt;
                    else msgsToSend.unshift({ role: 'system', content: sessionSpecificPrompt, reasoningContent: null, fileMetadata: null, searchMetadata: null, mode: currentMode, roleId: "default" });
                }

                // 添加推理/搜索标记
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
                    const timeoutPromise = new Promise((_, reject) => {
                        setTimeout(() => reject(new Error("Request timed out")), 60000);
                    });

                    await Promise.race([
                        invoke("ask_ai", {
                            msg: msgsToSend,
                            onEvent,
                            temperature: activePreset?.temperature,
                            max_tokens: activePreset?.maxTokens,
                            explicitProviderId: currentProviderId,
                            explicitModelId: currentModelId,
                            stream: isStreamEnabled
                        }),
                        timeoutPromise
                    ]);

                    if (!isStreamEnabled) {
                        const lastMsg = currentMessages.value[currentMessages.value.length - 1];
                        if (lastMsg && lastMsg.role === 'assistant') {
                            if (lastMsg.content === "__LOADING__") lastMsg.content = "";
                            lastMsg.content = aiFullContent;
                        }
                    }
                } finally {
                    unlistenSearch();
                    unlistenMemory();
                }

                // 🛡️ Fix bug: Ensure loading state is cleared even if AI returns no content
                const lastMsg = currentMessages.value[currentMessages.value.length - 1];
                if (lastMsg.content === '__LOADING__') {
                    if (aiFullContent) {
                        lastMsg.content = aiFullContent;
                    } else {
                        console.warn(`[sendMessage] Model ${currentModelId} returned no content.`);
                        lastMsg.content = ""; // Clear loading state
                    }
                }

                // 保存助手回复
                await saveAssistantResponse(sessionId, aiFullContent, lastMsg.reasoningContent || null, null, lastMsg.searchMetadata || null, currentModelId, currentProviderId);

                // 如果用户停止了，跳过后续模型
                if (!isGenerating.value) break;
            }

            // 自动总结标题
            const msgCount = currentMessages.value.filter(m => m.content && m.content !== "__LOADING__").length;
            const isDefaultTitle = !activeSession.value?.title ||
                ["新对话", "默认会话", "New Chat", "默认对话"].includes(activeSession.value?.title);

            if (msgCount >= 2 && isDefaultTitle) {
                autoSummaryTitle(sessionId);
            }
        } catch (error: any) {
            console.error("对话失败:", error);
            const lastMsg = currentMessages.value[currentMessages.value.length - 1];
            if (lastMsg && lastMsg.role === 'assistant' && lastMsg.content === '__LOADING__') {
                let errorMsg = error.message || String(error);
                if (errorMsg.includes('timed out')) errorMsg = '请求超时 (60s)，请检查网络或稍后重试。';
                else if (errorMsg.includes('quota') || errorMsg.includes('429')) errorMsg = '请求速率超过限制或配额不足。';

                lastMsg.content = '';
                lastMsg.error = { message: errorMsg, type: 'error' };
            }
        } finally {
            isGenerating.value = false;
            pausedChunks.value = { content: [], reasoning: [] };
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
    // 🕵️ 实时同步监听：当用户在 UI 修改模型/预设时，如果当前有活跃会话，立即持久化
    watch(
        [() => configStore.settings.selectedModelId, () => configStore.settings.defaultPresetId],
        async ([newModel, newPreset]) => {
            if (isInternalSync || !activeId.value) return;

            console.log("🛠️ 检测到 UI 配置变更，正在同步至会话:", activeId.value);
            try {
                await invoke("update_session_config", {
                    id: activeId.value,
                    presetId: newPreset,
                    modelId: newModel,
                    systemPrompt: activeSession.value?.system_prompt || null
                });

                // 同步本地内存状态
                if (activeSession.value) {
                    activeSession.value.preset_id = newPreset;
                    activeSession.value.model_id = newModel;
                }
            } catch (e) {
                console.error("同步会话配置失败:", e);
            }
        }
    );

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
