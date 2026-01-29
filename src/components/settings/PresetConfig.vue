<script setup>
import { computed } from 'vue';
import { useConfigStore } from '../../stores/config';

const props = defineProps({

  presetId: String
});

const configStore = useConfigStore();

const preset = computed(() => {
  return configStore.settings.presets.find(p => p.id === props.presetId);
});

const handleParamChange = (key, value) => {
  if (preset.value) {
    configStore.updatePreset(props.presetId, { [key]: value });
  }
};

// UI Icons
const TWEAK_ICON_SVG = `<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M4 21v-7m0-4V3m8 18v-9m0-4V3m8 18v-5m0-4V3M2 14h4m4-8h4m4 8h4"/></svg>`;
</script>

<template>
  <div class="preset-config-pane" v-if="preset">
    <!-- Parameters Section -->
    <section class="pane-section">
      <div class="pane-header-row">
        <span class="pane-label">参数设置</span>
        <button class="ghost-tool-btn" v-html="TWEAK_ICON_SVG"></button>
      </div>
      <div class="params-grid">
        <div class="param-item">
          <div class="param-top">
            <label>回复随机度 (Temperature)</label>
            <span>{{ preset.temperature }}</span>
          </div>
          <input 
            type="range" 
            v-model.number="preset.temperature" 
            min="0" 
            max="2" 
            step="0.1" 
            @input="handleParamChange('temperature', preset.temperature)"
          />
        </div>
        <div class="param-item">
          <div class="param-top">
            <label>单次回复最大长度 (Max Tokens)</label>
            <span>{{ preset.maxTokens }}</span>
          </div>
          <input 
            type="number" 
            v-model.number="preset.maxTokens" 
            placeholder="默认"
            @change="handleParamChange('maxTokens', preset.maxTokens)"
            class="param-number-input"
          />
        </div>
      </div>
    </section>

    <!-- System Prompt Section -->
    <section class="pane-section">
      <div class="pane-header-row">
        <span class="pane-label">系统提示词 (System Prompt)</span>
      </div>
      <div class="prompt-textarea-wrapper">
        <textarea 
          v-model="preset.systemPrompt" 
          placeholder="给模型定义一个角色或任务..."
          @change="handleParamChange('systemPrompt', preset.systemPrompt)"
          class="prompt-textarea modern-scroll"
        ></textarea>
      </div>
      <div class="pane-hint">系统提示词会在对话开始前发送给模型，用于设定其行为模式。</div>
    </section>

    <!-- Advanced Configuration Placeholder -->
    <section class="pane-section">
      <div class="pane-header-row">
        <span class="pane-label">高级配置</span>
      </div>
      <div class="advanced-grid">
        <div class="param-item disabled">
          <div class="param-top">
            <label>Top P (正在开发)</label>
          </div>
          <input type="range" disabled value="1" />
        </div>
        <div class="param-item disabled">
          <div class="param-top">
            <label>Presence Penalty (正在开发)</label>
          </div>
          <input type="range" disabled value="0" />
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.preset-config-pane {
  display: flex;
  flex-direction: column;
  gap: 32px;
  color: #d1d1d1;
}

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

.pane-label {
  font-size: 14px;
  font-weight: 700;
  color: #fff;
  letter-spacing: 0.2px;
}

.ghost-tool-btn { background: transparent; border: none; padding: 4px; border-radius: 4px; color: #666; cursor: pointer; display: flex; width: 24px; height: 24px; }
.ghost-tool-btn:hover { background: rgba(255,255,255,0.1); color: #ccc; }

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

.prompt-textarea-wrapper {
  background: #111214;
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  padding: 4px;
}

.prompt-textarea {
  width: 100%;
  height: 120px;
  background: transparent;
  border: none;
  color: #fff;
  font-size: 14px;
  line-height: 1.6;
  resize: vertical;
  outline: none;
  padding: 8px 12px;
}

.pane-hint {
  font-size: 12px;
  color: #4a4d52;
  margin: 0;
}

.advanced-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  opacity: 0.5;
}

.param-item.disabled {
  pointer-events: none;
}
</style>
