<script setup>
import { ref, nextTick, watch } from 'vue';
const props = defineProps(['isCollapsed']); // 接收父组件的状态
const emit = defineEmits(['search']);

const isSearching = ref(false);
const searchQuery = ref("");
const inputRef = ref(null);

// 🚩 核心逻辑：一旦侧边栏折叠，强行关闭搜索状态，防止“收起来还在”
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
      <button v-if="!isSearching" class="minimal-icon-btn" @click="toggleSearch(true)">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
      </button>

      <div v-else class="search-box">
        <input ref="inputRef" v-model="searchQuery" placeholder="搜索..." @input="e => emit('search', e.target.value)" />
        <button @click="toggleSearch(false)">✕</button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.search-wrapper {
  padding: 4px 16px 4px 28px; /* 展开时的轴线对齐 */
  height: 40px;
  display: flex;
  align-items: center;
}
/* 这里不再需要报错的 v-bind(isCollapsed) */
</style>