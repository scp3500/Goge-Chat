<script setup>
defineProps({
  icon: String,
  modelName: String,
  date: String
});
</script>

<template>
  <div class="assistant-header">
    <div class="header-left">
      <div class="avatar-icon" v-html="icon"></div>
      <span class="model-name-text">{{ modelName }}</span>
      <div class="divider">|</div>
      <span class="model-name-text" style="opacity: 0.7;">New API</span>
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
}
.avatar-icon :deep(svg) { width: 100%; height: 100%; }

.model-name-text {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-color-white);
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
