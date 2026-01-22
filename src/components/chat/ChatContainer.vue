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
 */
const triggerScroll = async () => {
  await nextTick();
  if (messageListRef.value?.scrollToBottom) {
    messageListRef.value.scrollToBottom();
  }
};

const handleStop = async () => {
  await chatStore.stopGeneration();
};

const handleSend = async (text) => {
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

// 监听消息变化
watch(
  () => currentMessages.value?.length,
  () => triggerScroll(),
  { deep: true }
);

// 为父组件或外部暴露更新位置的方法
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
.chat-main-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: #1e1e1f; /* 浅色底座（侧边栏同色） */
  
  /* --- 🛠️ 悬浮控制旋钮 1：外圈留白 --- */
  /* 增大这个值，岛屿就会缩小，悬浮感增强 */
  padding: 5px 5px; 
  /* ---------------------------------- */
  
  box-sizing: border-box;
  overflow: hidden;
}

.message-list-wrapper {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  position: relative;
  background: #131314; /* 岛屿深色 */
  
  /* --- 🛠️ 悬浮控制旋钮 2：顶部圆角 --- */
  border-top-left-radius: 12px; 
  border-top-right-radius: 12px; 
  /* ---------------------------------- */
}

.chat-input-wrapper {
  flex-shrink: 0;
  padding: 0;
  z-index: 10;
  background: #131314; /* 必须与 wrapper 一致，确保岛屿是一体的 */
  
  /* --- 🛠️ 悬浮控制旋钮 3：底部圆角 --- */
  border-bottom-left-radius: 12px; 
  border-bottom-right-radius: 12px; 
  /* ---------------------------------- */
  
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