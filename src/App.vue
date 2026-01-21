<script setup>
import { ref, onMounted } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useConfigStore } from './stores/config';
import { useChatStore } from './stores/chat';

// 导入组件 - 请确保路径与你的目录结构一致
import SettingsModal from "./components/settings/SettingsModal.vue"; 
import TitleBar from "./components/TitleBar.vue";
import SideBar from "./components/SideBar.vue";
import ChatContainer from "./components/chat/ChatContainer.vue";

const appWindow = getCurrentWindow();
const configStore = useConfigStore();
const chatStore = useChatStore();

const isMaximized = ref(false); 
const showSettings = ref(false); 

onMounted(async () => {
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
</script>

<template>
  <div class="app-layout" :class="{ 'is-maximized': isMaximized }">
    <TitleBar 
      :is-settings="showSettings" 
      @open-settings="showSettings = true" 
      @back-home="showSettings = false" 
    />
    
    <div class="content-area">
      <transition name="view-fade" mode="out-in">
        <div v-if="!showSettings" class="main-view">
          <SideBar 
            :active="chatStore.activeId" 
            :list="chatStore.historyList" 
            @create="chatStore.createSession" 
            @select="id => chatStore.activeId = id" 
            @delete="id => chatStore.deleteSession(id)" 
            @rename="chatStore.renameSession"
          />
          
          <ChatContainer 
            v-if="chatStore.activeId !== null"
            :key="chatStore.activeId"
          />
          
          <div v-else class="empty-chat">
            <p>选择或创建一个对话开始</p>
          </div>
        </div>

        <SettingsModal v-else @close="showSettings = false" />
      </transition>
    </div>
  </div>
</template>

<style>
/* 全局基础重置 */
html, body, #app { 
  overflow: hidden !important; 
  height: 100%; 
  margin: 0; 
  background: transparent; 
}

/* 视图切换动画 */
.view-fade-enter-active, .view-fade-leave-active { transition: all 0.25s ease; }
.view-fade-enter-from { opacity: 0; transform: translateX(10px); }
.view-fade-leave-to { opacity: 0; transform: translateX(-10px); }
</style>

<style scoped>
.app-layout { 
  display: flex; 
  flex-direction: column; 
  height: 100vh; 
  background: var(--bg-main, #131314); 
  color: #e3e3e3; 
  
  /* ✨ 核心修复：非最大化时的圆角 */
  border-radius: 12px; 
  /* ✨ 核心修复：防止子元素溢出圆角边界 */
  overflow: hidden; 
  
  /* 增加一个极其微妙的边框，提升质感 */
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-sizing: border-box;
  transition: border-radius 0.2s ease;
}

/* 窗口最大化时，平滑切换回直角 */
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
}

.empty-chat { 
  flex: 1; 
  display: flex; 
  align-items: center; 
  justify-content: center; 
  color: #555; 
  font-size: 0.9rem;
}
</style>