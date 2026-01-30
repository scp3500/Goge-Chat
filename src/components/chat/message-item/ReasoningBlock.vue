<script setup>
import { BRAIN_SVG } from '../../../constants/icons.ts';

defineProps({
  content: String,
  isLoading: Boolean,
  isExpanded: Boolean
});

defineEmits(['toggle']);
</script>

<template>
  <div class="reasoning-container">
    <div class="reasoning-status" @click="$emit('toggle')">
      <span class="status-icon" v-html="BRAIN_SVG"></span>
      <span class="status-text">{{ isLoading ? '正在思考...' : '思考过程' }}</span>
      <span class="status-arrow" :class="{ 'is-expanded': isExpanded }">
        <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </span>
    </div>
    <Transition name="collapse">
      <div v-if="isExpanded" class="reasoning-content">
        <div class="reasoning-inner">{{ content }}</div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.reasoning-container { margin-bottom: 16px; display: flex; flex-direction: column; }
.reasoning-status { display: flex; align-items: center; gap: 6px; padding: 4px 8px; cursor: pointer; color: var(--text-color); opacity: 0.6; font-size: 13px; border-radius: 6px; width: fit-content; }
.status-icon { width: 14px; height: 14px; display: flex; align-items: center; color: var(--text-color-white); opacity: 0.8; }
.status-arrow { transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
.status-arrow.is-expanded { transform: rotate(180deg); }
.reasoning-content { margin-top: 4px; padding-left: 12px; position: relative; overflow: hidden; }
.reasoning-inner { font-size: 14px; line-height: 1.6; color: var(--text-color); opacity: 0.6; white-space: pre-wrap; padding: 4px 0 8px 0; }

.collapse-enter-active, .collapse-leave-active { transition: all 0.3s ease-out; max-height: 500px; }
.collapse-enter-from, .collapse-leave-to { max-height: 0; opacity: 0; }
</style>
