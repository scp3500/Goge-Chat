<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useConfigStore } from '../../stores/config';
import { useUIStore } from '../../stores/ui';
import { 
  SEARCH_SVG, 
  VISION_SVG, 
  TOOLS_SVG, 
  BRAIN_SVG, 
  GLOBE_SVG, 
  CHEVRON_DOWN_SVG,
  CHECK_SVG,
  AI_EVO_SVG
} from '../../constants/icons';
import { getProviderIcon } from '../../assets/icons';

const props = defineProps({
  minimal: {
    type: Boolean,
    default: false
  },
  direction: {
    type: String,
    default: 'down' // 'up' or 'down'
  },
  fullWidth: {
    type: Boolean,
    default: false
  },
  menuId: {
    type: String,
    default: 'model-selector'
  }
});

const configStore = useConfigStore();
const uiStore = useUIStore();
const isOpen = computed(() => uiStore.isMenuOpen(props.menuId));
const searchQuery = ref('');
const activeFilter = ref('all');

// 切换下拉框
const toggleDropdown = () => {
  console.log(`🔄 ModelSelector(${props.menuId}): toggleDropdown`, !isOpen.value);
  uiStore.toggleMenu(props.menuId);
};

// 关闭下拉框
const closeDropdown = () => {
  uiStore.setActiveMenu(null);
};

// 点击外部关闭
const selectorRef = ref(null);
const handleClickOutside = (event) => {
  if (selectorRef.value && !selectorRef.value.contains(event.target)) {
    closeDropdown();
  }
};

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
});

// 获取当前选中的模型和提供商
const currentModel = computed(() => {
  const providers = configStore.settings.providers || [];
  const selectedId = configStore.settings.selectedModelId;
  
  if (!selectedId) return null;

  for (const provider of providers) {
    if (provider.models && provider.models.includes(selectedId)) {
      return {
        id: selectedId,
        name: selectedId,
        provider: provider
      };
    }
  }
  // 如果没找到，尝试返回默认提供商的模型
  const defaultProv = providers.find(p => p.id === configStore.settings.defaultProviderId);
  if (defaultProv && defaultProv.models && defaultProv.models.length > 0) {
    return {
      id: defaultProv.models[0],
      name: defaultProv.models[0],
      provider: defaultProv
    };
  }
  return null;
});

// 过滤后的提供商和模型列表
const filteredProviders = computed(() => {
  const query = searchQuery.value.toLowerCase();
  const filter = activeFilter.value;
  
  return configStore.enabledProviders.map(provider => {
    const matchedModels = provider.models.filter(model => {
      // 搜索匹配
      const matchesSearch = model.toLowerCase().includes(query);
      if (!matchesSearch) return false;
      
      // 过滤器匹配 (Mock 逻辑，因为目前没有模型元数据)
      if (filter === 'all') return true;
      if (filter === 'vision') return model.includes('vision') || model.includes('v');
      if (filter === 'reasoning') return model.includes('reasoner') || model.includes('reason');
      if (filter === 'tools') return false; // 暂时没有工具模型标识
      if (filter === 'web') return false;
      if (filter === 'free') return provider.id === 'ollama'; // Mock
      
      return true;
    });
    
    return {
      ...provider,
      matchedModels
    };
  }).filter(p => p.matchedModels.length > 0);
});

// 选择模型
const selectModel = (providerId, modelId) => {
  configStore.updateConfig({
    defaultProviderId: providerId,
    selectedModelId: modelId
  });
  closeDropdown();
};

// 判断是否为视觉模型
const isVisionModel = (model) => model.toLowerCase().includes('vision') || model.toLowerCase().includes('-v');
// 判断是否为推理模型
const isReasoningModel = (model) => model.toLowerCase().includes('reasoner') || model.toLowerCase().includes('reason');

</script>

<template>
  <div class="model-selector" :class="{ 'full-width': fullWidth }" ref="selectorRef">
    <!-- 选择器按钮 -->
    <button 
      class="selector-btn" 
      :class="{ 'active': isOpen, 'minimal-mode': minimal }"
      @click.stop="toggleDropdown" 
      @mousedown.stop 
      :title="minimal ? (currentModel?.id || '选择模型') : ''"
    >
      <span class="provider-icon">
        <template v-if="minimal">
          <span v-html="AI_EVO_SVG" class="ai-logo-white"></span>
        </template>
        <template v-else>
          <span v-html="getProviderIcon(currentModel?.provider?.icon || 'default')" class="provider-icon-inner"></span>
        </template>
      </span>
      <span v-if="!minimal" class="model-name">{{ currentModel?.id || '选择模型' }}</span>
      <span class="chevron" v-html="CHEVRON_DOWN_SVG"></span>
    </button>

    <!-- 下拉面板 -->
    <Transition name="fade-slide">
      <div v-if="isOpen" class="dropdown-panel" :class="[direction === 'up' ? 'pop-up' : 'pop-down']" @mousedown.stop>
        <!-- 搜索栏 -->
        <div class="search-header">
          <div class="search-box">
            <span class="search-icon" v-html="SEARCH_SVG"></span>
            <input 
              v-model="searchQuery" 
              type="text" 
              placeholder="搜索模型..." 
              autoFocus
              @click.stop
            />
          </div>
        </div>

        <!-- 过滤器列表 -->
        <div class="filters-container">
          <span class="filter-label">标签</span>
          <div class="filters-bar">
            <button 
              class="filter-chip" 
              :class="{ active: activeFilter === 'all' }"
              @click="activeFilter = 'all'"
            >全部</button>
            <button 
              class="filter-chip" 
              :class="{ active: activeFilter === 'vision' }"
              @click="activeFilter = 'vision'"
            >
              <span v-html="VISION_SVG"></span>
            </button>
            <button 
              class="filter-chip" 
              :class="{ active: activeFilter === 'reasoning' }"
              @click="activeFilter = 'reasoning'"
            >
              <span v-html="BRAIN_SVG"></span>
            </button>
            <button 
              class="filter-chip" 
              :class="{ active: activeFilter === 'free' }"
              @click="activeFilter = 'free'"
            >免费</button>
          </div>
        </div>

        <!-- 模型列表 -->
        <div class="models-list custom-scrollbar">
          <div v-for="provider in filteredProviders" :key="provider.id" class="provider-group">
            <div class="provider-label">{{ provider.name }}</div>
            
            <div 
              v-for="model in provider.matchedModels" 
              :key="model"
              class="model-item"
              :class="{ 'selected': configStore.settings.selectedModelId === model && configStore.settings.defaultProviderId === provider.id }"
              @click="selectModel(provider.id, model)"
            >
              <div class="model-info">
                <span v-html="getProviderIcon(provider.icon)" class="model-icon"></span>
                <span class="model-text">{{ model }}</span>
              </div>
              <div class="model-badges">
                <span v-if="isVisionModel(model)" class="badge vision" v-html="VISION_SVG" title="支持视觉"></span>
                <span v-if="isReasoningModel(model)" class="badge reasoning" v-html="BRAIN_SVG" title="支持推理"></span>
                <span v-if="configStore.settings.selectedModelId === model && configStore.settings.defaultProviderId === provider.id" class="badge check" v-html="CHECK_SVG"></span>
              </div>
            </div>
          </div>
          
          <div v-if="filteredProviders.length === 0" class="no-results">
            没有找到匹配的模型
          </div>
        </div>

        <!-- 底部提示 -->
        <div class="menu-hint">
          <span>ESC 关闭</span>
          <span>↵ 确认</span>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.model-selector {
  position: relative;
  display: inline-block;
  -webkit-app-region: no-drag;
}

/* 当作为对话框全宽嵌入组件时，取消相对定位，使面板相对于 input-wrapper 对齐 */
.model-selector.full-width {
  position: static;
}

.selector-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background: transparent;
  border: none;
  border-radius: 99px;
  padding: 4px 16px; /* 稍微增加内边距 */
  color: #a1a1a1; /* 稍微调亮一点基础颜色 */
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  height: 28px; /* 稍微变矮一点点更精致 */
}

.selector-btn:hover {
  background: rgba(255, 255, 255, 0.08); /* 现代透明悬浮感 */
  color: #fff;
}

.selector-btn.active {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
}

/* 简约小图标模式 */
.selector-btn.minimal-mode {
  padding: 0;
  margin: 0;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0;
  background: transparent;
  opacity: 0.6;
  line-height: 0;
}

.selector-btn.minimal-mode:hover {
  background: rgba(255, 255, 255, 0.1);
  opacity: 1;
}

.selector-btn.minimal-mode.active {
  background: rgba(255, 255, 255, 0.15);
  opacity: 1;
  color: #fff;
}

/* 极简模式去掉箭头 */
.selector-btn.minimal-mode .chevron {
  display: none; 
}

.selector-btn.minimal-mode .provider-icon {
  font-size: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ai-logo-white {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}

.ai-logo-white :deep(svg) {
  width: 18px;
  height: 18px;
}


.provider-icon {
  font-size: 14px;
}

.model-name {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.chevron {
  display: flex;
  align-items: center;
  transition: transform 0.2s;
  opacity: 0.6;
}

.selector-btn.active .chevron {
  transform: rotate(180deg);
}

/* 下拉面板 */
.dropdown-panel {
  position: absolute;
  left: 0;
  width: 320px;
  background: rgba(24, 24, 26, 0.98);
  backdrop-filter: blur(40px) saturate(200%);
  -webkit-backdrop-filter: blur(40px) saturate(200%);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 20px;
  box-shadow: 0 -10px 40px rgba(0, 0, 0, 0.5);
  z-index: 1000;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 6px;
}

.dropdown-panel.pop-down {
  top: calc(100% + 12px);
}

.dropdown-panel.pop-up {
  bottom: calc(100% - 1px);
  left: 0 !important;
  right: 0 !important;
  margin: 0 auto !important;
  width: 92% !important;
  border-bottom: none;
  border-radius: 20px 20px 0 0;
  top: auto;
}

.full-width .dropdown-panel.pop-up {
  left: 0;
  right: 0;
  width: 100%;
}

.search-header {
  padding: 10px 10px 6px;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 10px;
  padding: 8px 12px;
  transition: all 0.2s;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.search-box:focus-within {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.15);
}


.search-icon {
  color: #888;
  display: flex;
}

.search-box input {
  background: transparent;
  border: none;
  color: #eee;
  font-size: 13px;
  width: 100%;
  outline: none;
}

/* 过滤器 */
.filters-container {
  padding: 8px 10px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.filter-label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.3);
  white-space: nowrap;
}

.filters-bar {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  scrollbar-width: none;
}

.filters-bar::-webkit-scrollbar { display: none; }

.filter-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.04);
  border-radius: 8px;
  padding: 4px 10px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}

.filter-chip:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ccc;
}

.filter-chip.active {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
  border-color: rgba(255, 255, 255, 0.15);
}


.filter-chip span {
  display: flex;
  align-items: center;
}

/* 模型列表 */
.models-list {
  max-height: 400px;
  overflow-y: auto;
  padding: 4px 0;
}

.provider-group {
  margin-bottom: 8px;
}

.provider-label {
  padding: 8px 12px 4px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.3);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.8px;
}

.model-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  cursor: pointer;
  transition: all 0.15s;
  position: relative;
  border-radius: 8px;
  margin: 0 4px;
}

.model-item:hover {
  background: rgba(255, 255, 255, 0.08);
}

.model-item.selected {
  background: rgba(74, 222, 128, 0.1);
  border: 1px solid rgba(74, 222, 128, 0.2);
}

.model-item.selected .model-text {
  color: #4ade80;
}

.model-item.selected .badge.check {
  color: #4ade80;
}

.model-item.selected::before {
  display: none; /* 移除之前的横条 */
}


.model-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.model-icon {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(220, 220, 225, 0.85);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
  color: #1a1a1b;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1), inset 0 1px 0.5px rgba(255, 255, 255, 0.15);
}

.model-item:hover .model-icon {
  transform: translateY(-1px);
  background: rgba(230, 230, 235, 0.95);
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.15);
}

.model-item.selected .model-icon {
  background: #f0f0f5;
  border-color: #4ade80;
  box-shadow: 0 0 0 2px rgba(74, 222, 128, 0.2), 0 2px 6px rgba(0, 0, 0, 0.15);
}

.model-icon :deep(svg) {
  width: 18px;
  height: 18px;
  object-fit: contain;
}

.provider-icon-inner {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.provider-icon-inner :deep(svg) {
  width: 14px;
  height: 14px;
}

.model-text {
  font-size: 13px;
  color: #ccc;
}

.model-item.selected .model-text {
  color: #fff;
  font-weight: 500;
}

.model-badges {
  display: flex;
  align-items: center;
  gap: 6px;
}

.badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 4px;
  opacity: 0.6;
}

.badge.vision { color: #a18cd1; }
.badge.reasoning { color: #fbc2eb; }
.badge.check { opacity: 1; color: #4facfe; }

/* 滚动条 */
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}

.no-results {
  padding: 20px;
  text-align: center;
  color: #555;
  font-size: 13px;
}

.menu-hint {
  display: flex;
  justify-content: center;
  gap: 16px;
  padding: 10px 12px;
  color: rgba(255, 255, 255, 0.2);
  font-size: 11px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  margin-top: 8px;
}

/* 动画 */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.fade-slide-enter-from,
.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.98);
}

.dropdown-panel.pop-up.fade-slide-enter-from,
.dropdown-panel.pop-up.fade-slide-leave-to {
  transform: translateY(0) scale(0.98);
}
</style>
