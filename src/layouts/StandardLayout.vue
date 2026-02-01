<script setup>
import { useChatStore } from '../stores/chat';
import SideBar from "../components/sidebar/layout/SideBar.vue";
import ChatContainer from "../components/chat/ChatContainer.vue";

const chatStore = useChatStore();
import { useSettingsStore } from '../stores/settings';
const settingsStore = useSettingsStore();

const handleSessionSelect = (id) => {
  chatStore.switchSession(id);
  if (settingsStore.isModalOpen) {
    settingsStore.closeSettings();
    chatStore.setChatViewActive(true);
  }
};

const handleOpenProfile = () => {
    settingsStore.openSettings('profile');
    chatStore.setChatViewActive(false);
};
</script>

<template>
  <div class="standard-layout">
    <SideBar 
      :active="chatStore.activeId" 
      :list="chatStore.historyList" 
      @create="chatStore.createSession" 
      @select="handleSessionSelect"
      @delete="id => chatStore.deleteSession(id)"
      @rename="chatStore.renameSession"
      @reorder="newList => chatStore.reorderSessions(newList)"
      @reorder-folders="newList => chatStore.reorderFolders(newList)"
      @new-folder="chatStore.createFolder('新建文件夹')"
    />
    <ChatContainer 
      v-if="chatStore.activeId !== null"
      :key="chatStore.activeId"
    />
    <div v-else class="empty-chat">
      <p>选择或创建一个对话开始</p>
    </div>
  </div>
</template>

<style scoped>
.standard-layout {
  display: flex;
  width: 100%;
  height: 100%;
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
