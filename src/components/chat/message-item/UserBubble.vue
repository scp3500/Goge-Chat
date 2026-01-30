<script setup>
import { ref, watch, nextTick } from 'vue';

const props = defineProps({
  content: String,
  isEditing: Boolean
});

const emit = defineEmits(['update-edit-content', 'cancel-edit', 'save-edit']);

const editTextarea = ref(null);

watch(() => props.isEditing, (newVal) => {
  if (newVal) {
    nextTick(() => {
      editTextarea.value?.focus();
    });
  }
});

const formatUserText = (text) => {
  if (!text) return '';
  let cleanText = text.replace(/\[REASON\]|\[SEARCH\]/g, '');
  const attachmentTag = "\n\n--- 附件内容 ---";
  const index = cleanText.indexOf(attachmentTag);
  if (index !== -1) {
    cleanText = cleanText.substring(0, index);
  }
  return cleanText.trim();
};
</script>

<template>
  <div class="message-bubble" :class="{ 'is-editing': isEditing }">
    <template v-if="isEditing">
      <textarea
        ref="editTextarea"
        :value="content"
        class="edit-textarea modern-scroll"
        @input="$emit('update-edit-content', $event.target.value)"
        @keydown.esc="$emit('cancel-edit')"
        @keydown.ctrl.enter="$emit('save-edit')"
      ></textarea>
      <div class="edit-actions">
        <button class="edit-cancel" @click="$emit('cancel-edit')">取消</button>
        <button class="edit-save" @click="e => $emit('save-edit', e)">保存并重新生成</button>
      </div>
    </template>
    <template v-else>
      <div class="user-text">{{ formatUserText(content) }}</div>
    </template>
  </div>
</template>

<style scoped>
.message-bubble { 
  padding: 12px 16px; 
  border-radius: 18px; 
  background: var(--bg-user-bubble); 
  color: var(--color-user-bubble-text); 
  max-width: 100%; 
  word-wrap: break-word; 
  white-space: pre-wrap; 
  border: 1px solid var(--border-glass);
}

.message-bubble.is-editing { 
  width: 100%; 
  padding: 12px; 
  background: var(--bg-glass); 
  border: 1px solid var(--border-glass-bright); 
  border-radius: 14px; 
}

.edit-textarea { 
  width: 100%; 
  min-height: 100px; 
  background: transparent; 
  border: none; 
  color: var(--text-color-white); 
  font-size: 15px; 
  line-height: 1.6; 
  resize: vertical; 
  outline: none; 
  font-family: inherit; 
}

.edit-actions { 
  display: flex; 
  justify-content: flex-end; 
  gap: 10px; 
  margin-top: 10px; 
}

.edit-actions button { 
  padding: 6px 14px; 
  border-radius: 6px; 
  font-size: 13px; 
  cursor: pointer; 
  border: none; 
}

.edit-cancel { 
  background: var(--bg-glass-active); 
  color: var(--text-tertiary); 
}

.edit-save { 
  background: var(--color-primary); 
  color: #fff; 
}
</style>
