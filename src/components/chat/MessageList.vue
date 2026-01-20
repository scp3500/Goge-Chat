<script setup>
import { ref, nextTick, watch, onMounted, onUnmounted } from 'vue';
import { Marked } from 'marked'; 
import { markedHighlight } from "marked-highlight";
import { invoke } from '@tauri-apps/api/core';
import { debounce } from 'lodash-es';
import hljs from 'highlight.js';

const props = defineProps(['messages', 'sessionId', 'initialScrollPos']);
const emit = defineEmits(['update-pos']);

const scrollRef = ref(null);
const isRestoring = ref(false); 
let activeSessionId = null;

/**
 * ✨ 配置解析实例
 */
const customMarked = new Marked(
  markedHighlight({
    langPrefix: 'hljs language-',
    highlight(code, lang) {
      const language = hljs.getLanguage(lang) ? lang : 'plaintext';
      return hljs.highlight(code, { language }).value;
    }
  })
);
customMarked.setOptions({ breaks: true, gfm: true, mangle: false, headerIds: false });

/**
 * ✨ 给父组件调用的指令：吸底
 */
const scrollToBottom = () => {
  if (isRestoring.value || !scrollRef.value) return;
  scrollRef.value.scrollTop = scrollRef.value.scrollHeight;
};

defineExpose({ scrollToBottom });

/**
 * ✨ 位置保存
 */
const handleScroll = debounce((e) => {
  if (isRestoring.value || !props.sessionId) return;
  const currentPos = Math.floor(e.target.scrollTop);
  if (props.initialScrollPos > 0 && currentPos === 0) return;
  
  invoke('update_session_scroll', { id: props.sessionId, pos: currentPos })
    .catch(err => console.error("Persistence Error:", err));
  emit('update-pos', currentPos);
}, 300);

/**
 * ✨ 坐标恢复
 */
watch([() => props.sessionId, () => props.messages], async ([newId, newMsgs]) => {
  if (!newId) return;
  if (activeSessionId !== newId) {
    isRestoring.value = true;
    activeSessionId = newId;
  }
  if (isRestoring.value && newMsgs && newMsgs.length > 0) {
    await nextTick(); 
    if (scrollRef.value) {
      scrollRef.value.scrollTop = props.initialScrollPos || 0;
      setTimeout(() => {
        isRestoring.value = false;
      }, 600);
    }
  }
}, { immediate: true, deep: true });

onMounted(() => {
  scrollRef.value?.addEventListener('scroll', handleScroll);
});

onUnmounted(() => {
  scrollRef.value?.removeEventListener('scroll', handleScroll);
});
</script>

<template>
  <div class="message-display modern-scroll" ref="scrollRef">
    <div class="scroll-content-wrapper">
      <div v-for="(m, i) in messages" :key="i" 
           class="message-row" 
           :class="String(m.role || 'user').toLowerCase()">
        <div v-if="m.role === 'user'" class="message-bubble">
          <div class="user-text">{{ m.content }}</div>
        </div>
        <div v-else class="assistant-content-wrapper">
          <template v-if="m.content === '__LOADING__'">
            <div class="typing-indicator"><span></span><span></span><span></span></div>
          </template>
          <div v-else v-html="customMarked.parse(m.content)" class="markdown-body"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 消息基础布局 */
.message-display { flex: 1; padding: 40px 6% 60px 6%; display: flex; flex-direction: column; overflow-y: auto; overflow-anchor: auto; scroll-behavior: auto !important; }
.scroll-content-wrapper { display: flex; flex-direction: column; gap: 48px; width: 100%; max-width: 900px; margin: 0 auto; }
.message-row { display: flex; width: 100%; animation: fadeIn 0.3s ease-out; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(5px); } to { opacity: 1; transform: translateY(0); } }

.message-row.user { justify-content: flex-end; }
.message-row.user .message-bubble { background: #3c4043; padding: 14px 20px; border-radius: 20px 20px 4px 20px; color: #fff; max-width: 80%; word-wrap: break-word; box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1); }
.assistant-content-wrapper { width: 100%; }

/* 加载动画 */
.typing-indicator { display: flex; align-items: center; gap: 6px; padding: 10px 0; background: transparent !important; width: fit-content; }
.typing-indicator span { width: 6px; height: 6px; background-color: rgba(255, 255, 255, 0.25); border-radius: 50%; animation: sophisticated-bounce 1.4s infinite ease-in-out; }
.typing-indicator span:nth-child(2) { animation-delay: 0.2s; }
.typing-indicator span:nth-child(3) { animation-delay: 0.4s; }
@keyframes sophisticated-bounce { 0%, 60%, 100% { transform: translateY(0); opacity: 0.3; } 30% { transform: translateY(-6px); opacity: 1; background-color: #fff; } }

/* Markdown 整体样式 */
.markdown-body { font-size: 16px; line-height: 1.7; color: #e3e3e3; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif; }

/* 代码块容器及其内嵌滚动条修复 */
:deep(.markdown-body pre) { 
  background-color: #1e1e1e !important; 
  padding: 1.2rem; 
  border-radius: 12px; 
  overflow-x: auto; /* 核心：允许横向滚动 */
  margin: 1.5rem 0;
  border: 1px solid rgba(255, 255, 255, 0.05);
  font-family: inherit !important; 
  font-size: 15px;
  /* 解决某些浏览器下的内缩问题 */
  white-space: pre;
}

/* ✨ 【关键修复】：为代码块内部横向滚动条应用自定义样式 */
:deep(.markdown-body pre::-webkit-scrollbar) {
  height: 5px; /* 横向滚动条高度 */
}

:deep(.markdown-body pre::-webkit-scrollbar-thumb) {
  background: rgba(255, 255, 255, 0.15); /* 稍微比背景亮一点点 */
  border-radius: 10px;
}

:deep(.markdown-body pre::-webkit-scrollbar-thumb:hover) {
  background: rgba(255, 255, 255, 0.25);
}

:deep(.markdown-body pre::-webkit-scrollbar-track) {
  background: transparent;
}

/* 语法高亮 */
:deep(.hljs-keyword), :deep(.hljs-selector-tag), :deep(.hljs-literal), :deep(.hljs-section), :deep(.hljs-link), :deep(.hljs-type), :deep(.hljs-built_in) { color: #569cd6; }
:deep(.hljs-function), :deep(.hljs-title), :deep(.hljs-title.function_) { color: #dcdcaa; }
:deep(.hljs-string), :deep(.hljs-attribute), :deep(.hljs-symbol), :deep(.hljs-bullet), :deep(.hljs-addition) { color: #ce9178; }
:deep(.hljs-class), :deep(.hljs-title.class_), :deep(.hljs-name), :deep(.hljs-selector-id), :deep(.hljs-selector-class) { color: #4ec9b0; }
:deep(.hljs-comment), :deep(.hljs-quote) { color: #6a9955; font-style: normal; }
:deep(.hljs-number), :deep(.hljs-regexp) { color: #b5cea8; }
:deep(.hljs-variable), :deep(.hljs-template-variable), :deep(.hljs-attr) { color: #9cdcfe; }
:deep(.hljs-constant), :deep(.hljs-meta) { color: #4fc1ff; font-style: normal; }

/* 其他元素 */
:deep(.markdown-body table) { width: 100%; border-collapse: collapse; margin: 1.2rem 0; font-size: 15px; }
:deep(.markdown-body th), :deep(.markdown-body td) { padding: 10px 14px; border: 1px solid rgba(255, 255, 255, 0.12); text-align: left; }
:deep(.markdown-body th) { background-color: rgba(255, 255, 255, 0.05); font-weight: 600; color: #ffffff; }
:deep(.markdown-body :not(pre) > code) { background-color: rgba(255, 255, 255, 0.08) !important; padding: 0.15em 0.4em !important; border-radius: 5px !important; color: inherit !important; font-family: inherit !important; font-weight: 500 !important; border: 1px solid rgba(255, 255, 255, 0.05) !important; }

/* 现代全局滚动条 */
.modern-scroll::-webkit-scrollbar { width: 6px; }
.modern-scroll::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
.modern-scroll::-webkit-scrollbar-track { background: transparent; }
</style>