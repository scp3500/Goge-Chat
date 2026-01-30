<script setup>
import { ATTACHMENT_SVG } from '../../../constants/icons.ts';

defineProps({
  files: {
    type: Array,
    required: true
  }
});

const emit = defineEmits(['open-file']);

const handleOpenFile = (path) => {
  emit('open-file', path);
};
</script>

<template>
  <div class="message-file-attachments" v-if="files.length > 0">
    <div 
      v-for="file in files" 
      :key="file.path" 
      class="message-file-card"
      @dblclick="handleOpenFile(file.path)"
      title="双击打开文件"
    >
      <div class="m-file-icon" v-html="file.icon || ATTACHMENT_SVG"></div>
      <div class="m-file-info">
        <span class="m-file-name text-ellipsis">{{ file.name }}</span>
      </div>
      <button class="m-open-btn" @click.stop="handleOpenFile(file.path)" title="打开文件">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
          <polyline points="15 3 21 3 21 9"></polyline>
          <line x1="10" y1="14" x2="21" y2="3"></line>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.message-file-attachments {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 10px;
}

.message-file-card {
  display: flex;
  align-items: center;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 8px 12px;
  gap: 10px;
  cursor: pointer;
  transition: all 0.2s;
  min-width: 180px;
}

.message-file-card:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.2);
}

.m-file-icon {
  color: #818cf8;
  display: flex;
  align-items: center;
}

.m-file-icon :deep(svg) {
  width: 20px;
  height: 20px;
}

.m-file-info {
  flex: 1;
  overflow: hidden;
}

.m-file-name {
  font-size: 13px;
  color: #efefef;
  display: block;
}

.text-ellipsis {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.m-open-btn {
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  transition: all 0.2s;
}

.m-open-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}
</style>
