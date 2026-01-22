<script setup>
/**
 * SideBarHeader.vue - 侧边栏头部
 * 职责：管理侧边栏折叠开关，并在展开模式下提供极简搜索功能。
 */
import { ref, nextTick, watch } from 'vue';

const props = defineProps({
  isCollapsed: { type: Boolean, default: false }
});

const emit = defineEmits(['toggle', 'search']);

const isSearching = ref(false);
const searchQuery = ref("");
const inputRef = ref(null);

// 🚩 核心逻辑：当侧边栏折叠时，强制物理关闭搜索状态，防止“收起来还在”
watch(() => props.isCollapsed, (newVal) => {
  if (newVal) {
    isSearching.value = false;
    searchQuery.value = "";
    emit('search', ""); // 清除父组件的过滤结果
  }
});

const toggleSearch = async () => {
  // 如果侧边栏当前是折叠的，点击搜索应先通知父级展开
  if (props.isCollapsed) {
    emit('toggle');
    await nextTick();
  }
  
  isSearching.value = !isSearching.value;
  
  if (isSearching.value) {
    await nextTick();
    inputRef.value?.focus(); // 自动聚焦搜索框
  } else {
    searchQuery.value = "";
    emit('search', ""); // 关闭时重置搜索
  }
};

const handleInput = (e) => {
  emit('search', e.target.value);
};
</script>

<template>
  <header class="sidebar-header" :class="{ 'is-collapsed': isCollapsed }">
    <button class="icon-btn menu-btn" @click="emit('toggle')" title="切换侧边栏">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="3" y1="12" x2="21" y2="12"></line>
        <line x1="3" y1="6" x2="21" y2="6"></line>
        <line x1="3" y1="18" x2="21" y2="18"></line>
      </svg>
    </button>

    <div v-if="!isCollapsed" class="header-search-container">
      <Transition name="search-expand" mode="out-in">
        <div v-if="isSearching" class="search-input-wrapper">
          <span class="search-icon-inner">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="8"></circle>
              <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
          </span>
          <input 
            ref="inputRef"
            v-model="searchQuery"
            type="text" 
            placeholder="搜索记录..." 
            @input="handleInput"
            @keyup.esc="toggleSearch"
          />
          <button class="close-btn" @click="toggleSearch">✕</button>
        </div>
        
        <button v-else class="icon-btn search-trigger" @click="toggleSearch">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
        </button>
      </Transition>
    </div>
  </header>
</template>

<style scoped>
.sidebar-header {
  height: 64px;
  display: flex;
  align-items: center;
  /* 🚩 展开模式：黄金 28px 轴线对齐 */
  padding: 0 16px 0 28px; 
  width: 100%;
  box-sizing: border-box;
  transition: all 0.3s cubic-bezier(0.05, 0.7, 0.1, 1);
}

/* 🚩 窄模式：由于搜索容器已被物理移除，menu-btn 将在此完美居中 */
.sidebar-header.is-collapsed {
  padding: 0;
  justify-content: center;
}

.icon-btn {
  background: transparent;
  border: none;
  /* 🚩 使用纯白色调，极简风格 */
  color: #ffffff; 
  cursor: pointer;
  padding: 8px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  flex-shrink: 0;
  opacity: 0.8;
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  opacity: 1;
}

.menu-btn {
  /* 🚩 展开模式：补偿图标透明边距，使三横杠精准对齐轴线 */
  margin-left: -8px; 
}

.is-collapsed .menu-btn {
  /* 窄模式：取消补偿，回归物理中心 */
  margin-left: 0;
}

.header-search-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  min-width: 0;
}

.search-input-wrapper {
  display: flex;
  align-items: center;
  /* 极简无边框设计，仅在悬停时感知背景 */
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 20px;
  padding: 0 12px;
  height: 36px;
  width: 100%;
  box-sizing: border-box;
}

.search-icon-inner { 
  display: flex;
  color: #ffffff;
  margin-right: 8px; 
  opacity: 0.6; 
}

input {
  flex: 1;
  background: transparent;
  border: none;
  color: #ffffff;
  font-size: 13px;
  outline: none;
  min-width: 0;
}

.close-btn { 
  background: transparent; 
  border: none; 
  color: #9aa0a6; 
  cursor: pointer; 
  padding: 4px; 
  font-size: 14px; 
}

.close-btn:hover { color: #ffffff; }

/* 搜索框动画：平滑淡入 */
.search-expand-enter-active,
.search-expand-leave-active {
  transition: all 0.2s ease;
}
.search-expand-enter-from { opacity: 0; transform: translateX(8px); }
.search-expand-leave-to { opacity: 0; transform: translateX(8px); }
</style>