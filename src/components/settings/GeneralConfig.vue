<script setup>
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { useConfigStore } from '../../stores/config';
import { BRAIN_SVG, BOX_SVG, SPARKLES_SVG, CHEVRON_DOWN_SVG } from '../../constants/icons';
import ImageCropperModal from '../modals/ImageCropperModal.vue';
import { open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';

const configStore = useConfigStore();

// Avatar Cropper
const showCropper = ref(false);
const cropImgSrc = ref('');
const handleAvatarUpload = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'Images',
                extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif']
            }]
        });
        
        if (selected) {
            // Read file as binary
            const content = await readFile(selected);
            // Convert to base64
            const base64 = btoa(
                new Uint8Array(content)
                  .reduce((data, byte) => data + String.fromCharCode(byte), '')
            );
            const mimeType = selected.toLowerCase().endsWith('.png') ? 'image/png' : 
                             selected.toLowerCase().endsWith('.gif') ? 'image/gif' : 
                             'image/jpeg';
            
            cropImgSrc.value = `data:${mimeType};base64,${base64}`;
            showCropper.value = true;
        }
    } catch (e) {
        console.error('Failed to open file dialog:', e);
    }
};

const handleCropConfirm = async (data) => {
    try {
        await configStore.uploadAvatar(data);
    } catch (e) {
        console.error('Failed to upload avatar in GeneralConfig:', e);
    }
    showCropper.value = false;
};

const presets = computed(() => configStore.settings.presets || []);
const allModels = computed(() => {
  const models = [];
  try {
    const providers = configStore.settings.providers || [];
    providers.forEach(p => {
      if (p.enabled && p.models) {
        p.models.forEach(m => {
          const modelId = typeof m === 'string' ? m : m.id;
          const modelName = typeof m === 'string' ? m : (m.name || m.id);
          models.push({
            id: modelId,
            providerName: p.name,
            displayName: `${p.name} - ${modelName}`,
            features: typeof m === 'string' ? [] : (m.features || [])
          });
        });
      }
    });
  } catch (e) {
    console.error('[GeneralConfig] Failed to compute allModels:', e);
  }
  return models;
});

const isPromptTemplateMatch = computed(() => {
  const lib = configStore.settings.promptLibrary || [];
  return lib.some(p => p.content === configStore.settings.defaultSystemPrompt);
});

const activeDropdown = ref(null);
const promptSearchQuery = ref('');

const currentModelName = computed(() => {
    const list = allModels.value || [];
    const model = list.find(m => m.id === configStore.settings.globalModelId);
    return model ? model.displayName : '选择模型';
});

const currentPresetName = computed(() => {
    const list = presets.value || [];
    const preset = list.find(p => p.id === configStore.settings.globalPresetId);
    return preset ? preset.name : '选择配置预设';
});

const currentPromptItem = computed(() => {
    const lib = configStore.settings.promptLibrary || [];
    return lib.find(p => p.content === configStore.settings.defaultSystemPrompt);
});

const filteredPrompts = computed(() => {
    const query = (promptSearchQuery.value || '').toLowerCase();
    const lib = configStore.settings.promptLibrary || [];
    return lib.filter(p => 
        (p.name || '').toLowerCase().includes(query) || 
        (p.description || '').toLowerCase().includes(query)
    );
});

const updateDefaultModel = (id) => {
  configStore.updateConfig({ globalModelId: id });
  activeDropdown.value = null;
};

const updateDefaultPreset = (id) => {
  configStore.updateConfig({ globalPresetId: id });
  activeDropdown.value = null;
};

const onSelectPromptTemplate = (promptId) => {
  if (!promptId) {
      activeDropdown.value = null;
      return;
  }
  const lib = configStore.settings.promptLibrary || [];
  const item = lib.find(p => p.id === promptId);
  if (item) {
    configStore.updateConfig({ defaultSystemPrompt: item.content });
  }
  activeDropdown.value = null;
};

// Click outside logic
const handleClickOutside = (e) => {
    if (activeDropdown.value && !e.target.closest('.modern-dropdown')) {
        activeDropdown.value = null;
    }
};

onMounted(() => {
    window.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
    window.removeEventListener('click', handleClickOutside);
});

const resolveAvatarSrc = (path) => {
    if (!path) return '';
    if (path.startsWith('data:') || path.startsWith('http')) return path;
    return convertFileSrc(path);
};
</script>

<template>
  <div class="general-config">
    <div class="config-list">
      <!-- 默认模型 -->
      <section class="config-card">
        <div class="card-header">
          <div class="icon-wrap" v-html="BOX_SVG"></div>
          <div class="title-wrap">
            <label>默认 AI 模型</label>
            <span class="hint">开启新对话时自动选中的模型</span>
          </div>
        </div>
        <div class="input-wrap">
          <div class="modern-dropdown" 
               :class="{ 'is-open': activeDropdown === 'model' }"
               @click.stop="activeDropdown = activeDropdown === 'model' ? null : 'model'">
            <div class="dropdown-trigger">
              <span class="item-name">{{ currentModelName }}</span>
              <span class="chev-icon" v-html="CHEVRON_DOWN_SVG"></span>
            </div>
            
            <Transition name="pop-up">
              <div v-if="activeDropdown === 'model'" class="dropdown-portal modern-scroll" @click.stop>
                <div 
                  v-for="m in allModels" 
                  :key="m.id" 
                  class="portal-item"
                  :class="{ 'active': configStore.settings.globalModelId === m.id }"
                  @click="updateDefaultModel(m.id)">
                  <div class="item-info">
                    <span class="item-name">{{ m.displayName }}</span>
                  </div>
                </div>
              </div>
            </Transition>
          </div>
        </div>
      </section>

      <!-- 用户头像 -->
      <section class="config-card">
        <div class="card-header">
          <div class="icon-wrap">👤</div>
          <div class="title-wrap">
            <label>用户头像</label>
            <span class="hint">显示在聊天的用户气泡旁</span>
          </div>
          <div class="avatar-preview-wrap" @click="handleAvatarUpload">
             <img v-if="configStore.userAvatarUrl" :src="configStore.userAvatarUrl" class="avatar-small" />
             <div v-else class="avatar-placeholder">+</div>
          </div>
        </div>
      </section>

      <!-- 默认预设 -->
      <section class="config-card">
        <div class="card-header">
          <div class="icon-wrap" v-html="SPARKLES_SVG"></div>
          <div class="title-wrap">
            <label>默认配置预设</label>
            <span class="hint">新对话将继承此预设的模型参数及提示词</span>
          </div>
        </div>
        <div class="input-wrap">
          <div class="modern-dropdown" 
               :class="{ 'is-open': activeDropdown === 'preset' }"
               @click.stop="activeDropdown = activeDropdown === 'preset' ? null : 'preset'">
            <div class="dropdown-trigger">
              <span class="item-name">{{ currentPresetName }}</span>
              <span class="chev-icon" v-html="CHEVRON_DOWN_SVG"></span>
            </div>
            
            <Transition name="pop-up">
              <div v-if="activeDropdown === 'preset'" class="dropdown-portal modern-scroll" @click.stop>
                <div 
                  v-for="p in presets" 
                  :key="p.id" 
                  class="portal-item"
                  :class="{ 'active': configStore.settings.globalPresetId === p.id }"
                  @click="updateDefaultPreset(p.id)">
                  <div class="item-info">
                    <span class="item-name">{{ p.name }}</span>
                  </div>
                </div>
              </div>
            </Transition>
          </div>
        </div>
      </section>

      <!-- 兜底提示词 -->
      <section class="config-card">
        <div class="card-header">
          <div class="icon-wrap" v-html="BRAIN_SVG"></div>
          <div class="title-wrap">
            <label>默认系统提示词</label>
            <span class="hint">新对话未配置提示词时，将自动激活此角色范式</span>
          </div>
        </div>
          
        <div class="input-wrap">
          <div class="modern-dropdown" 
               :class="{ 'is-open': activeDropdown === 'prompt' }"
               @click.stop="activeDropdown = activeDropdown === 'prompt' ? null : 'prompt'">
            <div class="dropdown-trigger">
              <span class="item-icon">{{ currentPromptItem?.icon || '✍️' }}</span>
              <span class="item-name">{{ currentPromptItem?.name || '手动编辑内容' }}</span>
              <span class="chev-icon" v-html="CHEVRON_DOWN_SVG"></span>
            </div>
            
            <Transition name="pop-up">
              <div v-if="activeDropdown === 'prompt'" class="dropdown-portal modern-scroll" @click.stop>
                <div class="dropdown-search">
                  <input v-model="promptSearchQuery" placeholder="搜索提示词范式..." />
                </div>
                <div 
                  class="portal-item" 
                  :class="{ 'active': !isPromptTemplateMatch }"
                  @click="onSelectPromptTemplate('')">
                  <span class="item-icon">✍️</span>
                  <span class="item-name">手动编辑 / 自定义内容</span>
                </div>
                <div class="divider-small"></div>
                <div 
                  v-for="p in filteredPrompts" 
                  :key="p.id" 
                  class="portal-item"
                  :class="{ 'active': configStore.settings.defaultSystemPrompt === p.content }"
                  @click="onSelectPromptTemplate(p.id)">
                  <span class="item-icon">{{ p.icon }}</span>
                  <div class="item-info">
                    <span class="item-name">{{ p.name }}</span>
                    <span class="item-desc">{{ p.description }}</span>
                  </div>
                </div>
              </div>
            </Transition>
          </div>
        </div>
      </section>



      <!-- 流式传输 -->
      <section class="config-card">
        <div class="card-header">
           <div class="icon-wrap" v-html="SPARKLES_SVG"></div>
           <div class="title-wrap">
             <label>流式响应模式</label>
             <span class="hint">开启后 AI 将像打字机一样逐字输出</span>
           </div>
        </div>
        <div class="input-wrap">
           <label class="toggle-switch">
             <input type="checkbox" v-model="configStore.settings.enableStream" @change="configStore.updateConfig({ enableStream: configStore.settings.enableStream })" />
             <span class="slider"></span>
           </label>
        </div>
      </section>
    </div>

    <!-- Image Cropper -->
    <ImageCropperModal 
      :show="showCropper"
      :imgSrc="cropImgSrc"
      :fixedBox="false"
      @close="showCropper = false"
      @confirm="handleCropConfirm"
    />
  </div>
</template>

<style scoped>
.general-config {
  display: flex;
  flex-direction: column;
  gap: 28px;
  animation: fadeIn 0.4s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.config-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-card {
  background: var(--bg-card);
  border: 1px solid var(--border-card);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  transition: all 0.3s ease;
}

.config-card:hover {
  background: var(--bg-glass-hover);
  border-color: var(--border-glass-bright);
}

.card-header {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.icon-wrap {
  width: 32px;
  height: 32px;
  background: var(--color-primary-bg);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary);
  flex-shrink: 0;
}

.icon-wrap :deep(svg) {
  width: 18px;
  height: 18px;
}

.input-wrap {
  width: 100%;
}

.modern-dropdown {
    position: relative;
    cursor: pointer;
    user-select: none;
}

.dropdown-trigger {
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--bg-glass-active);
    border: 1px solid var(--border-glass-bright);
    border-radius: 12px;
    padding: 10px 16px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.modern-dropdown:hover .dropdown-trigger {
    background: var(--bg-glass-hover);
    border-color: var(--color-primary-border);
}

.modern-dropdown.is-open .dropdown-trigger {
    background: var(--bg-input-focus);
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-alpha-10);
}

.dropdown-trigger .item-name { 
    font-size: 14px; 
    font-weight: 500; 
    color: var(--text-color-white); 
    flex: 1;
}
.dropdown-trigger .chev-icon { 
    color: var(--text-tertiary); 
    display: flex;
    transition: transform 0.3s;
}
.is-open .chev-icon { transform: rotate(180deg); }

.dropdown-portal {
    position: absolute;
    top: calc(100% + 10px);
    left: 0;
    right: 0;
    max-height: 320px;
    background: var(--bg-menu);
    backdrop-filter: blur(20px);
    border: 1px solid var(--border-glass-bright);
    border-radius: 16px;
    box-shadow: var(--shadow-main);
    z-index: 1000;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.dropdown-search {
    padding: 4px 8px 8px;
}
.dropdown-search input {
    width: 100%;
    background: var(--bg-input-dim);
    border: 1px solid var(--border-glass);
    border-radius: 8px;
    padding: 8px 12px;
    color: var(--text-color-white);
    font-size: 13px;
    outline: none;
}

.portal-item {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 10px;
    transition: all 0.2s;
    cursor: pointer;
}

.portal-item:hover {
    background: var(--bg-glass-hover);
}

.portal-item.active {
    background: var(--color-primary-bg);
    border-left: 3px solid var(--color-primary);
}

.portal-item .item-icon { font-size: 18px; padding-top: 2px; }
.item-info { display: flex; flex-direction: column; gap: 2px; flex: 1; }
.item-info .item-name { font-size: 14px; font-weight: 600; color: var(--text-color-white); }
.item-info .item-desc { font-size: 11px; color: var(--text-tertiary); line-height: 1.4; }

.divider-small {
    height: 1px;
    background: var(--border-glass);
    margin: 4px 8px;
}

.title-wrap {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}

.title-wrap label {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-color-white);
}

.hint {
  font-size: 12px;
  color: var(--text-tertiary);
  line-height: 1.4;
}

.pop-up-enter-active, .pop-up-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.pop-up-enter-from, .pop-up-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.95);
}

/* Toggle Switch Styles */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--bg-input-dim);
  transition: .4s;
  border-radius: 24px;
  border: 1px solid var(--border-glass);
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 2px;
  bottom: 2px;
  background-color: var(--text-tertiary);
  transition: .4s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: var(--color-primary-bg);
  border-color: var(--color-primary);
}

input:checked + .slider:before {
  transform: translateX(20px);
  background-color: var(--color-primary);
  box-shadow: 0 0 10px var(--color-primary);
}



/* Small Toggle Switch */
.toggle-switch.small {
  width: 36px;
  height: 20px;
}
.toggle-switch.small .slider:before {
  height: 14px;
  width: 14px;
  left: 2px;
  bottom: 2px;
}
.toggle-switch.small input:checked + .slider:before {
  transform: translateX(16px);
}

.expand-section-enter-active, .expand-section-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
  max-height: 200px;
  opacity: 1;
}
.expand-section-enter-from, .expand-section-leave-to {
  max-height: 0;
  opacity: 0;
  transform: translateY(-10px);
}

.avatar-preview-wrap {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  overflow: hidden;
  background: var(--bg-input);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-glass);
  margin-left: auto;
}
.avatar-small { width: 100%; height: 100%; object-fit: cover; }
.avatar-placeholder { color: var(--text-tertiary); font-size: 20px; }
</style>
