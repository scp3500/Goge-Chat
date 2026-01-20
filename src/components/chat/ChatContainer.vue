<script setup>
import { ref, watch, computed, nextTick } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";

// 引用同级组件
import MessageList from "./MessageList.vue";
import ChatInput from "./ChatInput.vue";

const props = defineProps(["currentId", "sessions"]);
const emit = defineEmits(["scroll-update"]);

const messages = ref([]);
const messageListRef = ref(null);
const isGenerating = ref(false);

const currentSession = computed(() => {
  const found = props.sessions?.find((s) => s.id === props.currentId);
  return found || null;
});

const triggerScroll = async () => {
  await nextTick();
  if (messageListRef.value?.scrollToBottom) {
    messageListRef.value.scrollToBottom();
  }
};

const handleStop = async () => {
  isGenerating.value = false;
  console.log("🛑 正在向后端发送物理中断指令...");
  try {
    await invoke("stop_ai_generation");
  } catch (err) {
    console.error("后端中断失败:", err);
  }
};

watch(
  () => props.currentId,
  async (newId) => {
    if (newId !== null) {
      try {
        if (isGenerating.value) await handleStop();
        const history = await invoke("get_messages", { sessionId: newId });
        messages.value =
          history && history.length > 0
            ? history
            : [{ role: "assistant", content: "你好！我是 GoleChat。" }];
      } catch (err) {
        console.error("[前端排查] 调用 get_messages 失败:", err);
      }
    }
  },
  { immediate: true }
);

async function handleSend(text) {
  if (!props.currentId || !text.trim() || isGenerating.value) return;
  try {
    await invoke("reset_ai_generation");
  } catch (e) {}
  isGenerating.value = true;

  try {
    await invoke("save_message", {
      sessionId: props.currentId,
      role: "user",
      content: text,
    });
    messages.value.push({ role: "user", content: text });

    // ✨ 【关键修复】：将空字符串改为 __LOADING__ 标识符
    messages.value.push({ role: "assistant", content: "__LOADING__" });
    triggerScroll();

    const onEvent = new Channel();
    let aiFullContent = "";

    onEvent.onmessage = (chunk) => {
      if (!isGenerating.value) return;

      const lastMsg = messages.value[messages.value.length - 1];
      
      // ✨ 收到第一个 chunk 时，瞬间抹除加载标识
      if (lastMsg.content === "__LOADING__") {
        lastMsg.content = "";
      }

      lastMsg.content += chunk;
      aiFullContent += chunk;

      triggerScroll();
    };

    await invoke("ask_ai", {
      msg: messages.value.slice(0, -1),
      onEvent,
    });

    if (aiFullContent.trim().length > 0) {
      await invoke("save_message", {
        sessionId: props.currentId,
        role: "assistant",
        content: aiFullContent,
      });
      console.log("💾 已强行持久化截断内容");
    }
  } catch (error) {
    console.error("[前端排查] 对话过程出错:", error);
  } finally {
    isGenerating.value = false;
  }
}
</script>

<template>
  <main class="chat-main-layout">
    <div class="message-list-wrapper">
      <Transition name="list-fade">
        <MessageList
          v-if="currentId"
          :key="currentId"
          :messages="messages"
          :sessionId="currentId"
          :initialScrollPos="currentSession?.last_scroll_pos || 0"
          ref="messageListRef"
          @update-pos="(pos) => $emit('scroll-update', currentId, pos)"
        />
      </Transition>
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
.chat-main-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg-main, #131314);
  overflow: hidden;
}

.message-list-wrapper {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  position: relative;
}

.chat-input-wrapper {
  flex-shrink: 0;
  padding: 0;
  z-index: 10;
  background: var(--bg-main, #131314);
  box-shadow: none !important;
  border: none !important;
  outline: none !important;
  border-top-left-radius: 48px;
  overflow: hidden;
}

.list-fade-enter-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.list-fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

:deep(.message-list-wrapper > *) {
  scrollbar-gutter: stable;
}
</style>