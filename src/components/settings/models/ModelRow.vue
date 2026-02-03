<script setup lang="ts">
import { 
  VISION_SVG, 
  GLOBE_SVG, 
  TOOLS_SVG,
  SETTINGS_SVG
} from '../../../constants/icons';
import { ModelInfo } from '../../../types/config';
import { getProviderIcon } from '../../../assets/icons';

const props = defineProps<{
  model: ModelInfo;
  providerIcon: string;
}>();

const emit = defineEmits<{
  (e: 'edit', model: ModelInfo): void;
  (e: 'remove', id: string): void;
}>();

const BRAIN_SVG_SMALL = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A5 5 0 0 0 8 8c0 1.3.5 2.6 1.5 3.5.8.8 1.3 1.5 1.5 2.5"></path><line x1="9" y1="18" x2="15" y2="18"></line><line x1="10" y1="22" x2="14" y2="22"></line></svg>`;

</script>

<template>
  <div class="model-row-v3">
    <div class="model-main-v3">
      <div class="model-avatar-v3">
        <span class="provider-icon-inner" v-html="getProviderIcon(providerIcon)"></span>
      </div>
      <span class="model-name-v3">{{ model.name || model.id }}</span>
      
      <!-- Feature Tags -->
      <div v-if="model.features" class="feature-tags-v3">
        <span v-if="model.features.includes('vision')" class="feat-tag vision" v-html="VISION_SVG" title="视觉识别"></span>
        <span v-if="model.features.includes('web')" class="feat-tag web" v-html="GLOBE_SVG" title="联网搜索"></span>
        <span v-if="model.features.includes('reasoning')" class="feat-tag reasoning" v-html="BRAIN_SVG_SMALL" title="深度思考"></span>
        <span v-if="model.features.includes('tools')" class="feat-tag tools" v-html="TOOLS_SVG" title="工具调用"></span>
      </div>
    </div>

    <div class="model-actions-v3">
      <button class="action-btn-v3" @click="emit('edit', model)" v-html="SETTINGS_SVG" title="编辑模型"></button>
      <button class="action-btn-v3 remove" @click="emit('remove', model.id)" title="移除模型">
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.model-row-v3 {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    transition: background 0.2s;
    border-bottom: 1px solid var(--border-glass-dim, var(--glass-white-03));
}
.model-row-v3:last-child { border-bottom: none; }
.model-row-v3:hover { background: var(--bg-glass-hover); }

.model-main-v3 {
    display: flex;
    align-items: center;
    gap: 14px;
    flex: 1;
}

.model-avatar-v3 {
    width: 32px;
    height: 32px;
    background: var(--bg-model-icon, var(--bg-main));
    border: 1px solid var(--border-glass);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.provider-icon-inner {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    color: var(--color-model-icon, var(--color-primary));
}

.provider-icon-inner :deep(svg) {
    width: 100%;
    height: 100%;
}

.model-name-v3 {
    font-size: 14px;
    color: var(--text-color-white);
    font-weight: 500;
}

.feature-tags-v3 {
    display: flex;
    gap: 8px;
    margin-left: 12px;
}

.feat-tag {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    opacity: 0.8;
}

.feat-tag.vision { background: var(--color-success-alpha-10, rgba(0,255,100,0.1)); color: var(--color-success); }
.feat-tag.web { background: var(--color-primary-alpha-10, rgba(0,100,255,0.1)); color: var(--color-primary); }
.feat-tag.reasoning { background: var(--color-reasoning-bg); color: var(--color-reasoning); }
.feat-tag.tools { background: var(--color-warning-alpha-10, rgba(255,150,0,0.1)); color: var(--color-warning); }

.feat-tag :deep(svg) { width: 14px; height: 14px; }

.model-actions-v3 {
    display: flex;
    gap: 8px;
    align-items: center;
}

.action-btn-v3 {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 6px;
    border-radius: 6px;
    display: flex;
    transition: all 0.2s;
}
.action-btn-v3:hover { background: var(--bg-glass-active); color: var(--text-color-white); }
.action-btn-v3.remove:hover { color: var(--color-danger); background: var(--color-danger-alpha-10); }

.action-btn-v3 :deep(svg) { width: 14px; height: 14px; }
</style>
