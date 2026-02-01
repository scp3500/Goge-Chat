<script setup>
/**
 * SideBar.vue - 侧边栏主容器
 * 职责：物理宽度切换，并强制子组件进入“窄模式”或“展开模式”。
 */
import { ref } from 'vue';
import SideBarHeader from './SideBarHeader.vue';
import NewChatBtn from '../NewChatBtn.vue';
import HistoryList from './HistoryList.vue';
import { useChatStore } from '../../../stores/chat';

const chatStore = useChatStore();

const props = defineProps({
  active: { type: [String, Number, null], default: null },
  list: { type: Array, default: () => [] }
});

const emit = defineEmits(['create', 'select', 'delete', 'rename', 'reorder', 'reorder-folders', 'newFolder']);

// 侧边栏折叠状态：true = 72px 窄模式, false = 300px 展开模式
const isCollapsed = ref(false); 
const searchQuery = ref("");

const handleSearch = (query) => {
  searchQuery.value = query;
};

const toggleSidebar = () => {
  isCollapsed.value = !isCollapsed.value;
};
</script>

<template>
  <aside 
    class="sidebar" 
    :class="{ 'is-collapsed': isCollapsed }"
  >
    <SideBarHeader
      :is-collapsed="isCollapsed"
      @toggle="toggleSidebar"
      @search="handleSearch"
      @new-folder="emit('newFolder')"
    />

    <div class="main-content-wrapper">
      <NewChatBtn
        :is-collapsed="isCollapsed"
        @click="emit('create')"
        @new-folder="emit('newFolder')"
      />
      
      <HistoryList
        :is-collapsed="isCollapsed"
        :list="props.list"
        :active="props.active"
        :filter="searchQuery"
        @select="(id) => emit('select', id)"
        @delete="(id) => emit('delete', id)"
        @rename="(id, title) => emit('rename', id, title)"
        @reorder="(newList) => emit('reorder', newList)"
        @reorder-folders="(newList) => emit('reorder-folders', newList)"
        @dblclick="isCollapsed = true"
      />

      <footer class="sidebar-footer">
        <div class="user-info">
          {{ isCollapsed ? 'V2' : 'Goge Chat v2' }}
        </div>
      </footer>
    </div>
  </aside>
</template>

<style scoped>
.sidebar { 
  /* 🚩 展开模式：物理宽度三重锁死 */
  width: 260px; 
  min-width: 260px;
  max-width: 260px;
  
  --collapsed-width: 66px;

  /* 动力学曲线：确保缩放极其顺滑 */
  transition: all 0.3s cubic-bezier(0.05, 0.7, 0.1, 1);
  background: var(--bg-sidebar); 
  height: 100vh; 
  display: flex; 
  flex-direction: column; 
  
  /* --- 🩺 手术位置：彻底删除这一行 --- */
  border-right: none;
  /* ---------------------------------- */

  /* Local overrides for dark/vibrant sidebar frames */


  /* 🛡️ 核心：切掉所有溢出内容 */
  overflow: hidden; 
  box-sizing: border-box; 
  flex-shrink: 0; 
}

/* 🚩 只给不需要选中的 UI 元素加这个属性 */
.icon-btn, .new-chat-pill, .sidebar-footer {
  user-select: none;
}

/* 🚩 窄模式：强制物理收缩 */
.sidebar.is-collapsed { 
  width: var(--collapsed-width) !important; 
  min-width: var(--collapsed-width) !important;
  max-width: var(--collapsed-width) !important;
}

.main-content-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  /* 🚩 解除固定宽度限制 */
  width: 100%;
  min-width: 0; 
  transition: opacity 0.2s ease;
}

/* 🚩 核心“模式切换”黑科技 */
.sidebar.is-collapsed :deep(.btn-text),
.sidebar.is-collapsed :deep(.header-search-container),
.sidebar.is-collapsed :deep(.title-text),
.sidebar.is-collapsed :deep(.more-btn) {
  display: none !important;
  pointer-events: none;
  opacity: 0;
}

.sidebar-footer { 
  padding: 16px 20px 16px 28px; 
  border-top: none; 
  font-size: 10px; 
  color: var(--text-color);
  opacity: 0.5;
  text-align: left; 
  white-space: nowrap; 
  flex-shrink: 0;
  transition: all 0.3s;
}

/* 🚩 窄模式页脚：彻底居中 */
.is-collapsed .sidebar-footer {
  padding: 16px 0;
  text-align: center;
  width: 100%;
}
</style>