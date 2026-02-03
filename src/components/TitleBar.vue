<script setup>
import { ref, onMounted, computed } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { convertFileSrc } from '@tauri-apps/api/core';
import { HOME_SVG, SETTINGS_SVG, MINIMIZE_SVG, MAXIMIZE_SVG, CLOSE_SVG, SUN_SVG, MOON_SVG, HISTORY_SVG } from '../constants/icons.ts';
import ModelSelector from './chat/ModelSelector.vue';
import PresetSelector from './chat/PresetSelector.vue';
import { useConfigStore } from '../stores/config';

const props = defineProps({
  isSettings: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['open-settings', 'back-home', 'toggle-sidebar', 'toggle-history']);

const appWindow = getCurrentWindow();
const isMaximized = ref(false);
const configStore = useConfigStore();

const isLight = computed(() => configStore.settings.theme === 'light');

const toggleTheme = () => {
  const nextTheme = isLight.value ? 'dark' : 'light';
  configStore.updateConfig({ theme: nextTheme });
};

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
    event.target.closest('.selectors-group') ||
    event.target.closest('.history-btn')
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

const toggleSidebar = () => {
  emit('toggle-sidebar');
};

const toggleHistory = () => {
  emit('toggle-history');
};

const resolveAvatarSrc = (path) => {
  if (!path) return '';
  if (path.startsWith('data:') || path.startsWith('http')) return path;
  return convertFileSrc(path);
};
</script>

<template>
  <header class="titlebar" 
          :class="{ 
            'maximized': isMaximized, 
            'is-chat-mode': configStore.settings.chatMode.enabled 
          }" 
          @mousedown="handleGlobalDrag">
    <!-- Left Segment: Logo/Brand + Avatar -->
    <div class="titlebar-left">
      <!-- User Avatar: Show in Normal Mode titlebar only -->
      <div v-if="false" class="user-avatar-container">
        <img v-if="configStore.userAvatarUrl" 
             :src="configStore.userAvatarUrl" 
             class="user-avatar" />
        <div v-else class="avatar-placeholder">U</div>
      </div>
      <span class="app-name" v-if="!configStore.settings.chatMode.enabled">Goge Chat</span>
    </div>
    
    <!-- Center Segment: Flexible space for dragging -->
    <div class="titlebar-center"></div>
    
    <!-- Right Segment: Selectors + Theme + Window Controls -->
    <div class="titlebar-right">
      <div class="selectors-group" v-if="!isSettings && !configStore.settings.chatMode.enabled">
        <ModelSelector class="titlebar-model-selector" menuId="titlebar-model" />
        <div class="v-divider"></div>
        <PresetSelector class="titlebar-preset-selector" menuId="titlebar-preset" />
      </div>

      <div class="window-controls">
        <template v-if="!configStore.settings.chatMode.enabled">
          <button
            @click.stop="toggleTheme"
            class="control-btn theme-toggle"
            :title="isLight ? '切换深色模式' : '切换浅色模式'"
            v-html="isLight ? SUN_SVG : MOON_SVG"
          ></button>

          <button
            v-if="isSettings"
            @click.stop="$emit('back-home')"
            class="control-btn settings-btn"
            title="返回首页"
            v-html="HOME_SVG"
          ></button>

          <button
            v-if="!isSettings"
            @click.stop="openSettings"
            class="control-btn settings-btn"
            title="设置"
            v-html="SETTINGS_SVG"
          ></button>
        </template>

        <!-- History Toggle Button (Shown in Chat Mode) -->
        <button 
          @click.stop="toggleHistory" 
          class="control-btn history-btn" 
          title="历史记录" 
          v-html="HISTORY_SVG" 
          v-if="configStore.settings.chatMode.enabled"
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
  background: var(--bg-titlebar); 
  display: flex;
  align-items: center; 
  padding: 0 10px; 
  -webkit-app-region: drag; 
  position: relative;
  z-index: 100;
  border-bottom: none;
}

.titlebar-left { display: flex; align-items: center; flex-shrink: 0; gap: 8px; }

/* 用户头像样式 (通用 CSS 变量) */
.user-avatar-container {
  display: flex;
  align-items: center;
  justify-content: center;
  transform: translate(var(--user-avatar-offset-x), var(--user-avatar-offset-y));
  transition: transform 0.2s ease;
  -webkit-app-region: no-drag;
}

.user-avatar {
  width: var(--user-avatar-size);
  height: var(--user-avatar-size);
  border-radius: var(--user-avatar-radius);
  object-fit: cover;
  transition: all 0.2s ease;
}

.avatar-placeholder {
  width: var(--user-avatar-size);
  height: var(--user-avatar-size);
  background: var(--color-primary);
  color: white;
  border-radius: var(--user-avatar-radius);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: calc(var(--user-avatar-size) * 0.45);
  transition: all 0.2s ease;
}

.titlebar-center { flex: 1; height: 100%; }
.titlebar-right { display: flex; align-items: center; gap: 12px; }

.app-name { 
  font-weight: 600; 
  color: var(--color-title-text); 
  font-size: 11px; 
  margin-left: 2px; 
  letter-spacing: 0.5px; 
  opacity: 0.8; 
}

/* Chat Mode (Immersive) Overrides */
.titlebar.is-chat-mode {
  background: transparent;
  width: 100%;
  position: absolute;
  top: 0;
  right: 0;
  pointer-events: none; /* Let clicks pass through to content header */
  border-bottom: none;
}

.titlebar.is-chat-mode .window-controls {
  pointer-events: auto; /* Re-enable clicking on buttons */
  color: var(--text-color);
}

.titlebar.is-chat-mode .control-btn {
  color: var(--text-color);
}

.selectors-group {
    display: flex;
    align-items: center;
    gap: 4px;
    background: var(--bg-selectors);
    padding: 2px;
    border-radius: 99px;
    border: 1px solid var(--border-selectors);
}

.v-divider {
    width: 1px;
    height: 14px;
    background: var(--border-selectors);
}

/* Target selectors specifically for titlebar context */
.selectors-group :deep(.selector-btn) {
  color: var(--color-title-text);
}
.selectors-group :deep(.selector-btn:hover),
.selectors-group :deep(.selector-btn.active) {
  color: var(--color-title-text-bright);
}
.selectors-group :deep(.chevron) {
  color: var(--color-title-text);
}


/* 返回按钮样式 */
.back-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: none;
  color: var(--color-title-text);
  font-size: 12px;
  font-weight: bold;
  cursor: pointer;
  -webkit-app-region: no-drag;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.2s;
}
.back-btn:hover {
  background: var(--bg-glass-hover);
  color: var(--color-title-text-bright);
}

.window-controls { display: flex; height: 100%; align-items: center; -webkit-app-region: no-drag; gap: 8px; margin-right: 8px; }
.control-btn { 
  background: transparent; 
  border: 1px solid transparent; 
  color: var(--color-window-controls); 
  width: 30px; 
  height: 30px; 
  display: flex; align-items: center; justify-content: center; cursor: pointer;
  -webkit-app-region: no-drag;
  border-radius: var(--radius-window-btn, 6px);
  transition: all 0.2s ease;
}
.control-btn:hover { 
  background: var(--bg-window-btn); 
  border-color: var(--border-window-btn); 
  color: var(--color-title-text-bright); 
}
.settings-btn:hover { color: var(--color-primary); }
.theme-toggle:hover { color: var(--color-primary); }
.theme-toggle :deep(svg),
.settings-btn :deep(svg) { width: 14px; height: 14px; }
.close-btn:hover { background: #c42b1c !important; color: white !important; }


</style>