<script setup>
import { ref, onMounted } from 'vue';
import AppNavBar from '../components/layout/AppNavBar.vue';
import SocialSidebar from '../components/layout/SocialSidebar.vue';
import { useConfigStore } from '../stores/config';
import { useSettingsStore } from '../stores/settings';
import { invoke } from '@tauri-apps/api/core';

const configStore = useConfigStore();
const settingsStore = useSettingsStore();

const activeModule = ref('chat');
const isCollapsed = ref(false);

onMounted(async () => {
  try {
    const savedState = await invoke('get_social_setting', { key: 'sidebar_collapsed' });
    if (savedState !== null) {
      isCollapsed.value = savedState === 'true';
    }
  } catch (e) {
    console.error('Failed to load sidebar state:', e);
  }
});

const toggleCollapse = async () => {
  isCollapsed.value = !isCollapsed.value;
  try {
    await invoke('set_social_setting', { 
      key: 'sidebar_collapsed', 
      value: isCollapsed.value.toString() 
    });
  } catch (e) {
    console.error('Failed to save sidebar state:', e);
  }
};

const handleOpenSettings = () => {
  settingsStore.openSettings();
};
</script>

<template>
  <div class="main-layout">
    <AppNavBar 
      v-model:activeModule="activeModule"
      :is-collapsed="isCollapsed"
      @toggleCollapse="toggleCollapse"
      @openSettings="handleOpenSettings"
    />
    
    <SocialSidebar 
      :is-collapsed="isCollapsed"
    />

    <main class="content-view">
      <!-- In chatMode, we render the content based on activeModule -->
      <slot v-if="activeModule === 'chat'"></slot>
      <div v-else class="placeholder-module">
        <p>{{ activeModule }} 模块正在开发中...</p>
      </div>
    </main>
  </div>
</template>

<style scoped>
.main-layout {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: var(--bg-main);
}

.content-view {
  flex: 1;
  height: 100%;
  overflow: hidden;
  position: relative;
  display: flex;
  flex-direction: column;
}

.placeholder-module {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-color);
  opacity: 0.4;
}
</style>
