<script setup>
import { ref, watch, onMounted, nextTick } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import { useConfigStore } from "../../stores/config";
import { useChatStore } from "../../stores/chat";
import MessageList from "./MessageList.vue";
import ChatInput from "./ChatInput.vue";

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

const loadMessages = async (contactId) => {
  try {
    const history = await invoke("get_social_messages", { contactId });
    messages.value = history;
  } catch (e) {
    console.error("Failed to load social messages:", e);
  }
};

const triggerScroll = async () => {
  await nextTick();
  setTimeout(() => {
    if (messageListRef.value?.scrollToBottom) {
      messageListRef.value.scrollToBottom();
    }
  }, 10);
};

watch(() => props.activeContact?.id, (newId) => {
  if (newId) {
    loadMessages(newId);
    triggerScroll();
  }
}, { immediate: true });

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
      created_at: new Date().toISOString()
    };
    messages.value.push(assistantMsg);
  }
  
  isGenerating.value = true;
  triggerScroll();

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
        
        if (assistantMsg.content === "__LOADING__") assistantMsg.content = "";
        assistantMsg.content += content;
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
    if (assistantMsg.id) {
        await invoke("update_social_message", { id: assistantMsg.id, content: aiFullContent });
    } else {
        const savedId = await invoke("save_social_message", {
            contactId,
            role: "assistant",
            content: aiFullContent
        });
        assistantMsg.id = savedId;
    }
    assistantMsg.content = aiFullContent;

  } catch (e) {
    console.error("Social chat AI error:", e);
    assistantMsg.content = "发生错误: " + e;
  } finally {
    isGenerating.value = false;
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
        role: "user", 
        content: userText 
    });
    
    messages.value.push({ 
      id: savedUserId,
      role: "user", 
      content: userText,
      created_at: new Date().toISOString() 
    });
    triggerScroll();

    // 2. Trigger AI
    await triggerAIRequest();

  } catch (e) {
    console.error("Social chat send error:", e);
  }
};

const handleStop = async () => {
    isGenerating.value = false;
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
       <span class="contact-name">{{ activeContact.name }}</span>
       <span class="model-badge">{{ activeContact.model }}</span>
    </header>

    <div class="message-list-wrapper">
        <MessageList
          :key="activeContact.id"
          :messages="messages"
          :sessionId="activeContact.id.toString()"
          :themeOverride="'wechat'"
          :showSystemPrompt="false"
          :assistantAvatar="activeContact.avatar"
          :assistantName="activeContact.name"
          :initialScrollPos="chatStore.getSessionScroll(activeContact.id.toString())"
          ref="messageListRef"
          @delete="handleDelete"
          @regenerate="handleRegenerate"
          @save-edit="handleSaveEdit"
        />
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
  background: #f5f5f5; /* Classic WeChat light grey background */
}

/* Force Light Background for MessageList in Social Mode */
.message-list-wrapper {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  background: #f5f5f5;
  overflow: hidden;
}

.chat-header {
    height: 60px;
    padding: 0 24px;
    display: flex;
    align-items: center;
    gap: 12px;
    border-bottom: 1px solid #e5e5e5;
    background: #f5f5f5;
    z-index: 10;
}

.contact-name {
    font-size: 1.1rem;
    font-weight: 600;
    color: #1a1a1a;
}

.model-badge {
    font-size: 0.75rem;
    padding: 2px 8px;
    background: #e0e0e0;
    border-radius: 4px;
    color: #666;
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

:global(.app-dark) .model-badge {
    background: #333;
    color: #888;
}
</style>
