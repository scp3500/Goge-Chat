<script setup>
import { ref, watch, onMounted, nextTick } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
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
const messageListRef = ref(null);
const isLoadingMore = ref(false); // ⏳ Loading state
const allLoaded = ref(false);     // 🏁 End of history
const PAGE_SIZE = 20;

// ⚡️ Load recent messages initially
const loadMessages = async (contactId) => {
  try {
    messages.value = []; // Clear current
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
    
    triggerScroll('auto'); // ⚡️ Instant scroll to bottom on initial load
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

const triggerScroll = async (behavior = 'auto') => {
  await nextTick();
  // Wait for blur 'out' phase (approx 125ms) + enter phase start
  setTimeout(() => {
    if (messageListRef.value?.scrollToBottom) {
      messageListRef.value.scrollToBottom(behavior);
    }
  }, 80);
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

watch(() => props.activeContact?.id, async (newId) => {
  if (newId) {
    // 1. Init Session first
    await initSessions(newId);
    // 2. Then load messages
    await loadMessages(newId);
  }
}, { immediate: true });

// Watch for session changes (clicked in sidebar)
watch(() => chatStore.activeSocialSessionId, async (newSid) => {
  if (newSid && props.activeContact?.id) {
    // 🔄 Touch session to update its accessed time
    try {
        await invoke("touch_social_session", { id: newSid });
    } catch (e) {
        console.warn("Failed to touch session:", e);
    }
    
    // Update title
    try {
        const sessions = await invoke("get_social_sessions", { contactId: props.activeContact.id });
        const s = sessions.find(x => x.id === newSid);
        if (s) activeSessionTitle.value = s.title;
    } catch(e) {}
    
    await loadMessages(props.activeContact.id);
  }
});

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
      }
    };

    // Construct message history for AI
    // If regenerating, we context up to BUT NOT INCLUDING the regenerating message
    const msgIndex = messages.value.indexOf(assistantMsg);
    const history = messages.value.slice(0, msgIndex).map(m => ({
        role: m.role,
        content: m.content
    }));

    // Add system prompt using REFRESHED data
    if (currentContact.prompt) {
        history.unshift({ role: "system", content: currentContact.prompt });
    }

    await invoke("ask_ai", {
      msg: history,
      onEvent,
      explicitProviderId: configStore.settings.defaultProviderId, 
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
            content: aiFullContent
        });
        msgInArray.id = savedId;
    }
    msgInArray.content = aiFullContent;

    // 4. 🧠 Auto Summary Check
    const validMsgCount = messages.value.filter(m => m.content !== "__LOADING__").length;
    // Check if title is default. Weak check using string inclusion or exact match.
    // Better to check against "默认会话" or "新对话"
    const isDefaultTitle = ["默认会话", "新对话"].includes(activeSessionTitle.value);
    
    if (validMsgCount >= 2 && isDefaultTitle && chatStore.activeSocialSessionId) {
        console.log("🧠 Triggering Auto Summary...");
        autoSummaryTitle(chatStore.activeSocialSessionId);
    }

  } catch (e) {
    console.error("Social chat AI error:", e);
    msgInArray.content = "发生错误: " + e;
  } finally {
    isGenerating.value = false;
    chatStore.isGenerating = false; // ⚡️ Sync state end
  }
};

const handleSend = async (text) => {
  if (!text.trim() || isGenerating.value) return;

  const contactId = props.activeContact.id;
  const userText = text.trim();

  try {
    // 1. Save and add user message locally with ID
    const savedUserId = await invoke("save_social_message", { 
        contactId,
        sessionId: chatStore.activeSocialSessionId, // ✨ Pass Session ID
        role: "user", 
        content: userText 
    });
    
    messages.value.push({ 
      id: savedUserId,
      role: "user", 
      content: userText,
      created_at: new Date().toISOString().replace('T', ' ').replace('Z', '') 
    });
    triggerScroll('smooth'); // 🌊 Smooth scroll for user action

    // 2. Trigger AI
    await triggerAIRequest();

  } catch (e) {
    console.error("Social chat send error:", e);
  }
};

// 🆕 Auto Summary Title Logic
const autoSummaryTitle = async (sessionId) => {
    try {
        const prompt = "请总结以上对话的标题(8-10字)。直接返回标题文字，不要代码，不要标点符号。";
        
        // Filter out loading messages
        const filteredMsgs = messages.value.filter(m => m.content !== "__LOADING__");
        
        // Take first few turns + prompt
        const summaryMsgs = [
            ...filteredMsgs.slice(0, 5).map(m => ({
                role: m.role,
                content: m.content
            })),
            { role: "user", content: prompt }
        ];

        console.log("=== [Social] Requesting Auto Title ===");
        const rawTitle = await invoke("generate_title", { msg: summaryMsgs });
        console.log("✨ Generated Title:", rawTitle);

        let finalTitle = rawTitle.trim();
        if (finalTitle.length > 10) finalTitle = finalTitle.substring(0, 10);

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
    if (!messageId) {
        // Fallback: search for the ID in local messages if mismatch
        const msg = messages.value[index];
        if (msg?.id) messageId = msg.id;
    }
    
    if (!messageId) {
        console.error("Critical: Cannot delete message without ID at index:", index);
        return;
    }

    try {
        await invoke("delete_social_message", { id: messageId });
        // Logic fix: Ensure we splice the correct index in the ORIGIN messages array if filtered
        // MessageList filters message.role !== 'system'
        messages.value.splice(index, 1);
    } catch (e) {
        console.error("Failed to delete social message:", e);
    }
};

const handleRegenerate = async (messageId, index) => {
    if (isGenerating.value) return;
    
    const targetMsg = messages.value[index];
    if (!targetMsg || targetMsg.role !== 'assistant') return;

    // Fixed: Do NOT delete subsequent messages as requested by user.
    // Instead, just regenerate THIS specific message in place.
    await triggerAIRequest(targetMsg);
};

const handleSaveEdit = async (messageId, index, newContent) => {
    try {
        await invoke("update_social_message", { id: messageId, content: newContent });
        
        // Update local state
        const msg = messages.value[index];
        if (msg) {
            msg.content = newContent;
            
            // If it's a user message, we usually want to regenerate the following assistant response
            if (msg.role === 'user') {
                if (index < messages.value.length - 1) {
                    const nextMsg = messages.value[index + 1];
                    if (nextMsg.role === 'assistant') {
                        await triggerAIRequest(nextMsg);
                    }
                }
            }
        }
    } catch (e) {
        console.error("Save edit failed:", e);
    }
};
</script>

<template>
  <main class="social-chat-container">
    <header class="chat-header" data-tauri-drag-region>
       <div class="header-info">
           <span class="session-topic">{{ activeContact.name }}</span>
           <transition name="status-fade">
               <span v-if="isGenerating" class="typing-status">
                   &nbsp;正在输入<span class="dot-anim">...</span>
               </span>
           </transition>
       </div>
    </header>

    <div class="message-list-wrapper">
        <Transition name="message-blur" mode="out-in">
          <MessageList
            :key="activeContact.id + '-' + chatStore.activeSocialSessionId"
            :messages="messages"
            :sessionId="activeContact.id.toString()"
            :themeOverride="'wechat'"
            :showSystemPrompt="false"
            :assistantAvatar="resolveAvatarSrc(activeContact.avatar, activeContact.id)"
            :assistantName="activeContact.name"
            :initialScrollPos="chatStore.getSessionScroll(activeContact.id.toString())"
            :loadingMore="isLoadingMore"
            ref="messageListRef"
            @delete="handleDelete"
            @regenerate="handleRegenerate"
            @save-edit="handleSaveEdit"
            @load-more="loadMoreMessages"
          />
        </Transition>
    </div>

    <ChatInput
      :is-generating="isGenerating"
      :override-send="true"
      @send="handleSend"
      @stop="handleStop"
    />
  </main>
</template>

<style scoped>
.social-chat-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  width: 100%;
  background: var(--bg-chat-island); /* Swapped to darker/grayer */
}

/* Force Light Background for MessageList in Social Mode */
.message-list-wrapper {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
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
    background: #1a1a1a;
    border-color: #333;
}

:global(.app-dark) .contact-name {
    color: #fff;
}

:global(.app-dark) .typing-status {
    color: #777;
}
</style>
