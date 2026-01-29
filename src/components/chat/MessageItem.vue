<script setup>
import { ref, nextTick, onMounted, watch, computed } from 'vue';
import SearchSources from './SearchSources.vue';
import { useChatStore } from "../../stores/chat"; 
import { REFRESH_SVG, COPY_SVG, MORE_SVG, CHECK_SVG, BRAIN_SVG, EDIT_SVG, TRASH_SVG, ATTACHMENT_SVG } from '../../constants/icons.ts';
import { renderMarkdown } from '../../services/markdown';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener'; // Import openUrl
import { useConfigStore } from '../../stores/config'; // Import config store
import { getProviderIcon } from '../../assets/icons'; // Import icon helper

const props = defineProps({
  m: Object,
  index: Number,
  sessionId: String,
  isEditing: Boolean
});

const emit = defineEmits(['start-edit', 'cancel-edit', 'save-edit', 'delete', 'regenerate']);

const chatStore = useChatStore();
const configStore = useConfigStore(); // Init config store

// 💡 格式化时间
const formattedDate = computed(() => {
  if (!props.m.created_at) return '';
  const date = new Date(props.m.created_at + 'Z'); // Assume UTC usually, or append Z if needed for Rust's default format
  // Simply formatting as MM/DD HH:mm
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const day = date.getDate().toString().padStart(2, '0');
  const hours = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');
  return `${month}/${day} ${hours}:${minutes}`;
});

// 💡 获取模型图标
const modelIcon = computed(() => {
    // 1. 如果有明确的模型名称
    const modelId = props.m.model;
    if (modelId) {
        // Find provider that has this model
        const provider = configStore.settings.providers.find(p => p.models.includes(modelId));
        if (provider) {
            return getProviderIcon(provider.icon);
        }
    }
    // Fallback: Try to guess from text or just return default
    return getProviderIcon('gemini'); // As per request: "chat with gemini not provider", defaulting to Gemini icon
});

// 💡 获取显示的模型名称
const displayModelName = computed(() => {
    if (props.m.model) return props.m.model;
    // Fallback for old messages
    return "Gemini";
});

// 💡 展开/折叠推理过程
const isReasoningExpanded = ref(false);
const toggleReasoning = () => {
  isReasoningExpanded.value = !isReasoningExpanded.value;
};

// 💡 只有在不生成时，或者是老消息时，才显示功能按钮
const showActionButtons = computed(() => {
  if (!chatStore.isGenerating) return true;
  // 如果正在生成，隐藏最后两条消息（当前的 User 和 Assistant）的按钮
  const total = chatStore.currentMessages.length;
  return props.index < total - 2;
});

// 解析搜索元数据
const searchResults = computed(() => {
  if (!props.m.searchMetadata) return [];
  try {
    return JSON.parse(props.m.searchMetadata);
  } catch (e) {
    return [];
  }
});

// 💡 统一复制函数
const doCopy = async (text, el) => {
  try {
    await navigator.clipboard.writeText(text);
    const original = el.innerHTML;
    // 临时改变内容为“已复制”+图标
    el.innerHTML = `<span class="copy-text">已复制</span>${CHECK_SVG}`;
    el.classList.add('copied');
    setTimeout(() => {
      el.innerHTML = original;
      el.classList.remove('copied');
    }, 2000);
  } catch (err) { console.error('复制失败', err); }
};

// 💡 格式化用户文本,每30个字符换行
const formatUserText = (text) => {
  if (!text) return '';
  // 如果是 [REASON] 或 [SEARCH] 等标记，先清理掉展示
  let cleanText = text.replace(/\[REASON\]|\[SEARCH\]/g, '');
  // 去掉附件正文内容的显示
  const attachmentTag = "\n\n--- 附件内容 ---";
  const index = cleanText.indexOf(attachmentTag);
  if (index !== -1) {
    cleanText = cleanText.substring(0, index);
  }
  return cleanText.trim();
};

const hasVisibleContent = computed(() => {
  const formatted = formatUserText(props.m.content);
  return formatted.length > 0;
});

const parsedFiles = computed(() => {
  if (!props.m.fileMetadata) return [];
  try {
    return JSON.parse(props.m.fileMetadata);
  } catch (e) {
    console.error("解析文件元数据失败:", e);
    return [];
  }
});

const handleOpenFile = async (path) => {
  try {
    await invoke('open_file', { path });
  } catch (e) {
    console.error("打开文件失败:", e);
  }
};

// 💡 保持原功能的按钮注入
const messageRef = ref(null);
const injectCodeButtons = () => {
  nextTick(() => {
    if (!messageRef.value || !showActionButtons.value) return;
    // 查找 wrapper，如果已经有wrapper则跳过，或者查找 pre not(.processed)
    const pres = messageRef.value.querySelectorAll('pre:not(.processed)');
    
    pres.forEach(pre => {
      pre.classList.add('processed');
      
      // 1. 获取语言类型
      const codeEl = pre.querySelector('code');
      let lang = 'text';
      if (codeEl) {
        // 尝试从 class 中解析 language-xxx
        const classes = codeEl.className.split(' ');
        const langClass = classes.find(c => c.startsWith('language-'));
        if (langClass) {
          lang = langClass.replace('language-', '');
        } else if (codeEl.className.includes('hljs')) {
            // fallback logic if needed
        }
      }

      // 2. 创建容器
      const wrapper = document.createElement('div');
      wrapper.className = 'code-block-wrapper';
      
      // 3. 创建头部
      const header = document.createElement('div');
      header.className = 'code-block-header';
      
      // 语言标签
      const langSpan = document.createElement('span');
      langSpan.className = 'code-lang';
      langSpan.textContent = lang;
      
      // 复制按钮
      const btn = document.createElement('button');
      btn.className = 'code-copy-btn';
      // 默认文字 + 图标
      btn.innerHTML = `<span class="copy-text">复制代码</span>${COPY_SVG}`;
      btn.onclick = (e) => { 
        e.stopPropagation(); 
        doCopy(pre.innerText.trim(), btn); 
      };

      header.appendChild(langSpan);
      header.appendChild(btn);
      
      // 4. DOM 重组
      if (pre.parentNode) {
        pre.parentNode.insertBefore(wrapper, pre);
        wrapper.appendChild(header);
        wrapper.appendChild(pre);
      }
    });
  });
};

onMounted(injectCodeButtons);
watch(() => props.m.content, injectCodeButtons);
watch(showActionButtons, (val) => {
  if (val) injectCodeButtons();
});

const editTextarea = ref(null);

watch(() => props.isEditing, (newVal) => {
  if (newVal) {
    nextTick(() => {
      editTextarea.value?.focus();
    });
  }
});
// 💡 处理链接点击
// 💡 处理链接点击
const handleLinkClick = async (event) => {
  const link = event.target.closest('a');
  if (link && link.href) {
    event.preventDefault();
    try {
      await openUrl(link.href);
    } catch (err) {
      console.error('Failed to open link:', err);
    }
  }
};
</script>

<template>
  <div class="message-row" :class="String(m.role || 'user').toLowerCase()" ref="messageRef">
    
    <div v-if="m.role === 'user'" class="message-bubble-wrapper">
      <div class="user-turn-content">
        <!-- 文字气泡 -->
        <div v-if="hasVisibleContent || isEditing" class="message-bubble" :class="{ 'is-editing': isEditing }">
          <template v-if="isEditing">
            <textarea
              ref="editTextarea"
              :value="m.content"
              class="edit-textarea modern-scroll"
              @input="$emit('update-edit-content', $event.target.value)"
              @keydown.esc="$emit('cancel-edit')"
              @keydown.ctrl.enter="$emit('save-edit')"
            ></textarea>
            <div class="edit-actions">
              <button class="edit-cancel" @click="$emit('cancel-edit')">取消</button>
              <button class="edit-save" @click="e => $emit('save-edit', e)">保存并重新生成</button>
            </div>
          </template>
          <template v-else>
            <div class="user-text">{{ formatUserText(m.content) }}</div>
          </template>
        </div>

        <!-- 文件显示区 (独立气泡) -->
        <div v-if="parsedFiles.length > 0" class="message-file-attachments">
          <div 
            v-for="file in parsedFiles" 
            :key="file.path" 
            class="message-file-card"
            @dblclick="handleOpenFile(file.path)"
            title="双击打开文件"
          >
            <div class="m-file-icon" v-html="file.icon || ATTACHMENT_SVG"></div>
            <div class="m-file-info">
              <span class="m-file-name text-ellipsis">{{ file.name }}</span>
            </div>
            <button class="m-open-btn" @click.stop="handleOpenFile(file.path)" title="打开文件">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                <polyline points="15 3 21 3 21 9"></polyline>
                <line x1="10" y1="14" x2="21" y2="3"></line>
              </svg>
            </button>
          </div>
        </div>
      </div>
      
      <div v-if="!isEditing && showActionButtons" class="msg-action-bar-user">
        <button class="action-btn" title="编辑" @click="$emit('start-edit')" v-html="EDIT_SVG"></button>
        <button class="action-btn" title="删除" @click="e => $emit('delete', e)" v-html="TRASH_SVG"></button>
      </div>
    </div>

    <div v-else class="assistant-content-wrapper">
      <!-- 🆕 Assistant Header -->
      <div class="assistant-header" v-if="m.content !== '__LOADING__'">
        <div class="header-left">
          <div class="avatar-icon" v-html="modelIcon"></div>
          <span class="model-name-text">{{ displayModelName }}</span>
          <div class="divider">|</div>
          <span class="model-name-text" style="opacity: 0.7;">New API</span>
        </div>
        <span class="time-text">{{ formattedDate }}</span>
      </div>

      <template v-if="m.content === '__LOADING__' && !m.reasoningContent">
        <div class="typing-indicator"><span></span><span></span><span></span></div>
      </template>
      <template v-else>
        <div v-if="m.reasoningContent" class="reasoning-container">
          <div class="reasoning-status" @click="toggleReasoning">
            <span class="status-icon" v-html="BRAIN_SVG"></span>
            <span class="status-text">{{ m.content === '__LOADING__' ? '正在思考...' : '思考过程' }}</span>
            <span class="status-arrow" :class="{ 'is-expanded': isReasoningExpanded }">
              <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="6 9 12 15 18 9"></polyline>
              </svg>
            </span>
          </div>
          <Transition name="collapse">
            <div v-if="isReasoningExpanded" class="reasoning-content">
              <div class="reasoning-inner">{{ m.reasoningContent }}</div>
            </div>
          </Transition>
        </div>

        <!-- 搜索结果显示 -->
        <SearchSources 
          v-if="m.searchStatus || searchResults.length > 0"
          :results="searchResults"
          :status="m.searchStatus || 'done'"
          :query="m.searchQuery"
        />

        <div v-if="m.content !== '__LOADING__'" v-html="renderMarkdown(m.content)" class="markdown-body" @click="handleLinkClick"></div>
        <div v-else-if="m.reasoningContent" class="typing-indicator small"><span></span><span></span><span></span></div>
        
        <div v-if="m.content !== '__LOADING__' && showActionButtons" class="msg-action-bar-bottom">
          <button class="action-btn" title="重新生成" @click="chatStore.regenerateAction(index)" v-html="REFRESH_SVG"></button>
          <button class="action-btn" title="复制全文" @click="e => doCopy(m.content, e.currentTarget)" v-html="COPY_SVG"></button>
          <button class="action-btn delete-btn" title="删除" @click="e => $emit('delete', e)" v-html="TRASH_SVG"></button>
          <button class="action-btn" title="更多" v-html="MORE_SVG"></button>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
/* 拷贝原来的样式 */
.assistant-content-wrapper { position: relative; width: 100%; display: flex; flex-direction: column; gap: 8px; }

/* 🆕 Header Styles */
.assistant-header {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 8px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.avatar-icon {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}
.avatar-icon :deep(svg) { width: 100%; height: 100%; }

.model-name-text {
  font-size: 15px;
  font-weight: 600;
  color: #e3e3e3;
  letter-spacing: 0.3px;
}

.divider {
  color: rgba(255, 255, 255, 0.2);
  font-size: 14px;
  margin: 0 4px;
}

.time-text {
  font-size: 12px;
  color: #7b7d81; /* Dimmer text for time */
  margin-left: 2px;
  font-family: 'Roboto Mono', monospace;
}
.msg-action-bar-bottom { display: flex; gap: 4px; margin-top: 10px; padding-left: 2px; }
.action-btn { background: transparent; border: none; color: #707070; cursor: pointer; padding: 6px; border-radius: 6px; display: flex; align-items: center; transition: all 0.2s; }
.action-btn:hover { color: #ffffff; background: rgba(255, 255, 255, 0.06); }
.action-btn.delete-btn:hover { color: #ff4d4f; background: rgba(255, 77, 79, 0.1); }

.message-bubble-wrapper { display: flex; flex-direction: column; align-items: flex-end; max-width: 85%; }
.user-turn-content { display: flex; flex-direction: column; align-items: flex-end; gap: 8px; width: 100%; }
.msg-action-bar-user { display: flex; gap: 4px; margin-top: 4px; opacity: 0; visibility: hidden; transition: all 0.2s; }
.message-bubble-wrapper:hover .msg-action-bar-user { opacity: 1; visibility: visible; }

.message-bubble { padding: 12px 16px; border-radius: 18px; background: #3a3a3c; color: #fff; max-width: 100%; word-wrap: break-word; white-space: pre-wrap; }
.message-bubble.is-editing { width: 100%; padding: 12px; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.12); border-radius: 14px; }
.edit-textarea { width: 100%; min-height: 100px; background: transparent; border: none; color: #fff; font-size: 15px; line-height: 1.6; resize: vertical; outline: none; font-family: inherit; }
.edit-actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 10px; }
.edit-actions button { padding: 6px 14px; border-radius: 6px; font-size: 13px; cursor: pointer; border: none; }
.edit-cancel { background: rgba(255, 255, 255, 0.05); color: rgba(255, 255, 255, 0.6); }
.edit-save { background: #4f46e5; color: #fff; }

.message-row { display: flex; width: 100%; }
.message-row.user { justify-content: flex-end; }

/* Modern Typing Indicator - Minimal & Clean */
.typing-indicator {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 8px 0; /* No bubble padding */
  background: transparent; /* No background */
  border: none; /* No border */
  border-radius: 0;
  width: fit-content;
  margin-top: 2px;
  box-shadow: none;
  backdrop-filter: none;
}

/* Reset small modifer as it's now the default, but keep for specificity if needed */
.typing-indicator.small {
  padding: 8px 0;
  margin: 0;
}

.typing-indicator span {
  width: 6px; /* Smaller dots */
  height: 6px;
  background: rgba(255, 255, 255, 0.5); /* Subtle color */
  border-radius: 50%;
  animation: liquid-flow 1.4s ease-in-out infinite both;
  box-shadow: none; /* Remove dot shadow */
}

.typing-indicator span:nth-child(1) { animation-delay: -0.32s; }
.typing-indicator span:nth-child(2) { animation-delay: -0.16s; }
.typing-indicator span:nth-child(3) { animation-delay: 0s; }

@keyframes liquid-flow {
  0%, 100% {
    transform: translateY(0);
    opacity: 0.4;
  }
  50% {
    transform: translateY(-4px); /* Reduce excessive bounce */
    opacity: 1;
    background: #ffffff;
  }
}


.markdown-body { font-size: 16px; line-height: 1.7; color: #e3e3e3; }
.reasoning-container { margin-bottom: 16px; display: flex; flex-direction: column; }
.reasoning-status { display: flex; align-items: center; gap: 6px; padding: 4px 8px; cursor: pointer; color: rgba(255, 255, 255, 0.45); font-size: 13px; border-radius: 6px; width: fit-content; }
.status-icon { width: 14px; height: 14px; display: flex; align-items: center; color: #ffffff; opacity: 0.8; }
.status-arrow { transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
.status-arrow.is-expanded { transform: rotate(180deg); }
.reasoning-content { margin-top: 4px; padding-left: 12px; position: relative; overflow: hidden; }
.reasoning-inner { font-size: 14px; line-height: 1.6; color: rgba(255, 255, 255, 0.4); white-space: pre-wrap; padding: 4px 0 8px 0; }
.collapse-enter-active, .collapse-leave-active { transition: all 0.3s ease-out; max-height: 500px; }
.collapse-enter-from, .collapse-leave-to { max-height: 0; opacity: 0; }

/* 代码块增强样式 (Gemini Reference Match) */
:deep(.code-block-wrapper) {
  margin: 1.5rem 0;
  border-radius: 0 !important;
  background-color: transparent !important;
  border: none !important;
  padding: 0 !important;
  width: 100%;
  box-sizing: border-box;
  scrollbar-gutter: auto !important;
}

:deep(.code-block-header) {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px; 
  background-color: var(--bg-code-header) !important; /* ⚡️ 修复：使用变量 */
  border-bottom: none;
  font-family: 'Google Sans', 'Segoe UI', system-ui, sans-serif;
  font-size: 13px;
  color: var(--text-color); /* 使用变量 */
  user-select: none;
  width: 100%;
  box-sizing: border-box;
  
  /* 吸顶效果 */
  position: sticky;
  top: 0;
  z-index: 10;
  
  /* ⚡️ 修复：上圆角 (强制) */
  border-radius: 12px 12px 0 0 !important;
  
  margin-bottom: 0;
  box-shadow: none;
}

/* ⚡️ 修复：伪元素遮罩 - 使用页面背景色变量 */
:deep(.code-block-header)::after {
  content: '';
  position: absolute;
  left: 0;
  bottom: -4px;
  width: 100%;
  height: 4px;
  background-color: var(--bg-main); /* ⚡️ 核心修复：使用 --bg-main 变量 */
  z-index: 10;
}

:deep(.code-lang) {
  font-weight: 500;
  opacity: 0.9;
}

:deep(.markdown-body pre) {
  margin: 0 !important;
  border: none !important;
  /* ⚡️ 修复：下圆角 (强制) */
  border-radius: 0 0 12px 12px !important;
  /* ⚡️ 修复：padding 适配 4px 缝隙 */
  padding: 20px 20px 16px 20px !important; 
  background-color: var(--bg-code) !important; /* ⚡️ 修复：使用变量 */
  font-family: 'Roboto Mono', 'Consolas', monospace;
  font-size: 14px;
  line-height: 1.6;
}

:deep(.code-copy-btn) {
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #c4c7c5; /* Google 灰色图标 */
  cursor: pointer;
  padding: 8px; /* 增加点击区域 */
  border-radius: 50%; /* 圆形按钮或者圆角矩形 */
  transition: all 0.2s ease;
  width: 32px;
  height: 32px;
}

:deep(.code-copy-btn:hover) {
  background-color: rgba(255, 255, 255, 0.08);
  color: #e3e3e3;
}

:deep(.code-copy-btn svg) {
  width: 18px; /* 图标稍大 */
  height: 18px;
}

:deep(.copy-text) {
  display: none; /* 回归 Google 风格，不显示文字 */
}

:deep(.copied) { 
  color: #4ade80 !important; 
}

/* 消息中的文件卡片样式 */
.message-file-attachments {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 10px;
}

.message-file-card {
  display: flex;
  align-items: center;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 8px 12px;
  gap: 10px;
  cursor: pointer;
  transition: all 0.2s;
  min-width: 180px;
}

.message-file-card:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.2);
}

.m-file-icon {
  color: #818cf8;
  display: flex;
  align-items: center;
}

.m-file-icon :deep(svg) {
  width: 20px;
  height: 20px;
}

.m-file-info {
  flex: 1;
  overflow: hidden;
}

.m-file-name {
  font-size: 13px;
  color: #efefef;
  display: block;
}

.text-ellipsis {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.m-open-btn {
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  transition: all 0.2s;
}

.m-open-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}
</style>
