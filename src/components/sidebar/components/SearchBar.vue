<script setup>
import { ref, nextTick, watch } from 'vue';
const props = defineProps(['isCollapsed']); 
const emit = defineEmits(['search']);

const isSearching = ref(false);
const searchQuery = ref("");
const inputRef = ref(null);

watch(() => props.isCollapsed, (val) => {
  if (val) {
    isSearching.value = false;
    searchQuery.value = "";
    emit('search', "");
  }
});

const toggleSearch = async (state) => {
  isSearching.value = state;
  if (state) {
    await nextTick();
    inputRef.value?.focus();
  } else {
    searchQuery.value = "";
    emit('search', "");
  }
};
</script>

<template>
  <div v-if="!isCollapsed" class="search-wrapper">
    <Transition name="search-fade" mode="out-in">
      <button v-if="!isSearching" class="minimal-search-trigger" @click="toggleSearch(true)">
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
      </button>

      <div v-else class="search-capsule">
        <svg class="inner-icon" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input 
          ref="inputRef" 
          v-model="searchQuery" 
          placeholder="搜索对话..." 
          class="modern-input"
          @input="e => emit('search', e.target.value)" 
        />
        <button class="clear-btn" @click="toggleSearch(false)">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.search-wrapper {
  padding: 0 16px 0 28px; /* 严格控制上下 padding 为 0，靠容器高度撑开 */
  height: 48px;
  display: flex;
  align-items: center;
  overflow: hidden;
}

/* 🚩 触发按钮：保持极其干净的视觉 */
.minimal-search-trigger {
  background: transparent;
  border: none;
  color: var(--text-color);
  opacity: 0.6;
  cursor: pointer;
  padding: 8px;
  border-radius: 8px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.minimal-search-trigger:hover {
  color: var(--text-color-white);
  background: var(--bg-glass-hover);
}

/* 🚩 胶囊：现代化设计的精髓（内嵌、微光、无界） */
.search-capsule {
  display: flex;
  align-items: center;
  background: var(--bg-glass); /* 极低透明度，显得高级 */
  border: 1px solid var(--border-glass); /* 微微的边缘线 */
  border-radius: 10px;
  padding: 0 10px;
  width: 100%;
  height: 32px; /* 固定高度，这是防止跳动的关键 */
  box-sizing: border-box;
}

.inner-icon {
  color: var(--text-color);
  opacity: 0.5;
  flex-shrink: 0;
}

/* 🚩 Input：解决打字跳动的核心配置 */
.modern-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-color-white);
  font-size: 13px;
  padding: 0 8px;
  height: 30px;      /* 略小于父容器，留出余地 */
  line-height: 30px; /* 强制行高与高度一致，锁死基准线 */
  font-family: inherit;
}

.modern-input::placeholder {
  color: var(--text-color);
  opacity: 0.4;
}

.clear-btn {
  background: transparent;
  border: none;
  color: var(--text-color);
  opacity: 0.4;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s;
}

.clear-btn:hover {
  color: var(--text-color-white);
  background: var(--bg-glass-hover);
}

/* 动画：丝滑淡入淡出 */
.search-fade-enter-active,
.search-fade-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.search-fade-enter-from,
.search-fade-leave-to {
  opacity: 0;
  transform: translateY(2px);
}
</style>