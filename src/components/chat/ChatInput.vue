<script setup>
import { ref, watch, nextTick } from 'vue';
import { storeToRefs } from 'pinia';
import { useChatStore } from "../../stores/chat"; 

const chatStore = useChatStore();
const { isGenerating } = storeToRefs(chatStore);

const inputMsg = ref("");
const textareaRef = ref(null);

const adjustHeight = () => {
  const el = textareaRef.value;
  if (!el) return;
  el.style.height = 'auto';
  el.style.height = (el.scrollHeight > 200 ? 200 : el.scrollHeight) + 'px';
};

watch(inputMsg, () => {
  nextTick(adjustHeight);
});

const handleAction = async () => {
  if (isGenerating.value) {
    await chatStore.stopGeneration();
  } else {
    if (!inputMsg.value.trim()) return;
    const msgToProcess = inputMsg.value;
    inputMsg.value = ""; 
    nextTick(() => {
      if (textareaRef.value) textareaRef.value.style.height = 'auto';
    });
    await chatStore.sendMessage(msgToProcess);
  }
};

const onKeydown = (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    handleAction();
  }
};
</script>

<template>
  <div class="input-area">
    <div class="input-wrapper">
      <textarea 
        ref="textareaRef"
        v-model="inputMsg" 
        rows="1"
        @keydown="onKeydown"
        placeholder="发送消息..." 
        class="chat-input modern-scroll" 
      ></textarea>
      
      <button 
        class="action-btn" 
        @click="handleAction" 
        :class="{ 'is-stop': isGenerating }"
        :disabled="!isGenerating && !inputMsg.trim()"
      >
        <template v-if="isGenerating">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <rect x="6" y="6" width="12" height="12" rx="2" />
          </svg>
        </template>
        <template v-else>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="22" y1="2" x2="11" y2="13"></line>
            <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
          </svg>
        </template>
      </button>
    </div>
  </div>
</template>

<style scoped>
.input-area { 
  /* --- 🩺 样式手术：定义宽度变量 --- */
  --input-width-percent: 80%; /* 👈 核心控制点：修改这个百分比即可控制左右边距 */
  /* ------------------------------- */
  
  padding: 0; 
  width: 100%;
  background: transparent !important; 
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-bottom: 24px; 
  padding-top: 8px;
}

.input-wrapper { 
  /* 使用变量控制宽度，自动成比例 */
  width: var(--input-width-percent); 
  
  /* 建议给一个舒适的上限，防止在 4K 屏上拉得太长导致阅读困难 */
  max-width: 800px; 
  
  background: #1c1c1e; 
  border: none; 
  box-shadow: none; 
  display: flex; 
  align-items: flex-end; 
  padding: 10px 16px; 
  border-radius: 12px; 
  transition: all 0.2s ease;
}

/* 🩺 改动原因说明：
 * 1. 引入 --input-width-percent 变量：将宽度从 90% 降至 80%，左右边距会自动从 5% 扩大到 10%。
 * 2. 移除固定的 margin 设置：通过父级的 align-items: center 配合宽度百分比，实现完美的居中比例缩放。
 */

.input-wrapper:focus-within { 
  background: #252527;
  border: none; 
}

.chat-input { 
  flex: 1; 
  background: transparent; 
  border: none; 
  color: #fff; 
  padding: 8px 4px; 
  outline: none; 
  font-size: 15px; 
  line-height: 1.5;
  resize: none !important; 
  max-height: 200px;
  font-family: inherit;
  overflow-y: auto; 
}

.chat-input::-webkit-resizer {
  display: none !important;
}

.action-btn { 
  background: transparent; 
  color: #888; 
  border: none; 
  width: 32px;
  height: 32px;
  border-radius: 8px;
  margin-left: 8px;
  margin-bottom: 2px;
  cursor: pointer; 
  transition: all 0.2s; 
  display: flex; 
  align-items: center; 
  justify-content: center;
  flex-shrink: 0;
}

.input-wrapper:has(.chat-input:not(:placeholder-shown)) .action-btn {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}

.action-btn.is-stop {
  background: #ff4d4f;
  color: #fff;
}

.action-btn:disabled { 
  opacity: 0.15; 
  background: transparent;
}

.modern-scroll::-webkit-scrollbar { width: 4px; }
.modern-scroll::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
</style>