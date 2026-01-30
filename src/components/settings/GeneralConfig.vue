<script setup>
import { computed } from 'vue';
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
            <label>全局兜底提示词</label>
            <span class="hint">当所选预设未配置提示词时，将自动使用此内容作为系统指令</span>
          </div>
        </div>
        <div class="input-wrap">
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
  color: #fff;
  margin: 0 0 4px 0;
}

.subtitle {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.4);
}

.config-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  transition: all 0.3s ease;
}

.config-card:hover {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.1);
}


.card-header {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.icon-wrap {
  width: 32px;
  height: 32px;
  background: rgba(129, 140, 248, 0.1);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #818cf8;
  flex-shrink: 0;
}

.icon-wrap :deep(svg) {
  width: 18px;
  height: 18px;
}

.title-wrap {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.title-wrap label {
  font-size: 14px;
  font-weight: 600;
  color: #fff;
}

.hint {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.3);
  line-height: 1.4;
}

.input-wrap select, .input-wrap textarea {
  width: 100%;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  color: #fff;
  padding: 10px 12px;
  font-family: inherit;
  font-size: 14px;
  outline: none;
  transition: all 0.2s;
}

.input-wrap select:focus, .input-wrap textarea:focus {
  border-color: #818cf8;
  background: rgba(0, 0, 0, 0.3);
}

.input-wrap textarea {
  resize: none;
  line-height: 1.6;
}
</style>
