<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';

import { useConfigService } from './services/configService';
// 注意：暂时保留文件名，逻辑上它已经是一个 Full Page 了
import SettingsModal from "./components/settings/SettingsModal.vue"; 
import TitleBar from "./components/TitleBar.vue";
import SideBar from "./components/SideBar.vue";
import ChatContainer from "./components/chat/ChatContainer.vue";

const appWindow = getCurrentWindow();
const configStore = useConfigService();

const isMaximized = ref(false); 
const activeId = ref(null); 
const historyList = ref([]);
const showSettings = ref(false); // 核心切换开关

// 加载会话历史 [cite: 2026-01-20]
const loadData = async () => {
  try {
    const sessions = await invoke('get_sessions');
    historyList.value = sessions;
    // 自动选中第一条记录
    if (sessions.length > 0 && activeId.value === null) {
      activeId.value = sessions[0].id;
    }
  } catch (e) { console.error("DB加载失败", e); }
};

/**
 * 【核心修复】：实时同步滚动位置到内存 [cite: 2026-01-20]
 * 当 ChatContainer 传回新位置时，直接修改 historyList 里的对应项
 */
const handleScrollSync = (id, pos) => {
  const session = historyList.value.find(s => s.id === id);
  if (session) {
    session.last_scroll_pos = pos;
    // console.log(`内存已同步: 会话 ${id} -> 位置 ${pos}`);
  }
};

// 新建对话 [cite: 2026-01-20]
const handleCreate = async () => {
  const newId = await invoke('create_session', { title: "新对话" });
  // 插入到列表首位，并初始化位置为 0
  historyList.value.unshift({ id: newId, title: "新对话", last_scroll_pos: 0 });
  activeId.value = newId; 
};

// 删除对话 [cite: 2026-01-20]
const handleDelete = async (id) => {
  await invoke('delete_session', { id });
  historyList.value = historyList.value.filter(item => item.id !== id);
  if (activeId.value === id) activeId.value = historyList.value[0]?.id || null;
};

// 重命名对话 [cite: 2026-01-20]
const handleRename = async (id, newTitle) => {
  try {
    await invoke('update_session_title', { id, title: newTitle });
    const item = historyList.value.find(i => i.id === id);
    if (item) item.title = newTitle;
  } catch (e) { console.error("重命名失败", e); }
};

onMounted(async () => {
  await configStore.init(); 
  await loadData();
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
            :active="activeId" 
            :list="historyList" 
            @create="handleCreate" 
            @select="id => activeId = id" 
            @delete="id => handleDelete(id)" 
            @rename="(id, t) => handleRename(id, t)"
          />
          
          <ChatContainer 
            v-if="activeId !== null"
            :key="activeId"
            :currentId="activeId" 
            :sessions="historyList"
            @scroll-update="handleScrollSync"
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
html, body, #app { overflow: hidden !important; height: 100%; margin: 0; background: transparent; }

/* 视图切换动画 [cite: 2026-01-20] */
.view-fade-enter-active, .view-fade-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}
.view-fade-enter-from { opacity: 0; transform: translateX(10px); }
.view-fade-leave-to { opacity: 0; transform: translateX(-10px); }
</style>

<style scoped>
.app-layout { 
  display: flex; flex-direction: column; height: 100vh; 
  background: var(--bg-main, #131314); 
  color: #e3e3e3; 
  border-radius: 12px; border: 1px solid #333; overflow: hidden;
}
.app-layout.is-maximized { border-radius: 0; border: none; }
.content-area { flex: 1; position: relative; overflow: hidden; }
.main-view { display: flex; width: 100%; height: 100%; }
.empty-chat { flex: 1; display: flex; align-items: center; justify-content: center; color: #555; font-size: 14px; }
</style>