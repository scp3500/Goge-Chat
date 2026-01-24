<script setup>
import { ref } from 'vue';
import { EYE_OPEN_SVG, EYE_CLOSED_SVG } from '../../../constants/icons.ts';

defineProps({
  providerName: String,
  providerId: String,
  settings: Object,
  configStore: Object
});

const showApiKey = ref(false);
</script>

<template>
  <div class="config-content">
    <div class="config-group">
      <div class="group-header">
        <label>{{ providerName }} API Key</label>
      </div>
      <div class="input-row">
        <div class="input-box">
          <input
            :type="showApiKey ? 'text' : 'password'"
            v-model="settings.apiKey"
            :placeholder="`在此输入您的 ${providerId === 'ollama' ? 'Ollama 地址' : 'sk-...'}`"
            @change="configStore.updateConfig(settings)"
          />
          <button
            class="eye-btn"
            @click="showApiKey = !showApiKey"
            v-html="showApiKey ? EYE_OPEN_SVG : EYE_CLOSED_SVG"
          ></button>
        </div>
      </div>
      <p class="hint">点击眼睛图标切换显示/隐藏密钥。</p>
    </div>
  </div>
</template>

<style scoped>
.config-group { margin-bottom: 24px; }
.group-header { margin-bottom: 12px; }
.group-header label { display: block; font-size: 13px; color: #b5bac1; }
.input-box { background: #131314; border: 1px solid rgba(255, 255, 255, 0.05); border-radius: 8px; padding: 12px 16px; display: flex; align-items: center; gap: 8px; }
.input-box input { background: transparent; border: none; color: #fff; outline: none; flex: 1; font-family: monospace; }
.eye-btn { background: transparent; border: none; color: #888; cursor: pointer; padding: 4px; border-radius: 4px; display: flex; align-items: center; justify-content: center; transition: color 0.2s; }
.eye-btn:hover { color: #fff; }
.hint { font-size: 12px; color: #80868b; margin-top: 8px; }
</style>
