<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWindow, PhysicalSize, currentMonitor } from '@tauri-apps/api/window';
import { emit as tauriEmit } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
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
import SocialContactProfile from "./components/social/SocialContactProfile.vue";
import AppNavBar from "./components/layout/AppNavBar.vue";
import MinimalistOverlay from "./components/chat/MinimalistOverlay.vue";

const appWindow = getCurrentWindow();
const configStore = useConfigStore();
const uiStore = useUIStore();
const chatStore = useChatStore();

const isMaximized = ref(false); 
const settingsStore = useSettingsStore();

const activeModule = ref('chat');
const isMinimalistMode = ref(false);
const wasMaximizedBeforeMinimalist = ref(false); // 记住进入极简模式前的窗口状态

// 处理打开设置
const handleOpenSettings = (category) => {
    settingsStore.openSettings(category);
    uiStore.isHistoryOpen = false;   // 🚪 自动隐藏历史会话
    chatStore.setChatViewActive(false);  // 通知聊天 store 视图已切换
};

// 处理返回聊天
const handleBackToChat = () => {
    settingsStore.closeSettings();
    chatStore.setChatViewActive(true);  // 通知聊天 store 视图已激活
}; 

// 处理切换最小化模式
const handleToggleMinimalist = async () => {
    isMinimalistMode.value = !isMinimalistMode.value;
    
    try {
        if (isMinimalistMode.value) {
            // 进入极简模式：记住当前状态，然后最大化
            document.documentElement.classList.add('minimalist-root-active');
            wasMaximizedBeforeMinimalist.value = await appWindow.isMaximized();
            if (!wasMaximizedBeforeMinimalist.value) {
                await appWindow.maximize();
            }
            
            // 🎯 关键修复：启用窗口点击穿透，让桌面和其他应用可以点击
            await invoke('set_window_ignore_cursor_events', { ignore: true });
            
            // 📌 关键补丁：进入极简模式立即置顶
            await appWindow.setAlwaysOnTop(true);
        } else {
            // 退出极简模式：恢复之前的窗口状态
            document.documentElement.classList.remove('minimalist-root-active');
            
            // 🎯 关键修复：禁用窗口点击穿透，恢复正常点击
            await invoke('set_window_ignore_cursor_events', { ignore: false });
            
            // 🔓 关键补丁：退出极简模式取消置顶
            await appWindow.setAlwaysOnTop(false);
            
            // 如果之前不是最大化的，恢复回去
            if (!wasMaximizedBeforeMinimalist.value) {
                await appWindow.unmaximize();
            }
            
            // 延迟发射滚动指令，等待主界面渲染完成
            setTimeout(async () => {
                await tauriEmit('request-social-chat-scroll', { behavior: 'smooth' });
            }, 150);
        }
    } catch (e) {
        console.warn("Failed to toggle window state for minimalist mode:", e);
    }
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
      'is-chat-mode': configStore.settings.chatMode.enabled,
      'is-minimalist': isMinimalistMode
    }"
    @dragover.prevent
    @drop.prevent
  >
    <!-- Social Mode: Sidebar-first layout -->
    <template v-if="configStore.settings.chatMode.enabled">
      <div v-show="!isMinimalistMode" class="full-layout-wrapper">
        <AppNavBar 
          v-model:activeModule="activeModule"
          :is-collapsed="!uiStore.isLeftSidebarOpen"
          :is-in-settings="settingsStore.isModalOpen"
          @toggleCollapse="uiStore.isLeftSidebarOpen = !uiStore.isLeftSidebarOpen"
          @openSettings="handleOpenSettings"
          @openProfile="handleOpenSettings('profile')"
          @backHome="handleBackToChat" 
          @toggleMinimalist="handleToggleMinimalist"
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
              @update:activeModule="(val) => { 
                activeModule = val; 
                handleBackToChat(); 
              }"
              v-slot="{ activeContact, activeModule: slotActiveModule }"
            >
              <SocialContactProfile 
                v-if="slotActiveModule === 'address_book' && activeContact"
                :active-contact="activeContact"
                @startChat="activeModule = 'chat'"
              />
              <SocialChatContainer 
                v-else-if="activeContact"
                :active-contact="activeContact"
                @show-profile="activeModule = 'address_book'"
              />
            </MainLayout>
          </div>
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

  <!-- 最小化聊天模式覆盖层 - 提升到顶层，不受 app-layout 约束 -->
  <MinimalistOverlay 
    :visible="isMinimalistMode && configStore.settings.chatMode?.enabled"
    @close="handleToggleMinimalist"
    @send="() => {}"
  />
</template>

<style>
/* ... 全局基础重置保持不变 ... */
html, body, #app { 
  overflow: hidden !important; 
  height: 100%; 
  margin: 0; 
  background: transparent !important; 
  background-color: transparent !important;
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
  
  /* 🛡️ 终极修剪补丁：强制使用 clip-path 进行物理裁剪，解决 backdrop-filter 边缘溢出问题 */
  clip-path: inset(0 round 12px);
}

.app-layout.is-chat-mode {
  flex-direction: row;
}

/* 全局极简模式补丁：解除 HTM/App 的所有裁剪限制 */
:global(html.minimalist-root-active),
:global(html.minimalist-root-active body),
:global(html.minimalist-root-active #app) {
  overflow: visible !important;
  background: transparent !important;
  width: 100vw !important;
  height: 100vh !important;
  display: block !important; /* 禁掉 flex 带来的布局限制 */
}

.app-layout.is-minimalist {
  position: fixed !important;
  inset: 0 !important;
  width: 100vw !important;
  height: 100vh !important;
  background: transparent !important;
  background-color: transparent !important;
  border: none !important;
  box-shadow: none !important;
  backdrop-filter: none !important;
  -webkit-backdrop-filter: none !important;
  clip-path: none !important;
  border-radius: 0 !important;
  overflow: visible !important;
  z-index: 9998;
  /* 关键：防止透明的主布局容器拦截点击 */
  pointer-events: none !important; 
  transform: none !important;
}

.full-layout-wrapper {
  display: flex;
  flex-direction: inherit;
  width: 100%;
  height: 100%;
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
  clip-path: none;
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