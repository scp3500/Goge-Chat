<script setup>
/**
 * NewChatBtn.vue - 轴线对齐调优版
 * 职责：通过负边距补偿，确保图标重心完美回归 28px 黄金线。
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
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
          </svg>
        </span> 
        
        <span v-if="!isCollapsed" class="btn-text"> 新对话</span>
      </div>
    </button>
  </div>
</template>

<style scoped>
.action-area { 
  /* 🚩 基础容器：锁定 28px 黄金轴线起点 */
  padding: 4px 16px 4px 28px; 
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
  color: #9aa0a6;
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
  overflow: visible; /* 🛡️ 允许悬停背景稍微溢出以对齐 */
  
  /* 🚩 核心修复：通过负边距，让按钮的视觉重心左移 */
  margin-left: -8px; 
}

.new-chat-pill:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #fff;
  /* 保持轻量，取消边框或仅用极淡颜色 */
  border-color: rgba(255, 255, 255, 0.05); 
}

/* 窄模式适配 */
.collapsed .new-chat-pill {
  width: 40px;
  margin-left: 0; /* 窄模式回归物理中心 */
  justify-content: center;
}

.pill-content {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  /* 🚩 补偿逻辑：28(padding) - 8(margin) + 8(content padding) = 28px 精准对齐 */
  padding-left: 8px;
  gap: 12px;
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
}

.btn-text {
  font-size: 14px;
  font-weight: 500;
  letter-spacing: 0.2px;
  white-space: nowrap;
}
</style>