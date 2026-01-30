<script setup>
import { ref, nextTick, onMounted, watch, computed } from 'vue';
import { useChatStore } from "../../stores/chat"; 
import { renderMarkdown } from '../../services/markdown';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useConfigStore } from '../../stores/config';
import { getProviderIcon } from '../../assets/icons';

// Import new components
import MessageHeader from './message-item/MessageHeader.vue';
import UserBubble from './message-item/UserBubble.vue';
import AssistantBubble from './message-item/AssistantBubble.vue';
import MessageActions from './message-item/MessageActions.vue';
import FileAttachment from './message-item/FileAttachment.vue';

const props = defineProps({
  m: Object,
  index: Number,
  sessionId: String,
  isEditing: Boolean
});

const emit = defineEmits(['start-edit', 'cancel-edit', 'save-edit', 'delete', 'regenerate', 'update-edit-content']);

const chatStore = useChatStore();
const configStore = useConfigStore();

// 💡 格式化时间
const formattedDate = computed(() => {
  if (!props.m.created_at) return '';
  const date = new Date(props.m.created_at + 'Z');
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const day = date.getDate().toString().padStart(2, '0');
  const hours = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');
  return `${month}/${day} ${hours}:${minutes}`;
});

// 💡 获取模型图标
const modelIcon = computed(() => {
    const modelId = props.m.model;
    if (modelId) {
        const provider = configStore.settings.providers.find(p => p.models.includes(modelId));
        if (provider) {
            return getProviderIcon(provider.icon);
        }
    }
    return getProviderIcon('gemini');
});

// 💡 获取显示的模型名称
const displayModelName = computed(() => {
    if (props.m.model) return props.m.model;
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
  const total = chatStore.currentMessages.length;
  return props.index < total - 2;
});

// 💡 统一复制函数
const doCopy = async (text, el) => {
  try {
    await navigator.clipboard.writeText(text);
    const original = el.innerHTML;
    el.innerHTML = `<span class="copy-text">已复制</span><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
    el.classList.add('copied');
    setTimeout(() => {
      el.innerHTML = original;
      el.classList.remove('copied');
    }, 2000);
  } catch (err) { console.error('复制失败', err); }
};

const hasVisibleContent = computed(() => {
  return props.m.content && props.m.content.trim().length > 0;
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

const messageRef = ref(null);
const injectCodeButtons = () => {
  nextTick(() => {
    if (!messageRef.value || !showActionButtons.value) return;
    const pres = messageRef.value.querySelectorAll('pre:not(.processed)');
    
    pres.forEach(pre => {
      pre.classList.add('processed');
      const codeEl = pre.querySelector('code');
      let lang = 'text';
      if (codeEl) {
        const classes = codeEl.className.split(' ');
        const langClass = classes.find(c => c.startsWith('language-'));
        if (langClass) {
          lang = langClass.replace('language-', '');
        }
      }

      const wrapper = document.createElement('div');
      wrapper.className = 'code-block-wrapper';
      const header = document.createElement('div');
      header.className = 'code-block-header';
      const langSpan = document.createElement('span');
      langSpan.className = 'code-lang';
      langSpan.textContent = lang;
      
      const btn = document.createElement('button');
      btn.className = 'code-copy-btn';
      btn.innerHTML = `<span class="copy-text">复制代码</span><svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>`;
      btn.onclick = (e) => { 
        e.stopPropagation(); 
        doCopy(pre.innerText.trim(), btn); 
      };

      header.appendChild(langSpan);
      header.appendChild(btn);
      
      if (pre.parentNode) {
        pre.parentNode.insertBefore(wrapper, pre);
        wrapper.appendChild(header);
        wrapper.appendChild(pre);
      }
    });
  });
};

onMounted(injectCodeButtons);
watch(showActionButtons, (val) => {
  if (val) injectCodeButtons();
});

// --- 🚅 Adaptive Performance-Based Throttling ---
const renderedContent = ref("");
let renderTimeout = null;
let lastRenderTime = 0;
let currentThrottleDelay = 16; 

const updateRenderedContent = (content) => {
  if (content === "__LOADING__") {
      renderedContent.value = "";
      return;
  }
  
  const start = performance.now();
  renderedContent.value = renderMarkdown(content);
  const duration = performance.now() - start;

  if (duration < 5) {
      currentThrottleDelay = 16;
  } else if (duration < 16) {
      currentThrottleDelay = 50;
  } else {
      currentThrottleDelay = 150;
  }
  
  injectCodeButtons(); 
};

watch(() => props.m.content, (newVal) => {
    const isActiveMessage = props.index === chatStore.currentMessages.length - 1;
    const shouldThrottle = chatStore.isGenerating && isActiveMessage;

    if (!shouldThrottle) {
        if (renderTimeout) clearTimeout(renderTimeout);
        updateRenderedContent(newVal);
    } else {
        const now = Date.now();
        const timeSinceLast = now - lastRenderTime;

        if (timeSinceLast >= currentThrottleDelay) {
            if (renderTimeout) clearTimeout(renderTimeout);
            updateRenderedContent(newVal);
            lastRenderTime = now;
        } else {
            if (renderTimeout) clearTimeout(renderTimeout);
            renderTimeout = setTimeout(() => {
                updateRenderedContent(newVal);
                lastRenderTime = Date.now();
            }, currentThrottleDelay - timeSinceLast);
        }
    }
}, { immediate: true });

watch(() => chatStore.isGenerating, (isGen) => {
    if (!isGen) {
        const isActiveMessage = props.index === chatStore.currentMessages.length - 1;
        if (isActiveMessage) {
            if (renderTimeout) clearTimeout(renderTimeout);
            updateRenderedContent(props.m.content);
        }
    }
});

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
        <UserBubble 
          v-if="hasVisibleContent || isEditing"
          :content="m.content"
          :is-editing="isEditing"
          @update-edit-content="$emit('update-edit-content', $event)"
          @cancel-edit="$emit('cancel-edit')"
          @save-edit="$emit('save-edit', $event)"
        />

        <FileAttachment 
          :files="parsedFiles"
          @open-file="handleOpenFile"
        />
      </div>
      
      <MessageActions 
        role="user"
        :show="!isEditing && showActionButtons"
        @edit="$emit('start-edit')"
        @delete="$emit('delete', $event)"
      />
    </div>

    <div v-else class="assistant-content-wrapper">
      <MessageHeader 
        v-if="m.content !== '__LOADING__'"
        :icon="modelIcon"
        :model-name="displayModelName"
        :date="formattedDate"
      />

      <AssistantBubble 
        :message="m"
        :rendered-content="renderedContent"
        :is-reasoning-expanded="isReasoningExpanded"
        @toggle-reasoning="toggleReasoning"
        @link-click="handleLinkClick"
      />
      
      <MessageActions 
        role="assistant"
        :show="m.content !== '__LOADING__' && showActionButtons"
        @regenerate="chatStore.regenerateAction(index)"
        @copy="e => doCopy(m.content, e.currentTarget)"
        @delete="$emit('delete', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.assistant-content-wrapper { position: relative; width: 100%; display: flex; flex-direction: column; gap: 8px; }

.message-bubble-wrapper { display: flex; flex-direction: column; align-items: flex-end; max-width: 85%; }
.message-bubble-wrapper:hover :deep(.msg-action-bar-user) {
  opacity: 1;
  visibility: visible;
}
.user-turn-content { display: flex; flex-direction: column; align-items: flex-end; gap: 8px; width: 100%; }

.message-row { display: flex; width: 100%; }
.message-row.user { justify-content: flex-end; }

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
  background-color: var(--bg-code-header) !important;
  border-bottom: none;
  font-family: 'Google Sans', 'Segoe UI', system-ui, sans-serif;
  font-size: 13px;
  color: var(--text-color);
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

:deep(.code-block-header)::after {
  content: '';
  position: absolute;
  left: 0;
  bottom: -4px;
  width: 100%;
  height: 4px;
  background-color: var(--bg-main);
  z-index: 10;
}

:deep(.code-lang) {
  font-weight: 500;
  opacity: 0.9;
}

:deep(.markdown-body pre) {
  margin: 0 !important;
  border: none !important;
  border-radius: 0 0 12px 12px !important;
  padding: 20px 20px 16px 20px !important; 
  background-color: var(--bg-code) !important;
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
  color: #c4c7c5;
  cursor: pointer;
  padding: 8px;
  border-radius: 50%;
  transition: all 0.2s ease;
  width: 32px;
  height: 32px;
}

:deep(.code-copy-btn:hover) {
  background-color: var(--bg-glass-hover);
  color: #e3e3e3;
}

:deep(.code-copy-btn svg) {
  width: 18px;
  height: 18px;
}

:deep(.copy-text) {
  display: none;
}

:deep(.copied) { 
  color: #4ade80 !important; 
}
</style>

