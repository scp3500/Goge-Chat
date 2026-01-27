<script setup>
import { ref, nextTick, onMounted, watch, computed } from 'vue';
import { storeToRefs } from 'pinia';
import { useChatStore } from "../../stores/chat";
import { STOP_SVG, SEND_SVG, PLUS_SVG, BRAIN_SVG, GLOBE_SVG, CLOSE_SVG, ATTACHMENT_SVG } from '../../constants/icons';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

const chatStore = useChatStore();
const { isGenerating, useReasoning, useSearch, searchProvider } = storeToRefs(chatStore);

const searchProviders = [
  { id: 'all', name: '全网搜索', icon: GLOBE_SVG },
  { id: 'google', name: 'Google', icon: '<svg viewBox="0 0 24 24" width="16" height="16"><path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/><path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/><path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l3.66-2.84z"/><path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/></svg>' },
  { id: 'bing', name: 'Bing', icon: '<svg viewBox="0 0 24 24" width="16" height="16"><path fill="#008373" d="M10.5 4L4 6.5v11.5l6.5 2.5l7.5-4.5V9.5z"/><path fill="#00A294" d="M20 9l-9.5-5V20l9.5-5z"/></svg>' },
  { id: 'baidu', name: 'Baidu', icon: '<svg viewBox="0 0 24 24" width="16" height="16"><path fill="#2932e1" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.5 13.5c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5zm-9 0c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5zm4.5-5c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5z"/></svg>' },
];

const showSearchMenu = ref(false);
const activeSearchProvider = computed(() => searchProviders.find(p => p.id === searchProvider.value) || searchProviders[0]);

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
        showSearchMenu.value = true;
    } else {
        showSearchMenu.value = !showSearchMenu.value;
    }
};

const selectSearchProvider = (id) => {
    searchProvider.value = id;
    showSearchMenu.value = false;
};

// 点击外部关闭菜单
onMounted(() => {
    window.addEventListener('click', () => {
        showSearchMenu.value = false;
    });
});

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
          <div class="search-btn-wrapper" style="position: relative;">
            <button
              class="icon-btn attach-btn"
              @click="handleSearchClick"
              :title="useSearch ? '切换搜索源/关闭' : '开启网络搜索'"
              :class="{ 'active-search': useSearch }"
            >
              <span v-html="activeSearchProvider.icon"></span>
            </button>
            
            <!-- 搜索源选择菜单 -->
            <Transition name="fade-slide">
              <div v-if="showSearchMenu" class="search-menu-popup modern-scroll" @click.stop>
                <div class="menu-header">网络搜索</div>
                <div 
                  v-for="provider in searchProviders" 
                  :key="provider.id"
                  class="menu-item"
                  :class="{ active: searchProvider === provider.id }"
                  @click="selectSearchProvider(provider.id)"
                >
                  <span class="provider-icon" v-html="provider.icon"></span>
                  <span class="provider-name">{{ provider.name }}</span>
                  <span class="free-badge">免费</span>
                </div>
                <div class="menu-footer" @click="useSearch = false; showSearchMenu = false">
                   关闭搜索
                </div>
              </div>
            </Transition>
          </div>
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

.attach-btn.active-search {
  color: #818cf8;
  opacity: 1;
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
  background: rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 6px 10px;
  gap: 8px;
  min-width: 120px;
  max-width: 200px;
  position: relative;
  border: 1px solid rgba(255, 255, 255, 0.1);
  transition: all 0.2s ease;
}

.file-card:hover {
  background: rgba(255, 255, 255, 0.12);
  border-color: rgba(255, 255, 255, 0.2);
}

.file-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #818cf8;
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
  background: rgba(0, 0, 0, 0.3);
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
  background: rgba(239, 68, 68, 0.8);
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

/* 搜索菜单样式 */
.search-menu-popup {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 12px;
  background: rgba(30, 31, 32, 0.95);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  width: 180px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  z-index: 100;
  overflow: hidden;
}

.menu-header {
  padding: 10px 14px;
  font-size: 11px;
  color: #80868b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.menu-item {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  gap: 10px;
  cursor: pointer;
  transition: background 0.2s;
}

.menu-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.menu-item.active {
  background: rgba(129, 140, 248, 0.15);
}

.provider-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.provider-name {
  flex: 1;
  font-size: 13px;
  color: #e3e3e3;
}

.free-badge {
  font-size: 10px;
  color: #818cf8;
  background: rgba(129, 140, 248, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

.menu-footer {
  padding: 10px 14px;
  font-size: 12px;
  color: #ef4444;
  text-align: center;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  cursor: pointer;
  background: rgba(239, 68, 68, 0.05);
}

.menu-footer:hover {
  background: rgba(239, 68, 68, 0.1);
}

/* 动画 */
.fade-slide-enter-active, .fade-slide-leave-active {
  transition: all 0.2s ease;
}
.fade-slide-enter-from, .fade-slide-leave-to {
  opacity: 0;
  transform: translateY(10px);
}
</style>