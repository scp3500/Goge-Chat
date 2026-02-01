<script setup>
import { ref, watch, nextTick } from 'vue';
const props = defineProps(['item', 'isActive', 'isEditingId', 'isCollapsed']);
const emit = defineEmits(['select', 'rename', 'enter-edit', 'contextmenu', 'dblclick']);

const tempTitle = ref(props.item.title);
const inputRef = ref(null);

// 监听编辑状态同步
watch(() => props.isEditingId, async (newId) => {
  if (newId === props.item.id) {
    tempTitle.value = props.item.title;
    await nextTick();
    inputRef.value?.focus();
    inputRef.value?.select();
  }
});

const handleRename = () => {
  const trimmed = tempTitle.value.trim();
  if (trimmed) emit('rename', props.item.id, trimmed);
};
</script>

<template>
  <div
    class="history-item"
    :class="{
      'active': isActive,
      'editing': isEditingId === item.id,
      'collapsed': isCollapsed
    }"
    @click="$emit('select', item.id)"
    @dblclick="$emit('dblclick')"
    @contextmenu.prevent="$emit('contextmenu', item.id, $event)"
  >
    <div class="active-indicator"></div>

    <div class="content-wrapper">
      <input
        v-if="isEditingId === item.id"
        ref="inputRef"
        v-model="tempTitle"
        class="edit-input"
        @keyup.enter="handleRename"
        @blur="handleRename"
        @click.stop
      />
      <span v-else class="title-text">{{ item.title?.replace(/^[cr]:/g, '') }}</span>
    </div>

    <button v-if="!isEditingId" class="more-btn" @click.stop="$emit('contextmenu', item.id, $event)">⋯</button>
  </div>
</template>

<style scoped>
/* 🚩 定义布局变量，方便未来做外观设置 */
.history-item {
  --item-padding-left: 20px; /* 统一的文字起始留白 */
  --accent-white: var(--text-color-white);
  
  height: 40px;
  margin: 2px 8px;
  width: auto;
  padding-right: 12px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  cursor: pointer;
  position: relative;
  transition: all 0.2s ease;
  color: var(--text-color);
}

.history-item:hover { background: var(--bg-glass-hover); color: var(--color-sidebar-text-hover); }
.history-item.active { background: var(--bg-glass-active); color: var(--color-primary); }

.history-item.collapsed {
  margin: 2px 0;
  width: 40px;
  justify-content: center;
  padding-right: 0;
}

.history-item.collapsed .content-wrapper {
  padding-left: 0;
  justify-content: center;
}

.active-indicator {
  position: absolute;
  left: 0;
  width: 3px;
  height: 14px;
  background: var(--color-primary);
  border-radius: 0 4px 4px 0;
  opacity: 0;
  transform: scaleY(0.5);
  transition: all 0.2s ease;
}

.history-item.active .active-indicator { opacity: 1; transform: scaleY(1); }

.content-wrapper { 
  flex: 1; 
  min-width: 0; 
  display: flex; 
  align-items: center; 
  height: 100%;
  /* 🚩 关键：锁定文字的物理起始点 */
  padding-left: var(--item-padding-left); 
}

.title-text {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 🚩 核心：带留白的对齐修复 */
.edit-input {
  width: 100%;
  background: var(--bg-input-focus);
  border: 1px solid var(--border-glass-bright);
  color: var(--text-color-white);
  font-size: 13px;
  border-radius: 4px;
  outline: none;
  /* 对齐计算：
     1. 我们想要文字起始于 padding-left 的位置。
     2. Input 本身有 1px 边框和 6px 内边距。
     3. 所以 margin-left 设为 -7px，把 input 的边框“顶”回去，
        让光标正好对准原本 span 文字的起始像素。
  */
  padding: 4px 6px;
  margin-left: -7px; 
  height: 26px;
}

.more-btn {
  opacity: 0;
  background: transparent;
  border: none;
  color: var(--text-color);
  opacity: 0.5;
  font-size: 18px;
  cursor: pointer;
  margin-left: 8px;
}

.history-item:hover .more-btn { opacity: 1; }
</style>