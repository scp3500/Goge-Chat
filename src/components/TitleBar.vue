<script setup>
import { ref, onMounted } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { HOME_SVG, SETTINGS_SVG, MINIMIZE_SVG, MAXIMIZE_SVG, CLOSE_SVG } from '../constants/icons.ts';
import ModelSelector from './chat/ModelSelector.vue';

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
  if (
    event.target.closest('.window-controls') || 
    event.target.closest('.back-btn') || 
    event.target.closest('.model-selector')
  ) return;
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
    <!-- Left Segment: Logo/Brand -->
    <div class="titlebar-left">
      <template v-if="!isSettings">
        <span class="app-name">Goge Chat</span>
      </template>
      <button
        v-else
        class="back-btn"
        @click="$emit('back-home')"
        title="返回首页"
        v-html="HOME_SVG"
      ></button>
    </div>
    
    <!-- Center Segment: Model Selector -->
    <div class="titlebar-center">
      <ModelSelector v-if="!isSettings" class="titlebar-model-selector" menuId="titlebar-model" />
    </div>
    
    <!-- Right Segment: Window Controls -->
    <div class="titlebar-right">
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
    </div>
  </header>
</template>

<style scoped>
.titlebar { 
  height: 35px; 
  background: var(--bg-main); 
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center; 
  padding: 0 10px; 
  -webkit-app-region: drag; 
  position: relative;
  z-index: 100;
}

.titlebar-left { display: flex; align-items: center; justify-self: start; }
.titlebar-center { display: flex; justify-content: center; align-items: center; }
.titlebar-right { display: flex; justify-content: flex-end; align-items: center; justify-self: end; }

.app-name { font-weight: 600; color: #aaa; font-size: 11px; margin-left: 2px; letter-spacing: 0.5px; opacity: 0.7; }
.titlebar-model-selector { z-index: 10; }

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