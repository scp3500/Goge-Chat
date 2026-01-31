<script setup>
import { computed } from 'vue';

const props = defineProps({
  icon: String,
  modelName: String,
  providerName: String,
  date: String
});

const isImage = computed(() => {
  if (!props.icon) return false;
  const i = props.icon.trim();
  // If it's a raw SVG string, it's not an "image" URL
  if (i.startsWith('<svg')) return false;
  
  const low = i.toLowerCase();
  return low.startsWith('http') || low.startsWith('https') || low.startsWith('asset') || low.includes('/') || low.includes('\\') || low.startsWith('data:image');
});
</script>

<template>
  <div class="assistant-header">
    <div class="header-left">
      <div class="avatar-icon" :class="{ 'is-image': isImage }">
        <template v-if="isImage">
          <img :src="icon" class="avatar-image-actual" />
        </template>
        <template v-else>
          <div v-html="icon"></div>
        </template>
      </div>
      <span class="model-name-text">{{ modelName }}</span>
      <div class="divider" v-if="providerName">|</div>
      <span class="model-name-text" style="opacity: 0.7;" v-if="providerName">{{ providerName }}</span>
    </div>
    <span class="time-text">{{ date }}</span>
  </div>
</template>

<style scoped>
.assistant-header {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 8px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.avatar-icon {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-color-white);
  flex-shrink: 0;
}

.avatar-icon.is-image {
  width: 40px;
  height: 40px;
  border-radius: 8px;
}
.avatar-icon :deep(svg) { width: 100%; height: 100%; }

.avatar-image-actual {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.model-name-text {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-assistant-name);
  letter-spacing: 0.3px;
}

.divider {
  color: var(--text-color);
  opacity: 0.25;
  font-size: 14px;
  margin: 0 4px;
}

.time-text {
  font-size: 12px;
  color: #7b7d81;
  margin-left: 2px;
  font-family: 'Roboto Mono', monospace;
}
</style>
