<script setup>
import { ref, watch, nextTick } from 'vue';

const props = defineProps({
  isGenerating: {
    type: Boolean,
    default: false
  }
});

const inputMsg = ref("");
const textareaRef = ref(null);
const emit = defineEmits(['send', 'stop']);

const adjustHeight = () => {
  const el = textareaRef.value;
  if (!el) return;
  el.style.height = 'auto';
  el.style.height = (el.scrollHeight > 200 ? 200 : el.scrollHeight) + 'px';
};

watch(inputMsg, () => {
  nextTick(adjustHeight);
});

const handleAction = () => {
  if (props.isGenerating) {
    emit('stop');
  } else {
    if (!inputMsg.value.trim()) return;
    emit('send', inputMsg.value);
    inputMsg.value = "";
    nextTick(() => {
      if (textareaRef.value) textareaRef.value.style.height = 'auto';
    });
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
        placeholder="给 Alice 发送消息..." 
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
  /* ✨ 核心修复 1：彻底切掉边缘多余空间，直接填满父容器宽度 */
  padding: 0; 
  width: 100%;
  background: transparent !important; 
  display: flex;
  flex-direction: column;
  align-items: center;
  /* 底部留出间距，确保气泡不贴底，同时不遮挡左侧圆角 */
  padding-bottom: 20px; 
  padding-top: 10px;
}

.input-wrapper { 
  /* ✨ 核心修复 2：调整气泡占比，让非对话框部分变透明 */
  width: 90%; 
  max-width: 880px; 
  background: #25262b; 
  display: flex; 
  align-items: flex-end; 
  padding: 10px 14px; 
  border-radius: 16px; 
  border: 1px solid rgba(255, 255, 255, 0.08); 
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
}

.input-wrapper:focus-within { 
  border-color: rgba(255, 255, 255, 0.3); 
  background: #2c2d33;
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
  /* ✨ 核心修复 3：三重封锁，彻底杀掉右侧白方块 */
  resize: none !important; 
  appearance: none !important;
  -webkit-appearance: none !important;
  
  max-height: 200px;
  font-family: inherit;
  overflow-y: auto; 
}

/* ✨ 暴力清理 Webkit 原生手柄伪元素 */
.chat-input::-webkit-resizer {
  display: none !important;
  background: transparent;
}

.action-btn { 
  background: #fff; 
  color: #000; 
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

.action-btn.is-stop {
  background: #ADD8E6;
  color: #fff;
}

.action-btn:disabled { 
  opacity: 0.15; 
  background: #888;
}

.modern-scroll::-webkit-scrollbar { width: 4px; }
.modern-scroll::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
</style>