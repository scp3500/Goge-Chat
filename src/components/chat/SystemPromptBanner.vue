<script setup>
import { ref, computed } from 'vue';
import { useConfigStore } from '../../stores/config';
import { BRAIN_SVG, CHEVRON_DOWN_SVG } from '../../constants/icons';
import { DEFAULT_SYSTEM_PROMPT } from '../../constants/prompts';

import { useChatStore } from '../../stores/chat';

const configStore = useConfigStore();
const chatStore = useChatStore();
const isExpanded = ref(false);

const currentSession = computed(() => chatStore.activeSession);
const activePreset = computed(() => {
  return configStore.settings.presets?.find(p => p.id === configStore.settings.defaultPresetId) || configStore.settings.presets?.[0];
});

const systemPrompt = computed(() => {
    return currentSession.value?.system_prompt || activePreset.value?.systemPrompt || configStore.settings.defaultSystemPrompt || DEFAULT_SYSTEM_PROMPT;
});

const promptName = computed(() => {
  // 1. 如果有会话特定提示词
  if (currentSession.value?.system_prompt) {
      const matched = configStore.settings.promptLibrary.find(p => p.content === currentSession.value.system_prompt);
      return matched ? matched.name : '会话特定';
  }

  // 2. 如果完全没有设置，显示默认
  if (!activePreset.value?.systemPrompt) return '默认';
  
  // 3. 在库中查找匹配的内容
  const matched = configStore.settings.promptLibrary.find(p => p.content === activePreset.value.systemPrompt);
  if (matched) return matched.name;
  
  // 4. 都不匹配显示自定义
  return '自定义提示词';
});

const toggleExpand = () => {
  isExpanded.value = !isExpanded.value;
};
</script>

<template>
  <div v-if="systemPrompt" class="system-prompt-minimal" :class="{ expanded: isExpanded }">
    <div class="prompt-header" @click="toggleExpand">
      <span class="status-icon" v-html="BRAIN_SVG"></span>
      <span class="status-text">系统提示词</span>
      <span :class="promptName === '默认' ? 'default-tag' : 'preset-tag'">{{ promptName }}</span>
      <span class="status-arrow" :class="{ 'is-expanded': isExpanded }">
        <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </span>
    </div>
    <Transition name="collapse">
      <div v-if="isExpanded" class="prompt-content">
        <div class="prompt-inner" :class="{ 'is-default': !activePreset?.systemPrompt }">{{ systemPrompt }}</div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.system-prompt-minimal {
  margin-bottom: 24px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  width: 100%;
}

.prompt-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 13px;
  border-radius: 6px;
  width: fit-content;
  transition: background-color 0.2s ease;
}

.prompt-header:hover {
  background-color: var(--bg-glass-hover);
}

.status-icon {
  width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  color: var(--color-primary);
  opacity: 0.8;
}

.status-icon :deep(svg) {
  width: 100%;
  height: 100%;
}

.status-text {
  font-weight: 500;
}

.preset-tag {
  font-size: 11px;
  background: var(--bg-glass-active);
  padding: 1px 6px;
  border-radius: 4px;
  color: var(--text-tertiary);
  margin: 0 4px;
}

.status-arrow {
  display: flex;
  align-items: center;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  opacity: 0.5;
}

.status-arrow.is-expanded {
  transform: rotate(180deg);
}

.default-tag {
  font-size: 10px;
  background: var(--bg-glass-active);
  color: var(--color-primary);
  padding: 0 4px;
  border-radius: 4px;
  margin: 0 4px;
}

.prompt-content {
  margin-top: 4px;
  padding-left: 12px;
  width: 100%;
  overflow: hidden;
}

.prompt-inner {
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-secondary);
  white-space: pre-wrap;
  padding: 8px 12px;
  border-left: 1px solid var(--border-glass-bright);
}

.prompt-inner.is-default {
  color: var(--text-tertiary);
  font-style: italic;
}

.collapse-enter-active, .collapse-leave-active {
  transition: all 0.3s ease-out;
  max-height: 800px;
}

.collapse-enter-from, .collapse-leave-to {
  max-height: 0;
  opacity: 0;
}
</style>
