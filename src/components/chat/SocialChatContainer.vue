<script setup>
import { ref, watch, onMounted, nextTick } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import { useConfigStore } from "../../stores/config";
import MessageList from "./MessageList.vue";
import ChatInput from "./ChatInput.vue";

const props = defineProps({
  activeContact: {
    type: Object,
    required: true
  }
});

const configStore = useConfigStore();
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

const handleSend = async (text) => {
  if (!text.trim() || isGenerating.value) return;

  const contactId = props.activeContact.id;
  const userText = text.trim();

  // 1. Save and add user message
  try {
    await invoke("save_social_message", { 
        contactId, 
        role: "user", 
        content: userText 
    });
    messages.value.push({ role: "user", content: userText });
    triggerScroll();

    // 2. Add loading assistant message
    messages.value.push({
      role: "assistant",
      content: "__LOADING__",
      model: props.activeContact.model
    });
    isGenerating.value = true;
    triggerScroll();

    // 3. Prepare AI request
    const onEvent = new Channel();
    let aiFullContent = "";

    onEvent.onmessage = (data) => {
      if (data.startsWith("c:")) {
        const content = data.substring(2);
        aiFullContent += content;
        
        const lastMsg = messages.value[messages.value.length - 1];
        if (lastMsg.content === "__LOADING__") lastMsg.content = "";
        lastMsg.content += content;
      }
    };

    // Construct message history for AI
    const history = messages.value.slice(0, -1).map(m => ({
        role: m.role,
        content: m.content
    }));

    // Add system prompt
    if (props.activeContact.prompt) {
        history.unshift({ role: "system", content: props.activeContact.prompt });
    }

    await invoke("ask_ai", {
      msg: history,
      onEvent,
      explicitProviderId: configStore.settings.defaultProviderId, // Ideally should match model
      explicitModelId: props.activeContact.model
    });

    // 4. Save final assistant response
    await invoke("save_social_message", {
        contactId,
        role: "assistant",
        content: aiFullContent
    });

  } catch (e) {
    console.error("Social chat error:", e);
    const lastMsg = messages.value[messages.value.length - 1];
    if (lastMsg) lastMsg.content = "发生错误: " + e;
  } finally {
    isGenerating.value = false;
  }
};

const handleStop = async () => {
    isGenerating.value = false;
    try { await invoke("stop_ai_generation"); } catch (err) { console.error(err); }
};
</script>

<template>
  <main class="social-chat-container">
    <header class="chat-header">
       <span class="contact-name">{{ activeContact.name }}</span>
       <span class="model-badge">{{ activeContact.model }}</span>
    </header>

    <div class="message-list-wrapper">
      <MessageList
        :key="activeContact.id"
        :messages="messages"
        :sessionId="activeContact.id.toString()"
        :themeOverride="'wechat'"
        ref="messageListRef"
      />
    </div>

    <footer class="chat-input-wrapper">
      <ChatInput
        :is-generating="isGenerating"
        @send="handleSend"
        @stop="handleStop"
      />
    </footer>
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

.chat-input-wrapper {
  background: #f5f5f5;
  border-top: 1px solid #e5e5e5;
  padding: 8px 16px 16px 16px;
}

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
