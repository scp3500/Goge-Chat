<script setup>
import { ref, nextTick, watch, onMounted, onUnmounted } from 'vue';
import { Marked } from 'marked'; 
import { markedHighlight } from "marked-highlight";
import { debounce } from 'lodash-es';
import hljs from 'highlight.js';
import { useChatStore } from "../../stores/chat"; 

const props = defineProps(['messages', 'sessionId', 'initialScrollPos']);
const emit = defineEmits(['update-pos']);

const chatStore = useChatStore();
const scrollRef = ref(null);
const isRestoring = ref(false); 

/**
 * ✨ 配置解析实例 (保持原有逻辑)
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
 * ✨ 吸底指令
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
  // 🩺 只有在非恢复状态且非加载状态时才记录位置，防止跳变时误记录 0
  if (isRestoring.value || !props.sessionId || chatStore.isLoading) return;
  const currentPos = Math.floor(e.target.scrollTop);
  chatStore.updateSessionScroll(props.sessionId, currentPos);
  emit('update-pos', currentPos);
}, 300);

/**
 * 🛠️ 【深度修复】：坐标恢复探针
 * 逻辑改动：增加对 chatStore.isLoading 的监听，确保“数据搬完”后再开始探针
 */
watch([() => props.sessionId, () => chatStore.isLoading], async ([newId, loading]) => {
  // 只有当 ID 存在，且 Store 已经完成从 Rust 的数据加载时才执行
  if (!newId || loading) return;
  
  isRestoring.value = true;

  const performRestore = async (retryCount = 0) => {
    await nextTick();
    
    if (props.messages && props.messages.length > 0 && scrollRef.value) {
      await nextTick(); 
      
      const targetPos = props.initialScrollPos || 0;
      
      setTimeout(() => {
        if (!scrollRef.value) return;
        
        scrollRef.value.scrollTop = targetPos;

        // 验证机制
        if (Math.abs(scrollRef.value.scrollTop - targetPos) > 5 && targetPos > 0 && retryCount < 8) {
          performRestore(retryCount + 1);
        } else {
          setTimeout(() => { isRestoring.value = false; }, 100);
        }
      }, 50);

    } else if (retryCount < 15) {
      setTimeout(() => performRestore(retryCount + 1), 50);
    } else {
      isRestoring.value = false;
    }
  };

  performRestore();
}, { immediate: true });

onMounted(() => {
  scrollRef.value?.addEventListener('scroll', handleScroll);
});

onUnmounted(() => {
  scrollRef.value?.removeEventListener('scroll', handleScroll);
});
</script>

<template>
  <div class="message-display modern-scroll" ref="scrollRef">
    <Transition name="list-fade">
      <div v-if="!chatStore.isLoading" :key="sessionId" class="scroll-content-wrapper">
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
            <div v-else v-html="customMarked.parse(m.content || '')" class="markdown-body"></div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* 消息基础布局 */
.message-display { 
  flex: 1; 
  padding: 40px 6% 60px 6%; 
  display: flex; 
  flex-direction: column; 
  overflow-y: auto; 
  position: relative; /* 新增：为绝对定位提供参考 */
  
  /* ✨ 【关键修复】：彻底禁用滚动锚定 */
  overflow-anchor: none !important; 
  scroll-behavior: auto !important; 
}

.scroll-content-wrapper { 
  display: flex; 
  flex-direction: column; 
  gap: 48px; 
  width: 100%; 
  max-width: 900px; 
  margin: 0 auto; 
  /* 确保切换时内容不超出容器导致滚动条抖动 */
  backface-visibility: hidden;
}

/* ✨ 现代转场动画 */
.list-fade-enter-active {
  transition: all 0.3s ease-out;
}
.list-fade-leave-active {
  /* 🩺 修复点：离开时不占位，防止两个列表同时存在导致撑开容器高度 */
  position: absolute;
  width: 100%;
  opacity: 0;
}
.list-fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
  filter: blur(4px);
}

.message-row { display: flex; width: 100%; animation: fadeIn 0.3s ease-out; }

@keyframes fadeIn { 
  from { opacity: 0; transform: translateY(5px); } 
  to { opacity: 1; transform: translateY(0); } 
}

.message-row.user { justify-content: flex-end; }
.message-row.user .message-bubble { 
  background: #3c4043; 
  padding: 14px 20px; 
  border-radius: 20px 20px 4px 20px; 
  color: #fff; 
  max-width: 80%; 
  word-wrap: break-word; 
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1); 
}

.assistant-content-wrapper { width: 100%; }

/* 加载动画 */
.typing-indicator { display: flex; align-items: center; gap: 6px; padding: 10px 0; background: transparent !important; width: fit-content; }
.typing-indicator span { 
  width: 6px; height: 6px; 
  background-color: rgba(255, 255, 255, 0.25); 
  border-radius: 50%; 
  animation: sophisticated-bounce 1.4s infinite ease-in-out; 
}
.typing-indicator span:nth-child(2) { animation-delay: 0.2s; }
.typing-indicator span:nth-child(3) { animation-delay: 0.4s; }

@keyframes sophisticated-bounce { 
  0%, 60%, 100% { transform: translateY(0); opacity: 0.3; } 
  30% { transform: translateY(-6px); opacity: 1; background-color: #fff; } 
}

/* Markdown 样式 (保持原有) */
.markdown-body { 
  font-size: 16px; 
  line-height: 1.7; 
  color: #e3e3e3; 
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif; 
}

/* ✨ Markdown 表格现代样式 (保持原有) */
:deep(.markdown-body table) {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  margin: 1.5rem 0;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  overflow: hidden;
}

:deep(.markdown-body th) {
  background-color: rgba(255, 255, 255, 0.05);
  padding: 12px 16px;
  text-align: left;
  font-weight: 600;
  color: #ffffff;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

:deep(.markdown-body td) {
  padding: 10px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  color: #e3e3e3;
}

:deep(.markdown-body tr:last-child td) {
  border-bottom: none;
}

:deep(.markdown-body tr:nth-child(even)) {
  background-color: rgba(255, 255, 255, 0.02);
}

/* 行内代码 */
:deep(.markdown-body :not(pre) > code) {
  color: #ffffff !important;
  background-color: rgba(255, 255, 255, 0.1) !important;
  border: 1px solid rgba(255, 255, 255, 0.2) !important;
  padding: 0.15em 0.4em !important;
  border-radius: 6px !important;
  font-family: inherit !important;
  font-weight: 500 !important;
}

/* 代码块 (pre) */
:deep(.markdown-body pre) { 
  background-color: #1e1e1e !important; 
  padding: 1.2rem; 
  border-radius: 12px; 
  overflow-x: auto; 
  margin: 1.5rem 0;
  border: 1px solid rgba(255, 255, 255, 0.05);
  font-family: inherit !important; 
  font-size: 15px;
  white-space: pre;
  overflow-anchor: none !important;
}

:deep(.markdown-body pre::-webkit-scrollbar) { height: 5px; }
:deep(.markdown-body pre::-webkit-scrollbar-thumb) { 
  background: rgba(255, 255, 255, 0.15); 
  border-radius: 10px; 
}

/* 语法高亮 */
:deep(.hljs-keyword) { color: #569cd6; }
:deep(.hljs-string) { color: #ce9178; }
:deep(.hljs-comment) { color: #6a9955; }
:deep(.hljs-function), :deep(.hljs-title), :deep(.hljs-title.function_) { color: #dcdcaa; }
:deep(.hljs-variable), :deep(.hljs-attr) { color: #9cdcfe; }

/* 全局滚动条 */
.modern-scroll::-webkit-scrollbar { width: 6px; }
.modern-scroll::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
.modern-scroll::-webkit-scrollbar-track { background: transparent; }
</style>