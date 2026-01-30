<script setup>
/**
 * SideBarHeader.vue - 侧边栏头部
 * 职责：管理侧边栏折叠开关，并在展开模式下提供极简搜索功能。
 */
import { ref, nextTick, watch } from 'vue';
import { MENU_SVG, SEARCH_SVG, FOLDER_PLUS_SVG } from '../../../constants/icons.ts';

const props = defineProps({
  isCollapsed: { type: Boolean, default: false }
});

const emit = defineEmits(['toggle', 'search', 'newFolder']);

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
    <button class="icon-btn menu-btn" @click="emit('toggle')" title="切换侧边栏" v-html="MENU_SVG"></button>

    <div v-if="!isCollapsed" class="header-search-container">
      <button
        class="icon-btn folder-btn"
        title="新建文件夹"
        @click="emit('newFolder')"
        v-html="FOLDER_PLUS_SVG"
      ></button>
      <Transition name="search-expand" mode="out-in">
        <div v-if="isSearching" class="search-input-wrapper">
          <span class="search-icon-inner" v-html="SEARCH_SVG"></span>
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
        
        <button v-else class="icon-btn search-trigger" @click="toggleSearch" v-html="SEARCH_SVG"></button>
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
  /* 🚩 使用主题头部图标颜色变量 */
  color: var(--color-header-icon);
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
  background: var(--bg-glass-hover);
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
  gap: 4px;
}

.search-input-wrapper {
  display: flex;
  align-items: center;
  /* 极简无边框设计，仅在悬停时感知背景 */
  background: var(--bg-selectors);
  border: 1px solid var(--border-selectors);
  border-radius: 20px;
  padding: 0 12px;
  height: 36px;
  width: 100%;
  box-sizing: border-box;
}

.search-icon-inner {
  display: flex;
  color: var(--text-color-white);
  margin-right: 8px;
  opacity: 0.6;
}

input {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-color-white);
  font-size: 13px;
  outline: none;
  min-width: 0;
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--text-color);
  cursor: pointer;
  padding: 4px;
  font-size: 14px;
}

.close-btn:hover { color: var(--text-color-white); }

/* 搜索框动画：平滑淡入 */
.search-expand-enter-active,
.search-expand-leave-active {
  transition: all 0.2s ease;
}
.search-expand-enter-from { opacity: 0; transform: translateX(8px); }
.search-expand-leave-to { opacity: 0; transform: translateX(8px); }
</style>