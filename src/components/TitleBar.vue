<script setup>
import { ref, onMounted } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();
const isMaximized = ref(false);

// 1. 接收来自 App.vue 的状态，判断当前是否在设置页
defineProps({
  isSettings: {
    type: Boolean,
    default: false
  }
});

// 2. 增加 back-home 信号
const emit = defineEmits(['open-settings', 'back-home']);

const updateMaximizedState = async () => {
  isMaximized.value = await appWindow.isMaximized();
};

onMounted(async () => {
  await updateMaximizedState();
  await appWindow.onResized(updateMaximizedState);
});

const handleGlobalDrag = async (event) => {
  if (event.target.closest('.window-controls') || event.target.closest('.back-btn')) return;
  await appWindow.startDragging();
};

const minimizeWin = async () => await appWindow.minimize();
const toggleMaximizeWin = async () => {
  await appWindow.toggleMaximize();
  setTimeout(updateMaximizedState, 150); 
};
const closeWin = async () => await appWindow.destroy();

const openSettings = () => {
  emit('open-settings');
};
</script>

<template>
  <header class="titlebar" :class="{ 'maximized': isMaximized }" @mousedown="handleGlobalDrag">
    <div class="window-title">
      <span v-if="!isSettings" class="app-name">Goge Chat</span>
      
      <button 
        v-else 
        class="back-btn" 
        @click="$emit('back-home')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="19" y1="12" x2="5" y2="12"></line>
          <polyline points="12 19 5 12 12 5"></polyline>
        </svg>
        <span>返回首页</span>
      </button>
    </div>
    
    <div class="window-controls">
      <button 
        v-if="!isSettings" 
        @click.stop="openSettings" 
        class="control-btn settings-btn" 
        title="设置"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
      </button>

      <button @click.stop="minimizeWin" class="control-btn">
        <svg width="12" height="1"><rect width="12" height="1" fill="currentColor"/></svg>
      </button>

      <button @click.stop="toggleMaximizeWin" class="control-btn">
        <svg width="10" height="10" viewBox="0 0 10 10"><path d="M0 0h10v10H0z" fill="none" stroke="currentColor" stroke-width="1"/></svg>
      </button>

      <button @click.stop="closeWin" class="control-btn close-btn">
        <svg width="10" height="10"><path d="M0 0l10 10M10 0L0 10" fill="none" stroke="currentColor" stroke-width="1.2"/></svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.titlebar { 
  height: 35px; 
  /* --- 🩺 手术：同步侧边栏背景色 --- */
  background: #1e1e1f; 
  /* ------------------------------ */
  display: flex; 
  justify-content: space-between; 
  align-items: center; 
  padding: 0 10px; 
  -webkit-app-region: drag; 
}

.window-title { font-size: 12px; color: #888; flex: 1; display: flex; align-items: center; }

/* 返回按钮样式 */
.back-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: none;
  color: #5865f2;
  font-size: 12px;
  font-weight: bold;
  cursor: pointer;
  -webkit-app-region: no-drag;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.2s;
}
.back-btn:hover {
  background: rgba(88, 101, 242, 0.15);
  color: #7289da;
}

.window-controls { display: flex; height: 100%; align-items: center; -webkit-app-region: no-drag; }
.control-btn { 
  background: transparent; border: none; color: #888; width: 42px; height: 28px; 
  display: flex; align-items: center; justify-content: center; cursor: pointer;
  -webkit-app-region: no-drag;
}
.control-btn:hover { background: #333; color: white; }
.settings-btn:hover { color: white; }
.close-btn:hover { background: #c42b1c !important; }
</style>