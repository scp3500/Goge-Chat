<script setup>
import { ref, onMounted } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { HOME_SVG, SETTINGS_SVG, MINIMIZE_SVG, MAXIMIZE_SVG, CLOSE_SVG } from '../constants/icons.ts';

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
        title="返回首页"
        v-html="HOME_SVG"
      ></button>
    </div>
    
    <div class="window-controls">
      <button
        v-if="!isSettings"
        @click.stop="openSettings"
        class="control-btn settings-btn"
        title="设置"
        v-html="SETTINGS_SVG"
      ></button>

      <button @click.stop="minimizeWin" class="control-btn" v-html="MINIMIZE_SVG"></button>

      <button @click.stop="toggleMaximizeWin" class="control-btn" v-html="MAXIMIZE_SVG"></button>

      <button @click.stop="closeWin" class="control-btn close-btn" v-html="CLOSE_SVG"></button>
    </div>
  </header>
</template>

<style scoped>
.titlebar { 
  height: 35px; 
  /* --- 🩺 手术：同步侧边栏背景色 --- */
  background: var(--bg-main); 
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
  color: var(--text-color-white);
  font-size: 12px;
  font-weight: bold;
  cursor: pointer;
  -webkit-app-region: no-drag;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.2s;
}
.back-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: var(--text-color-white);
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