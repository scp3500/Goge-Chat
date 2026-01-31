<script setup>
import { computed } from 'vue';
import SearchSources from '../SearchSources.vue';
import ReasoningBlock from './ReasoningBlock.vue';

const props = defineProps({
  message: Object,
  renderedContent: String,
  isReasoningExpanded: Boolean,
  isChatMode: { type: Boolean, default: false }
});

const emit = defineEmits(['toggle-reasoning', 'link-click']);

const searchResults = computed(() => {
  if (!props.message.searchMetadata) return [];
  try {
    return JSON.parse(props.message.searchMetadata);
  } catch (e) {
    return [];
  }
});
</script>

<template>
  <div class="assistant-bubble-content" :class="{ 'standard-mode': !isChatMode }">
    <template v-if="message.content === '__LOADING__' && !message.reasoningContent">
      <div class="typing-indicator"><span></span><span></span><span></span></div>
    </template>
    <template v-else>
      <ReasoningBlock 
        v-if="message.reasoningContent"
        :content="message.reasoningContent"
        :is-loading="message.content === '__LOADING__'"
        :is-expanded="isReasoningExpanded"
        @toggle="$emit('toggle-reasoning')"
      />

      <!-- 搜索结果显示 -->
      <SearchSources 
        v-if="message.searchStatus || searchResults.length > 0"
        :results="searchResults"
        :status="message.searchStatus || 'done'"
        :query="message.searchQuery"
      />

      <div v-if="message.content !== '__LOADING__'" v-html="renderedContent" class="markdown-body" @click="$emit('link-click', $event)"></div>
      <div v-else-if="message.reasoningContent" class="typing-indicator small"><span></span><span></span><span></span></div>
    </template>
  </div>
</template>

<style scoped>

.assistant-bubble-content {
  background: var(--bg-assistant-bubble);
  border: 1px solid var(--border-assistant-bubble);
  padding: 6px 12px;
  width: fit-content;
  max-width: 90%;
}

.assistant-bubble-content.standard-mode {
  width: 100%;
  max-width: 100%;
  background: transparent;
  border: none;
  padding: 0;
}

.markdown-body { 
  font-size: 15px; 
  line-height: 1.6; 
  color: var(--text-color-white); 
  display: inline-block;
  width: fit-content;
  max-width: 100%;
}

.standard-mode .markdown-body {
  display: block;
  width: 100%;
}

.markdown-body :deep(p) { 
  margin: 0 !important; 
  display: inline; /* Prevent p from stretching bubble */
}

.standard-mode .markdown-body :deep(p) {
  display: block;
  margin-bottom: 1.2em !important;
}
.markdown-body :deep(*:last-child) { margin-bottom: 0 !important; }
.markdown-body :deep(*:first-child) { margin-top: 0 !important; }

/* Modern Typing Indicator - Minimal & Clean */
.typing-indicator {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 8px 0;
  background: transparent;
  border: none;
  border-radius: 0;
  width: fit-content;
  margin-top: 2px;
  box-shadow: none;
  backdrop-filter: none;
}

.typing-indicator.small {
  padding: 8px 0;
  margin: 0;
}

.typing-indicator span {
  width: 6px;
  height: 6px;
  background: var(--text-color);
  opacity: 0.5;
  border-radius: 50%;
  animation: liquid-flow 1.4s ease-in-out infinite both;
  box-shadow: none;
}

.typing-indicator span:nth-child(1) { animation-delay: -0.32s; }
.typing-indicator span:nth-child(2) { animation-delay: -0.16s; }
.typing-indicator span:nth-child(3) { animation-delay: 0s; }

@keyframes liquid-flow {
  0%, 100% {
    transform: translateY(0);
    opacity: 0.4;
  }
  50% {
    transform: translateY(-4px);
    opacity: 1;
    background: var(--text-color-white);
  }
}
</style>
