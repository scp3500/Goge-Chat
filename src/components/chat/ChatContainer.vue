<script setup>
import { ref, watch, computed, nextTick } from "vue";
import { storeToRefs } from "pinia";
import { useChatStore } from "../../stores/chat"; // 确保路径正确

// 引用同级组件
import MessageList from "./MessageList.vue";
import ChatInput from "./ChatInput.vue";

const chatStore = useChatStore();
// 使用 storeToRefs 保持响应式
const { activeId, currentMessages, isGenerating, activeSession, isChatViewActive } = storeToRefs(chatStore);

const messageListRef = ref(null);

/**
 * 💡 触发滚动逻辑
 */
const triggerScroll = async () => {
  await nextTick();
  // 再次等待一帧，确保 v-show 的 display 切换已完成且布局已重绘
  setTimeout(() => {
    if (messageListRef.value?.scrollToBottom) {
      messageListRef.value.scrollToBottom();
    }
  }, 10);
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
      // 关键修复：如果当前会话正在生成消息，不要重新从数据库加载！
      // 这里的 store 状态是最新的，包含正在生成的临时消息。
      // 如果重载，会因为数据库还没保存而丢失 assistant 消息，导致追加到 user 气泡。
      if (chatStore.generatingSessionId === newId && chatStore.isGenerating) {
        console.log("🚫 Skipping loadMessages because generating session is active:", newId);
        triggerScroll();
      } else {
        await chatStore.loadMessages(newId);
        triggerScroll();
      }
    }
  },
  { immediate: true }
);

// 监听视图激活状态（从设置返回时触发）
watch(
  isChatViewActive,
  (isActive) => {
    if (isActive) {
      console.log("👀 Chat view active, triggering scroll restoration");
      triggerScroll();
    }
  }
);

// 监听消息数量增加（新消息添加时才滚动，避免流式完成时的跳动）
let previousMessageCount = 0;
watch(
  () => currentMessages.value?.length,
  (newCount) => {
    if (newCount > previousMessageCount) {
      triggerScroll();
    }
    previousMessageCount = newCount || 0;
  }
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
  background: var(--bg-sidebar); /* 确保背景与侧边栏/标题栏一致，消除接缝 */
  
  /* --- 🛠️ [可调参数] 悬浮外边距：上 右 下 左 --- */
  /* 这里控制对话框距离窗口边缘的距离，例如 "0 6px 6px 0" 代表右边和下边有6px缝隙 */
  padding: 0 4px 4px 0px; 
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
  background: var(--bg-chat-island); /* 岛屿深色 */

  /* --- 🛠️ 悬浮控制旋钮 2：顶部圆角 --- */
  /* --- 🛠️ [可调参数] 顶部圆角半径 --- */
  /* 修改这里的 12px 可以调整圆角大小 */
  border-top-left-radius: 12px;
  border-top-right-radius: 12px;
  /* ---------------------------------- */
}

.chat-input-wrapper {
  flex-shrink: 0;
  padding: 0;
  z-index: 10;
  background: var(--bg-chat-island); /* 必须与 wrapper 一致，确保岛屿是一体的 */

  /* --- 🛠️ 悬浮控制旋钮 3：底部圆角 --- */
  /* --- 🛠️ [可调参数] 底部圆角半径 --- */
  /* 修改这里的 12px 可以调整圆角大小 */
  border-bottom-left-radius: 12px;
  border-bottom-right-radius: 12px;
  /* ---------------------------------- */
  
  /* overflow: hidden; */
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