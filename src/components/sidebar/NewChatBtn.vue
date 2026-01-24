<script setup>
/**
 * NewChatBtn.vue - 终极对齐调优版
 */
defineProps({
  isCollapsed: { type: Boolean, default: false }
});

const emit = defineEmits(['click', 'newFolder']);
</script>

<template>
  <div class="action-area" :class="{ 'collapsed': isCollapsed }">
    <button
      v-if="!isCollapsed"
      class="icon-btn-pill new-folder-btn"
      title="新建文件夹"
      @click="emit('newFolder')"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        <line x1="12" y1="11" x2="12" y2="17"></line>
        <line x1="9" y1="14" x2="15" y2="14"></line>
      </svg>
    </button>

    <button class="action-pill new-chat-pill" @click="$emit('click')" :title="isCollapsed ? '新对话' : ''">
      <div class="pill-content">
        <span class="icon plus-icon">
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
  padding: 4px 16px;
  width: 100%;
  box-sizing: border-box;
  transition: all 0.3s cubic-bezier(0.05, 0.7, 0.1, 1);
  display: flex;
  align-items: center;
  gap: 4px;
}

.action-area.collapsed {
  padding: 8px 0;
  display: flex;
  justify-content: center;
}

.action-pill {
  flex: 1;
  height: 40px;
  background: transparent;
  border: 1px solid transparent;
  color: var(--text-color);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
}

.icon-btn-pill {
  width: 40px;
  height: 40px;
  background: transparent;
  border: 1px solid transparent;
  color: var(--text-color);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.action-pill:hover,
.icon-btn-pill:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-color-white);
}

.new-folder-btn {
  margin-left: -6px;
}

.new-chat-pill {
  margin-right: -6px;
}

/* 窄模式回归物理中心 */
.collapsed .action-pill {
  width: 40px;
  flex: 0 0 40px;
  margin: 0;
  justify-content: center;
}

.pill-content {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  padding: 0 8px;
  gap: 8px;
  width: 100%;
}

.collapsed .pill-content {
  padding: 0;
  justify-content: center;
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.plus-icon {
  transform: translateX(1px);
}

.btn-text {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
}
</style>