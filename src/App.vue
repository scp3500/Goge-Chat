<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useConfigStore } from './stores/config';
import { useChatStore } from './stores/chat';
import { useSettingsStore } from './stores/settings';

// 导入组件
import SettingsModal from "./components/settings/SettingsModal.vue"; 
import TitleBar from "./components/TitleBar.vue";
import ChatContainer from "./components/chat/ChatContainer.vue";
import StandardLayout from "./layouts/StandardLayout.vue";
import MainLayout from "./layouts/MainLayout.vue";
import SocialChatContainer from "./components/chat/SocialChatContainer.vue";

const appWindow = getCurrentWindow();
const configStore = useConfigStore();
const chatStore = useChatStore();

const isMaximized = ref(false); 
const settingsStore = useSettingsStore();

// 处理打开设置
const handleOpenSettings = () => {
    settingsStore.openSettings();
    chatStore.setChatViewActive(false);  // 通知聊天 store 视图已切换
};

// 处理返回聊天
const handleBackToChat = () => {
    settingsStore.closeSettings();
    chatStore.setChatViewActive(true);  // 通知聊天 store 视图已激活
}; 


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
    
    // 初始化窗口状态并监听变化
    isMaximized.value = await appWindow.isMaximized();
    await appWindow.onResized(async () => {
        isMaximized.value = await appWindow.isMaximized();
    });
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
    :class="{ 'is-maximized': isMaximized }"
    @dragover.prevent
    @drop.prevent
  >
    <TitleBar 
      :is-settings="settingsStore.isModalOpen" 
      @open-settings="handleOpenSettings" 
      @back-home="handleBackToChat" 
    />
    
    <div class="content-area">
      <transition name="view-fade" mode="out-in">
        <!-- Standard Layout (Normal Mode) -->
        <StandardLayout v-if="!configStore.settings.chatMode.enabled && !settingsStore.isModalOpen" />
        
        <!-- Main Layout (Immersive Mode) - Always stays mounted even if settings are open -->
        <MainLayout 
          v-else-if="configStore.settings.chatMode.enabled"
          v-slot="{ activeContact }"
        >
          <SocialChatContainer 
            v-if="activeContact"
            :active-contact="activeContact"
          />
        </MainLayout>
      </transition>

      <!-- Settings Modal: Only show at root level if NOT in chatMode (for Standard Layout compatibility) -->
      <transition name="view-fade">
        <SettingsModal 
          v-show="settingsStore.isModalOpen && !configStore.settings.chatMode.enabled" 
          @close="handleBackToChat" 
        />
      </transition>
    </div>
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
  border: 1px solid var(--border-glass);
  box-sizing: border-box;
  transition: border-radius 0.2s ease;
  
  /* 🩺 增加视觉稳定性补丁 */
  user-select: none;
  
  /* 🧊 全局毛玻璃层：让窗口背后带一点透感 */
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
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