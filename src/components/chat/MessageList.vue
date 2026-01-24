<script setup>
import { ref, nextTick, watch, onMounted, onUnmounted } from 'vue';
import { debounce } from 'lodash-es';
import { useChatStore } from "../../stores/chat"; 
import { REFRESH_SVG, COPY_SVG, MORE_SVG, CHECK_SVG, BRAIN_SVG } from '../../constants/icons.ts';
import { renderMarkdown } from '../../services/markdown';
import { useScrollRestore } from './composables/useScrollRestore';

const props = defineProps(['messages', 'sessionId', 'initialScrollPos']);
const emit = defineEmits(['update-pos', 'refresh']);

const chatStore = useChatStore();
const scrollRef = ref(null);
const isRestoring = ref(false); 
const { performRestore } = useScrollRestore();

// 💡 展开/折叠推理过程
const expandedReasoning = ref(new Set());
const toggleReasoning = (index) => {
  if (expandedReasoning.value.has(index)) {
    expandedReasoning.value.delete(index);
  } else {
    expandedReasoning.value.add(index);
  }
};

// 自动展开包含推理内容的最后一条消息
const autoExpandLastReasoning = () => {
  if (!props.messages) return;
  props.messages.forEach((m, i) => {
    if (m.reasoning_content && m.reasoning_content.trim()) {
      expandedReasoning.value.add(i);
    }
  });
};

// 💡 统一复制函数
const doCopy = async (text, el) => {
  try {
    await navigator.clipboard.writeText(text);
    const original = el.innerHTML;
    el.innerHTML = CHECK_SVG;
    el.classList.add('copied');
    setTimeout(() => {
      el.innerHTML = original;
      el.classList.remove('copied');
    }, 2000);
  } catch (err) { console.error('复制失败', err); }
};

// 💡 格式化用户文本，每30个字符换行
const formatUserText = (text) => {
  return text.replace(/(.{30})/g, '$1\n');
};

// 💡 保持原功能的按钮注入
const injectCodeButtons = () => {
  nextTick(() => {
    const pres = document.querySelectorAll('.markdown-body pre:not(.has-copy-btn)');
    pres.forEach(pre => {
      pre.classList.add('has-copy-btn');
      const btn = document.createElement('button');
      btn.className = 'code-copy-btn';
      btn.innerHTML = COPY_SVG;
      btn.onclick = (e) => { e.stopPropagation(); doCopy(pre.innerText.trim(), btn); };
      pre.prepend(btn);
    });
  });
};

// 💡 暴露给父组件的滚动方法
defineExpose({ scrollToBottom: () => {
  if (!isRestoring.value && scrollRef.value) scrollRef.value.scrollTop = scrollRef.value.scrollHeight;
}});

const handleScroll = debounce((e) => {
  if (isRestoring.value || !props.sessionId || chatStore.isLoading) return;
  chatStore.updateSessionScroll(props.sessionId, Math.floor(e.target.scrollTop));
  emit('update-pos', Math.floor(e.target.scrollTop));
}, 300);

// 核心监听：数据变化触发按钮注入
watch(() => props.messages, injectCodeButtons, { deep: true });

// 核心监听：切换会话触发坐标恢复
watch([() => props.sessionId, () => chatStore.isLoading], async ([newId, loading]) => {
  if (!newId || loading) return;
  isRestoring.value = true;
  
  if (props.messages?.length > 0) {
    await performRestore(scrollRef.value, props.initialScrollPos || 0);
    injectCodeButtons();
    autoExpandLastReasoning();
  }
  
  setTimeout(() => { isRestoring.value = false; }, 500);
}, { immediate: true });

onMounted(() => {
  scrollRef.value?.addEventListener('scroll', handleScroll);
  injectCodeButtons();
});

onUnmounted(() => scrollRef.value?.removeEventListener('scroll', handleScroll));
</script>

<template>
  <div class="message-display modern-scroll" ref="scrollRef">
    <Transition name="list-fade">
      <div v-if="!chatStore.isLoading" :key="sessionId" class="scroll-content-wrapper">
        <div v-for="(m, i) in messages" :key="i" 
             class="message-row" :class="String(m.role || 'user').toLowerCase()">
          
          <div v-if="m.role === 'user'" class="message-bubble">
            <div class="user-text">{{ formatUserText(m.content) }}</div>
          </div>

          <div v-else class="assistant-content-wrapper">
            <template v-if="m.content === '__LOADING__' && !m.reasoning_content">
              <div class="typing-indicator"><span></span><span></span><span></span></div>
            </template>
            <template v-else>
              <!-- 🧠 推理过程展示 -->
              <div v-if="m.reasoning_content" class="reasoning-container">
                <div class="reasoning-status" @click="toggleReasoning(i)">
                  <span class="status-icon" v-html="BRAIN_SVG"></span>
                  <span class="status-text">{{ m.content === '__LOADING__' ? '正在思考...' : '思考过程' }}</span>
                  <span class="status-arrow" :class="{ 'is-expanded': expandedReasoning.has(i) }">
                    <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5">
                      <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
                  </span>
                </div>
                <Transition name="collapse">
                  <div v-if="expandedReasoning.has(i)" class="reasoning-content">
                    <div class="reasoning-inner">{{ m.reasoning_content }}</div>
                  </div>
                </Transition>
              </div>

              <!-- 正文内容 -->
              <div v-if="m.content !== '__LOADING__'" v-html="renderMarkdown(m.content)" class="markdown-body"></div>
              <div v-else-if="m.reasoning_content" class="typing-indicator small"><span></span><span></span><span></span></div>
              
              <div v-if="m.content !== '__LOADING__'" class="msg-action-bar-bottom">
                <button class="action-btn" title="重新生成" @click="emit('refresh', m)" v-html="REFRESH_SVG"></button>
                <button class="action-btn" title="复制全文" @click="e => doCopy(m.content, e.currentTarget)" v-html="COPY_SVG"></button>
                <button class="action-btn" title="更多" v-html="MORE_SVG"></button>
              </div>
            </template>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.assistant-content-wrapper { 
  position: relative; 
  width: 100%;
  display: flex;
  flex-direction: column;
}

/* 🚩 左下角工具栏：默认显示 */
.msg-action-bar-bottom {
  display: flex;
  gap: 4px;
  margin-top: 10px;
  opacity: 1;
  visibility: visible;
  transform: translateY(0);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  padding-left: 2px;
}

.assistant-content-wrapper:hover .msg-action-bar-bottom { 
  opacity: 1; 
  visibility: visible;
  transform: translateY(0);
}

.action-btn {
  background: transparent;
  border: none;
  color: #707070;
  cursor: pointer;
  padding: 6px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  transition: all 0.2s;
}

.action-btn:hover {
  color: #ffffff;
  background: rgba(255, 255, 255, 0.06);
}

/* 🚩 代码块：确保滚动条显示 */
:deep(.markdown-body pre) { 
  position: relative; 
  display: block; 
  overflow-x: auto !important;
  overflow-y: hidden;
  background-color: var(--bg-code) !important; 
  padding: 1.2rem; 
  padding-right: 3.5rem;
  border-radius: 12px; 
  margin: 1.5rem 0; 
  border: 1px solid rgba(255, 255, 255, 0.05);
}

:deep(.code-copy-btn) {
  position: absolute; 
  top: 10px; 
  right: 10px;
  z-index: 50;
  display: flex;
  background: transparent !important;
  border: none;
  color: #888;
  padding: 6px;
  cursor: pointer;
  opacity: 0;
  visibility: hidden;
  transition: opacity 0.2s ease, visibility 0.2s, color 0.2s ease;
}

:deep(pre:hover .code-copy-btn) { 
  opacity: 1; 
  visibility: visible;
}

:deep(.code-copy-btn:hover) { 
  color: #ffffff !important; 
}

.copied, :deep(.copied) { color: #4ade80 !important; }

/* --- 基础 UI 框架 --- */
.message-display { flex: 1; padding: 40px 6% 60px 6%; display: flex; flex-direction: column; overflow-y: auto; position: relative; overflow-anchor: none !important; scroll-behavior: auto !important; }
.scroll-content-wrapper { display: flex; flex-direction: column; gap: 48px; width: 100%; margin: 0 auto; backface-visibility: hidden; }

.list-fade-enter-active { transition: all 0.3s ease-out; }
.list-fade-leave-active { position: absolute; width: 100%; opacity: 0; }
.list-fade-enter-from { opacity: 0; transform: translateY(10px); filter: blur(4px); }

.message-row { display: flex; width: 100%; animation: fadeIn 0.3s ease-out; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(5px); } to { opacity: 1; transform: translateY(0); } }

.message-row.user { justify-content: flex-end; }

.typing-indicator { display: flex; align-items: center; gap: 6px; padding: 10px 0; background: transparent !important; width: fit-content; }
.typing-indicator span { width: 6px; height: 6px; background-color: rgba(255, 255, 255, 0.25); border-radius: 50%; animation: sophisticated-bounce 1.4s infinite ease-in-out; }
.typing-indicator span:nth-child(2) { animation-delay: 0.2s; }
.typing-indicator span:nth-child(3) { animation-delay: 0.4s; }
@keyframes sophisticated-bounce { 0%, 60%, 100% { transform: translateY(0); opacity: 0.3; } 30% { transform: translateY(-6px); opacity: 1; background-color: #fff; } }

.markdown-body { font-size: 16px; line-height: 1.7; color: #e3e3e3; }

/* 🧠 推理过程样式 - 现代化重构 */
.reasoning-container {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
}

.reasoning-status {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  cursor: pointer;
  user-select: none;
  color: rgba(255, 255, 255, 0.45);
  font-size: 13px;
  border-radius: 6px;
  width: fit-content;
  transition: all 0.2s ease;
}

.reasoning-status:hover {
  background: rgba(255, 255, 255, 0.05);
  color: rgba(255, 255, 255, 0.7);
}

.status-icon {
  width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  color: #818cf8;
}

.status-arrow {
  display: flex;
  align-items: center;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  opacity: 0.6;
}

.status-arrow.is-expanded {
  transform: rotate(180deg);
}

.reasoning-content {
  margin-top: 4px;
  padding-left: 12px;
  position: relative;
  overflow: hidden;
}

/* 现代化左侧线条 */
.reasoning-content::before {
  content: "";
  position: absolute;
  left: 0;
  top: 4px;
  bottom: 4px;
  width: 2px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.reasoning-inner {
  font-size: 14px;
  line-height: 1.6;
  color: rgba(255, 255, 255, 0.4);
  white-space: pre-wrap;
  padding: 4px 0 8px 0;
}

.collapse-enter-active, .collapse-leave-active {
  transition: all 0.3s ease-out;
  max-height: 500px;
}
.collapse-enter-from, .collapse-leave-to {
  max-height: 0;
  opacity: 0;
  padding-bottom: 0;
}

.typing-indicator.small {
  padding: 5px 0;
}
.typing-indicator.small span {
  width: 4px;
  height: 4px;
}

/* 🚩 表格样式 */
:deep(.markdown-body table) { width: 100%; border-collapse: separate; border-spacing: 0; margin: 1.5rem 0; border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 10px; overflow: hidden; }
:deep(.markdown-body th) { background-color: rgba(255, 255, 255, 0.05); padding: 12px 16px; text-align: left; font-weight: 600; color: #ffffff; border-bottom: 1px solid rgba(255, 255, 255, 0.1); }
:deep(.markdown-body td) { padding: 10px 16px; border-bottom: 1px solid rgba(255, 255, 255, 0.05); color: #e3e3e3; }
:deep(.markdown-body tr:last-child td) { border-bottom: none; }
:deep(.markdown-body tr:nth-child(even)) { background-color: rgba(255, 255, 255, 0.02); }

:deep(.markdown-body :not(pre) > code) { color: #C2C5C3 !important; background-color: rgba(255, 255, 255, 0.1) !important; border: 0px solid rgba(255, 255, 255, 0.2) !important; padding: 0.15em 0.4em !important; border-radius: 6px !important; font-family: inherit !important; font-weight: 500 !important; }

/* 🚩 代码高亮颜色已在main.css中定义 */

.modern-scroll::-webkit-scrollbar { width: 6px; }
.modern-scroll::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
.modern-scroll::-webkit-scrollbar-track { background: transparent; }
</style>