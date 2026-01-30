<script setup>
import { computed, ref } from 'vue';
import { useConfigStore } from '../../stores/config';
import { BRAIN_SVG, BOX_SVG, SPARKLES_SVG } from '../../constants/icons';

const configStore = useConfigStore();

const allModels = computed(() => {
  const models = [];
  configStore.settings.providers.forEach(p => {
    if (p.enabled) {
      p.models.forEach(modelId => {
        models.push({
          id: modelId,
          providerName: p.name,
          displayName: `${p.name} - ${modelId}`
        });
      });
    }
  });
  return models;
});

const presets = computed(() => configStore.settings.presets || []);

const updateDefaultModel = (e) => {
  configStore.updateConfig({ globalModelId: e.target.value });
};

const updateDefaultPreset = (e) => {
  configStore.updateConfig({ globalPresetId: e.target.value });
};

const updateDefaultPrompt = (val) => {
  configStore.updateConfig({ defaultSystemPrompt: val });
};

const isPromptTemplateMatch = computed(() => {
  return configStore.settings.promptLibrary.some(p => p.content === configStore.settings.defaultSystemPrompt);
});

const onSelectPromptTemplate = (e) => {
  const val = e.target.value;
  if (!val) return;
  const item = configStore.settings.promptLibrary.find(p => p.id === val);
  if (item) {
    configStore.updateConfig({ defaultSystemPrompt: item.content });
  }
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
          <select :value="configStore.settings.globalModelId" @change="updateDefaultModel">
            <option v-for="m in allModels" :key="m.id" :value="m.id">{{ m.displayName }}</option>
          </select>
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
          <select :value="configStore.settings.globalPresetId" @change="updateDefaultPreset">
            <option v-for="p in presets" :key="p.id" :value="p.id">{{ p.name }}</option>
          </select>
        </div>
      </section>

      <!-- 兜底提示词 -->
      <section class="config-card">
        <div class="card-header">
          <div class="icon-wrap" v-html="BRAIN_SVG"></div>
          <div class="title-wrap">
            <label>默认系统提示词</label>
            <span class="hint">当新对话未自定义提示词时，将自动使用此内容作为系统指令</span>
          </div>
        </div>
        <div class="input-wrap">
          <select @change="onSelectPromptTemplate">
            <option value="" :selected="!isPromptTemplateMatch">-- 手动编辑 / 自定义内容 --</option>
            <option 
              v-for="p in configStore.settings.promptLibrary" 
              :key="p.id" 
              :value="p.id"
              :selected="configStore.settings.defaultSystemPrompt === p.content"
            >
              {{ p.icon }} {{ p.name }}
            </option>
          </select>
        </div>
        <div class="input-wrap mt-2">
          <textarea 
            v-model="configStore.settings.defaultSystemPrompt" 
            @change="updateDefaultPrompt($event.target.value)"
            rows="4"
            placeholder="例如：你是一个简洁专业的 AI 助手..."
          ></textarea>
        </div>
      </section>
    </div>
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

.header-section h2 {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color-white);
  margin: 0 0 4px 0;
}

.subtitle {
  font-size: 13px;
  color: var(--text-color);
  opacity: 0.4;
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

.library-select-container {
  position: relative;
  margin-left: auto;
}

.mini-tool-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--color-primary-bg);
  border: 1px solid var(--color-primary-border);
  border-radius: 8px;
  padding: 4px 10px;
  color: var(--color-primary-light);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.mini-tool-btn:hover {
  background: var(--color-primary-border);
}

.mini-tool-btn .icon :deep(svg) {
  width: 12px;
  height: 12px;
}

.library-dropdown-menu {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  width: 180px;
  max-height: 200px;
  background: var(--bg-main);
  border: 1px solid var(--border-glass-bright);
  border-radius: 12px;
  box-shadow: var(--shadow-main);
  z-index: 100;
  overflow-y: auto;
  padding: 6px;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.dropdown-item:hover {
  background: var(--bg-glass-hover);
}

.item-icon { font-size: 16px; }
.item-name { font-size: 13px; color: var(--text-color-white); }

.pop-up-enter-active, .pop-up-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.pop-up-enter-from, .pop-up-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.95);
}

.title-wrap {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.title-wrap label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-color-white);
}

.hint {
  font-size: 12px;
  color: var(--text-color);
  opacity: 0.35;
  line-height: 1.4;
}

.input-wrap select, .input-wrap textarea {
  width: 100%;
  background: var(--bg-input-dim);
  border: 1px solid var(--border-card);
  border-radius: 10px;
  color: var(--text-color-white);
  padding: 10px 12px;
  font-family: inherit;
  font-size: 14px;
  outline: none;
  transition: all 0.2s;
}

.input-wrap select:focus, .input-wrap textarea:focus {
  border-color: var(--color-primary);
  background: var(--bg-input-focus);
}

.input-wrap textarea {
  resize: none;
  line-height: 1.6;
}

.mt-2 {
  margin-top: 12px;
}
</style>
