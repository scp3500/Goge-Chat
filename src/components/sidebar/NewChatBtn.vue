<script setup>
/**
 * NewChatBtn.vue - 终极对齐调优版
 */
defineProps({
  isCollapsed: { type: Boolean, default: false }
});

defineEmits(['click']);
</script>

<template>
  <div class="action-area" :class="{ 'collapsed': isCollapsed }">
    <button class="new-chat-pill" @click="$emit('click')" :title="isCollapsed ? '新对话' : ''">
      <div class="pill-content">
        <span class="plus-icon">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
          </svg>
        </span> 
        
        <span v-if="!isCollapsed" class="btn-text">新对话</span>
      </div>
    </button>
  </div>
</template>

<style scoped>
.action-area {
  /* 🚩 稍微缩小容器左边距，为负 margin 提供更多腾挪空间 */
  padding: 4px 16px 4px 22px;
  width: 100%;
  box-sizing: border-box;
  transition: all 0.3s cubic-bezier(0.05, 0.7, 0.1, 1);
}

.action-area.collapsed {
  padding: 8px 0;
  display: flex;
  justify-content: center;
}

.new-chat-pill {
  width: 100%;
  height: 40px;
  background: transparent;
  border: 1px solid transparent;
  color: var(--text-color);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;

  /* 🚩 增加负 margin，让 hover 时的背景更靠左 */
  margin-left: -6px;
}

.new-chat-pill:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-color-white);
}

/* 窄模式回归物理中心 */
.collapsed .new-chat-pill {
  width: 40px;
  margin-left: 0;
  justify-content: center;
}

.pill-content {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  padding-left: 6px;
  gap: 10px;
  width: 100%;
}

.collapsed .pill-content {
  padding-left: 0;
  justify-content: center;
}

.plus-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  
  /* 🚩 终极必杀：如果觉得还往右差一点，就改这个 -1px 或 -2px */
  /* transform 可以在不影响布局的情况下，物理平移图标的视觉位置 */
  transform: translateX(1px); 
}

.btn-text {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
}
</style>