<script setup>
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from 'vue';
import { invoke, Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useConfigStore } from "../../stores/config";
import { useChatStore } from "../../stores/chat";
import MessageList from "./MessageList.vue";
import ChatInput from "./ChatInput.vue";
import { getDefaultAvatar, resolveSocialAvatar } from "../../utils/social";
import { convertFileSrc } from "@tauri-apps/api/core";

const resolveAvatarSrc = (path, id) => {
  // If path exists, resolve it; otherwise use default avatar directly
  return path ? resolveSocialAvatar(path) : getDefaultAvatar(id);
};

const props = defineProps({
  activeContact: {
    type: Object,
    required: true
  }
});

const configStore = useConfigStore();
const chatStore = useChatStore();
const messages = ref([]);
const isGenerating = ref(false);
const isTyping = ref(false); // 🎭 Immersive mode typing indicator
const messageListRef = ref(null);
const isLoadingMore = ref(false); // ⏳ Loading state
const allLoaded = ref(false);     // 🏁 End of history
const PAGE_SIZE = 20;

// ⚡️ Load recent messages initially
const loadMessages = async (contactId) => {
  try {
    messages.value = []; // Clear current
    isInitialScrollDone.value = false; // 🙈 Hide immediately
    allLoaded.value = false;
    
    // Fetch last N messages
    const history = await invoke("get_recent_social_messages", { 
      contactId,
      sessionId: chatStore.activeSocialSessionId, // ✨ Pass Session ID
      limit: PAGE_SIZE 
    });
    
    messages.value = history;
    if (history.length < PAGE_SIZE) {
      allLoaded.value = true;
    }
    
    // 🛡️ Empty state: Show immediately since there's no scrolling needed
    if (history.length === 0) {
        isInitialScrollDone.value = true;
    }
    
    triggerScroll('auto'); // ⚡️ Scroll immediately (polling will handle timing)
  } catch (e) {
    console.error("Failed to load social messages:", e);
  }
};

// 📜 Load more older messages (Pagination)
const loadMoreMessages = async () => {
  if (isLoadingMore.value || allLoaded.value || !props.activeContact?.id) return;
  
  const oldestMsgId = messages.value[0]?.id;
  if (!oldestMsgId) return;

  try {
    isLoadingMore.value = true;
    const startTime = Date.now(); // ⏱️ Start timer
    
    // Save scroll height BEFORE loading to restore position
    const listEl = messageListRef.value?.$el?.querySelector('.message-list-scroll');
    const oldScrollHeight = listEl?.scrollHeight || 0;
    const oldScrollTop = listEl?.scrollTop || 0;

    const olderMessages = await invoke("get_social_messages_paginated", { 
      contactId: props.activeContact.id,
      sessionId: chatStore.activeSocialSessionId, // ✨ Pass Session ID
      limit: PAGE_SIZE,
      beforeId: oldestMsgId
    });

    // ⏳ Ensure minimum spinner visibility (300ms) for smoother UX
    const elapsed = Date.now() - startTime;
    if (elapsed < 300) {
      await new Promise(resolve => setTimeout(resolve, 300 - elapsed));
    }

    isLoadingMore.value = false; // 💡 Hide spinner BEFORE measuring height to avoid jump

    if (olderMessages.length > 0) {
      messages.value = [...olderMessages, ...messages.value];
    } else {
      allLoaded.value = true;
    }

    // 📍 Restore scroll position
    await nextTick();
    if (listEl) {
      const newScrollHeight = listEl.scrollHeight;
      const heightDiff = newScrollHeight - oldScrollHeight;
      // If we were at top (scrollTop=0), we want to stay at the same relative position
      listEl.scrollTop = oldScrollTop + heightDiff; 
    }

  } catch (e) {
    console.error("Failed to load more messages:", e);
    isLoadingMore.value = false;
  }
};

const isInitialScrollDone = ref(true); // 🛡️ Default to visible to avoid "forever empty" bug

const triggerScroll = async (behavior = 'auto', attempt = 0) => {
  await nextTick();
  
  // 🛡️ Retry logic: If ref is missing (e.g. during transition), wait and retry
  if (!messageListRef.value) {
      if (attempt < 20) { // Try for up to ~1 second (50ms * 20)
          setTimeout(() => triggerScroll(behavior, attempt + 1), 50);
      } else {
          // Give up after timeout to prevent infinite hidden state
          isInitialScrollDone.value = true;
      }
      return;
  }
  
  // Ref exists, perform scroll
  if (messageListRef.value.scrollToBottom) {
    messageListRef.value.scrollToBottom(behavior);
    
    // Reveal list if hidden
    if (!isInitialScrollDone.value) {
        // Use double requestAnimationFrame to ensure rendering is complete
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                isInitialScrollDone.value = true;
            });
        });
    }
  }
};

const activeSessionTitle = ref("");

// 🆕 Initialize Session Logic
const initSessions = async (contactId) => {
  try {
    const sessions = await invoke("get_social_sessions", { contactId });
    
    if (sessions.length > 0) {
      // 1. Prioritize currently persisted active session
      let targetSession = null;
      if (chatStore.activeSocialSessionId) {
          targetSession = sessions.find(s => s.id === chatStore.activeSocialSessionId);
      }
      
      // 2. Fallback to most recent if persisted one is invalid or not set
      if (!targetSession) {
          targetSession = sessions[0];
          chatStore.updateSocialSessionId(targetSession.id);
      }
      
      activeSessionTitle.value = targetSession.title;
    } else {
      // No sessions, create default
      const newId = await invoke("create_social_session", { contactId, title: "默认会话" });
      chatStore.updateSocialSessionId(newId);
      activeSessionTitle.value = "默认会话";
    }
  } catch(e) {
    console.error("Session init failed:", e);
  }
};

// 🧠 记忆同步控制器 (Memory Sync Controller)
const activeSyncSessions = new Set();
const hasNewMessages = ref(false); // 追踪当前会话是否产生了新交互

const syncCurrentMemoryOnLeave = async (contactSnapshot, sessionId) => {
    if (!contactSnapshot || !sessionId) return;
    
    // 🛡️ [限制修复]：如果当前会话没有产生任何新消息，则跳过同步，避免垃圾提取
    if (!hasNewMessages.value) {
        console.log(`[Memory] 会话 ${sessionId} 无新交互，跳过结算`);
        return;
    }
    
    // 如果该会话已经在同步中，不要重复触发
    if (activeSyncSessions.has(sessionId)) return;
    
    activeSyncSessions.add(sessionId);
    try {
        const strRoleId = String(contactSnapshot.id);
        const intSessionId = parseInt(sessionId, 10);
        
        console.warn(`🚀 [Memory] 正在执行结算同步 | 角色: ${contactSnapshot.name} (ID: ${strRoleId}) | 会话: ${intSessionId}`);
        
        await invoke("trigger_fact_sync", {
            sessionId: intSessionId,
            roleId: strRoleId,
            mode: "Social"
        });
        
        console.log(`✅ [Memory] 同步请求已确认`);
    } catch (e) {
        console.error(`❌ [Memory] 同步异常:`, e);
    } finally {
        activeSyncSessions.delete(sessionId);
    }
};

// 追踪“当前正处于稳定状态”的上下文
let lastActiveContext = { contact: null, sessionId: null };

// --- 核心逻辑：统一离场监控 ---

// 1. 组件初始化/销毁钩子
let typingUnlisten = null;
let retractionUnlisten = null;
let newMessageUnlisten = null;

onMounted(async () => {
    console.log("🟢 [SocialChat] 进入聊天容器");
    
    // 🎭 Listen for immersive mode events
    try {
        typingUnlisten = await listen('typing-status', (event) => {
            const { contactId, isTyping: typing } = event.payload;
            if (contactId === props.activeContact?.id) {
                isTyping.value = typing;
            }
        });
        
        retractionUnlisten = await listen('message-retracted', (event) => {
            const { messageId } = event.payload;
            const index = messages.value.findIndex(m => m.id === messageId);
            if (index !== -1) {
                messages.value.splice(index, 1);
            }
        });
        
        // 🆕 Listen for new messages from immersive mode
        newMessageUnlisten = await listen('new-social-message', (event) => {
            const { messageId, contactId, sessionId, role, content, createdAt } = event.payload;
            
            console.log(`📨 [new-social-message] 收到消息:`, {
                messageId,
                contactId,
                sessionId,
                role,
                content: content.substring(0, 50),
                currentContact: props.activeContact?.id,
                currentSession: chatStore.activeSocialSessionId
            });
            
            // Only add if it's for the current contact and session
            if (contactId === props.activeContact?.id && sessionId === chatStore.activeSocialSessionId) {
                // Check if message already exists (avoid duplicates)
                const exists = messages.value.find(m => m.id === messageId);
                if (!exists) {
                    console.log(`✅ [new-social-message] 添加消息到当前会话 (role: ${role})`);
                    messages.value.push({
                        id: messageId,
                        role,
                        content,
                        created_at: createdAt
                    });
                    
                    // Auto-scroll to new message
                    nextTick(() => {
                        triggerScroll('smooth');
                    });
                } else {
                    console.log(`⚠️ [new-social-message] 消息已存在，跳过`);
                }
            } else {
                console.log(`❌ [new-social-message] 消息不属于当前会话，忽略`);
            }
        });
        
        console.log("🎭 [Immersive] Event listeners registered");
    } catch (e) {
        console.error("Failed to register immersive event listeners:", e);
    }
    // 🛡️ 修复：不要在这里立即快照，否则 SID 变化前会锁定错误的上下文
});

onUnmounted(() => {
    console.log("🚪 [SocialChat] 离开聊天容器，执行最后结算...");
    if (lastActiveContext.contact && lastActiveContext.sessionId) {
        syncCurrentMemoryOnLeave(lastActiveContext.contact, lastActiveContext.sessionId);
    }
    
    // 🎭 Cleanup immersive event listeners
    if (typingUnlisten) typingUnlisten();
    if (retractionUnlisten) retractionUnlisten();
    if (newMessageUnlisten) newMessageUnlisten();
});

// 2. 深度监控上下文变换：角色 ID 或 会话 ID 任何一个变了，都视为“切换”
watch(
  () => ({
    cid: props.activeContact?.id,
    sid: chatStore.activeSocialSessionId
  }),
  async (newCtx, oldCtx) => {
    // A. 如果 oldCtx 有值，说明是从一个有效会话“切出来”的，触发结算
    if (oldCtx?.cid && oldCtx?.sid) {
        // 🎭 取消旧会话的所有待执行行为，防止消息出现在错误的会话中
        try {
            await invoke("cancel_immersive_behaviors", { sessionId: oldCtx.sid });
            console.log(`🛑 [Session-Switch] 已取消会话 ${oldCtx.sid} 的待执行行为`);
        } catch (e) {
            console.warn("Failed to cancel old session behaviors:", e);
        }
        
        // 🛡️ 核心修复：确保同步时使用“离开那一瞬间”的旧快照 ID 和 旧 Context 
        if (lastActiveContext.contact && String(lastActiveContext.contact.id) === String(oldCtx.cid)) {
            console.log(`📤 [Sync-Trigger] 正在离开角色: ${lastActiveContext.contact.name} (SID: ${oldCtx.sid})`);
            syncCurrentMemoryOnLeave(lastActiveContext.contact, oldCtx.sid);
            lastActiveContext.sessionId = null; 
        }
    }

    // B. 处理“新进入”的逻辑
    if (newCtx.cid) {
      if (newCtx.cid !== oldCtx?.cid) {
          // Case 1: 角色变了，需要先拉取该角色的会话列表，再决定打开哪个 SID
          console.log(`📥 [Context] 角色变更为: ${newCtx.cid}, 初始化会话...`);
          await initSessions(newCtx.cid);
          // 🚀 [核心修复]：不再 return，确保即便 Session ID 没变也会继续向下执行加载逻辑
      }
      
      // Case 2: 角色没变，但 Session ID 变了 (或者刚初始化完)
      // 继续向下执行加载逻辑，不要 return
      console.log(`📥 [Context] 确认上下文: ${newCtx.cid} | Session: ${newCtx.sid}`);
      
      // 更新当前的稳定上下文快照，标记当前为“可信且对齐”的聊天状态
      lastActiveContext = {
          contact: { ...props.activeContact },
          sessionId: chatStore.activeSocialSessionId
      };
      
      hasNewMessages.value = false; // 🔄 重置新消息标志位，进入新上下文
      
      console.log(`🎯 [Context] 上下文锁定: ${lastActiveContext.contact.name} | Session: ${lastActiveContext.sessionId}`);
      
      // 更新 UI (标题和消息)
      try {
          const sessions = await invoke("get_social_sessions", { contactId: newCtx.cid });
          const target = sessions.find(s => s.id === chatStore.activeSocialSessionId);
          if (target) activeSessionTitle.value = target.title;
          
          await invoke("touch_social_session", { id: chatStore.activeSocialSessionId });
      } catch(e) {}
      
      await loadMessages(newCtx.cid);
    }
  },
  { immediate: true, deep: true }
);

// 🚀 Software Init Sync removed to prevent ghost memory resurrection
// Only sync on actual deliberate 'leave' actions now.

const triggerAIRequest = async (targetMessage = null) => {
  if (isGenerating.value) return;

  const contactId = props.activeContact.id;
  
  // 1. Prepare/Add assistant message locally
  let assistantMsg;
  if (targetMessage) {
    assistantMsg = targetMessage;
    assistantMsg.content = "__LOADING__";
  } else {
    assistantMsg = {
      role: "assistant",
      content: "__LOADING__",
      model: props.activeContact.model,
      created_at: new Date().toISOString().replace('T', ' ').replace('Z', '')
    };
    messages.value.push(assistantMsg);
  }
  
  const msgInArray = targetMessage ? assistantMsg : messages.value[messages.value.length - 1];
  
  isGenerating.value = true;
  chatStore.isGenerating = true; // ⚡️ Sync state for auto-scroll
  triggerScroll('smooth'); // 🌊 Smooth scroll for AI start

  try {
  // 2. Prepare AI request
    const onEvent = new Channel();
    let aiFullContent = "";

    // 🔄 RE-FETCH CONTACT INFO TO GET LATEST PROMPT/MODEL
    let currentContact = props.activeContact;
    try {
        const contacts = await invoke("get_social_contacts");
        const updated = contacts.find(c => c.id === props.activeContact.id);
        if (updated) currentContact = updated;
    } catch (e) {
        console.warn("Failed to refresh contact info, using prop data:", e);
    }

    const isStreamEnabled = configStore.settings.chatMode?.enabled
      ? configStore.settings.chatMode.enableStream
      : configStore.settings.enableStream;

    onEvent.onmessage = (data) => {
      if (data.startsWith("c:")) {
        const content = data.substring(2);
        aiFullContent += content;
        
        if (msgInArray.content === "__LOADING__") msgInArray.content = "";
        msgInArray.content += content;
        
        // ⚡️ FORCE SCROLL TO BOTTOM ON CHUNK (Social Mode Exclusive)
        if (messageListRef.value?.scrollToBottom) {
             messageListRef.value.scrollToBottom();
        }
      }
    };

    // Construct message history for AI
    // If regenerating, we context up to BUT NOT INCLUDING the regenerating message
    const msgIndex = messages.value.indexOf(assistantMsg);
    const history = messages.value.slice(0, msgIndex).map(m => ({
        role: m.role,
        content: m.content,
        mode: "Social",
        role_id: m.role === 'assistant' ? String(props.activeContact.id) : undefined
    }));

    // Add system prompt using REFRESHED data
    if (currentContact.prompt) {
        history.unshift({ role: "system", content: currentContact.prompt });
    }

    await invoke("ask_ai", {
      msg: history,
      onEvent,
      explicitProviderId: currentContact.provider || configStore.settings.defaultProviderId, 
      explicitModelId: currentContact.model,
      stream: isStreamEnabled
    });

    // 3. Save/Update assistant response in database
    if (msgInArray.id) {
        await invoke("update_social_message", { id: msgInArray.id, content: aiFullContent });
    } else {
        const savedId = await invoke("save_social_message", {
            contactId,
            sessionId: chatStore.activeSocialSessionId, // ✨ Pass Session ID
            role: "assistant",
            content: aiFullContent,
            fileMetadata: null
        });
        msgInArray.id = savedId;
    }
    msgInArray.content = aiFullContent;

    // 4. 🧠 Auto Summary Check
    const validMsgCount = messages.value.filter(m => m.content !== "__LOADING__").length;
    // Check if title is default. Weak check using string inclusion or exact match.
    // Better to check against "默认会话" or "新对话"
    const isDefaultTitle = ["默认会话", "新对话"].includes(activeSessionTitle.value);
    const shouldReSummary = validMsgCount % 20 === 0; // 每20条重新总结一次
    
    if (chatStore.activeSocialSessionId && ( (validMsgCount >= 2 && isDefaultTitle) || shouldReSummary )) {
        console.log("🧠 Triggering Adaptive Summary...");
        autoSummaryTitle(chatStore.activeSocialSessionId);
    }

  } catch (e) {
    console.error("Social chat AI error:", e);
    msgInArray.content = "发生错误: " + e;
  } finally {
    isGenerating.value = false;
    chatStore.isGenerating = false; // ⚡️ Sync state end
    // ⚡️ FINAL SCROLL TO BOTTOM
    triggerScroll('smooth');
  }
};

const handleSend = async (text, fileMetadata = null) => {
  if (!text.trim() && !fileMetadata) return;
  if (isGenerating.value) return;

  const contactId = props.activeContact.id;
  const userText = text.trim();

  try {
    // 🎭 Cancel any pending immersive behaviors first
    try {
        await invoke("cancel_immersive_behaviors", { 
            sessionId: chatStore.activeSocialSessionId 
        });
    } catch (e) {
        console.warn("Failed to cancel immersive behaviors:", e);
    }
    
    // 1. Save and add user message locally with ID
    const savedUserId = await invoke("save_social_message", { 
        contactId,
        sessionId: chatStore.activeSocialSessionId, // ✨ Pass Session ID
        role: "user", 
        content: userText,
        fileMetadata // ✨ Support files
    });
    
    messages.value.push({ 
      id: savedUserId,
      role: "user", 
      content: userText,
      fileMetadata, // ✨ Support files
      created_at: new Date().toISOString().replace('T', ' ').replace('Z', '') 
    });
    hasNewMessages.value = true; // ✍️ 标记产生了新交互
    triggerScroll('smooth'); // 🌊 Smooth scroll for user action

    // 2. 🎭 Check if immersive mode is enabled
    if (configStore.settings.immersiveMode?.enabled) {
        // 🎭 Immersive Mode: Generate AI response WITHOUT saving/displaying
        // The backend will handle the immersive display through events
        
        isGenerating.value = true;
        chatStore.isGenerating = true;
        
        try {
            // Generate AI response (we need the content but won't save it)
            const onEvent = new Channel();
            let aiFullContent = "";
            
            // Re-fetch contact info for latest prompt/model
            let currentContact = props.activeContact;
            try {
                const contacts = await invoke("get_social_contacts");
                const updated = contacts.find(c => c.id === props.activeContact.id);
                if (updated) currentContact = updated;
            } catch (e) {
                console.warn("Failed to refresh contact info:", e);
            }
            
            onEvent.onmessage = (data) => {
                if (data.startsWith("c:")) {
                    aiFullContent += data.substring(2);
                }
            };
            
            // Build message history
            const history = messages.value.map(m => ({
                role: m.role,
                content: m.content,
                mode: "Social",
                role_id: m.role === 'assistant' ? String(contactId) : undefined
            }));
            
            if (currentContact.prompt) {
                history.unshift({ role: "system", content: currentContact.prompt });
            }
            
            // Generate AI response
            await invoke("ask_ai", {
                msg: history,
                onEvent,
                explicitProviderId: currentContact.provider || configStore.settings.defaultProviderId,
                explicitModelId: currentContact.model,
                stream: false // Don't stream in immersive mode
            });
            
            // Now send through immersive mode (backend will handle display)
            await invoke("send_social_message_immersive", {
                sessionId: chatStore.activeSocialSessionId,
                contactId,
                content: aiFullContent
            });
            
        } catch (e) {
            console.error("Immersive AI generation failed:", e);
            // Fallback: add error message
            messages.value.push({
                role: "assistant",
                content: "发生错误: " + e,
                created_at: new Date().toISOString().replace('T', ' ').replace('Z', '')
            });
        } finally {
            isGenerating.value = false;
            chatStore.isGenerating = false;
        }
        
    } else {
        // Traditional mode: direct AI request with immediate display
        await triggerAIRequest();
    }

  } catch (e) {
    console.error("Social chat send error:", e);
  }
};

// 🆕 Auto Summary Title Logic
const autoSummaryTitle = async (sessionId) => {
    try {
        const prompt = "请根据对话内容总结一个简洁生动的标题(20字以内)。直接返回标题文字，不要包含引号或多余标点。";
        
        // Filter out loading messages
        const filteredMsgs = messages.value.filter(m => m.content !== "__LOADING__");
        
        // ✨ Use LAST few messages for "latest" context
        const summaryMsgs = [];
        
        // Add Character Context (System Prompt)
        if (props.activeContact.prompt) {
            summaryMsgs.push({ role: "system", content: props.activeContact.prompt });
        }
        
        // Take LAST 6 messages + prompt
        summaryMsgs.push(
            ...filteredMsgs.slice(-6).map(m => ({
                role: m.role,
                content: m.content
            })),
            { role: "user", content: prompt }
        );

        console.log("=== [Social] Requesting Adaptive Title Update ===");
        const rawTitle = await invoke("generate_title", { 
            msg: summaryMsgs,
            explicitProviderId: props.activeContact.provider || configStore.settings.defaultProviderId,
            explicitModelId: props.activeContact.model
        });
        
        let finalTitle = rawTitle.trim().replace(/["'“”]/g, '');
        if (finalTitle.length > 30) finalTitle = finalTitle.substring(0, 30);

        if (finalTitle && finalTitle.length > 0 && finalTitle !== "新对话") {
            // Update DB
            await invoke("update_social_session_title", { 
                id: sessionId, 
                title: finalTitle 
            });
            // Update Local State
            activeSessionTitle.value = finalTitle;

            // 🔄 Sync: Notify sidebar to reload
            chatStore.triggerSocialSessionRefresh();
        }
    } catch (e) {
        console.error("Auto summary failed:", e);
    }
};

const handleStop = async () => {
    isGenerating.value = false;
    chatStore.isGenerating = false; // ⚡️ Sync state stop
    try { await invoke("stop_ai_generation"); } catch (err) { console.error(err); }
};

const handleDelete = async (messageId, index) => {
    // If we have an ID, delete from DB first
    if (messageId) {
        try {
            await invoke("delete_social_message", { id: messageId });
        } catch (e) {
            console.error("Failed to delete social message from DB:", e);
        }
    }
    
    // Always remove from local UI if index is valid
    if (index >= 0 && index < messages.value.length) {
        messages.value.splice(index, 1);
    }
};

const handleRegenerate = async (messageId, index) => {
    if (isGenerating.value) return;
    
    const targetMsg = messages.value[index];
    if (!targetMsg || targetMsg.role !== 'assistant') return;

    // Instead, just regenerate THIS specific message in place.
    hasNewMessages.value = true;
    await triggerAIRequest(targetMsg);
};

const emit = defineEmits(['show-profile']);

const handleSaveEdit = async (messageId, index, content) => {
    try {
        await invoke("update_social_message", { id: messageId, content });
        const msg = messages.value.find(m => m.id === messageId);
        if (msg) {
            msg.content = content;
            hasNewMessages.value = true;
        }
    } catch (e) {
        console.error("Save edit failed:", e);
    }
};

const handleAvatarClick = () => {
    emit('show-profile');
};
</script>

<template>
  <main class="social-chat-container">
    <header class="chat-header" data-tauri-drag-region>
       <div class="header-info">
           <span class="session-topic">{{ activeContact.remark || activeContact.name }}</span>
           <transition name="status-fade">
               <span v-if="isGenerating || isTyping" class="typing-status">
                   &nbsp;正在输入<span class="dot-anim">...</span>
               </span>
           </transition>
       </div>
    </header>

    <div class="message-list-wrapper" :style="{ opacity: isInitialScrollDone ? 1 : 0 }">
        <Transition name="message-blur" mode="out-in">
          <MessageList
            :key="activeContact.id + '-' + chatStore.activeSocialSessionId"
            :messages="messages"
            :sessionId="activeContact.id.toString()"
            :themeOverride="'wechat'"
            :showSystemPrompt="false"
            :assistantAvatar="resolveAvatarSrc(activeContact.avatar, activeContact.id)"
            :assistantName="activeContact.remark || activeContact.name"
            :initialScrollPos="chatStore.getSessionScroll(activeContact.id.toString())"
            :loadingMore="isLoadingMore"
            ref="messageListRef"
            @delete="handleDelete"
            @regenerate="handleRegenerate"
            @save-edit="handleSaveEdit"
            @load-more="loadMoreMessages"
            @avatar-click="handleAvatarClick"
          />
        </Transition>
    </div>

    <div class="chat-input-island">
      <ChatInput
        :is-generating="isGenerating"
        :override-send="true"
        @send="handleSend"
        @stop="handleStop"
      />
    </div>
  </main>
</template>

<style scoped>
.social-chat-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg-chat-island); /* Restore solid background */
  padding: 0; /* Revert island style to fill frame */
  box-sizing: border-box;
}

/* Force Light Background for MessageList in Social Mode */
.message-list-wrapper {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  background: var(--bg-chat-island);
  overflow: hidden;
}

.chat-header {
    height: 52px; /* Slightly more compact header */
    padding: 0 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border-bottom: 1px solid var(--border-color); /* Use variable */
    background: var(--bg-chat-island);
    z-index: 10;
}

.chat-input-island {
    background: var(--bg-chat-island);
}

.header-info {
    display: flex;
    flex-direction: row;
    align-items: center;
}

.session-topic {
    font-size: 1rem;
    font-weight: 600;
    color: #333;
    line-height: 1.2;
}



.typing-status {
    font-size: 0.85rem;
    color: #888;
    display: inline-flex;
    align-items: center;
}

.dot-anim {
    display: inline-block;
    width: 12px;
    text-align: left;
    animation: dots 1.5s infinite;
}

@keyframes dots {
    0% { content: ''; }
    25% { content: '.'; }
    50% { content: '..'; }
    75% { content: '...'; }
}

/* 适配微信/QQ样式的点点点更生动的方式：使用伪类循环 */
.dot-anim::after {
    content: '';
    animation: dots-pseudo 1.5s infinite;
}

@keyframes dots-pseudo {
    0% { content: ''; }
    33% { content: '.'; }
    66% { content: '..'; }
    100% { content: '...'; }
}

/* Transition for status */
.status-fade-enter-active, .status-fade-leave-active {
    transition: all 0.3s ease;
}
.status-fade-enter-from, .status-fade-leave-to {
    opacity: 0;
    transform: translateY(5px);
}

/* Message List Blur Transition */
.message-blur-enter-active,
.message-blur-leave-active {
  transition: all 0.20s ease;
}

.message-blur-enter-from {
  opacity: 0;
  transform: scale(0.98);
  filter: blur(4px);
}

.message-blur-leave-to {
  opacity: 0;
  transform: scale(0.98);
  filter: blur(4px);
}

/* chat-input-wrapper removed to simplify layout and avoid overlap */

/* Dark Mode Overrides for Social Mode */
:global(.app-dark) .social-chat-container,
:global(.app-dark) .chat-header,
:global(.app-dark) .chat-input-wrapper,
:global(.app-dark) .message-list-wrapper {
    background: var(--bg-chat-island);
    border-color: var(--border-glass);
}

:global(.app-dark) .contact-name {
    color: var(--text-color-white);
}

:global(.app-dark) .typing-status {
    color: var(--text-tertiary);
}
</style>
