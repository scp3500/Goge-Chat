<script setup>
import { ref, nextTick, onMounted, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useChatStore } from "../../stores/chat";
import { STOP_SVG, SEND_SVG, PLUS_SVG, BRAIN_SVG } from '../../constants/icons';

const chatStore = useChatStore();
const { isGenerating, useReasoning } = storeToRefs(chatStore);

const inputMsg = ref("");
const textareaRef = ref(null);

// --- 🔧 高度自动伸缩逻辑 ---
const autoResize = () => {
  const element = textareaRef.value;
  if (!element) return;
  element.style.height = 'auto'; 
  element.style.height = element.scrollHeight + 'px';
};

watch(inputMsg, () => {
  nextTick(() => {
    autoResize();
  });
});

const handleAction = async () => {
  if (isGenerating.value) {
    await chatStore.stopGeneration();
  } else {
    if (!inputMsg.value.trim()) return;
    const msgToProcess = inputMsg.value;
    inputMsg.value = "";
    
    // 发送后重置高度
    nextTick(() => {
        if(textareaRef.value) {
            textareaRef.value.style.height = 'auto'; 
            textareaRef.value.style.height = '24px'; 
        }
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

const handleAttachClick = () => {
  console.log('Attach button clicked');
};

const handleThinkClick = () => {
  useReasoning.value = !useReasoning.value;
};

onMounted(() => {
  autoResize();
});
</script>

<template>
  <div class="input-area">
    <div class="input-wrapper" @click="textareaRef?.focus()">
      
      <div class="text-input-section">
        <textarea
          ref="textareaRef"
          v-model="inputMsg"
          @keydown="onKeydown"
          @click.stop
          placeholder="发送消息..."
          class="chat-input modern-scroll"
          rows="1"
        ></textarea>
      </div>

      <div class="tools-section" @click.stop>
        <div class="tools-left" style="display: flex; align-items: center; gap: 4px;">
          <button
            class="icon-btn attach-btn"
            @click="handleAttachClick"
            title="添加文件/图片"
          >
            <span v-html="PLUS_SVG"></span>
          </button>
          <button
            class="icon-btn attach-btn"
            @click="handleThinkClick"
            :title="useReasoning ? '关闭深度思考' : '开启深度思考'"
            :class="{ 'active-think': useReasoning }"
          >
            <span v-html="BRAIN_SVG"></span>
          </button>
        </div>

        <div class="tools-right">
          <button
            class="icon-btn action-btn"
            @click="handleAction"
            :class="{ 'is-stop': isGenerating }"
            :disabled="!isGenerating && !inputMsg.trim()"
          >
            <template v-if="isGenerating">
              <span v-html="STOP_SVG"></span>
            </template>
            <template v-else>
              <span v-html="SEND_SVG"></span>
            </template>
          </button>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
.input-area {
  width: 100%;
  display: flex;
  justify-content: center;
  padding: 10px 0 20px 0;
  background: transparent;
}

.input-wrapper {
  /* --- 📍 [修改宽度] 这里控制输入框的胖瘦 --- */
  width: 85%;      /* 之前是 95%，改小一点 */
  max-width: 800px; /* 之前是 900px，限制最大宽度 */
  /* -------------------------------------- */
  
  background: var(--bg-input-focus);
  border-radius: 30px;
  padding: 16px 20px 10px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition: background-color 0.2s ease, box-shadow 0.2s ease;
  border: none;
  box-shadow: 0 4px 30px rgba(0, 0, 0, 0.2);
  cursor: text;
}

.text-input-section {
  width: 100%;
  display: flex;
  padding: 0 2px; 
}

.chat-input {
  width: 100%;
  background: transparent;
  border: none;
  color: var(--text-color-white);
  font-size: 16px;
  line-height: 1.5;
  resize: none;
  outline: none;
  font-family: inherit;
  padding: 0;
  height: 24px; 
  min-height: 24px;
  max-height: 200px;
  overflow-y: hidden; 
  transition: none;
}

.chat-input:not([style*="height: auto"]) {
  overflow-y: auto;
}

.tools-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  padding-top: 10px;
  padding-bottom: 5px; 
}

/* --- 按钮基础样式 --- */
.icon-btn {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border: none;
  background: transparent;
  color: var(--text-color-white);
  transition: all 0.2s ease;
}

.icon-btn svg {
  width: 18px;
  height: 18px;
  fill: currentColor;
}

.attach-btn {
  opacity: 0.6;
}
.attach-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
  opacity: 1;
}

.attach-btn.active-think {
  color: #818cf8;
  opacity: 1;
}

/* --- 发送/停止 按钮逻辑 --- */

/* 1. 默认状态 (Send) - 幽灵模式 */
.action-btn {
  background-color: transparent; /* 平时透明 */
  color: white;
  opacity: 1;
  transition: background-color 0.2s ease, opacity 0.2s ease, transform 0.1s ease;
}

.action-btn:hover:not(:disabled) {
  background-color: rgba(255, 255, 255, 0.1); /* 悬停显示白圆 */
  transform: scale(1.05);
}

/* 2. 禁用状态 (Disabled) */
.action-btn:disabled {
  opacity: 0.3; 
  background-color: transparent !important; 
  cursor: default; /* 标准箭头，无禁止符号 */
}

/* 3. 停止状态 (Stop) - 实体常驻模式 */
.action-btn.is-stop {
  color: #818cf8; /* 薰衣草紫文字 */
  
  /* 关键修改：默认显示蓝紫色背景，而不是透明 */
  background-color: rgba(165,195,245, 0.2); 
  opacity: 1; 
}

.action-btn.is-stop:hover {
  /* 悬停时加深背景 */
  background-color: rgba(165, 195, 245, 0.35); 
}

.modern-scroll::-webkit-scrollbar { width: 4px; }
.modern-scroll::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
</style>