<script setup>
import { computed, ref } from 'vue';
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
  <div class="preset-config-container" v-if="preset">
    <!-- Header / Title -->
    <div class="config-header">
      <h2 class="section-title">参数配置</h2>
      <div class="header-subtitle">调整模型的回复风格与限制</div>
    </div>

    <!-- Main Card Settings -->
    <div class="config-card">
      <!-- Temperature Slider -->
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">
            <span>回复随机度</span>
            <span class="tag">Temperature</span>
          </div>
          <div class="setting-desc">值越大回复越具有创造性，值越小回复越严谨。</div>
        </div>
        <div class="control-wrapper">
          <div class="slider-display">{{ preset.temperature }}</div>
          <input 
            type="range" 
            v-model.number="preset.temperature" 
            min="0" 
            max="2" 
            step="0.1" 
            class="modern-range"
            :style="{ '--progress': (preset.temperature / 2) * 100 + '%' }"
            @input="handleParamChange('temperature', preset.temperature)"
          />
        </div>
      </div>

      <div class="divider"></div>

      <!-- Max Tokens Input -->
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">
            <span>最大回复长度</span>
            <span class="tag">Max Tokens</span>
          </div>
          <div class="setting-desc">限制单次交互的最大生成 Token 数量。</div>
        </div>
        <div class="control-wrapper">
          <input 
            type="number" 
            v-model.number="preset.maxTokens" 
            placeholder="无限制"
            @change="handleParamChange('maxTokens', preset.maxTokens)"
            class="modern-input number-input"
          />
        </div>
      </div>
    </div>




    <!-- Advanced Settings (Collapsed/Visual Only for now) -->
    <div class="advanced-section">
      <div class="section-header">
        <h3 class="sub-title">高级选项</h3>
        <span class="badge-dev">开发中</span>
      </div>
      <div class="advanced-grid dimmed">
        <div class="adv-item">
          <label>Top P</label>
          <div class="fake-slider"></div>
        </div>
        <div class="adv-item">
          <label>Presence Penalty</label>
          <div class="fake-slider"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.preset-config-container {
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding: 8px 4px;
  color: var(--text-color);
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* Typography & Headers */
.config-header {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 8px;
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-color-white);
  margin: 0;
  letter-spacing: -0.02em;
}

.sub-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-color-white);
  margin: 0;
}

.header-subtitle {
  font-size: 13px;
  color: var(--text-secondary);
}

/* Cards */
.config-card {
  background: var(--bg-glass);
  border: 1px solid var(--border-glass);
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 24px;
}

.setting-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

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

.tag-outline {
  font-size: 11px;
  border: 1px solid var(--border-glass-bright);
  padding: 2px 6px;
  border-radius: 4px;
  color: var(--text-tertiary);
  font-family: monospace;
}

.setting-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  line-height: 1.4;
}

.divider {
  height: 1px;
  background: var(--border-card);
  margin: 0 -20px;
}

/* Controls */
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
  margin-top: 0; /* Align thumb center with track */
}

.modern-range::-webkit-slider-thumb:hover {
  transform: scale(1.1);
}

.modern-input {
  background: var(--bg-input) !important;
  border: 1px solid var(--border-glass);
  border-radius: 6px;
  padding: 6px 10px;
  color: var(--text-color-white) !important;
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
  text-align: right;
  width: 100px;
}

.modern-input:focus {
  border-color: var(--color-primary);
}



/* Advanced Section */
.advanced-section {
  padding-top: 16px;
  border-top: 1px solid var(--border-glass);
}

.badge-dev {
  font-size: 10px;
  background: var(--bg-glass-active);
  color: var(--text-tertiary);
  padding: 2px 6px;
  border-radius: 4px;
}

.advanced-grid {
  margin-top: 16px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.dimmed {
  opacity: 0.4;
  pointer-events: none;
  filter: grayscale(100%);
}

.adv-item label {
  display: block;
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.fake-slider {
  height: 6px;
  background: var(--bg-glass-active);
  border-radius: 3px;
  position: relative;
}

.fake-slider::after {
  content: '';
  position: absolute;
  left: 30%;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 12px;
  height: 12px;
  background: var(--text-tertiary);
  border-radius: 50%;
}

/* Transitions */
.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: all 0.2s ease;
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateY(-5px);
  opacity: 0;
}
</style>
