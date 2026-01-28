<script setup>
import { computed } from 'vue';
import { useSettings } from '../../composables/useSettings';
import { useProviderConfig } from '../../composables/useProviderConfig';
import SidebarNav from './SidebarNav.vue';
import ProviderList from './ProviderList.vue';
import ModelConfig from './ModelConfig.vue';
import AppearanceConfig from './AppearanceConfig.vue';

// 使用 composables
const { configStore, settingsStore, currentCategoryTitle } = useSettings();
const { allProviders, toggleProvider } = useProviderConfig();

const emit = defineEmits(['close']);

// 主题列表
const themes = ['#202124', '#1b1b1f', '#0f0f10', '#2c2c32'];

// 提供商列表（转换为旧格式以兼容现有组件）
const providersForList = computed(() => 
  allProviders.value.map(p => ({
    id: p.id,
    name: p.name,
    icon: p.icon,
    status: p.enabled ? 'on' : 'off'
  }))
);

// 当前选中的供应商名称
const activeProviderName = computed(() => {
  const provider = allProviders.value.find(p => p.id === settingsStore.activeProviderId);
  return provider ? provider.name : '配置详情';
});

// 切换供应商状态
const handleToggleStatus = async (providerId) => {
  try {
    await toggleProvider(providerId);
  } catch (error) {
    console.error('切换提供商状态失败:', error);
  }
};

// 关闭设置
const handleClose = () => {
  settingsStore.closeSettings();
  emit('close');
};

</script>

<template>
  <div class="settings-layout">
    <!-- 左侧导航 -->
    <SidebarNav v-model:activeCategory="settingsStore.activeCategory" />

    <!-- 中间供应商列表 -->
    <ProviderList 
      v-if="settingsStore.activeCategory === 'models'"
      :providers="providersForList"
      v-model:activeProviderId="settingsStore.activeProviderId"
      @toggleStatus="handleToggleStatus"
    />

    <!-- 右侧详情面板 -->
    <main class="detail-panel modern-scroll">
      <div class="detail-container">
        
        <header class="detail-header">
          <div class="header-info">
            <h2>{{ settingsStore.activeCategory === 'models' ? activeProviderName : '界面外观与显示' }}</h2>
          </div>
        </header>

        <!-- 模型配置 -->
        <ModelConfig 
          v-if="settingsStore.activeCategory === 'models'"
          :providerId="settingsStore.activeProviderId"
        />

        <!-- 显示设置 -->
        <AppearanceConfig 
          v-else-if="settingsStore.activeCategory === 'appearance'"
          :themes="themes"
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
