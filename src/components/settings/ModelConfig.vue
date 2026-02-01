<script setup lang="ts">
import { ref, computed } from 'vue';
import { useConfigStore } from '../../stores/config';
import { 
  SEARCH_SVG, 
  PLUS_SVG,
  REFRESH_SVG,
  CHEVRON_DOWN_SVG,
  CLOSE_SVG
} from '../../constants/icons';
import { ModelInfo } from '../../types/config';
import { getProviderIcon } from '../../assets/icons';

// Import Sub-components
import ModelGroup from './models/ModelGroup.vue';
import ModelAddModal from './models/ModelAddModal.vue';
import ModelDiscoveryModal from './models/ModelDiscoveryModal.vue';

const props = defineProps<{
  providerId: string;
}>();

const configStore = useConfigStore();

// State
const listSearchQuery = ref('');
const showAddModal = ref(false);
const showManageModal = ref(false);
const fetchingModels = ref(false);
const discoveredModels = ref<any[]>([]);
const editingModel = ref<ModelInfo | null>(null);
const connectivityStatus = ref<'idle' | 'testing' | 'success' | 'error'>('idle');
const connectivityError = ref('');
const showApiKey = ref(false);
const showTestModelDropdown = ref(false);
const showInternalList = ref(false);

const providerConfig = computed(() => {
  return configStore.settings.providers.find(p => p.id === props.providerId);
});

// Grouped and Filtered Models List
const filteredGroupedModels = computed(() => {
  if (!providerConfig.value) return {};
  
  const rawModels = providerConfig.value.models || [];
  // Normalize models to ModelInfo format
  const normalizedModels: ModelInfo[] = rawModels.map(m => {
    if (typeof m === 'string') {
        const modelId = m;
        let group = '';
        if (modelId.startsWith('gpt-4')) group = 'GPT-4 Series';
        else if (modelId.startsWith('gpt-3.5')) group = 'GPT-3.5 Series';
        else if (modelId.startsWith('claude-3')) group = 'Claude 3';
        else if (modelId.startsWith('gemini-1.5')) group = 'Gemini 1.5';
        
        return {
            id: modelId,
            name: modelId,
            group: group,
            features: []
        };
    }
    return m;
  });

  const query = listSearchQuery.value.toLowerCase();
  
  // Filter models based on search query
  const filtered = query 
    ? normalizedModels.filter(m => m.id.toLowerCase().includes(query) || m.name.toLowerCase().includes(query))
    : normalizedModels;

  // Group by group name and ensure unique IDs
  const seenIds = new Set<string>();
  return filtered.reduce((groups: Record<string, ModelInfo[]>, model) => {
    if (seenIds.has(model.id)) return groups;
    seenIds.add(model.id);

    const group = model.group || '默认分组';
    if (!groups[group]) groups[group] = [];
    groups[group].push(model);
    return groups;
  }, {});
});

// CRUD Operations
const startEditModel = (model: ModelInfo) => {
    editingModel.value = model;
    showAddModal.value = true;
};

const handleSaveModel = (model: ModelInfo) => {
    if (!providerConfig.value) return;
    
    const models = [...providerConfig.value.models];
    const index = models.findIndex(m => (typeof m === 'string' ? m : m.id) === model.id);
    
    if (index > -1) {
        models[index] = model;
    } else {
        models.push(model);
    }
    
    configStore.updateProvider(props.providerId, { models });
    closeAddModal();
};

const removeModelById = (id: string) => {
    if (!providerConfig.value) return;
    if (!confirm(`确定要移除模型 ${id} 吗？`)) return;
    
    const models = providerConfig.value.models.filter(m => (typeof m === 'string' ? m : m.id) !== id);
    configStore.updateProvider(props.providerId, { models });
};

const closeAddModal = () => {
    showAddModal.value = false;
    editingModel.value = null;
};

// Discovery Logic
const fetchProviderModels = async () => {
    if (!providerConfig.value) return;
    
    if (!providerConfig.value.apiKey && props.providerId !== 'ollama') {
        alert('请先配置 API Key');
        return;
    }
    
    fetchingModels.value = true;
    discoveredModels.value = [];
    try {
        let baseUrl = providerConfig.value.baseUrl || '';
        let url = '';
        let headers: Record<string, string> = { 'Accept': 'application/json' };

        if (props.providerId === 'gemini') {
            url = `${baseUrl}/v1beta/models?key=${providerConfig.value.apiKey}`;
        } else if (props.providerId === 'ollama') {
            url = `${baseUrl}/api/tags`;
        } else if (props.providerId === 'claude') {
            url = `${baseUrl}/v1/models`;
            headers['x-api-key'] = providerConfig.value.apiKey;
            headers['anthropic-version'] = '2023-06-01';
            headers['dangerously-allow-browser'] = 'true';
        } else {
            url = baseUrl;
            if (url.includes('siliconflow.cn') || !url.includes('/v1')) {
                // For SiliconFlow and others without /v1, we need /v1/models
                if (!url.endsWith('/v1')) {
                    if (!url.endsWith('/')) url += '/';
                    url += 'v1/models';
                } else {
                    url += '/models';
                }
            } else {
                if (!url.endsWith('/models')) {
                    if (!url.endsWith('/')) url += '/';
                    url += 'models';
                }
            }
            headers['Authorization'] = `Bearer ${providerConfig.value.apiKey}`;
        }
        
        const resp = await fetch(url, { headers });
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
        const data = await resp.json();
        
        if (props.providerId === 'ollama') {
            if (data.models) {
                discoveredModels.value = data.models.map((m: any) => ({
                    id: m.name,
                    name: m.name,
                    group: 'Ollama Local'
                }));
            }
        } else if (data.models) {
            discoveredModels.value = data.models.map((m: any) => ({
                id: m.name.includes('/') ? m.name.split('/').pop() : m.id || m.name,
                name: m.displayName || m.name.split('/').pop(),
                group: 'API Discovery'
            }));
        } else if (data.data) {
            discoveredModels.value = data.data.map((m: any) => ({
                id: m.id,
                name: m.id,
                group: 'API Discovery'
            }));
        }
        
        if (discoveredModels.value.length > 0) {
            showManageModal.value = true;
        } else {
            alert('获取成功，但未返回模型列表。');
        }
    } catch (e: any) {
        alert(`获取失败: ${e.message}`);
    } finally {
        fetchingModels.value = false;
    }
};

const addFromDiscovery = (model: any) => {
    if (!providerConfig.value) return;
    const models = [...providerConfig.value.models];
    if (models.some(m => (typeof m === 'string' ? m : m.id) === model.id)) return;
    
    models.push({
        id: model.id,
        name: model.name || model.id,
        group: model.group || '',
        features: []
    });
    configStore.updateProvider(props.providerId, { models });
};

const isModelAdded = (id: string) => {
    return providerConfig.value?.models.some(m => (typeof m === 'string' ? m : m.id) === id) || false;
};

// connectivity Check
const checkConnectivity = async () => {
    if (!providerConfig.value) return;
    
    connectivityStatus.value = 'testing';
    connectivityError.value = '';
    
    try {
        let baseUrl = providerConfig.value.baseUrl || '';
        let headers: Record<string, string> = { 
            'Content-Type': 'application/json',
            'Accept': 'application/json' 
        };
        let method = 'POST';
        let body: any = null;
        let url = '';

        const currentTestModel = providerConfig.value.lastTestModelId || (providerConfig.value.models[0] ? (typeof providerConfig.value.models[0] === 'string' ? providerConfig.value.models[0] : providerConfig.value.models[0].id) : '');

        if (!currentTestModel) {
            throw new Error('请先添加或选择一个模型进行测试');
        }

        // Construct URL and Body based on Provider Type
        if (props.providerId === 'gemini') {
            // Google Native: POST :generateContent
            url = `${baseUrl}/v1beta/models/${currentTestModel}:generateContent?key=${providerConfig.value.apiKey}`;
            body = {
                contents: [{ parts: [{ text: "Hi" }] }]
            };
        } else if (props.providerId === 'claude') {
            // Anthropic Native: POST /v1/messages
            url = `${baseUrl}/v1/messages`;
            headers['x-api-key'] = providerConfig.value.apiKey;
            headers['anthropic-version'] = '2023-06-01';
            headers['dangerously-allow-browser'] = 'true';
            body = {
                model: currentTestModel,
                max_tokens: 10,
                messages: [{ role: "user", content: "Hi" }]
            };
        } else if (props.providerId === 'ollama') {
            // Ollama: POST /api/chat
            url = `${baseUrl}/api/chat`; // Ollama usually uses /api/chat
            body = {
                model: currentTestModel,
                messages: [{ role: "user", content: "Hi" }],
                stream: false
            };
        } else {
            // OpenAI Compatible (default): POST /v1/chat/completions
            if (providerConfig.value.disableUrlSuffix) {
                url = baseUrl;
            } else {
                url = baseUrl;
                if (!url.endsWith('/v1/chat/completions')) {
                    if (url.endsWith('/')) url += 'v1/chat/completions';
                    else url += '/v1/chat/completions';
                }
            }
            
            if (providerConfig.value.apiKey) {
                headers['Authorization'] = `Bearer ${providerConfig.value.apiKey}`;
            }

            body = {
                model: currentTestModel,
                messages: [{ role: "user", content: "Hi" }],
                max_tokens: 5,
                stream: false
            };
        }
        
        console.log(`[Connectivity Test] ${method} ${url}`, body);

        const resp = await fetch(url, { 
            method, 
            headers, 
            body: JSON.stringify(body) 
        });

        if (!resp.ok) {
            const errorData = await resp.json().catch(() => ({}));
            console.error('[Connectivity Test] Failed:', errorData);
            throw new Error(errorData.error?.message || `HTTP ${resp.status}: ${resp.statusText}`);
        }
        
        // Optional: Check if response body is valid JSON?
        await resp.json();

        connectivityStatus.value = 'success';
        
        // Auto-reset status after 2 seconds
        setTimeout(() => {
            if (connectivityStatus.value === 'success') {
                connectivityStatus.value = 'idle';
            }
        }, 2000);

    } catch (e: any) {
        console.error('[Connectivity Test] Error:', e);
        connectivityStatus.value = 'error';
        connectivityError.value = e.message;
        
        // Auto-reset error status after 3 seconds too? Or keep it to show error?
        // User asked to "maintain previous button", implying reset.
        // But for error, maybe they want to read it. 
        // Let's reset the BUTTON state if it was successful. 
        // If error, usually we might want to keep it red until they click again, but user said "don't forget to close the output".
        // Let's strictly follow "test success -> close". Error might stay.
    }
};

const selectTestModel = (modelId: string) => {
    configStore.updateProvider(props.providerId, { lastTestModelId: modelId });
    connectivityStatus.value = 'idle';
    connectivityError.value = '';
};

const updateApiKey = (val: string) => {
    configStore.updateProvider(props.providerId, { apiKey: val });
    connectivityStatus.value = 'idle'; // Reset status on change
};

const updateBaseUrl = (val: string) => {
    configStore.updateProvider(props.providerId, { baseUrl: val });
    connectivityStatus.value = 'idle';
};

const updateDisableSuffix = (val: boolean) => {
    configStore.updateProvider(props.providerId, { disableUrlSuffix: val });
    connectivityStatus.value = 'idle';
};

const handleStartCheck = () => {
    if (!providerConfig.value?.models.length) {
        checkConnectivity(); // If no models, just try basic URL check
    } else {
        showTestModelDropdown.value = true;
    }
};

const confirmTest = () => {
    checkConnectivity();
};

</script>

<template>
  <div class="models-section">
    <!-- Provider Basic Settings -->
    <div class="provider-config-v3">
        <!-- API Key Row -->
        <div class="config-item-v3">
            <div class="item-label-row-v3">
                <label>API 密钥</label>
                <span class="label-hint-v3">多个密钥使用逗号分隔</span>
            </div>
            <div class="input-with-action-v3">
                <input 
                    :type="showApiKey ? 'text' : 'password'" 
                    :value="providerConfig?.apiKey"
                    @input="e => updateApiKey((e.target as HTMLInputElement).value)"
                    placeholder="输入您的 API Key..."
                    class="modern-input-v3"
                />
                <button class="toggle-eye-v3" @click="showApiKey = !showApiKey" title="显示/隐藏密钥">
                    <svg v-if="showApiKey" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path><line x1="1" y1="1" x2="23" y2="23"></line></svg>
                    <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>
                </button>
                <div class="action-divider-v3"></div>
                <button 
                    class="check-conn-btn-v2" 
                    :class="connectivityStatus" 
                    @click="handleStartCheck" 
                    :disabled="connectivityStatus === 'testing'"
                >
                    <span v-if="connectivityStatus === 'testing'" class="loading-spin"></span>
                    {{ 
                        connectivityStatus === 'testing' ? '检测中' : 
                        connectivityStatus === 'success' ? '通过' : 
                        connectivityStatus === 'error' ? '失败' : '检测' 
                    }}
                </button>
            </div>
        </div>

        <!-- Base URL Row -->
        <div class="config-item-v3">
            <div class="item-label-row-v3">
                <label>API 地址</label>
                <div class="label-actions-v3">
                    <span class="label-hint-v3" title="接口请求基地址">Base URL</span>
                    <div class="modern-toggle-v3" title="开启后将完全使用下方填写的路径，不再自动拼接后缀">
                        <span class="toggle-label-v3">自定义路径</span>
                        <div 
                            class="toggle-switch-v3" 
                            :class="{ active: providerConfig?.disableUrlSuffix }"
                            @click="updateDisableSuffix(!providerConfig?.disableUrlSuffix)"
                        >
                            <div class="toggle-dot-v3"></div>
                        </div>
                    </div>
                </div>
            </div>
            <div class="input-with-action-v3">
                <input 
                    type="text" 
                    :value="providerConfig?.baseUrl"
                    @input="e => updateBaseUrl((e.target as HTMLInputElement).value)"
                    :placeholder="props.providerId === 'ollama' ? 'http://localhost:11434' : 'https://api...'"
                    class="modern-input-v3"
                />
            </div>
            <div class="url-preview-v3">
                预览: <code>{{ 
                    (() => {
                        const base = providerConfig?.baseUrl || '默认';
                        if (providerConfig?.disableUrlSuffix) return base;
                        
                        // Dynamic preview based on provider type
                        if (props.providerId === 'gemini') return `${base}/v1beta/models/{model}:generateContent`;
                        if (props.providerId === 'claude') return `${base}/v1/messages`;
                        if (props.providerId === 'ollama') return `${base}/api/chat`;
                        
                        // Default / OpenAI Compatible
                        const u = base.replace(/\/+$/, '');
                        if (u.includes('/v1/chat/completions')) return u;
                        if (u.endsWith('/v1')) return `${u}/chat/completions`;
                        return `${u}/v1/chat/completions`;
                    })()
                }}</code>
            </div>
        </div>

        <!-- Error Message -->
        <div v-if="connectivityError" class="conn-error-msg-v3">
            {{ connectivityError }}
        </div>
    </div>

    <div class="divider-v3"></div>

    <div class="section-header">
      <h3 class="sub-title">模型管理</h3>
      <div class="header-badge">{{ providerConfig?.models.length || 0 }} Models</div>
      
      <div class="header-search-v3">
          <span v-html="SEARCH_SVG"></span>
          <input v-model="listSearchQuery" placeholder="搜索已添加模型..." />
      </div>

      <div class="header-actions">
        <span class="action-icon" @click="showAddModal = true" v-html="PLUS_SVG" title="手动添加"></span>
      </div>
    </div>

    <!-- Grouped Models List -->
    <div class="grouped-models-container modern-scroll">
      <ModelGroup 
        v-for="(groupModels, groupName) in filteredGroupedModels" 
        :key="groupName"
        :groupName="groupName"
        :models="groupModels"
        :providerIcon="providerConfig?.icon || 'default'"
        @edit="startEditModel"
        @remove="removeModelById"
      />
      
      <div v-if="Object.keys(filteredGroupedModels).length === 0" class="empty-list-v3">
          {{ listSearchQuery ? '未找到匹配模型' : '暂无模型，点击下方按钮开始添加' }}
      </div>
    </div>
    
    <div class="footer-actions-v3">
      <button class="manage-btn-v3" @click="fetchProviderModels" :disabled="fetchingModels">
        {{ fetchingModels ? '正在获取...' : '管理' }}
      </button>
      <button class="add-btn-v3" @click="showAddModal = true">
        添加
      </button>
    </div>

    <!-- Modals -->
    <ModelAddModal 
        :show="showAddModal" 
        :editingModel="editingModel"
        @close="closeAddModal"
        @save="handleSaveModel"
    />

    <ModelDiscoveryModal 
        :show="showManageModal"
        :providerId="props.providerId"
        :discoveredModels="discoveredModels"
        :isModelAdded="isModelAdded"
        @close="showManageModal = false"
        @add="addFromDiscovery"
    />

    <!-- User Requested Connectivity Test Modal -->
    <Transition name="fade">
      <div v-if="showTestModelDropdown" class="modal-overlay-v3" @click="showTestModelDropdown = false">
        <div class="test-modal-v3" @click.stop>
          <div class="modal-header-v3">
            <h3>请选择要检测的模型</h3>
            <button class="close-modal-btn-v3" @click="showTestModelDropdown = false" v-html="CLOSE_SVG"></button>
          </div>
          
          <div class="modal-body-v3">
            <div class="test-select-wrapper-v3">
                <div class="custom-select-v3" @click="showInternalList = !showInternalList" :class="{ open: showInternalList }">
                    <div class="selected-value-v3">
                        <span v-html="getProviderIcon(providerConfig?.icon || 'default')" class="p-icon"></span>
                        <span class="m-name">{{ providerConfig?.lastTestModelId || (providerConfig?.models[0] ? (typeof providerConfig?.models[0] === 'string' ? providerConfig?.models[0] : providerConfig?.models[0].name) : '未选择') }}</span>
                        <span class="p-name">| {{ providerConfig?.name }}</span>
                        <span class="chevron" v-html="CHEVRON_DOWN_SVG"></span>
                    </div>
                    
                    <Transition name="fade-slide">
                        <div v-if="showInternalList" class="select-dropdown-v3 modern-scroll">
                            <div 
                                v-for="model in providerConfig?.models" 
                                :key="typeof model === 'string' ? model : model.id"
                                class="select-item-v3"
                                :class="{ active: (typeof model === 'string' ? model : model.id) === (providerConfig?.lastTestModelId || (providerConfig?.models[0] ? (typeof providerConfig?.models[0] === 'string' ? providerConfig?.models[0] : providerConfig?.models[0].id) : '')) }"
                                @click.stop="selectTestModel(typeof model === 'string' ? model : model.id); showInternalList = false"
                            >
                                {{ typeof model === 'string' ? model : (model.name || model.id) }}
                            </div>
                        </div>
                    </Transition>
                </div>
            </div>

            <!-- Test Result Info -->
            <div v-if="connectivityStatus !== 'idle'" class="modal-status-v3" :class="connectivityStatus">
                <div v-if="connectivityStatus === 'testing'" class="status-loading-v3">
                    <span class="loading-spin"></span>
                    正在连接...
                </div>
                <div v-else-if="connectivityStatus === 'success'" class="status-success-v3">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                    连接成功
                </div>
                <div v-else-if="connectivityStatus === 'error'" class="status-error-v3">
                    <div class="error-header-v3">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                        连接失败
                    </div>
                    <div class="error-detail-v3">{{ connectivityError }}</div>
                </div>
            </div>
          </div>

          <div class="modal-footer-v3">
            <button class="btn-cancel-v3" @click="showTestModelDropdown = false">关闭</button>
            <button class="btn-confirm-v3" @click="confirmTest" :disabled="connectivityStatus === 'testing'">
                {{ connectivityStatus === 'testing' ? '正在测试...' : '立即测试' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* Models Section */
.models-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 12px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 4px;
}

.sub-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-color-white);
  margin: 0;
}

.header-badge {
    background: var(--bg-glass-active);
    color: var(--text-secondary);
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 12px;
    font-weight: 500;
}

/* Header Search */
.header-search-v3 {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--bg-input-dim, rgba(255,255,255,0.03));
    border: 1.5px solid var(--border-glass-bright);
    border-radius: 8px;
    padding: 4px 10px;
    flex: 1;
    max-width: 200px;
    margin-left: auto;
}
.header-search-v3 :deep(svg) { width: 14px; height: 14px; color: var(--text-tertiary); }
.header-search-v3 input {
    background: transparent;
    border: none;
    color: #fff;
    font-size: 12px;
    outline: none;
    width: 0;
    flex: 1;
}

.header-actions {
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
    cursor: pointer;
    transition: color 0.2s;
}

.header-actions:hover { color: var(--text-color-white); }
.action-icon :deep(svg) { width: 18px; height: 18px; }

/* Grouped List */
.grouped-models-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-height: 500px;
    overflow-y: auto;
    padding-right: 4px;
}

.empty-list-v3 {
    text-align: center;
    padding: 40px 20px;
    color: var(--text-tertiary);
    font-size: 13px;
    background: var(--bg-card-dim);
    border: 1px dashed var(--border-glass);
    border-radius: 12px;
}

/* Footer Buttons */
.footer-actions-v3 {
    display: flex;
    gap: 12px;
    margin-top: 8px;
}

.manage-btn-v3, .add-btn-v3 {
    flex: 1;
    height: 38px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
}

.manage-btn-v3 { 
    background: var(--bg-glass-active); 
    color: var(--text-color-white); 
    border: 1px solid var(--border-glass-bright);
}
.manage-btn-v3:hover { background: var(--bg-glass-hover); }
.manage-btn-v3:disabled { opacity: 0.5; cursor: not-allowed; }

.add-btn-v3 { 
    background: var(--color-primary); 
    color: #000; 
    font-weight: 600;
}
.add-btn-v3:hover { opacity: 0.9; transform: translateY(-1px); }

.btn-icon :deep(svg) { width: 16px; height: 16px; }

/* Provider Config styles */
.provider-config-v3 {
    background: var(--bg-card-dim);
    border: 1px solid var(--border-glass);
    border-radius: 12px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.config-item-v3 {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.item-label-row-v3 {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 4px;
}

.label-actions-v3 {
    display: flex;
    align-items: center;
    gap: 12px;
}

.custom-url-toggle-v3 {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-tertiary);
    cursor: pointer;
    user-select: none;
    transition: color 0.2s;
}

.custom-url-toggle-v3:hover {
    color: var(--color-primary);
}

.custom-url-toggle-v3 input {
    margin: 0;
    cursor: pointer;
    accent-color: var(--color-primary);
}

.config-item-v3 label {
    font-size: 13px;
    color: var(--text-color-white);
    font-weight: 700;
}

.label-hint-v3 {
    font-size: 11px;
    color: var(--text-tertiary);
}

.input-with-action-v3 {
    position: relative;
    display: flex;
    align-items: center;
    background: var(--bg-input-dim);
    border: 1.5px solid var(--border-glass-bright);
    border-radius: 8px;
    transition: all 0.2s;
}

.input-with-action-v3:focus-within {
    border-color: var(--color-primary);
    background: var(--bg-input-focus);
}

.modern-input-v3 {
    flex: 1;
    background: transparent;
    border: none;
    padding: 10px 12px;
    color: var(--text-color-white);
    font-size: 14px;
    outline: none;
}

.toggle-eye-v3 {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    display: flex;
    padding: 8px;
    opacity: 0.6;
    transition: opacity 0.2s;
}
.toggle-eye-v3:hover { opacity: 1; color: var(--color-primary); }

.action-divider-v3 {
    width: 1px;
    height: 16px;
    background: var(--border-glass);
    margin: 0 4px;
}

.check-conn-btn-v2 {
    background: transparent;
    border: none;
    color: var(--text-color-white);
    padding: 0 16px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    height: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    border-left: 1px solid var(--border-glass);
    border-radius: 0 8px 8px 0;
}

.check-conn-btn-v2:hover { background: var(--bg-glass-hover); }
.check-conn-btn-v2.testing { opacity: 0.7; }
.check-conn-btn-v2.success { color: #10b981; background: rgba(16, 185, 129, 0.1); }
.check-conn-btn-v2.error { color: #ef4444; background: rgba(239, 68, 68, 0.1); }

.url-preview-v3 {
    font-size: 12px;
    color: var(--text-tertiary);
    padding: 0 4px;
    opacity: 0.8;
    margin-top: 4px;
    word-break: break-all;
    overflow-wrap: break-word;
    max-width: 100%;
    line-height: 1.2;
    transform: scale(0.85);
    transform-origin: left top;
}

.url-preview-v3 code {
    background: transparent;
    padding: 0;
    border-radius: 0;
    color: inherit;
    font-family: var(--font-mono);
}

.conn-error-msg-v3 {
    font-size: 11px;
    color: #ef4444;
    padding: 8px 12px;
    background: rgba(239, 68, 68, 0.05);
    border: 1px solid rgba(239, 68, 68, 0.1);
    border-radius: 8px;
}

.divider-v3 {
    height: 1px;
    background: var(--border-glass);
    margin: 8px 0;
}

.loading-spin {
    width: 10px;
    height: 10px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

/* Modern Toggle Switch */
.modern-toggle-v3 {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
}

.toggle-label-v3 {
    font-size: 11px;
    color: var(--text-tertiary);
    font-weight: 500;
}

.toggle-switch-v3 {
    width: 36px;
    height: 20px;
    background: var(--bg-glass-active);
    border: 1px solid var(--border-glass-bright);
    border-radius: 20px;
    position: relative;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.toggle-switch-v3.active {
    background: var(--color-primary);
    border-color: rgba(255, 255, 255, 0.1);
}

.toggle-dot-v3 {
    width: 14px;
    height: 14px;
    background: var(--text-color-white, #fff);
    border-radius: 50%;
    position: absolute;
    top: 2px;
    left: 2px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.toggle-switch-v3.active .toggle-dot-v3 {
    left: 18px;
    background: var(--text-color-white, #fff);
}

/* Test Model Selector */
.test-model-wrapper-v3 {
    position: relative;
    height: 100%;
}

.test-model-select-v3 {
    height: 100%;
    background: transparent;
    border: none;
    padding: 0 12px;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    color: var(--text-color-white);
    transition: background 0.2s;
    min-width: 140px;
}

.test-model-select-v3:hover {
    background: var(--bg-glass-hover);
}

.test-model-icon-v3 {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-primary);
}

.test-model-icon-v3 :deep(svg) {
    width: 14px;
    height: 14px;
}

.test-model-name-v3 {
    font-size: 13px;
    font-weight: 500;
    max-width: 100px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    opacity: 0.9;
}

.chevron-v3 {
    margin-left: auto;
    display: flex;
    color: var(--text-tertiary);
}

.chevron-v3 :deep(svg) {
    width: 12px;
    height: 12px;
}

.test-model-dropdown-v3 {
    position: absolute;
    bottom: calc(100% + 8px);
    right: 0;
    width: 220px;
    max-height: 240px;
    background: var(--bg-dropdown, #1e1e24);
    backdrop-filter: blur(20px);
    border: 1px solid var(--border-dropdown, rgba(255,255,255,0.1));
    border-radius: 12px;
    box-shadow: 0 10px 30px rgba(0,0,0,0.5);
    z-index: 100;
    overflow-y: auto;
    padding: 6px;
}

.test-model-item-v3 {
    padding: 8px 12px;
    font-size: 13px;
    color: var(--text-secondary);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.test-model-item-v3:hover {
    background: var(--bg-glass-hover);
    color: var(--text-color-white);
}

.test-model-item-v3.selected {
    background: var(--bg-menu-active);
    color: var(--color-menu-active);
    font-weight: 600;
}

/* Connectivity Test Modal Styles */
.modal-overlay-v3 {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
}

.test-modal-v3 {
    width: 440px;
    background: var(--bg-modal, #1e1e24);
    border: 1px solid var(--border-glass);
    border-radius: 20px;
    box-shadow: 0 30px 60px rgba(0,0,0,0.4);
    display: flex;
    flex-direction: column;
    overflow: visible;
}

.modal-header-v3 {
    padding: 20px 24px 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.modal-header-v3 h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text-color-white);
}

.close-modal-btn-v3 {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: color 0.2s;
    display: flex;
    padding: 4px;
}
.close-modal-btn-v3:hover { color: var(--text-color-white); }
.close-modal-btn-v3 :deep(svg) { width: 18px; height: 18px; }

.modal-body-v3 {
    padding: 0 24px 24px;
}

.test-select-wrapper-v3 {
    position: relative;
    width: 100%;
}

.custom-select-v3 {
    background: var(--bg-input-dim, rgba(255,255,255,0.03));
    border: 1px solid var(--border-glass);
    border-radius: 12px;
    padding: 12px 16px;
    cursor: pointer;
    position: relative;
    transition: all 0.2s;
}

.custom-select-v3:hover {
    border-color: var(--border-glass-bright);
    background: var(--bg-input-focus);
}

.selected-value-v3 {
    display: flex;
    align-items: center;
    gap: 10px;
}

.p-icon {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-primary);
}
.p-icon :deep(svg) { width: 18px; height: 18px; }

.m-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-color-white);
}

.p-name {
    font-size: 14px;
    color: var(--text-tertiary);
}

.selected-value-v3 .chevron {
    margin-left: auto;
    color: var(--text-tertiary);
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.custom-select-v3.open .chevron {
    transform: rotate(180deg);
}

.select-dropdown-v3 {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    right: 0;
    background: var(--bg-dropdown, #1e1e24);
    border: 1px solid var(--border-glass);
    border-radius: 12px;
    max-height: 200px;
    overflow-y: auto;
    z-index: 100;
    padding: 6px;
    box-shadow: 0 10px 30px rgba(0,0,0,0.3);
}

.select-item-v3 {
    padding: 10px 14px;
    font-size: 13px;
    color: var(--text-secondary);
    border-radius: 8px;
    transition: all 0.2s;
}

.select-item-v3:hover {
    background: var(--bg-glass-hover);
    color: var(--text-color-white);
}

.select-item-v3.active {
    background: var(--bg-menu-active);
    color: var(--color-menu-active);
    font-weight: 600;
}

.modal-footer-v3 {
    padding: 0 24px 24px;
    display: flex;
    justify-content: flex-end;
    gap: 12px;
}

.btn-cancel-v3, .btn-confirm-v3 {
    padding: 8px 24px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
}

.btn-cancel-v3 {
    background: transparent;
    border: 1px solid var(--border-glass);
    color: var(--text-color-white);
}
.btn-cancel-v3:hover { background: var(--bg-glass-hover); }

.btn-confirm-v3 {
    background: var(--color-primary);
    border: none;
    color: #000;
}
.btn-confirm-v3:hover { opacity: 0.9; }

/* Existing Fade Animations */
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
