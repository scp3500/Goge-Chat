<script setup>
import { ref, watch, computed, nextTick } from "vue";
import { storeToRefs } from "pinia";
import { useChatStore } from "../../stores/chat"; // 确保路径正确

// 引用同级组件
import MessageList from "./MessageList.vue";
import ChatInput from "./ChatInput.vue";

const chatStore = useChatStore();
// 使用 storeToRefs 保持响应式
const { activeId, currentMessages, isGenerating, activeSession } = storeToRefs(chatStore);

const messageListRef = ref(null);

/**
 * 💡 触发滚动逻辑
 * 虽然逻辑移到了 Store，但操作 DOM（滚动条）依然是 UI 层的职责
 */
const triggerScroll = async () => {
  await nextTick();
  if (messageListRef.value?.scrollToBottom) {
    messageListRef.value.scrollToBottom();
  }
};

/**
 * 🩺 手术改动原因：
 * 1. 移除本地 messages ref，改用 store.currentMessages。
 * 2. 移除 handleStop 本地实现，直接调用 store.stopGeneration()。
 * 3. 移除 handleSend 复杂的 Channel 逻辑，封装进 store.sendMessage()。
 */

const handleStop = async () => {
  await chatStore.stopGeneration();
};

const handleSend = async (text) => {
  // 调用 Store 的发送方法，并在发送后触发滚动
  await chatStore.sendMessage(text);
  triggerScroll();
};

// 监听 activeId 变化，加载历史记录
watch(
  activeId,
  async (newId) => {
    if (newId) {
      await chatStore.loadMessages(newId);
      triggerScroll();
    }
  },
  { immediate: true }
);

// 监听消息变化（用于 AI 回复时的实时滚动）
watch(
  () => currentMessages.value?.length,
  () => triggerScroll(),
  { deep: true }
);

// 为父组件或外部暴露更新位置的方法（如果需要）
const handleScrollUpdate = (pos) => {
  if (activeId.value) {
    chatStore.updateSessionScroll(activeId.value, pos);
  }
};
</script>

<template>
  <main class="chat-main-layout">
    <div class="message-list-wrapper">
      <Transition name="list-fade" mode="out-in">
        <MessageList
          v-if="activeId"
          :key="activeId"
          :messages="currentMessages"
          :sessionId="activeId"
          :initialScrollPos="activeSession?.last_scroll_pos || 0"
          ref="messageListRef"
          @update-pos="handleScrollUpdate"
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
/* 保持原有样式不变，遵循最小改动原则 */
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
  border-top-left-radius: 48px;
  overflow: hidden;
}

.list-fade-enter-active,
.list-fade-leave-active {
  transition: opacity 0.2s ease;
}

.list-fade-enter-from,
.list-fade-leave-to {
  opacity: 0;
}

:deep(.message-list-wrapper > *) {
  scrollbar-gutter: stable;
}
</style>