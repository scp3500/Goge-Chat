<script setup lang="ts">
import { ref } from 'vue';

interface SearchResult {
  title: string;
  url: string;
  snippet: string;
}

const props = defineProps<{
  results: SearchResult[];
  status: 'searching' | 'done' | 'error';
  query?: string;
}>();

const isExpanded = ref(false);

const toggleExpand = () => {
  isExpanded.value = !isExpanded.value;
};

const openLink = (url: string) => {
  window.open(url, '_blank');
};

const getFavicon = (url: string) => {
  try {
    const domain = new URL(url).hostname;
    return `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
  } catch (e) {
    return '';
  }
};
</script>

<template>
  <div class="search-sources-container">
    <!-- 正在搜索状态 -->
    <div v-if="status === 'searching'" class="searching-indicator">
      <div class="pulse-dot"></div>
      <span class="status-text">正在深入搜索: {{ query }}...</span>
    </div>

    <!-- 搜索完成结果 -->
    <div v-else-if="status === 'done' && results.length > 0" class="results-card" :class="{ 'is-expanded': isExpanded }">
      <div class="card-header" @click="toggleExpand">
        <div class="header-left">
          <div class="results-icon">
            <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M21 21L15 15M17 10C17 13.866 13.866 17 10 17C6.13401 17 3 13.866 3 10C3 6.13401 6.13401 3 10 3C13.866 3 17 6.13401 17 10Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <span class="results-count">已找到 {{ results.length }} 个参考来源</span>
        </div>
        <div class="header-right">
          <span class="expand-label">{{ isExpanded ? '收起' : '查看来源' }}</span>
          <div class="chevron-icon" :class="{ 'rotated': isExpanded }">
            <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M6 9L12 15L18 9" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
        </div>
      </div>

      <Transition name="expand">
        <div v-if="isExpanded" class="results-list-wrapper">
          <div class="results-list modern-scroll">
            <div 
              v-for="(res, index) in results" 
              :key="index" 
              class="result-item"
              @click="openLink(res.url)"
            >
              <div class="result-top">
                <img v-if="getFavicon(res.url)" :src="getFavicon(res.url)" class="favicon" alt="" />
                <span class="result-title">{{ res.title }}</span>
                <div class="external-icon">
                  <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M18 13V19C18 19.5304 17.7893 20.0391 17.4142 20.4142C17.0391 20.7893 16.5304 21 16 21H5C4.46957 21 3.96086 20.7893 3.58579 20.4142C3.21071 20.0391 3 19.5304 3 19V8C3 7.46957 3.21071 6.96086 3.58579 6.58579C3.96086 6.21071 4.46957 6 5 6H11M15 3H21M21 3V9M21 3L10 14" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </div>
              </div>
              <p class="result-snippet">{{ res.snippet }}</p>
              <span class="result-url">{{ res.url }}</span>
            </div>
          </div>
        </div>
      </Transition>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="status === 'error'" class="error-indicator">
      <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" class="error-icon">
        <path d="M12 9V11M12 15H12.01M5.07183 19H18.9282C20.4678 19 21.4301 17.3333 20.6603 16L13.7321 4C12.9623 2.66667 11.0378 2.66667 10.268 4L3.33975 16C2.56995 17.3333 3.53224 19 5.07183 19Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      <span>搜索暂时不可用</span>
    </div>
  </div>
</template>

<style scoped>
.search-sources-container {
  margin-bottom: 12px;
  width: 100%;
}

.searching-indicator {
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--bg-glass-active);
  border-radius: 12px;
  border: 1px solid var(--border-glass-bright);
  width: fit-content;
}

.pulse-dot {
  width: 8px;
  height: 8px;
  background-color: var(--color-primary);
  border-radius: 50%;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0% { transform: scale(0.95); box-shadow: 0 0 0 0 var(--color-primary-alpha-70); }
  70% { transform: scale(1); box-shadow: 0 0 0 6px rgba(129, 140, 248, 0); }
  100% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(129, 140, 248, 0); }
}

.status-text {
  font-size: 13px;
  color: var(--color-primary);
  font-weight: 500;
}

.results-card {
  background: var(--bg-glass);
  border: 1px solid var(--border-glass);
  border-radius: 14px;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  max-width: 400px;
}

.results-card:hover {
  background: var(--bg-glass-hover);
  border-color: var(--border-glass-bright);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  cursor: pointer;
}

.header-left, .header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.results-icon {
  color: var(--color-primary);
  display: flex;
}

.results-icon svg {
  width: 16px;
  height: 16px;
}

.results-count {
  font-size: 13px;
  color: #e3e3e3;
  font-weight: 500;
}

.expand-label {
  font-size: 11px;
  color: #80868b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.chevron-icon {
  color: #80868b;
  display: flex;
  transition: transform 0.3s ease;
}

.chevron-icon.rotated {
  transform: rotate(180deg);
}

.chevron-icon svg {
  width: 16px;
  height: 16px;
}

.results-list-wrapper {
  border-top: 1px solid var(--border-glass);
}

.results-list {
  max-height: 300px;
  overflow-y: auto;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.result-item {
  padding: 10px;
  background: var(--bg-glass);
  border-radius: 10px;
  cursor: pointer;
  transition: background 0.2s;
}

.result-item:hover {
  background: var(--bg-glass-hover);
}

.result-top {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.favicon {
  width: 14px;
  height: 14px;
  border-radius: 3px;
}

.result-title {
  flex: 1;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.external-icon {
  color: var(--text-tertiary);
}

.external-icon svg {
  width: 12px;
  height: 12px;
}

.result-snippet {
  font-size: 12px;
  color: #b0b0b0;
  line-height: 1.4;
  margin-bottom: 4px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.result-url {
  font-size: 10px;
  color: #606060;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}

.error-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 12px;
  color: var(--color-danger);
  background: var(--color-danger-alpha-10);
  border-radius: 8px;
}

.error-icon {
  width: 14px;
  height: 14px;
}

/* 展开动画 */
.expand-enter-active, .expand-leave-active {
  transition: all 0.3s ease;
  max-height: 400px;
}
.expand-enter-from, .expand-leave-to {
  max-height: 0;
  opacity: 0;
}

.modern-scroll::-webkit-scrollbar { width: 4px; }
.modern-scroll::-webkit-scrollbar-thumb { background: var(--bg-glass-active); border-radius: 10px; }
</style>
