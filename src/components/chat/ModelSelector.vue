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

  // 🟢 Fix: Prioritize the currently selected provider (defaultProviderId)
  // This ensures that if multiple providers share the same model ID (e.g. 'gemini-1.5-flash'),
  // we display the icon/info for the *active* provider, not just the first one found.
  const activeProviderId = configStore.settings.defaultProviderId;
  if (activeProviderId) {
    const activeProvider = providers.find(p => p.id === activeProviderId);
    if (activeProvider) {
      const models = activeProvider.models || [];
      const foundInActive = models.find(m => {
          const id = typeof m === 'string' ? m : m.id;
          return id === selectedId;
      });
      
      if (foundInActive) {
        return {
          id: selectedId,
          name: typeof foundInActive === 'string' ? foundInActive : (foundInActive.name || selectedId),
          provider: activeProvider,
          data: typeof foundInActive === 'string' ? null : foundInActive
        };
      }
    }
  }

  // Fallback: search all providers if not found in active provider
  for (const provider of providers) {
    const models = provider.models || [];
    const foundModel = models.find(m => {
        const id = typeof m === 'string' ? m : m.id;
        return id === selectedId;
    });

    if (foundModel) {
      return {
        id: selectedId,
        name: typeof foundModel === 'string' ? foundModel : (foundModel.name || selectedId),
        provider: provider,
        data: typeof foundModel === 'string' ? null : foundModel
      };
    }
  }
  // 如果没找到，尝试返回默认提供商的模型
  const defaultProv = providers.find(p => p.id === configStore.settings.defaultProviderId);
  if (defaultProv && defaultProv.models && defaultProv.models.length > 0) {
    const firstModel = defaultProv.models[0];
    const id = typeof firstModel === 'string' ? firstModel : firstModel.id;
    return {
      id: id,
      name: typeof firstModel === 'string' ? firstModel : (firstModel.name || id),
      provider: defaultProv,
      data: typeof firstModel === 'string' ? null : firstModel
    };
  }
  return null;
});

// 过滤后的提供商和模型列表
const filteredProviders = computed(() => {
  const query = searchQuery.value.toLowerCase();
  const filter = activeFilter.value;
  
  return configStore.enabledProviders.map(provider => {
    const matchedModels = (provider.models || []).filter(m => {
      const modelId = typeof m === 'string' ? m : m.id;
      const modelName = typeof m === 'string' ? m : (m.name || m.id);
      const modelFeatures = typeof m === 'string' ? [] : (m.features || []);

      // 搜索匹配
      const matchesSearch = modelId.toLowerCase().includes(query) || modelName.toLowerCase().includes(query);
      if (!matchesSearch) return false;
      
      // 过滤器匹配
      if (filter === 'all') return true;
      if (filter === 'vision') return modelFeatures.includes('vision') || modelId.includes('vision') || modelId.includes('-v');
      if (filter === 'reasoning') return modelFeatures.includes('reasoning') || modelId.includes('reasoner') || modelId.includes('reason');
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
const selectModel = (providerId, model) => {
  const modelId = typeof model === 'string' ? model : model.id;
  configStore.updateConfig({
    defaultProviderId: providerId,
    selectedModelId: modelId
  });
  closeDropdown();
};

// 判断是否为视觉模型
const isVisionModel = (model) => {
    if (typeof model !== 'string' && model.features?.includes('vision')) return true;
    const modelId = typeof model === 'string' ? model : model.id;
    return modelId.toLowerCase().includes('vision') || modelId.toLowerCase().includes('-v');
};
// 判断是否为推理模型
const isReasoningModel = (model) => {
    if (typeof model !== 'string' && model.features?.includes('reasoning')) return true;
    const modelId = typeof model === 'string' ? model : model.id;
    return modelId.toLowerCase().includes('reasoner') || modelId.toLowerCase().includes('reason');
};

const getPanelStyle = computed(() => {
    if (!selectorRef.value) return {};
    const rect = selectorRef.value.getBoundingClientRect();
    
    if (props.fullWidth && props.direction === 'up') {
        const inputWrapper = selectorRef.value.closest('.input-wrapper');
        if (inputWrapper) {
            const wrapRect = inputWrapper.getBoundingClientRect();
            return {
                position: 'fixed',
                bottom: (window.innerHeight - wrapRect.top) + 'px',
                left: wrapRect.left + 'px',
                width: wrapRect.width + 'px',
                borderRadius: '20px 20px 0 0'
            };
        }
    }

    return {
        position: 'fixed',
        top: props.direction === 'down' ? (rect.bottom + 12) + 'px' : 'auto',
        bottom: props.direction === 'up' ? (window.innerHeight - rect.top + 1) + 'px' : 'auto',
        left: rect.left + 'px'
    };
});

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
    <Teleport to="body">
      <Transition name="fade-slide">
        <div v-if="isOpen" class="dropdown-panel-global" :class="[direction === 'up' ? 'pop-up' : 'pop-down']" :style="getPanelStyle" @mousedown.stop>
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
                :key="typeof model === 'string' ? model : model.id"
                class="model-item"
                :class="{ 'selected': configStore.settings.selectedModelId === (typeof model === 'string' ? model : model.id) && configStore.settings.defaultProviderId === provider.id }"
                @click="selectModel(provider.id, model)"
              >
                <div class="model-info">
                  <span v-html="getProviderIcon(provider.icon)" class="model-icon"></span>
                  <span class="model-text">{{ typeof model === 'string' ? model : (model.name || model.id) }}</span>
                </div>
                <div class="model-badges">
                  <span v-if="isVisionModel(model)" class="badge vision" v-html="VISION_SVG" title="支持视觉"></span>
                  <span v-if="isReasoningModel(model)" class="badge reasoning" v-html="BRAIN_SVG" title="支持推理"></span>
                  <span v-if="configStore.settings.selectedModelId === (typeof model === 'string' ? model : model.id) && configStore.settings.defaultProviderId === provider.id" class="badge check" v-html="CHECK_SVG"></span>
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
    </Teleport>
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
  border-radius: 99px;
  padding: 4px 16px; /* 稍微增加内边距 */
  color: var(--text-color); /* 稍微调亮一点基础颜色 */
  opacity: 0.8;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  height: 28px; /* 稍微变矮一点点更精致 */
}

.selector-btn:hover {
  background: var(--bg-glass-hover); /* 现代透明悬浮感 */
  color: var(--text-color-white);
  opacity: 1;
}

.selector-btn.active {
  background: var(--bg-glass-active);
  color: var(--text-color-white);
  opacity: 1;
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
  background: var(--bg-glass-hover);
  opacity: 1;
}

.selector-btn.minimal-mode.active {
  background: var(--bg-glass-active);
  opacity: 1;
  color: var(--text-color-white);
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
  color: var(--text-color-white);
}

.ai-logo-white {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-color-white);
}

.ai-logo-white :deep(svg) {
  width: 18px;
  height: 18px;
}


.provider-icon {
  font-size: 14px;
  color: var(--color-header-icon);
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
.dropdown-panel-global {
  position: fixed;
  width: 320px;
  background: var(--bg-dropdown);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid var(--border-dropdown);
  border-radius: 14px;
  box-shadow: 
    0 10px 30px -5px rgba(0, 0, 0, 0.3),
    inset 0 1px 1px rgba(255, 255, 255, 0.1);
  z-index: 100000;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 8px;
  background-clip: padding-box;
}

.dropdown-panel-global.pop-up {
  border-bottom: none;
  box-shadow: 
    0 -10px 30px -5px rgba(0, 0, 0, 0.3),
    inset 0 1px 1px rgba(255, 255, 255, 0.1);
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
  background: var(--bg-input);
  border-radius: 10px;
  padding: 8px 12px;
  transition: all 0.2s;
  border: 1px solid var(--border-glass-bright);
}

.search-box:focus-within {
  background: var(--bg-input-focus);
  border-color: var(--border-glass-bright);
}


.search-icon {
  color: var(--token-operator); /* 使用稍微淡一点的颜色 */
  display: flex;
}

.search-box input {
  background: transparent;
  border: none;
  color: var(--text-color-white);
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
  color: var(--text-dim);
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
  background: var(--bg-glass);
  border: 1px solid var(--border-glass);
  border-radius: 8px;
  padding: 4px 10px;
  color: var(--text-color);
  opacity: 0.6;
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}

.filter-chip:hover {
  background: var(--bg-glass-hover);
  color: var(--text-color-white);
  opacity: 1;
}

.filter-chip.active {
  background: var(--bg-glass-active);
  color: var(--text-color-white);
  border-color: var(--border-glass-bright);
  opacity: 1;
}


.filter-chip span {
  display: flex;
  align-items: center;
}

/* 模型列表 */
.models-list {
  max-height: 50vh; /* 使用视口高度百分比更灵活 */
  overflow-y: auto;
  padding: 4px 0;
}

.provider-group {
  margin-bottom: 8px;
}

.provider-label {
  padding: 8px 12px 4px;
  font-size: 11px;
  color: var(--text-dim);
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
  background: var(--bg-glass-hover);
}

.model-item.selected {
  background: var(--bg-menu-active);
  border: 1px solid var(--color-primary-border);
}

.model-item.selected .model-text {
  color: var(--color-menu-active);
}

.model-item.selected .badge.check {
  color: var(--color-menu-active);
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
  background: var(--bg-model-icon);
  border: 1px solid var(--border-glass);
  border-radius: 8px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
  color: var(--color-model-icon);
  box-shadow: var(--shadow-main);
}

.model-item:hover .model-icon {
  transform: translateY(-1px);
  background: var(--bg-model-icon-hover);
  box-shadow: 0 3px 8px var(--bg-mask);
}

.model-item.selected .model-icon {
  background: var(--bg-model-icon-active);
  border-color: var(--color-menu-active);
  box-shadow: 0 0 0 2px var(--color-primary-border), 0 2px 6px var(--bg-mask);
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
  color: var(--text-color);
  opacity: 0.8;
}

.model-item.selected .model-text {
  color: var(--text-color-white);
  font-weight: 500;
  opacity: 1;
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

.badge.vision { color: var(--badge-vision); }
.badge.reasoning { color: var(--badge-reasoning); }
.badge.check { opacity: 1; color: var(--badge-blue); }

/* 滚动条 */
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--bg-glass-active);
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: var(--bg-glass-hover);
}

.no-results {
  padding: 20px;
  text-align: center;
  color: var(--text-color);
  opacity: 0.6;
  font-size: 13px;
}

.menu-hint {
  display: flex;
  justify-content: center;
  gap: 16px;
  padding: 10px 12px;
  color: var(--text-color);
  opacity: 0.4;
  font-size: 11px;
  border-top: 1px solid var(--border-glass);
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
