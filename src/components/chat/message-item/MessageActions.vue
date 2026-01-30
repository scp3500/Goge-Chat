<script setup>
import { REFRESH_SVG, COPY_SVG, MORE_SVG, EDIT_SVG, TRASH_SVG } from '../../../constants/icons.ts';

defineProps({
  role: {
    type: String,
    default: 'user'
  },
  show: {
    type: Boolean,
    default: true
  }
});

defineEmits(['edit', 'delete', 'copy', 'regenerate']);
</script>

<template>
  <div v-if="show" :class="role === 'user' ? 'msg-action-bar-user' : 'msg-action-bar-bottom'">
    <template v-if="role === 'user'">
      <button class="action-btn" title="编辑" @click="$emit('edit', $event)" v-html="EDIT_SVG"></button>
      <button class="action-btn" title="删除" @click="$emit('delete', $event)" v-html="TRASH_SVG"></button>
    </template>
    <template v-else>
      <button class="action-btn" title="重新生成" @click="$emit('regenerate')" v-html="REFRESH_SVG"></button>
      <button class="action-btn" title="复制全文" @click="$emit('copy', $event)" v-html="COPY_SVG"></button>
      <button class="action-btn delete-btn" title="删除" @click="$emit('delete', $event)" v-html="TRASH_SVG"></button>
      <button class="action-btn" title="更多" v-html="MORE_SVG"></button>
    </template>
  </div>
</template>

<style scoped>
.msg-action-bar-user { 
  display: flex; 
  gap: 4px; 
  margin-top: 4px; 
  opacity: 0; 
  visibility: hidden; 
  transition: all 0.2s; 
}


.msg-action-bar-bottom { 
  display: flex; 
  gap: 4px; 
  margin-top: 10px; 
  padding-left: 2px; 
}

.action-btn { 
  background: transparent; 
  border: none; 
  color: var(--text-color); 
  opacity: 0.7;
  cursor: pointer; 
  padding: 6px; 
  border-radius: 6px; 
  display: flex; 
  align-items: center; 
  transition: all 0.2s; 
}

.action-btn:hover { 
  color: var(--text-color-white); 
  background: var(--bg-glass-hover); 
  opacity: 1;
}

.action-btn.delete-btn:hover { 
  color: var(--color-danger); 
  background: var(--color-danger-alpha-10); 
}
</style>
