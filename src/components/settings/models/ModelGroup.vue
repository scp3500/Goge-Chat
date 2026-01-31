<script setup lang="ts">
import { ref } from 'vue';
import { CHEVRON_DOWN_SVG } from '../../../constants/icons';
import { ModelInfo } from '../../../types/config';
import ModelRow from './ModelRow.vue';

const props = defineProps<{
  groupName: string;
  models: ModelInfo[];
  providerIcon: string;
}>();

const emit = defineEmits<{
  (e: 'edit', model: ModelInfo): void;
  (e: 'remove', id: string): void;
}>();

const isOpen = ref(true);

const toggleGroup = () => {
    isOpen.value = !isOpen.value;
};
</script>

<template>
  <div class="model-group-box" :class="{ 'is-closed': !isOpen }">
    <div class="group-header-v3" @click="toggleGroup">
      <span class="chev" :class="{ 'rotated': !isOpen }" v-html="CHEVRON_DOWN_SVG"></span>
      <span class="group-title">{{ groupName || '未分类' }}</span>
      <span class="group-count">{{ models.length }}</span>
    </div>
    
    <div v-show="isOpen" class="group-items-v3">
      <ModelRow 
        v-for="model in models" 
        :key="model.id" 
        :model="model"
        :providerIcon="providerIcon"
        @edit="emit('edit', $event)"
        @remove="emit('remove', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.model-group-box {
    background: var(--bg-card-dim, rgba(255,255,255,0.02));
    border: 1px solid var(--border-glass);
    border-radius: 12px;
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.model-group-box.is-closed {
    border-color: transparent;
    background: rgba(255,255,255,0.01);
}

.group-header-v3 {
    background: var(--bg-input-dim);
    padding: 10px 16px;
    display: flex;
    align-items: center;
    gap: 10px;
    border-bottom: 1px solid var(--border-glass);
    cursor: pointer;
    user-select: none;
}

.model-group-box.is-closed .group-header-v3 {
    border-bottom-color: transparent;
}

.group-header-v3 .chev { 
    color: var(--text-tertiary); 
    display: flex;
    transition: transform 0.3s ease;
}

.group-header-v3 .chev.rotated {
    transform: rotate(-90deg);
}

.group-title { 
    font-size: 14px; 
    font-weight: 600; 
    color: var(--text-secondary); 
    flex: 1;
}

.group-count {
    font-size: 11px;
    background: var(--bg-glass-active);
    color: var(--text-tertiary);
    padding: 1px 6px;
    border-radius: 10px;
}

.group-items-v3 {
    display: flex;
    flex-direction: column;
}

.group-header-v3 :deep(svg) { width: 10px; height: 10px; }
</style>
