<script setup>
import { ref, onMounted } from 'vue';
import AppNavBar from '../components/layout/AppNavBar.vue';
import SocialSidebar from '../components/layout/SocialSidebar.vue';
import SettingsModal from '../components/settings/SettingsModal.vue';
import { useConfigStore } from '../stores/config';
import { useSettingsStore } from '../stores/settings';
import { invoke } from '@tauri-apps/api/core';
import { AI_EVO_SVG } from '../constants/icons';

const configStore = useConfigStore();
const settingsStore = useSettingsStore();

const activeModule = ref('chat');
const isCollapsed = ref(false);
const selectedContact = ref(null);

const handleSelect = (contact) => {
  selectedContact.value = contact;
};

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

const handleCloseSettings = () => {
  settingsStore.closeSettings();
};
</script>

<template>
  <div class="main-layout">
    <AppNavBar 
      v-model:activeModule="activeModule"
      :is-collapsed="isCollapsed"
      :is-in-settings="settingsStore.isModalOpen"
      @toggleCollapse="toggleCollapse"
      @openSettings="handleOpenSettings"
      @backHome="handleCloseSettings"
    />
    
    <div class="sidebar-wrapper">
      <SocialSidebar 
        :is-collapsed="isCollapsed"
        @select="handleSelect"
      />
      <!-- Interaction Lock Overlay -->
      <div v-if="settingsStore.isModalOpen" class="sidebar-overlay"></div>
    </div>

    <main class="content-view">
      <!-- Settings Mode -->
      <SettingsModal 
        v-if="settingsStore.isModalOpen" 
        @close="handleCloseSettings" 
      />

      <!-- Chat Mode -->
      <template v-else>
        <template v-if="activeModule === 'chat'">
          <slot v-if="selectedContact" :active-contact="selectedContact"></slot>
          <div v-else class="social-placeholder">
             <div class="placeholder-icon" v-html="AI_EVO_SVG"></div>
             <p>选择一个联系人开始对话</p>
          </div>
        </template>
        <div v-else class="placeholder-module">
          <p>{{ activeModule }} 模块正在开发中...</p>
        </div>
      </template>
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

.sidebar-wrapper {
  position: relative;
  display: flex;
  height: 100%;
}

.sidebar-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.05); /* Slight tint */
  z-index: 10;
  cursor: not-allowed;
  pointer-events: auto; /* Capture clicks */
}

/* Ensure SocialSidebar doesn't shrink */
.sidebar-wrapper :deep(.social-sidebar) {
  height: 100%;
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

.social-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-color);
  opacity: 0.2;
  gap: 16px;
}

.placeholder-icon :deep(svg) {
  width: 80px;
  height: 80px;
}

.social-placeholder p {
  font-size: 1.1rem;
  letter-spacing: 1px;
}
</style>
