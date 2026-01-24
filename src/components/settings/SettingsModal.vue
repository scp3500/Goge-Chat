<script setup>
import { ref, computed } from 'vue';
import { useConfigService } from '../../services/configService';
import SidebarNav from './components/SidebarNav.vue';
import ProviderList from './components/ProviderList.vue';
import ModelConfig from './components/ModelConfig.vue';
import AppearanceConfig from './components/AppearanceConfig.vue';

// 1. 注入配置服务
const configStore = useConfigService();
const { settings } = configStore;

const emit = defineEmits(['close']);

// 2. 主题列表
const themes = ['#202124', '#1b1b1f', '#0f0f10', '#2c2c32'];

// 3. 布局控制变量
const activeCategory = ref('models');
const activeProviderId = ref('deepseek');

// 4. 供应商列表
const providers = ref([
  { id: 'deepseek', name: 'DeepSeek', icon: '🐋', status: 'on' },
  { id: 'openai', name: 'OpenAI', icon: '🤖', status: 'off' },
  { id: 'claude', name: 'Claude', icon: '🦜', status: 'off' },
  { id: 'gemini', name: 'Gemini', icon: '💎', status: 'off' },
  { id: 'ollama', name: 'Ollama', icon: '🦙', status: 'off' },
  { id: 'qwen', name: 'Qwen', icon: '🐑', status: 'off' }
]);

// 5. 切换供应商状态
const toggleProviderStatus = (providerId) => {
  const provider = providers.value.find(p => p.id === providerId);
  if (provider) {
    provider.status = provider.status === 'on' ? 'off' : 'on';
  }
};

// 当前选中的供应商名称
const activeProviderName = computed(() => {
  const p = providers.value.find(p => p.id === activeProviderId.value);
  return p ? p.name : '配置详情';
});

</script>

<template>
  <div class="settings-layout">
    <!-- 左侧导航 -->
    <SidebarNav v-model:activeCategory="activeCategory" />

    <!-- 中间供应商列表 -->
    <ProviderList 
      v-if="activeCategory === 'models'"
      :providers="providers"
      v-model:activeProviderId="activeProviderId"
      @toggleStatus="toggleProviderStatus"
    />

    <!-- 右侧详情面板 -->
    <main class="detail-panel modern-scroll">
      <div class="detail-container">
        
        <header class="detail-header">
          <div class="header-info">
            <h2>{{ activeCategory === 'models' ? activeProviderName : '界面外观与显示' }}</h2>
          </div>
        </header>

        <!-- 模型配置 -->
        <ModelConfig 
          v-if="activeCategory === 'models'"
          :providerName="activeProviderName"
          :providerId="activeProviderId"
          :settings="settings"
          :configStore="configStore"
        />

        <!-- 显示设置 -->
        <AppearanceConfig 
          v-else-if="activeCategory === 'appearance'"
          :settings="settings"
          :themes="themes"
          :configStore="configStore"
        />

        <!-- 占位符 -->
        <div v-else class="placeholder">
          <p>正在开发中...</p>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.settings-layout { display: flex; width: 100%; height: 100%; background: #131314; color: #e3e3e3; overflow: hidden; }
.detail-panel { flex: 1; background: #1e1f22; overflow-y: auto; }
.detail-container { max-width: 680px; margin: 0 auto; padding: 40px 24px; }
.detail-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 40px; }
.detail-header h2 { font-size: 20px; font-weight: 600; color: #fff; margin: 0; }
.placeholder { display: flex; align-items: center; justify-content: center; height: 200px; color: #888; }

/* 滚动条样式复原 */
.modern-scroll::-webkit-scrollbar { width: 8px; }
.modern-scroll::-webkit-scrollbar-track { background: transparent; }
.modern-scroll::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.05); border-radius: 10px; }
.modern-scroll::-webkit-scrollbar-thumb:hover { background: rgba(255, 255, 255, 0.1); }
</style>
