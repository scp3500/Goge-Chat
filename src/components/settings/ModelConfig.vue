
<template>
  <div class="model-config-container" v-if="providerConfig">
    <!-- Header -->
    <div class="config-header-row">
      <div class="header-info">
        <h2 class="provider-title">{{ providerConfig.name }}</h2>
        <span class="provider-id-badge">{{ providerId }}</span>
      </div>
      <label class="switch-modern">
        <input type="checkbox" :checked="providerConfig.enabled" @change="toggleEnabled" />
        <span class="slider"></span>
      </label>
    </div>

    <!-- Main Settings Card -->
    <div class="config-card">
      <!-- API Key -->
      <div class="setting-group">
        <div class="group-header">
          <label class="group-label">API 密钥</label>
          <button class="text-action-btn" :disabled="testingConnection" @click="testingConnection = true">
            {{ testingConnection ? '连接中...' : '测试连接' }}
          </button>
        </div>
        <div class="input-wrapper-modern">
          <input
            :type="showApiKey ? 'text' : 'password'"
            v-model="providerConfig.apiKey"
            placeholder="sk-..."
            @change="handleApiKeyChange"
            class="full-width-input"
          />
          <button class="icon-toggle-btn" @click="showApiKey = !showApiKey" v-html="showApiKey ? EYE_OPEN_SVG : EYE_CLOSED_SVG"></button>
        </div>
        <p class="field-hint">用于鉴权的私有密钥，将安全存储在本地。</p>
      </div>
      
      <div class="divider"></div>

      <!-- Base URL -->
      <div class="setting-group">
        <div class="group-header">
          <label class="group-label">Base URL</label>
          <label class="checkbox-label">
            <input 
              type="checkbox" 
              v-model="providerConfig.disableUrlSuffix" 
              @change="handleParamChange('disableUrlSuffix', providerConfig.disableUrlSuffix)"
            />
            <span>完全自定义</span>
          </label>
        </div>
        <div class="input-wrapper-modern">
          <input
            type="text"
            v-model="providerConfig.baseUrl"
            placeholder="https://api.example.com/v1"
            class="full-width-input"
            @change="handleBaseUrlChange"
          />
        </div>
        <p class="field-hint">
          当前请求地址: <span class="code-snippet">{{ providerConfig.baseUrl || 'https://...' }}{{ 
            providerConfig.disableUrlSuffix ? '' : 
            (providerConfig.id === 'gemini' ? '/v1beta/models' : '/chat/completions') 
          }}</span>
        </p>
      </div>

      <div class="divider"></div>

      <!-- Parameters (Temperature & Tokens) -->
       <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">
            <span>回复随机度</span>
            <span class="tag">Temperature</span>
          </div>
        </div>
        <div class="control-wrapper">
          <div class="slider-display">{{ providerConfig.temperature }}</div>
          <input 
            type="range" 
            v-model.number="providerConfig.temperature" 
            min="0" 
            max="2" 
            step="0.1" 
             class="modern-range"
            :style="{ '--progress': (providerConfig.temperature / 2) * 100 + '%' }"
            @input="handleParamChange('temperature', providerConfig.temperature)"
          />
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">
            <span>最大回复长度</span>
            <span class="tag">Max Tokens</span>
          </div>
        </div>
        <div class="control-wrapper">
          <input 
            type="number" 
            v-model.number="providerConfig.maxTokens" 
            placeholder="默认"
            @change="handleParamChange('maxTokens', providerConfig.maxTokens)"
            class="modern-input number-input"
          />
        </div>
      </div>
    </div>

    <!-- Models Management -->
    <div class="models-section">
      <div class="section-header">
        <h3 class="sub-title">模型列表</h3>
        <div class="actions-right">
          <button class="icon-btn-modern" title="添加模型" @click="addModel">
            <span v-html="PLUS_SVG"></span>
          </button> 
        </div>
      </div>

      <div class="tree-box-modern">
        <div class="tree-inner-group">
          <div class="tree-header-v2">
            <span class="tree-chev-v2" v-html="CHEVRON_DOWN_SVG"></span>
            <span class="tree-title-v2">{{ providerConfig.models?.length || 0 }} 个模型</span>
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
              <template #item="{ element: model, index }">
                <div class="tree-row-v2 draggable-model">
                  <div class="row-info-v2">
                    <div class="sparkle-avatar">
                      <svg viewBox="0 0 24 24" width="20" height="20">
                        <defs>
                          <linearGradient :id="'fire-grad-' + index" x1="0" y1="0" x2="1" y2="1">
                            <stop offset="0%" stop-color="var(--badge-reasoning)" />
                            <stop offset="100%" stop-color="var(--badge-vision)" />
                          </linearGradient>
                        </defs>
                        <path d="M12 2l2.3 7 7 2.3-7 2.3-2.3 7-2.3-7-7-2.3 7-2.3z" :fill="'url(#fire-grad-' + index + ')'" />
                      </svg>
                    </div>
                    
                    <!-- Inline Edit -->
                    <input 
                      v-if="editingIndex === index"
                      ref="editInputRef"
                      v-model="editingModelName"
                      class="model-edit-input"
                      @blur="saveEdit(index)"
                      @keydown.enter="saveEdit(index)"
                      @keydown.esc="cancelEdit"
                    />
                    <span 
                      v-else 
                      class="row-name-v2" 
                      @dblclick="startEdit(index, model)"
                    >
                      {{ model }}
                    </span>

                    <div class="circles-group">
                      <div v-for="(cap, idx) in getCapabilities(model)" :key="idx" class="cap-dot-v2" :style="{ backgroundColor: cap.bg, color: cap.color }">
                        {{ cap.icon }}
                      </div>
                    </div>
                  </div>
                  <div class="row-actions-v2">
                    <button class="row-mini-btn" v-html="SETTINGS_SVG" @click="startEdit(index, model)"></button>
                    <button class="row-mini-btn remove-btn" @click="removeModel(index)">
                      <span>–</span>
                    </button>
                  </div>
                </div>
              </template>
            </draggable>
          </div>
        </div>
      </div>
      
      <div class="list-footer-hint">双击名称可修改，拖拽头像可排序。</div>
    </div>
  </div>
</template>

<script setup>
import draggable from 'vuedraggable';
import { ref, computed, watch, nextTick } from 'vue';
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

// Editing State
const editingIndex = ref(-1);
const editingModelName = ref('');
const editInputRef = ref(null);

// 同步逻辑：仅在集合实质变化时更新本地列表，保持拖拽后的顺序稳定
watch(() => providerConfig.value?.models, (newVal) => {
  if (!newVal) {
    modelsList.value = [];
    return;
  }
  
  const localKeys = modelsList.value.join(',');
  const remoteKeys = newVal.join(',');
  
  if (localKeys === remoteKeys) return;
  
  // Checking distinct changes
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

// CRUD Operations
const addModel = async () => {
  const newModelName = 'new-model-' + Math.floor(Math.random() * 1000);
  modelsList.value.push(newModelName);
  await handleModelsReorder();
  
  // Automatically start editing the new model
  const newIndex = modelsList.value.length - 1;
  startEdit(newIndex, newModelName);
};

const removeModel = async (index) => {
  modelsList.value.splice(index, 1);
  await handleModelsReorder();
};

const startEdit = (index, model) => {
  editingIndex.value = index;
  editingModelName.value = model;
  nextTick(() => {
    if (editInputRef.value) editInputRef.value.focus();
  });
};

const saveEdit = async (index) => {
  if (editingIndex.value === -1) return;
  
  const newName = editingModelName.value.trim();
  if (newName) {
    modelsList.value[index] = newName;
    await handleModelsReorder();
  }
  
  cancelEdit();
};

const cancelEdit = () => {
  editingIndex.value = -1;
  editingModelName.value = '';
};

// 模型能力配置
const getCapabilities = (model) => [
  { icon: '👁', bg: 'var(--color-success-bg)', color: 'var(--color-success)' },
  { icon: '🌐', bg: 'var(--color-primary-bg)', color: 'var(--color-primary)' },
  { icon: '☀️', bg: 'var(--bg-glass-active)', color: 'var(--badge-vision)' },
  { icon: '🔧', bg: 'var(--color-warning-alpha-10)', color: 'var(--color-warning)' }
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

<style scoped>
/* HMR Trigger Force */
.model-config-container {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 8px 4px;
  color: var(--text-color);
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* Header */
.config-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.provider-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color-white);
  margin: 0;
}

.provider-id-badge {
  font-size: 11px;
  background: var(--bg-glass);
  padding: 2px 6px;
  border-radius: 4px;
  color: var(--text-color);
  opacity: 0.6;
  font-family: monospace;
}

/* Modern Switch */
.switch-modern {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
}
.switch-modern input { opacity: 0; width: 0; height: 0; }
.slider {
  position: absolute;
  cursor: pointer;
  top: 0; left: 0; right: 0; bottom: 0;
  background-color: var(--bg-input);
  transition: .3s;
  border-radius: 22px;
  border: 1px solid var(--border-glass);
}
.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 2px;
  bottom: 2px;
  background-color: var(--bg-selectors);
  transition: .3s;
  border-radius: 50%;
}
input:checked + .slider { background-color: var(--color-success); }
input:checked + .slider:before { transform: translateX(18px); background-color: #fff; }

/* Config Card */
.config-card {
  background: var(--bg-glass);
  border: 1px solid var(--border-glass);
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.divider {
  height: 1px;
  background: var(--border-card);
  margin: 0 -20px;
}

/* Setting Groups */
.setting-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.group-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-color-white);
}

.text-action-btn {
  background: transparent;
  border: none;
  font-size: 12px;
  color: var(--color-primary);
  cursor: pointer;
  padding: 0;
}
.text-action-btn:hover { text-decoration: underline; }

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-tertiary);
  cursor: pointer;
}

/* Modern Input Wrapper */
.input-wrapper-modern {
  display: flex;
  align-items: center;
  background: var(--bg-input);
  border: 1px solid var(--border-glass);
  border-radius: 8px;
  padding: 0 10px;
  transition: border-color 0.2s;
}

.input-wrapper-modern:focus-within {
  border-color: var(--color-primary);
}

.full-width-input {
  flex: 1;
  background: transparent;
  border: none;
  color: #fff;
  font-size: 13px;
  padding: 10px 0;
  outline: none;
  font-family: monospace;
}

.icon-toggle-btn {
  background: transparent;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 4px;
  display: flex;
}
.icon-toggle-btn:hover { color: var(--text-color); }

.field-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  margin: 0;
  line-height: 1.4;
}

.code-snippet {
  font-family: monospace;
  background: var(--bg-glass-active);
  padding: 2px 4px;
  border-radius: 4px;
  font-size: 11px;
}

/* Slider & Params (Copied from PresetConfig for consistency) */
.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 24px;
}

.setting-info { flex: 1; }

.setting-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-color-white);
}

.tag {
  font-size: 11px;
  background: var(--bg-glass);
  padding: 2px 6px;
  border-radius: 4px;
  color: var(--text-secondary);
  font-family: monospace;
}

.control-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 180px;
  justify-content: flex-end;
}

.slider-display {
  font-family: monospace;
  font-size: 13px;
  color: var(--color-primary);
  background: var(--color-primary-bg);
  padding: 2px 8px;
  border-radius: 4px;
  min-width: 36px;
  text-align: center;
}

.modern-range {
  -webkit-appearance: none;
  appearance: none;
  width: 140px;
  height: 6px;
  background: var(--bg-glass-active);
  border-radius: 3px;
  outline: none;
  position: relative;
}

.modern-range::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: var(--progress, 0%);
  background: var(--color-primary);
  border-radius: 3px;
}

.modern-range::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  background: #fff;
  border-radius: 50%;
  cursor: pointer;
  box-shadow: var(--shadow-main);
  position: relative;
  z-index: 2;
  transition: transform 0.1s;
  margin-top: 0;
}
.modern-range::-webkit-slider-thumb:hover { transform: scale(1.1); }

.modern-input {
  background: var(--bg-input);
  border: 1px solid var(--border-glass);
  border-radius: 6px;
  padding: 6px 10px;
  color: var(--text-color-white);
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
  text-align: right;
  width: 100px;
}
.modern-input:focus { border-color: var(--color-primary); }

/* Models Section */
.models-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 8px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.sub-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-color-white);
  margin: 0;
}

.icon-btn-modern {
  background: var(--bg-glass);
  border: none;
  border-radius: 6px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}
.icon-btn-modern:hover { background: var(--bg-glass-hover); color: var(--text-color-white); }
.icon-btn-modern :deep(svg) { width: 16px; height: 16px; }

/* Tree Box Modern (Restored & Refined) */
.tree-box-modern {
  background: var(--bg-glass);
  border-radius: 8px;
  border: 1px solid var(--border-glass); /* Slightly lighter border */
  overflow: hidden;
}

.tree-header-v2 {
  background: var(--bg-code-header); /* Slightly transparent */
  padding: 10px 14px;
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  border-bottom: 1px solid var(--border-glass);
}

.tree-chev-v2 { color: var(--text-tertiary); display: flex; transform: scale(0.8); }
.tree-title-v2 { font-size: 13px; font-weight: 600; color: var(--text-secondary); }

.tree-row-v2 {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: transparent;
  transition: background 0.2s;
  border-bottom: 1px solid var(--border-glass);
}

.tree-row-v2:last-child { border-bottom: none; }
.tree-row-v2:hover { background: var(--bg-glass-hover); }

.draggable-model {
  cursor: default;
}

/* Sparkle Avatar */
.sparkle-avatar {
  cursor: grab;
  width: 28px;
  height: 28px;
  background: var(--bg-main);
  border-radius: 6px;
  border: 1px solid var(--border-glass);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.sparkle-avatar:active { cursor: grabbing; }

.sortable-ghost {
  opacity: 0.4;
  background: var(--bg-glass-active) !important;
}

.row-info-v2 {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.row-name-v2 { 
  font-size: 14px; 
  color: var(--text-color); 
  font-weight: 500; 
  cursor: text;
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.model-edit-input {
  background: var(--bg-input-focus);
  border: 1px solid var(--color-primary);
  color: var(--text-color-white);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 14px;
  outline: none;
  flex: 1;
  min-width: 0;
}

.circles-group {
  display: flex;
  gap: 6px;
  margin-left: auto; /* Push to right of info area */
  padding-right: 12px;
}

.cap-dot-v2 {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
}

.row-actions-v2 {
  display: flex;
  gap: 4px;
}

.row-mini-btn {
  background: transparent;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 6px;
  display: flex;
  border-radius: 4px;
  transition: all 0.2s;
}

.row-mini-btn:hover { color: var(--text-color-white); background: var(--bg-glass-hover); }
.remove-btn:hover { color: var(--color-danger); background: var(--color-danger-alpha-10); }

.list-footer-hint {
  font-size: 11px;
  color: var(--text-tertiary);
  text-align: center;
  margin-top: 4px;
}
</style>
