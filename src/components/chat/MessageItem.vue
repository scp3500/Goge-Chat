<script setup>
import { ref, nextTick, onMounted, onUnmounted, watch, computed } from 'vue';
import { useChatStore } from "../../stores/chat"; 
import { renderMarkdown } from '../../services/markdown';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useConfigStore } from '../../stores/config';
import { useSettingsStore } from '../../stores/settings';
import { getProviderIcon } from '../../assets/icons';

// Import new components
import MessageHeader from './message-item/MessageHeader.vue';
import UserBubble from './message-item/UserBubble.vue';
import AssistantBubble from './message-item/AssistantBubble.vue';
import MessageActions from './message-item/MessageActions.vue';
import FileAttachment from './message-item/FileAttachment.vue';
import MessageError from './message-item/MessageError.vue';

const props = defineProps({
  m: Object,
  index: Number,
  sessionId: String,
  isEditing: Boolean,
  themeOverride: { type: String, default: null },
  assistantAvatar: { type: String, default: null },
  assistantName: { type: String, default: null }
});

const emit = defineEmits(['start-edit', 'cancel-edit', 'save-edit', 'delete', 'regenerate', 'update-edit-content']);

const chatStore = useChatStore();
const configStore = useConfigStore();
const settingsStore = useSettingsStore();

// --- 🔵 Chat Mode Logic ---
const isChatMode = computed(() => !!props.themeOverride || configStore.settings.chatMode?.enabled);

const chatModeTheme = computed(() => {
    if (props.themeOverride) return props.themeOverride;
    if (!isChatMode.value) return null;
    return configStore.settings.theme === 'light' 
        ? configStore.settings.chatMode.dayThemeId 
        : configStore.settings.chatMode.nightThemeId;
});

const showLoadingBar = computed(() => {
    if (!isChatMode.value) return true; // Normal mode: always show (unless globally disabled?)
    return configStore.settings.chatMode.enableLoadingBar;
});

const chatModeClasses = computed(() => {
    if (!isChatMode.value) return {};
    return {
        'chat-mode-active': true,
        'theme-wechat': chatModeTheme.value === 'wechat',
        'theme-darkplus': chatModeTheme.value === 'dark_plus',
        'no-header': true // Optional: hide header in chat mode
    };
});

// 💡 格式化时间
const formattedDate = computed(() => {
  if (!props.m.created_at) return '';
  // 🛡️ Fix: Handle both SQLite "YYYY-MM-DD HH:MM:SS" (needs Z) and ISO "YYYY-MM-DDTHH:MM:SSZ"
  let timeStr = props.m.created_at;
  if (!timeStr.includes('Z') && !timeStr.includes('+')) {
      // If it looks like SQLite local time, append Z for UTC
      timeStr += 'Z';
  }
  
  const date = new Date(timeStr);
  if (isNaN(date.getTime())) return props.m.created_at; // Fallback to raw string if still invalid

  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const day = date.getDate().toString().padStart(2, '0');
  const hours = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');
  return `${month}/${day} ${hours}:${minutes}`;
});

// 💡 辅助函数：根据模型 ID 寻找提供商
const findProviderByModelId = (modelId) => {
    if (!modelId) return null;
    return configStore.settings.providers?.find(p => 
        p.models?.some(m => (typeof m === 'string' ? m : m.id) === modelId)
    );
};

// 💡 获取模型图标
const modelIcon = computed(() => {
    // 0. 优先使用传入的助理头像 (Social Mode)
    if (props.assistantAvatar) {
        // 如果是本地路径且不是 data:uri 或 http 链接，则转换
        const url = props.assistantAvatar;
        // ⚡️ Fix: Use shared resolver to handle all path types (including local assets)
        return resolveSocialAvatar(url);
    }

    // 1. Try explicit providerId stored in message
    if (props.m.providerId) {
        const provider = configStore.settings.providers.find(p => p.id === props.m.providerId);
        if (provider) {
            return getProviderIcon(provider.icon);
        }
    }

    // 2. Fallback to model ID lookup
    const modelId = props.m.model;
    const provider = findProviderByModelId(modelId);
    if (provider) {
        return getProviderIcon(provider.icon);
    }
    // 特殊处理：如果是 deepseek 模型但没找着（比如由于迁移没匹配上），也给个图标而非默认 Gemini
    if (modelId?.includes('deepseek')) return getProviderIcon('deepseek');
    return getProviderIcon('gemini');
});

// 💡 获取显示的模型名称
const displayModelName = computed(() => {
    if (props.assistantName) return props.assistantName;
    if (props.m.model) return props.m.model;
    return "Gemini";
});

const displayProviderName = computed(() => {
    if (props.assistantName) return ""; // Hide provider in social mode
    // 1. Prefer explicit providerId stored in message (for new messages)
    if (props.m.providerId) {
        const provider = configStore.settings.providers.find(p => p.id === props.m.providerId);
        return provider ? provider.name : props.m.providerId;
    }

    // 2. Fallback: Guess based on model ID (for history)
    const modelId = props.m.model;
    const provider = findProviderByModelId(modelId);
    if (provider) {
        return provider.name;
    }
    if (modelId?.includes('deepseek')) return "DeepSeek";
    return "Google";
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

const parsedError = (errorObj) => {
    if (typeof errorObj === 'string') {
        try {
            return JSON.parse(errorObj);
        } catch {
            return { message: errorObj };
        }
    }
    return errorObj;
};

const handleCloseError = async () => {
    // 只是简单的从UI移除，或者尝试重新加载（当前选简单的删除）
    // 但因为是 assistant 的最后一条消息，实际上可能需要清理 currentMessages
    if (props.m.role === 'assistant' && props.index === chatStore.currentMessages.length - 1) {
        // Option 1: Just hide local error (maybe not enough?)
        // Option 2: Remove the message
        chatStore.currentMessages.splice(props.index, 1);
    }
};

const handleRetryError = async () => {
    // 关闭错误并重试
    if (props.m.role === 'assistant' && props.index === chatStore.currentMessages.length - 1) {
        chatStore.currentMessages.splice(props.index, 1);
        await chatStore.sendMessage("");
    }
};
import { resolveSocialAvatar } from '../../utils/social';

const resolveAvatarSrc = (path) => {
    return resolveSocialAvatar(path);
};

// --- Context Menu Logic ---
const showContextMenu = ref(false);
const contextMenuPos = ref({ x: 0, y: 0 });

const handleContextMenu = (e) => {
    if (!isChatMode.value) return;
    contextMenuPos.value = { x: e.clientX, y: e.clientY };
    showContextMenu.value = true;
};

const closeContextMenu = () => {
    showContextMenu.value = false;
};

const handleMenuEdit = () => {
    emit('start-edit');
    closeContextMenu();
};

const handleMenuDelete = (event) => {
    emit('delete', props.m.id, event);
    closeContextMenu();
};

const handleMenuRegenerate = (event) => {
    emit('regenerate', props.m.id, event);
    closeContextMenu();
};

onMounted(() => {
    window.addEventListener('click', closeContextMenu);
    injectCodeButtons();
});

onUnmounted(() => { // Ensure onUnmounted is imported if not already, or just add listener/remover correctly
    window.removeEventListener('click', closeContextMenu);
});
</script>

<template>
  <div class="message-row" 
       :class="[
         String(m.role || 'user').toLowerCase(), 
         chatModeClasses
       ]" 
       ref="messageRef">
    
    <div v-if="m.role === 'user'" class="message-bubble-wrapper" 
         :class="{ 'bubble-mode': isChatMode }">
      
      <div class="user-layout-container">
        <div class="user-content-group">
          <div class="user-turn-content">
            <UserBubble 
              v-if="hasVisibleContent || isEditing"
              :content="m.content"
              :is-editing="isEditing"
              @contextmenu.prevent="handleContextMenu($event)"
              @update-edit-content="$emit('update-edit-content', $event)"
              @cancel-edit="$emit('cancel-edit')"
              @save-edit="$emit('save-edit', props.m.id, $event)"
            />

            <FileAttachment 
              :files="parsedFiles"
              @open-file="handleOpenFile"
            />
          </div>
          
          <MessageActions 
            role="user"
        :show="!isEditing && showActionButtons && !isChatMode"
            @edit="$emit('start-edit', props.m.id, $event)"
            @delete="$emit('delete', props.m.id, $event)"
          />
        </div>

        <!-- User Avatar (Strict Social/WeChat Mode Only) -->
        <div v-if="configStore.settings.showUserAvatar && props.themeOverride === 'wechat'" 
             class="user-avatar-container click-enabled"
             @click="settingsStore.openSettings('profile')"
             title="个人资料设置">
             <div class="avatar-img"
                  :style="{ backgroundImage: configStore.userAvatarUrl ? `url('${configStore.userAvatarUrl}')` : 'none' }">
                  <span v-if="!configStore.userAvatarUrl">👤</span>
             </div>
        </div>
      </div>
    </div>

    <div v-else class="assistant-content-wrapper" :class="{ 'bubble-mode': isChatMode }">
      <MessageHeader 
        v-if="m.content !== '__LOADING__'"
        :icon="modelIcon"
        :model-name="displayModelName"
        :provider-name="displayProviderName"
        :date="formattedDate"
      />

      <div class="assistant-bubble-container" :class="{ 'loading-hidden': m.content === '__LOADING__' && !showLoadingBar }">
        <!-- If loading bar disabled and content is loading, show nothing or minimal spacer -->
        <AssistantBubble 
          v-if="!(m.content === '__LOADING__' && !showLoadingBar)"
          :message="m"
          :rendered-content="renderedContent"
          :is-reasoning-expanded="isReasoningExpanded"
          :is-chat-mode="isChatMode"
          @contextmenu.prevent="handleContextMenu($event)"
          @toggle-reasoning="toggleReasoning"
          @link-click="handleLinkClick"
        />
      </div>
      
      <MessageActions 
        role="assistant"
        :show="m.content !== '__LOADING__' && !m.error && showActionButtons && !isChatMode"
        @regenerate="$emit('regenerate', props.m.id, $event)"
        @copy="e => doCopy(m.content, e.currentTarget)"
        @delete="$emit('delete', props.m.id, $event)"
      />

      <!-- 🔴 Error Rendering -->
      <MessageError 
        v-if="m.error" 
        :error="parsedError(m.error)"
        @close="handleCloseError"
        @retry="handleRetryError"
      />
    </div>
    <!-- Context Menu for Social Mode -->
    <Teleport to="body">
      <div v-if="showContextMenu" 
           class="msg-context-menu" 
           :style="{ top: contextMenuPos.y + 'px', left: contextMenuPos.x + 'px' }"
           @click.stop>
        <div class="menu-item" @click="handleMenuEdit">编辑</div>
        <div v-if="m.role === 'assistant'" class="menu-item" @click="handleMenuRegenerate($event)">重新生成</div>
        <div class="menu-divider"></div>
        <div class="menu-item delete" @click="handleMenuDelete($event)">删除</div>
      </div>
    </Teleport>
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

/* User Avatar & Layout */
.user-layout-container {
  display: flex;
  flex-direction: row;
  align-items: flex-start; /* Avatar at top */
  justify-content: flex-end;
  gap: 8px;
  width: 100%;
}

.user-content-group {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  max-width: 100%;
  flex: 1;
}

.user-avatar-container {
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  margin-top: 0px; /* Align with top of bubble */
}

.click-enabled {
    cursor: pointer;
    transition: transform 0.2s ease;
}

.click-enabled:hover {
    transform: scale(1.05);
}

.avatar-img {
  width: 100%;
  height: 100%;
  border-radius: 8px; /* Rounded square */
  background-color: var(--bg-input-dim);
  background-size: cover;
  background-position: center;
  border: 1px solid var(--border-glass);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
}

/* Bubble Mode Styles */
.bubble-mode :deep(.message-bubble) {
  background-color: var(--bg-user-bubble); 
  border: 1px solid var(--border-glass);
  border-radius: 18px 18px 4px 18px; /* Classic bubble shape */
  padding: 8px 14px; /* Slightly larger padding */
  width: fit-content;
  max-width: 90%;
  color: var(--color-user-bubble-text);
  line-height: 1.5;
  font-size: 15px;
}
/* Adjust layout for bubble mode */
.bubble-mode .user-turn-content {
  align-items: flex-end;
}

/* Assistant Bubble Mode */
.bubble-mode .assistant-bubble-container :deep(.assistant-bubble-content) {
  background-color: var(--bg-assistant-bubble);
  border: 1px solid var(--border-assistant-bubble);
  border-radius: 4px 18px 18px 18px; /* Original asymmetric shape */
  padding: 8px 14px; /* Larger padding */
  margin-top: 4px;
}

.bubble-mode .assistant-bubble-container :deep(.markdown-body) {
  background-color: transparent;
  border: none;
  padding: 0;
}

.assistant-bubble-container {
  width: fit-content;
  max-width: 90%;
}

.message-row.assistant:not(.chat-mode-active) .assistant-bubble-container {
  width: 100%;
  max-width: 100%;
}

/* Adjust header in bubble mode */
.bubble-mode .assistant-content-wrapper {
  gap: 6px;
}

/* =========================================
   🔵 Chat Mode Themes
   ========================================= */

/* WECHAT THEME (Day) */
.theme-wechat .message-bubble-wrapper :deep(.message-bubble) {
  background-color: var(--bg-user-bubble);
  color: var(--color-user-bubble-text);
  border: 1px solid var(--border-user-bubble);
}

.theme-wechat .assistant-bubble-container :deep(.assistant-bubble-content) {
  background-color: var(--bg-assistant-bubble);
  border: 1px solid var(--border-assistant-bubble);
  color: var(--color-assistant-bubble-text);
  box-shadow: none;
}

.theme-wechat .assistant-bubble-container :deep(.markdown-body) {
  background-color: transparent;
  border: none;
  color: inherit;
  box-shadow: none;
}

/* WECHAT THEME (Night) */
:global(.app-dark) .theme-wechat .message-bubble-wrapper :deep(.message-bubble) {
  background-color: var(--bg-user-bubble);
  color: var(--color-user-bubble-text);
  border-color: var(--border-user-bubble);
}

:global(.app-dark) .theme-wechat .assistant-bubble-container :deep(.assistant-bubble-content) {
  background-color: var(--bg-assistant-bubble);
  color: var(--color-assistant-bubble-text);
  border-color: var(--border-assistant-bubble);
}


/* DARK++ THEME (Night) */
.theme-darkplus .message-bubble-wrapper :deep(.message-bubble) {
  background-color: #2b2d31; 
  color: #e0e0e0;
  border: 1px solid #3f4148;
}

.theme-darkplus .assistant-bubble-container :deep(.assistant-bubble-content) {
  background-color: #1e1e1e;
  border: 1px solid #333333;
  color: #cccccc;
  padding: 10px 14px;
}

.theme-darkplus .assistant-bubble-container :deep(.markdown-body) {
  background-color: transparent;
  border: none;
  color: inherit;
  width: fit-content;
  max-width: 90%;
}

/* Hide Loading State */
.loading-hidden {
  display: none;
}

/* Context Menu Styles */
.msg-context-menu {
  position: fixed;
  z-index: 99999;
  background: var(--bg-menu);
  border: 1px solid var(--border-menu);
  border-radius: 8px;
  padding: 4px;
  min-width: 100px;
  box-shadow: var(--shadow-main);
  backdrop-filter: blur(12px);
  animation: menuFadeIn 0.1s ease-out;
}

@keyframes menuFadeIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.msg-context-menu .menu-item {
  padding: 8px 12px;
  font-size: 13px;
  color: var(--text-color);
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
}

.msg-context-menu .menu-item:hover {
  background: var(--bg-menu-hover);
  color: var(--text-color-white);
}

.msg-context-menu .menu-item.delete {
  color: #ff6b6b;
}

.msg-context-menu .menu-item.delete:hover {
  background: rgba(255, 107, 107, 0.1);
}

.msg-context-menu .menu-divider {
  height: 1px;
  background: var(--border-menu);
  margin: 4px 0;
}
</style>

