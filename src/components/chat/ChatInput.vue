<script setup>
import { ref, nextTick, onMounted, watch, computed } from 'vue';
import { storeToRefs } from 'pinia';
import { useChatStore } from "../../stores/chat";
import { STOP_SVG, SEND_SVG, PAPERCLIP_SVG, BRAIN_SVG, GLOBE_SVG, CLOSE_SVG, ATTACHMENT_SVG } from '../../constants/icons';
import ModelSelector from './ModelSelector.vue';
import { useUIStore } from '../../stores/ui';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../../stores/settings';
import { useConfigStore } from '../../stores/config';
import SystemPromptWidget from './SystemPromptWidget.vue';



const chatStore = useChatStore();
const { isGenerating, useReasoning, useSearch, searchProvider } = storeToRefs(chatStore);

const searchProviders = [
  { id: 'all', name: '全网搜索', icon: GLOBE_SVG },
  { id: 'google', name: 'Google', icon: GLOBE_SVG },
  { id: 'bing', name: 'Bing', icon: GLOBE_SVG },
  { id: 'baidu', name: 'Baidu', icon: GLOBE_SVG },
];

const uiStore = useUIStore();
const settingsStore = useSettingsStore();
const configStore = useConfigStore();

const showSearchMenu = computed(() => uiStore.isMenuOpen('search-menu'));
const showPresetMenu = computed(() => uiStore.isMenuOpen('preset-menu'));
const activeSearchProvider = computed(() => searchProviders.find(p => p.id === searchProvider.value) || searchProviders[0]);

const showNameModal = ref(false);


const inputMsg = ref("");
const textareaRef = ref(null);
const selectedFiles = ref([]); // { name, path, icon }

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
    if (!inputMsg.value.trim() && selectedFiles.value.length === 0) return;
    
    let msgToProcess = inputMsg.value;
    
    // 如果有文件，读取内容并追加到 prompt (对于 DeepSeek-V3 这种不支持附件的情况)
    if (selectedFiles.value.length > 0) {
      let filesPrompt = "\n\n--- 附件内容 ---\n";
      for (const file of selectedFiles.value) {
        try {
          const content = await invoke('read_file_text_content', { path: file.path });
          filesPrompt += `\n文件名: ${file.name}\n内容:\n${content}\n`;
        } catch (e) {
          console.error("读取文件失败:", file.path, e);
          filesPrompt += `\n文件名: ${file.name}\n(读取失败: ${e})\n`;
        }
      }
      msgToProcess += filesPrompt;
    }

    inputMsg.value = "";
    
    // 清除已选文件
    const filesMetadata = selectedFiles.value.length > 0 ? JSON.stringify(selectedFiles.value) : null;
    selectedFiles.value = [];
    
    // 发送后重置高度
    nextTick(() => {
        if(textareaRef.value) {
            textareaRef.value.style.height = 'auto'; 
            textareaRef.value.style.height = '24px'; 
        }
    });
    
    // 这里 sendMessage 需要稍作调整以接受 metadata，或者通过 store 处理
    await chatStore.sendMessage(msgToProcess, filesMetadata, searchProvider.value);
  }
};

const onKeydown = (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    handleAction();
  }
};

const handleAttachClick = async () => {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Documents',
        extensions: ['txt', 'md', 'json', 'js', 'ts', 'py', 'rs', 'cpp', 'h', 'css', 'html']
      }]
    });
    if (selected && Array.isArray(selected)) {
      selected.forEach(path => {
        const name = path.split(/[\\/]/).pop();
        if (!selectedFiles.value.find(f => f.path === path)) {
          selectedFiles.value.push({
            name,
            path,
            icon: ATTACHMENT_SVG
          });
        }
      });
    } else if (selected) {
      const path = selected;
      const name = path.split(/[\\/]/).pop();
      if (!selectedFiles.value.find(f => f.path === path)) {
        selectedFiles.value.push({
          name,
          path,
          icon: ATTACHMENT_SVG
        });
      }
    }
  } catch (e) {
    console.error("选择文件失败:", e);
  }
};

const handleRemoveFile = (index) => {
  selectedFiles.value.splice(index, 1);
};

const handleSearchClick = (e) => {
    e.stopPropagation();
    if (!useSearch.value) {
        useSearch.value = true;
        uiStore.setActiveMenu('search-menu');
    } else {
        // 如果已经开启了搜索，再次点击图标则关闭搜索
        useSearch.value = false;
        uiStore.setActiveMenu(null);
    }
};

const selectSearchProvider = (id) => {
    searchProvider.value = id;
    uiStore.setActiveMenu(null);
};

// 点击外部关闭菜单
onMounted(() => {
    window.addEventListener('click', () => {
        uiStore.setActiveMenu(null);
    });
});

const handleThinkClick = () => {
  useReasoning.value = !useReasoning.value;
};

const selectPreset = (presetId) => {
  if (presetId === 'new') {
    showNameModal.value = true;
  } else {
    configStore.updateConfig({ defaultPresetId: presetId });
  }
  uiStore.setActiveMenu(null);
};

const handleCreatePreset = async (name) => {
  showNameModal.value = false;
  const newId = await configStore.addPreset(name);
  settingsStore.setActivePreset(newId);
  settingsStore.openSettings('presets');
};

onMounted(() => {
  autoResize();
});

</script>

<template>
  <div class="input-area">
    <div class="input-wrapper" @click="textareaRef?.focus()">
      
      <!-- 文件预览区 -->
      <div v-if="selectedFiles.length > 0" class="file-tray modern-scroll">
        <div v-for="(file, index) in selectedFiles" :key="file.path" class="file-card">
          <div class="file-icon" v-html="file.icon"></div>
          <div class="file-info">
            <span class="file-name">{{ file.name }}</span>
          </div>
          <button class="remove-file-btn" @click.stop="handleRemoveFile(index)">
            <span v-html="CLOSE_SVG"></span>
          </button>
        </div>
      </div>

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
            <span v-html="PAPERCLIP_SVG"></span>
          </button>
          
          <!-- 极简模型选择器 -->
          <ModelSelector minimal direction="up" fullWidth menuId="input-model" />

          <SystemPromptWidget />

          <button
            class="icon-btn attach-btn"
            @click="handleThinkClick"
            :title="useReasoning ? '关闭深度思考' : '开启深度思考'"
            :class="{ 'active-think': useReasoning }"
          >
            <span v-html="BRAIN_SVG"></span>
          </button>

          <div class="search-btn-wrapper" style="position: relative;">
            <button
              class="icon-btn attach-btn"
              @click="handleSearchClick"
              :title="useSearch ? '切换搜索源/关闭' : '开启网络搜索'"
              :class="{ 'active-search': useSearch }"
            >
              <span v-html="activeSearchProvider.icon"></span>
            </button>
          </div>
        </div>

        <div class="tools-right">
          <button
            class="icon-btn action-btn"
            @click="handleAction"
            :class="{ 'is-stop': isGenerating }"
            :disabled="!isGenerating && !inputMsg.trim() && selectedFiles.length === 0"
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

      <!-- 全局搜索源选择菜单 (移出到外层以支持全宽/居中) -->
      <Transition name="fade-slide">
        <div v-if="showSearchMenu" class="search-menu-popup modern-scroll" @click.stop>
          <div class="menu-list">
            <div 
              v-for="provider in searchProviders" 
              :key="provider.id"
              class="menu-item"
              :class="{ active: searchProvider === provider.id }"
              @click="selectSearchProvider(provider.id)"
            >
              <div class="menu-item-left">
                <span class="provider-icon" v-html="provider.icon"></span>
                <span class="provider-name">{{ provider.name }}</span>
              </div>
              <span class="free-badge">免费</span>
            </div>
          </div>
          
          <div class="menu-footer">
            <div class="footer-left">网络搜索</div>
            <div class="menu-shortcuts">
              <span>ESC 关闭</span>
              <span>▲▼ 选择</span>
              <span><span class="key">Ctrl</span> + ▲▼ 翻页</span>
              <span>↵ 确认</span>
            </div>
          </div>
        </div>
      </Transition>

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
  padding: 16px 20px 10px 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition: background-color 0.2s ease, box-shadow 0.2s ease;
  border: 1px solid var(--border-glass);
  box-shadow: var(--input-shadow);
  cursor: text;
  position: relative; /* 确保子绝父相 */
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
  font-size: 15px;
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
  padding: 0;           /* 强烈清除所有默认内边距 */
  margin: 0;
  line-height: 0;       /* 防止行感干扰 */
}

.icon-btn :deep(svg) {
  width: 18px;          /* 统一所有图标大小 */
  height: 18px;
}

.attach-btn {
  opacity: 0.6;
}
.attach-btn:hover {
  background-color: var(--bg-glass-hover);
  opacity: 1;
}

.attach-btn.active-think {
  color: var(--color-primary);
  opacity: 1;
}

.attach-btn.active-search {
  color: var(--color-primary);
  opacity: 1;
}

.attach-btn.active-config {
  color: var(--color-primary);
  opacity: 1;
}


/* 当处于蓝紫色(激活)状态时，悬停/点击反馈为白色 */
.attach-btn.active-think:hover,
.attach-btn.active-search:hover,
.attach-btn.active-think:active,
.attach-btn.active-search:active {
  color: #ffffff !important;
}

/* --- 文件托盘样式 --- */
.file-tray {
  display: flex;
  flex-wrap: nowrap;
  gap: 10px;
  padding: 2px 2px 8px 2px;
  overflow-x: auto;
  max-width: 100%;
}

.file-card {
  display: flex;
  align-items: center;
  background: var(--bg-glass);
  border-radius: 12px;
  padding: 6px 10px;
  gap: 8px;
  min-width: 120px;
  max-width: 200px;
  position: relative;
  border: 1px solid var(--border-glass);
  transition: all 0.2s ease;
}

.file-card:hover {
  background: var(--bg-glass-hover);
  border-color: var(--border-glass-bright);
}

.file-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary);
}

.file-icon :deep(svg) {
  width: 18px;
  height: 18px;
}

.file-info {
  flex: 1;
  overflow: hidden;
}

.file-name {
  font-size: 12px;
  color: var(--text-color-white);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}

.remove-file-btn {
  background: var(--bg-mask);
  border: none;
  border-radius: 50%;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: white;
  opacity: 0.6;
  transition: opacity 0.2s;
}

.remove-file-btn:hover {
  opacity: 1;
  opacity: 1;
  background: var(--color-danger);
}

.remove-file-btn :deep(svg) {
  width: 10px;
  height: 10px;
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
  background-color: var(--bg-glass-hover); /* 悬停显示白圆 */
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
  color: var(--color-primary); /* 薰衣草紫文字 */
  
  /* 关键修改：默认显示蓝紫色背景，而不是透明 */
  background-color: var(--bg-button-active); 
  opacity: 1; 
}

.action-btn.is-stop:hover {
  /* 悬停时加深背景 */
  background-color: var(--bg-glass-active); 
}

.modern-scroll::-webkit-scrollbar { width: 4px; }
.modern-scroll::-webkit-scrollbar-thumb { background: var(--bg-glass-active); border-radius: 10px; }

/* 搜索菜单样式 */
/* 搜索菜单样式 - 极致毛玻璃 */
.search-menu-popup {
  position: absolute;
  bottom: calc(100% - 1px);
  left: 0;      /* 设为 0 */
  right: 0;     /* 设为 0 */
  margin: 0 auto; /* 配合 width: 92% 实现完美居中 */
  width: 92%;
  width: 92%;
  background: var(--bg-menu);
  backdrop-filter: blur(40px) saturate(200%);
  -webkit-backdrop-filter: blur(40px) saturate(200%);
  border: 1px solid var(--border-menu);
  border-bottom: none;
  border-radius: 20px 20px 0 0;
  box-shadow: var(--shadow-main);
  z-index: 1000;
  overflow: hidden;
  padding: 6px;
}

.menu-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.menu-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  cursor: pointer;
  transition: all 0.2s;
  border-radius: 10px;
}

.menu-item:hover {
  background: var(--bg-glass-hover);
}

.menu-item.active {
  background: var(--color-success-bg);
  border: 1px solid var(--color-success-border);
}

.menu-item.active .provider-name {
  color: var(--color-success);
}

.menu-item.active .free-badge {
  color: var(--color-success);
  opacity: 0.5;
}

.menu-item-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.provider-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
}

.provider-icon :deep(svg) {
  width: 18px;
  height: 18px;
}

.provider-name {
  font-size: 14px;
  color: var(--text-color);
  font-weight: 500;
}

.free-badge {
  font-size: 12px;
  color: var(--text-dim);
}

.menu-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 14px 6px;
  border-top: 1px solid var(--border-glass);
  margin-top: 6px;
}

.footer-left {
  font-size: 12px;
  color: var(--text-dim);
  font-weight: 500;
}

.menu-shortcuts {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 11px;
  color: var(--text-dim);
}

.menu-shortcuts .key {
  color: var(--text-tertiary);
  background: var(--bg-glass-active);
  padding: 1px 4px;
  border-radius: 4px;
  margin-right: 2px;
}

/* 动画 */
.fade-slide-enter-active, .fade-slide-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}
.fade-slide-enter-from, .fade-slide-leave-to {
  opacity: 0;
  transform: translateY(15px);
}

/* 预置下拉菜单样式 */
.preset-dropdown {
  position: absolute;
  bottom: calc(100% + 10px);
  left: 0;
  width: 200px;
  background: var(--bg-menu);
  backdrop-filter: blur(40px) saturate(200%);
  border: 1px solid var(--border-menu);
  border-radius: 12px;
  box-shadow: var(--shadow-main);
  z-index: 1000;
  padding: 6px;
}

.preset-dropdown .menu-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  font-size: 13px;
  border-radius: 8px;
}

.preset-dropdown .new-preset {
  color: var(--color-success);
  font-weight: 500;
}

.preset-dropdown .menu-sep {
  height: 1px;
  background: var(--border-menu);
  margin: 4px 0;
}

.check-icon {
  color: var(--color-success);
  font-weight: bold;
}
</style>
