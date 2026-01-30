<script setup>
import { computed, ref, onMounted } from 'vue';
import { useConfigStore } from '../../stores/config';
import { PREBUILT_PROMPTS, DEFAULT_SYSTEM_PROMPT } from '../../constants/prompts';
import { BRAIN_SVG } from '../../constants/icons';

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

const showPrebuiltDropdown = ref(false);

const applyPrebuilt = (prompt) => {
  if (preset.value) {
    configStore.updatePreset(props.presetId, { systemPrompt: prompt.content });
    showPrebuiltDropdown.value = false;
  }
};

onMounted(() => {
  window.addEventListener('click', (e) => {
    if (!e.target.closest('.prebuilt-container-mini')) {
      showPrebuiltDropdown.value = false;
    }
  });
});

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

    <!-- System Prompt Section -->
    <div class="prompt-section">
      <div class="section-header">
        <div class="header-left">
          <h3 class="sub-title">系统提示词</h3>
          <span class="tag-outline">System Prompt</span>
        </div>
        
        <div class="prebuilt-container">
          <button class="text-btn" @click.stop="showPrebuiltDropdown = !showPrebuiltDropdown">
            <span class="icon" v-html="BRAIN_SVG"></span>
            <span>提示词库</span>
          </button>
          
          <Transition name="slide-fade">
            <div v-if="showPrebuiltDropdown" class="dropdown-menu modern-scroll">
              <div 
                v-for="p in configStore.settings.promptLibrary" 
                :key="`preset-dropdown-${p.id}`" 
                class="dropdown-item"
                @click="applyPrebuilt(p)"
              >
                <span class="item-icon">{{ p.icon }}</span>
                <span class="item-name">{{ p.name }}</span>
              </div>
            </div>
          </Transition>
        </div>
      </div>

      <div class="textarea-wrapper">
        <textarea 
          v-model="preset.systemPrompt" 
          :placeholder="configStore.settings.defaultSystemPrompt"
          @input="handleParamChange('systemPrompt', preset.systemPrompt)"
          class="modern-textarea modern-scroll"
          spellcheck="false"
        ></textarea>
        <!-- Placeholder overlay if empty to show it uses default -->
        <div v-if="!preset.systemPrompt?.trim()" class="placeholder-hint">
          <span class="hint-icon" v-html="BRAIN_SVG"></span>
          <span>将使用全局默认提示词...</span>
        </div>
      </div>
      <p class="section-hint">在此设定模型的角色和行为准则。留空则使用全局默认设置。</p>
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
  color: #e3e3e3;
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
  color: #fff;
  margin: 0;
  letter-spacing: -0.02em;
}

.sub-title {
  font-size: 15px;
  font-weight: 600;
  color: #fff;
  margin: 0;
}

.header-subtitle {
  font-size: 13px;
  color: #8a8d95;
}

/* Cards */
.config-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
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
  color: #fff;
}

.tag {
  font-size: 11px;
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  color: #aeb4bb;
  font-family: monospace;
}

.tag-outline {
  font-size: 11px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  padding: 2px 6px;
  border-radius: 4px;
  color: #aeb4bb;
  font-family: monospace;
}

.setting-desc {
  font-size: 12px;
  color: #6b7280;
  line-height: 1.4;
}

.divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.06);
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
  color: #818cf8;
  background: rgba(129, 140, 248, 0.1);
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
  background: rgba(255, 255, 255, 0.1);
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
  background: #818cf8;
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
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
  position: relative;
  z-index: 2;
  transition: transform 0.1s;
  margin-top: 0; /* Align thumb center with track */
}

.modern-range::-webkit-slider-thumb:hover {
  transform: scale(1.1);
}

.modern-input {
  background: #18191b;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 6px 10px;
  color: #fff;
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
  text-align: right;
  width: 100px;
}

.modern-input:focus {
  border-color: #818cf8;
}

/* Prompt Section */
.prompt-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.text-btn {
  background: transparent;
  border: none;
  color: #818cf8;
  font-size: 13px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 6px;
  transition: background 0.2s;
}

.text-btn:hover {
  background: rgba(129, 140, 248, 0.1);
}

.text-btn .icon :deep(svg) {
  width: 14px;
  height: 14px;
}

.prebuilt-container {
  position: relative;
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 8px;
  background: #1e1f22;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  width: 200px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  padding: 4px;
  z-index: 50;
  max-height: 300px;
  overflow-y: auto;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  cursor: pointer;
  border-radius: 6px;
  font-size: 13px;
  color: #d1d5db;
  transition: all 0.2s;
}

.dropdown-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #fff;
}

/* Textarea */
.textarea-wrapper {
  position: relative;
  border-radius: 12px;
  background: #111214;
  border: 1px solid rgba(255, 255, 255, 0.08);
  transition: border-color 0.2s, box-shadow 0.2s;
}

.textarea-wrapper:focus-within {
  border-color: rgba(129, 140, 248, 0.5);
  box-shadow: 0 0 0 2px rgba(129, 140, 248, 0.1);
}

.modern-textarea {
  width: 100%;
  height: 160px;
  background: transparent;
  border: none;
  color: #e5e7eb;
  padding: 16px;
  font-size: 14px;
  line-height: 1.6;
  resize: vertical;
  outline: none;
  font-family: inherit;
}

.placeholder-hint {
  position: absolute;
  top: 16px;
  left: 16px;
  color: rgba(255, 255, 255, 0.25);
  font-size: 13px;
  pointer-events: none;
  display: flex;
  align-items: center;
  gap: 6px;
}

.hint-icon :deep(svg) {
  width: 14px;
  height: 14px;
}

.section-hint {
  font-size: 12px;
  color: #6b7280;
  margin: -8px 0 0 0;
}

/* Advanced Section */
.advanced-section {
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.badge-dev {
  font-size: 10px;
  background: #374151;
  color: #9ca3af;
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
  color: #9ca3af;
  margin-bottom: 8px;
}

.fake-slider {
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
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
  background: #6b7280;
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
