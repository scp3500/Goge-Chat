<script setup>
import draggable from 'vuedraggable';
import { ref, computed, watch } from 'vue';
import { useProviderConfig } from '../../composables/useProviderConfig';
import { useConfigStore } from '../../stores/config';
import { 
  EYE_OPEN_SVG, 
  EYE_CLOSED_SVG, 
  SEARCH_SVG, 
  SETTINGS_SVG, 
  CHEVRON_DOWN_SVG,
  VISION_SVG,
  TOOLS_SVG,
  GLOBE_SVG,
  PLUS_SVG
} from '../../constants/icons.ts';

// 精确匹配原图图标
const TWEAK_ICON = `<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M4 21v-7m0-4V3m8 18v-9m0-4V3m8 18v-5m0-4V3M2 14h4m4-8h4m4 8h4"/></svg>`;
const ARROWS_ICON = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M7 15l5 5 5-5M7 9l5-5 5 5"/></svg>`;
const HELP_CIRCLE_ICON = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 015.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>`;
const MAGIC_STAR_ICON = `<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 3l1.8 5.7a1.5 1.5 0 001 1L21 12l-6.2 1.8a1.5 1.5 0 00-1 1L12 21l-1.8-5.7a1.5 1.5 0 00-1-1L3 12l6.2-1.8a1.5 1.5 0 001-1L12 3z"/></svg>`;
const LIST_MANAGE_ICON = `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>`;

const props = defineProps({
  providerId: String
});

const configStore = useConfigStore();
const { currentProvider, updateCurrentProvider } = useProviderConfig();

const showApiKey = ref(false);
const testingConnection = ref(false);

const providerConfig = computed(() => currentProvider.value);

// 本地模型列表状态，用于拖拽
const modelsList = ref([...(providerConfig.value?.models || [])]);

// 同步逻辑：仅在集合实质变化时更新本地列表，保持拖拽后的顺序稳定
watch(() => providerConfig.value?.models, (newVal) => {
  if (!newVal) {
    modelsList.value = [];
    return;
  }
  
  const localKeys = modelsList.value.join(',');
  const remoteKeys = newVal.join(',');
  
  if (localKeys === remoteKeys) return;
  
  const localSorted = [...modelsList.value].sort().join(',');
  const remoteSorted = [...newVal].sort().join(',');
  
  if (localSorted !== remoteSorted || modelsList.value.length !== newVal.length) {
    modelsList.value = [...newVal];
  }
}, { deep: true });

const handleModelsReorder = async () => {
  if (providerConfig.value) {
    await updateCurrentProvider({ models: [...modelsList.value] });
  }
};

// 模型能力配置
const getCapabilities = (model) => [
  { icon: '👁', bg: '#064e3b', color: '#10b981' },
  { icon: '🌐', bg: '#1e3a8a', color: '#3b82f6' },
  { icon: '☀️', bg: '#312e81', color: '#9d9ade' },
  { icon: '🔧', bg: '#451a03', color: '#f59e0b' }
];

const handleApiKeyChange = async () => {
  if (providerConfig.value) await updateCurrentProvider({ apiKey: providerConfig.value.apiKey });
};

const handleBaseUrlChange = async () => {
  if (providerConfig.value) await updateCurrentProvider({ baseUrl: providerConfig.value.baseUrl });
};

const toggleEnabled = async () => {
  if (providerConfig.value) await updateCurrentProvider({ enabled: !providerConfig.value.enabled });
};

// 后置保存逻辑：延迟保存防止输入频繁触发
let saveTimeout = null;
const debouncedUpdate = (config) => {
  if (saveTimeout) clearTimeout(saveTimeout);
  saveTimeout = setTimeout(async () => {
    await updateCurrentProvider(config);
  }, 500);
};

const handleParamChange = (key, value) => {
  debouncedUpdate({ [key]: value });
};
</script>

<template>
  <div class="config-pane-wrapper" v-if="providerConfig">
    <!-- Header -->
    <header class="top-nav">
      <div class="title-area">
        <span class="main-title">{{ providerConfig.name }}</span>
      </div>
      <label class="switch-green">
        <input type="checkbox" :checked="providerConfig.enabled" @change="toggleEnabled" />
        <span class="switch-blob"></span>
      </label>
    </header>

    <div class="thin-sep"></div>

    <!-- API Key Section -->
    <section class="pane-section">
      <div class="pane-header-row">
        <span class="pane-label">API 密钥</span>
        <button class="ghost-tool-btn" v-html="TWEAK_ICON"></button>
      </div>
      <div class="input-action-row">
        <div class="search-style-input">
          <input
            :type="showApiKey ? 'text' : 'password'"
            v-model="providerConfig.apiKey"
            placeholder="••••••••••••••••••••••••••••••••••••••••••••••••••"
            @change="handleApiKeyChange"
          />
          <button class="eye-toggle-btn" @click="showApiKey = !showApiKey" v-html="showApiKey ? EYE_OPEN_SVG : EYE_CLOSED_SVG"></button>
        </div>
        <button class="check-btn" :disabled="testingConnection" @click="testingConnection = true">
          {{ testingConnection ? '检测中' : '检测' }}
        </button>
      </div>
      <div class="pane-hint text-end">多个密钥使用逗号分隔</div>
    </section>

    <!-- API Base URL Section -->
    <section class="pane-section">
      <div class="pane-header-row">
        <div class="label-group">
          <span class="pane-label">API 地址</span>
          <span class="tiny-meta" v-html="ARROWS_ICON"></span>
          <span class="tiny-meta" v-html="HELP_CIRCLE_ICON"></span>
        </div>
        
        <!-- 自定义地址开关 -->
        <label class="custom-url-check">
          <input 
            type="checkbox" 
            v-model="providerConfig.disableUrlSuffix" 
            @change="handleParamChange('disableUrlSuffix', providerConfig.disableUrlSuffix)"
          />
          <span>自定义地址</span>
        </label>
      </div>
      <div class="search-style-input w-full-input">
        <input
          type="text"
          v-model="providerConfig.baseUrl"
          placeholder="https://ai.example.com"
          @change="handleBaseUrlChange"
        />
      </div>
      <div class="pane-hint">
        预览: {{ providerConfig.baseUrl || 'https://...' }}{{ 
          providerConfig.disableUrlSuffix ? '' : 
          (providerConfig.id === 'gemini' ? '/v1beta/models' : '/chat/completions') 
        }}
      </div>
    </section>

    <!-- Parameters Section (New) -->
    <section class="pane-section">
      <div class="pane-header-row">
        <span class="pane-label">参数设置</span>
        <button class="ghost-tool-btn" v-html="TWEAK_ICON"></button>
      </div>
      <div class="params-grid">
        <div class="param-item">
          <div class="param-top">
            <label>回复随机度 (Temperature)</label>
            <span>{{ providerConfig.temperature }}</span>
          </div>
          <input 
            type="range" 
            v-model.number="providerConfig.temperature" 
            min="0" 
            max="2" 
            step="0.1" 
            @input="handleParamChange('temperature', providerConfig.temperature)"
          />
        </div>
        <div class="param-item">
          <div class="param-top">
            <label>单次回复最大长度 (Max Tokens)</label>
            <span>{{ providerConfig.maxTokens }}</span>
          </div>
          <input 
            type="number" 
            v-model.number="providerConfig.maxTokens" 
            placeholder="默认"
            @change="handleParamChange('maxTokens', providerConfig.maxTokens)"
            class="param-number-input"
          />
        </div>
      </div>
    </section>

    <!-- Models Section -->
    <section class="pane-section">
      <div class="pane-header-row">
        <div class="label-group">
          <span class="pane-label">模型</span>
          <span class="mini-badge-v2">{{ providerConfig.models?.length || 0 }}</span>
          <button class="search-v2-btn" v-html="SEARCH_SVG"></button>
        </div>
        <button class="ghost-tool-btn" v-html="MAGIC_STAR_ICON"></button>
      </div>

      <div class="tree-box-modern">
        <div class="tree-inner-group">
          <div class="tree-header-v2">
            <span class="tree-chev-v2" v-html="CHEVRON_DOWN_SVG"></span>
            <span class="tree-title-v2">{{ providerId }}</span>
          </div>
          <div class="tree-items-v2">
            <draggable 
              v-model="modelsList" 
              item-key="self"
              @end="handleModelsReorder"
              animation="200"
              ghost-class="sortable-ghost"
              handle=".sparkle-avatar"
            >
              <template #item="{ element: model }">
                <div class="tree-row-v2 draggable-model">
                  <div class="row-info-v2">
                    <div class="sparkle-avatar">
                      <svg viewBox="0 0 24 24" width="20" height="20">
                        <defs>
                          <linearGradient :id="'fire-grad-' + model.replace(/[^a-zA-Z0-9]/g, '-')" x1="0" y1="0" x2="1" y2="1">
                            <stop offset="0%" stop-color="#fbbf24" />
                            <stop offset="100%" stop-color="#ea580c" />
                          </linearGradient>
                        </defs>
                        <path d="M12 2l2.3 7 7 2.3-7 2.3-2.3 7-2.3-7-7-2.3 7-2.3z" :fill="'url(#fire-grad-' + model.replace(/[^a-zA-Z0-9]/g, '-') + ')'" />
                      </svg>
                    </div>
                    <span class="row-name-v2">{{ model }}</span>
                    <div class="circles-group">
                      <div v-for="(cap, idx) in getCapabilities(model)" :key="idx" class="cap-dot-v2" :style="{ backgroundColor: cap.bg, color: cap.color }">
                        {{ cap.icon }}
                      </div>
                    </div>
                  </div>
                  <div class="row-actions-v2">
                    <button class="row-mini-btn" v-html="SETTINGS_SVG"></button>
                    <button class="row-mini-btn">➖</button>
                  </div>
                </div>
              </template>
            </draggable>
          </div>
        </div>
      </div>

      <div class="footer-btn-layout">
        <button class="manage-green-btn">
          <span class="btn-icon-wrap" v-html="LIST_MANAGE_ICON"></span>
          管理
        </button>
        <button class="add-dark-btn">
          <span class="btn-icon-wrap" v-html="PLUS_SVG"></span>
          添加
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.config-pane-wrapper {
  display: flex;
  flex-direction: column;
  gap: 22px;
  color: #d1d1d1;
}

/* Header */
.top-nav {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.title-area { display: flex; align-items: center; gap: 8px; }
.main-title { font-size: 18px; font-weight: 600; color: #fff; }

.ghost-tool-btn { background: transparent; border: none; padding: 4px; border-radius: 4px; color: #666; cursor: pointer; display: flex; width: 24px; height: 24px; }
.ghost-tool-btn:hover { background: rgba(255,255,255,0.1); color: #ccc; }
.thin-sep {
  height: 1px;
  background: rgba(255, 255, 255, 0.04);
  margin-top: -12px;
}

/* Green Switch */
.switch-green {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.switch-green input { opacity: 0; width: 0; height: 0; }

.switch-blob {
  position: absolute;
  cursor: pointer;
  top: 0; left: 0; right: 0; bottom: 0;
  background-color: #313338;
  transition: .3s;
  border-radius: 24px;
}

.switch-blob:before {
  position: absolute;
  content: "";
  height: 20px;
  width: 20px;
  left: 2px;
  bottom: 2px;
  background-color: #fff;
  transition: .3s;
  border-radius: 50%;
}

input:checked + .switch-blob { background-color: #23a559; }
input:checked + .switch-blob:before { transform: translateX(20px); }

/* Pane Sections */
.pane-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.pane-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.label-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pane-label {
  font-size: 14px;
  font-weight: 700;
  color: #fff;
  letter-spacing: 0.2px;
}

.tiny-meta {
  color: #777;
  display: flex;
}

.mini-badge-v2 {
  background: #2b2d31;
  padding: 1px 7px;
  border-radius: 12px;
  font-size: 11px;
  color: #777;
}

.search-v2-btn {
  background: transparent;
  border: none;
  color: #777;
  padding: 0;
  display: flex;
}

/* Inputs */
.input-action-row {
  display: flex;
  gap: 8px;
  height: 40px;
}

.search-style-input {
  flex: 1;
  background: #111214;
  border: 1px solid rgba(0, 0, 0, 0.2);
  border-radius: 6px;
  display: flex;
  align-items: center;
  padding: 0 12px;
}

.w-full-input { height: 42px; }

.search-style-input input {
  flex: 1;
  background: transparent;
  border: none;
  color: #fff;
  font-size: 14px;
  font-family: monospace;
  outline: none;
  height: 100%;
}

.eye-toggle-btn {
  background: transparent;
  border: none;
  color: #666;
  cursor: pointer;
}

.check-btn {
  background: #1e1f22;
  border: 1px solid rgba(255, 255, 255, 0.04);
  color: #fff;
  padding: 0 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.check-btn:hover { background: #2b2d31; }

.pane-hint {
  font-size: 12px;
  color: #4a4d52;
  margin: 0;
}

.text-end { text-align: right; }

/* Tree Box */
.tree-box-modern {
  background: rgba(0, 0, 0, 0.1);
  border-radius: 8px;
  border: 1px solid #2b2d31;
  overflow: hidden;
}

.tree-header-v2 {
  background: #2b2d31;
  padding: 10px 14px;
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}

.tree-chev-v2 { color: #777; display: flex; transform: scale(0.8); }
.tree-title-v2 { font-size: 14px; font-weight: 700; color: #fff; }

.tree-row-v2 {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
}

.tree-row-v2:hover { background: rgba(255, 255, 255, 0.02); }

.draggable-model {
  cursor: default;
}

.sparkle-avatar {
  cursor: grab;
}

.sparkle-avatar:active {
  cursor: grabbing;
}

.sortable-ghost {
  opacity: 0.4;
  background: rgba(255, 255, 255, 0.05) !important;
}

.row-info-v2 {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sparkle-avatar {
  width: 28px;
  height: 28px;
  background: #2b2d31;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.row-name-v2 { font-size: 14px; color: #dbdee1; font-weight: 500; }

.circles-group {
  display: flex;
  gap: 6px;
}

.cap-dot-v2 {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
}

.row-actions-v2 {
  display: flex;
  gap: 6px;
}

.row-mini-btn {
  background: transparent;
  border: none;
  color: #666;
  cursor: pointer;
  padding: 4px;
  display: flex;
}

.row-mini-btn:hover { color: #fff; }

/* Parameters Styles */
.params-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
.param-item { display: flex; flex-direction: column; gap: 8px; }
.param-top { display: flex; justify-content: space-between; font-size: 13px; color: #777; }
.param-top span { color: #fff; font-weight: 600; font-family: monospace; }
.param-number-input {
  background: #111214;
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 6px;
  padding: 8px 12px;
  color: #fff;
  font-size: 14px;
  outline: none;
}
.param-item input[type="range"] {
  width: 100%;
  height: 4px;
  background: #2b2d31;
  border-radius: 2px;
  appearance: none;
  outline: none;
}
.param-item input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  width: 14px;
  height: 14px;
  background: #fff;
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 0 10px rgba(0,0,0,0.5);
}

/* Footer Group */
.footer-btn-layout {
  display: flex;
  gap: 12px;
  margin-top: 12px;
}

.manage-green-btn {
  background: #248046;
  border: none;
  color: #fff;
  padding: 10px 24px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
}

.btn-icon-wrap { display: flex; }

.add-dark-btn {
  background: #2b2d31;
  border: 1px solid rgba(255, 255, 255, 0.05);
  color: #fff;
  padding: 10px 24px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15px;
  cursor: pointer;
}

.custom-url-check {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #888;
  cursor: pointer;
  user-select: none;
}

.custom-url-check input {
  cursor: pointer;
}

.custom-url-check:hover { color: #ccc; }
</style>
