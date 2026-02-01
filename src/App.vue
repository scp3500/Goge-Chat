<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWindow, PhysicalSize, currentMonitor } from '@tauri-apps/api/window';
import { useConfigStore } from './stores/config';
import { useChatStore } from './stores/chat';
import { useSettingsStore } from './stores/settings';
import { useUIStore } from './stores/ui';

// 导入组件
import SettingsModal from "./components/settings/SettingsModal.vue"; 
import TitleBar from "./components/TitleBar.vue";
import ChatContainer from "./components/chat/ChatContainer.vue";
import StandardLayout from "./layouts/StandardLayout.vue";
import MainLayout from "./layouts/MainLayout.vue";
import SocialChatContainer from "./components/chat/SocialChatContainer.vue";
import AppNavBar from "./components/layout/AppNavBar.vue";

const appWindow = getCurrentWindow();
const configStore = useConfigStore();
const uiStore = useUIStore();
const chatStore = useChatStore();

const isMaximized = ref(false); 
const settingsStore = useSettingsStore();

const activeModule = ref('chat');

// 处理打开设置
const handleOpenSettings = (category) => {
    settingsStore.openSettings(category);
    chatStore.setChatViewActive(false);  // 通知聊天 store 视图已切换
};

// 处理返回聊天
const handleBackToChat = () => {
    settingsStore.closeSettings();
    chatStore.setChatViewActive(true);  // 通知聊天 store 视图已激活
}; 


import { useFontLoader } from "./composables/useFontLoader";

const { loadFont } = useFontLoader();

// 🅰️ Global Font Injection
const updateGlobalFonts = () => {
    const enFont = loadFont(configStore.settings.fontFamilyEnglish, 'english');
    const zhFont = loadFont(configStore.settings.fontFamilyChinese, 'chinese');
    
    // Apply to :root via document.documentElement.style
    if (enFont) {
        document.documentElement.style.setProperty('--font-family-en', enFont);
    } else {
        document.documentElement.style.removeProperty('--font-family-en');
    }

    if (zhFont) {
        document.documentElement.style.setProperty('--font-family-zh', zhFont);
    } else {
        document.documentElement.style.removeProperty('--font-family-zh');
    }
};

// Watch for font changes
import { watch } from 'vue';
watch(() => [configStore.settings.fontFamilyEnglish, configStore.settings.fontFamilyChinese], () => {
    updateGlobalFonts();
}, { immediate: true }); 


/**
 * 🩺 核心修复：全局拦截函数
 */
const handleGlobalDragOver = (e) => {
  e.preventDefault(); 
};

onMounted(async () => {
    // 🩺 注入全局监听
    window.addEventListener('dragover', handleGlobalDragOver, false);
    window.addEventListener('drop', handleGlobalDragOver, false);

    // 并行初始化配置和聊天数据
    await Promise.all([
        configStore.init(),
        chatStore.loadData()
    ]);
    
    // After init, ensure fonts are applied (in case watch triggered before init content ready)
    updateGlobalFonts(); 
    
    // 初始化窗口状态并监听变化
    isMaximized.value = await appWindow.isMaximized();
    await appWindow.onResized(async () => {
        isMaximized.value = await appWindow.isMaximized();
    });

    // 🖥️ 智能分辨率自适应 (Smart Resolution Adaptation)
    try {
        const monitor = await currentMonitor();
        if (monitor) {
            const screenWidth = monitor.size.width;
            const screenHeight = monitor.size.height;
            const scaleFactor = monitor.scaleFactor;
            
            console.log(`[Resolution Debug] Physical: ${screenWidth}x${screenHeight}, Scale: ${scaleFactor}`);
            console.log(`[Resolution Debug] Logical (CSS Max): ${Math.floor(screenWidth / scaleFactor)}x${Math.floor(screenHeight / scaleFactor)}`);

            // 如果是高分屏 (例如 2K/4K，宽度大于 1920 物理像素)
            if (screenWidth > 1920) {
                 const currentSize = await appWindow.innerSize();
                 // 如果当前窗口还很小 (默认 1000px 宽)，则自动放大
                 if (currentSize.width <= 1200) {
                     // 目标：即宽占屏幕 60%~70%，高占 70%~80%
                     const targetWidth = Math.floor(screenWidth * 0.65);
                     const targetHeight = Math.floor(screenHeight * 0.75);
                     
                     // 使用 LogicalSize 或者 PhysicalSize (Tauri 2 推荐 PhysicalSize)
                     // 但在 JS API 中通常直接传对象或者特定类
                     // 这里简单的做法是设为 PhysicalSize
                     await appWindow.setSize(new PhysicalSize(targetWidth, targetHeight));
                     await appWindow.center();
                 }
            }
        }
    } catch (e) {
        console.warn('Failed to adapt window size:', e);
    }
});

// 🩺 卸载时移除监听
onUnmounted(() => {
    window.removeEventListener('dragover', handleGlobalDragOver);
    window.removeEventListener('drop', handleGlobalDragOver);
});
</script>

<template>
  <div 
    class="app-layout" 
    :class="{ 
      'is-maximized': isMaximized,
      'is-chat-mode': configStore.settings.chatMode.enabled 
    }"
    @dragover.prevent
    @drop.prevent
  >
    <!-- Social Mode: Sidebar-first layout -->
    <template v-if="configStore.settings.chatMode.enabled">
      <AppNavBar 
        v-model:activeModule="activeModule"
        :is-collapsed="!uiStore.isLeftSidebarOpen"
        :is-in-settings="settingsStore.isModalOpen"
        @toggleCollapse="uiStore.isLeftSidebarOpen = !uiStore.isLeftSidebarOpen"
        @openSettings="handleOpenSettings"
        @openProfile="handleOpenSettings('profile')"
        @backHome="handleBackToChat" 
      />
      <div class="main-container">
        <TitleBar 
          :is-settings="settingsStore.isModalOpen" 
          @open-settings="handleOpenSettings" 
          @back-home="handleBackToChat" 
          @toggle-sidebar="uiStore.isLeftSidebarOpen = !uiStore.isLeftSidebarOpen"
          @toggle-history="uiStore.isHistoryOpen = !uiStore.isHistoryOpen"
        />
        <div class="content-area">
          <MainLayout 
            :is-left-sidebar-open="uiStore.isLeftSidebarOpen"
            :is-history-open="uiStore.isHistoryOpen"
            :active-module="activeModule"
            v-slot="{ activeContact }"
          >
            <SocialChatContainer 
              v-if="activeContact"
              :active-contact="activeContact"
            />
          </MainLayout>
        </div>
      </div>
    </template>

    <!-- Normal Mode: Original Header-first layout -->
    <template v-else>
      <TitleBar 
        :is-settings="settingsStore.isModalOpen" 
        @open-settings="handleOpenSettings" 
        @back-home="handleBackToChat" 
      />
      
      <div class="content-area">
        <div v-show="!settingsStore.isModalOpen" class="layout-wrapper">
          <StandardLayout />
        </div>
        <SettingsModal 
          v-if="settingsStore.isModalOpen"
          class="settings-overlay"
          @close="handleBackToChat" 
        />
      </div>
    </template>
  </div>
</template>

<style>
/* ... 全局基础重置保持不变 ... */
html, body, #app { 
  overflow: hidden !important; 
  height: 100%; 
  margin: 0; 
  background: transparent; 
}

/* ... 视图切换动画保持不变 ... */
.view-fade-enter-active, .view-fade-leave-active { transition: all 0.25s ease; }
.view-fade-enter-from { opacity: 0; transform: translateX(10px); }
.view-fade-leave-to { opacity: 0; transform: translateX(-10px); }
</style>

<style scoped>
/* ... 你的样式 100% 保留 ... */
.app-layout { 
  display: flex; 
  flex-direction: column; 
  height: 100vh; 
  background: var(--bg-main); 
  color: var(--text-color); 
  border-radius: 12px; 
  overflow: hidden; 
  border: 1px solid var(--border-app);
  box-sizing: border-box;
  transition: border-radius 0.2s ease;
  
  /* 🩺 增加视觉稳定性补丁 */
  user-select: none;
  
  /* 🧊 全局毛玻璃层：让窗口背后带一点透感 */
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

.app-layout.is-chat-mode {
  flex-direction: row;
}

.main-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-layout.is-maximized { 
  border-radius: 0; 
  border: none;
}

.content-area { 
  flex: 1; 
  position: relative; 
  overflow: hidden; 
}

.layout-wrapper {
  width: 100%;
  height: 100%;
}

.settings-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 100;
  animation: slide-up 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slide-up {
  from { transform: translateY(10px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}


.main-view { 
  display: flex; 
  width: 100%; 
  height: 100%; 
  position: absolute;  /* 关键：绝对定位，防止动画期间挤占空间 */
  top: 0;
  left: 0;
}

.empty-chat { 
  flex: 1; 
  display: flex; 
  align-items: center; 
  justify-content: center; 
  color: var(--text-color); 
  opacity: 0.4;
  font-size: 0.9rem;
}
</style>