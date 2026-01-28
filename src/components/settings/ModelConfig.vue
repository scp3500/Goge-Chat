<script setup>
import { ref, computed } from 'vue';
import { useProviderConfig } from '../../composables/useProviderConfig';
import { useConfigStore } from '../../stores/config';
import { EYE_OPEN_SVG, EYE_CLOSED_SVG } from '../../constants/icons.ts';

const props = defineProps({
  providerId: String
});

const configStore = useConfigStore();
const { currentProvider, updateCurrentProvider } = useProviderConfig();

const showApiKey = ref(false);

// 当前提供商的配置（响应式）
const providerConfig = computed(() => currentProvider.value);

// 更新 API Key
const handleApiKeyChange = async () => {
  if (providerConfig.value && providerConfig.value.apiKey !== undefined) {
    try {
      await updateCurrentProvider({ apiKey: providerConfig.value.apiKey });
    } catch (error) {
      console.error('保存 API Key 失败:', error);
    }
  }
};

// 更新搜索实例 URL
const handleSearchUrlChange = async () => {
  try {
    await configStore.updateConfig({ 
      searchInstanceUrl: configStore.settings.searchInstanceUrl 
    });
  } catch (error) {
    console.error('保存搜索实例地址失败:', error);
  }
};

</script>

<template>
  <div class="config-content" v-if="providerConfig">
    <div class="config-group">
      <div class="group-header">
        <label>{{ providerConfig.name }} API Key</label>
      </div>
      <div class="input-row">
        <div class="input-box">
          <input
            :type="showApiKey ? 'text' : 'password'"
            v-model="providerConfig.apiKey"
            :placeholder="`在此输入您的 ${providerId === 'ollama' ? 'Ollama 地址' : 'sk-...'}`"
            @change="handleApiKeyChange"
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

    <!-- Base URL (如果需要自定义) -->
    <div class="config-group" v-if="providerConfig.baseUrl">
      <div class="group-header">
        <label>API Base URL</label>
      </div>
      <div class="input-row">
        <div class="input-box">
          <input
            type="text"
            v-model="providerConfig.baseUrl"
            placeholder="例如: https://api.example.com"
            @change="handleApiKeyChange"
          />
        </div>
      </div>
      <p class="hint">大多数情况下保持默认即可。</p>
    </div>

    <!-- SearXNG Instance URL -->
    <div class="config-group">
      <div class="group-header">
        <label>SearXNG 实例地址 (可选)</label>
      </div>
      <div class="input-row">
        <div class="input-box">
          <input
            type="text"
            v-model="configStore.settings.searchInstanceUrl"
            placeholder="例如: https://searx.be (留空使用默认列表)"
            @change="handleSearchUrlChange"
          />
        </div>
      </div>
      <p class="hint">默认使用公共开源实例，无需配置即可联网。你也可以设置自己的私有 SearXNG 实例。</p>
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
