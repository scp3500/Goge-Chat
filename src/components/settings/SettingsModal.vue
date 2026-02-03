<script setup>
import { computed } from 'vue';
import { useSettings } from '../../composables/useSettings';
import { useProviderConfig } from '../../composables/useProviderConfig';
import SidebarNav from './SidebarNav.vue';
import ProviderList from './ProviderList.vue';
import ModelConfig from './ModelConfig.vue';
import PresetList from './PresetList.vue';
import PresetConfig from './PresetConfig.vue';
import SystemPromptConfig from './SystemPromptConfig.vue';
import AppearanceConfig from './AppearanceConfig.vue';
import GeneralConfig from './GeneralConfig.vue';
import ChatModeConfig from './ChatModeConfig.vue';
import ProfileConfig from './ProfileConfig.vue';
import DataConfig from './DataConfig.vue';


// 使用 composables
const { configStore, settingsStore, currentCategoryTitle } = useSettings();
const { allProviders, toggleProvider, updateProvidersOrder } = useProviderConfig();

const emit = defineEmits(['close']);

// 主题列表
const themes = [
  'var(--theme-preset-1)',
  'var(--theme-preset-2)',
  'var(--theme-preset-3)',
  'var(--theme-preset-4)'
];

// 提供商列表（转换为旧格式以兼容现有组件）
const providersForList = computed(() => 
  allProviders.value.map(p => ({
    id: p.id,
    name: p.name,
    icon: p.icon,
    status: p.enabled ? 'on' : 'off',
    isCustom: p.isCustom || false
  }))
);

// 当前选中的供应商名称
const activeProviderName = computed(() => {
  const provider = allProviders.value.find(p => p.id === settingsStore.activeProviderId);
  return provider ? provider.name : '配置详情';
});

// 当前选中的预设名称
const activePresetName = computed(() => {
  const preset = configStore.settings.presets.find(p => p.id === settingsStore.activePresetId);
  return preset ? preset.name : '预设配置';
});


// 选择供应商
const handleProviderSelect = (providerId) => {
  settingsStore.setActiveProvider(providerId);
};

// 切换供应商状态
const handleStatusToggle = async (providerId) => {
  try {
    await toggleProvider(providerId);
  } catch (error) {
    console.error('切换提供商状态失败:', error);
  }
};


const handleAddProvider = async () => {
  try {
    const newId = await allProviders.value.find(p => p.id === 'custom_new')?.id ?? await useProviderConfig().addCustomProvider();
    // 这里的 addCustomProvider 已经在 store 里处理了自动选中，所以这里不需要额外操作
    // 但为了保险起见，或者如果是从 composable 调用的，我们可以再次确认选中
  } catch (error) {
    console.error('添加提供商失败:', error);
  }
};
const handleRenameProvider = async (providerId, newName) => {
  try {
    const { updateProvider } = useProviderConfig();
    await updateProvider(providerId, { name: newName });
  } catch (error) {
    console.error('重命名供应商失败:', error);
  }
};

const handleDeleteProvider = async (providerId) => {

  try {
    const provider = allProviders.value.find(p => p.id === providerId);
    if (!provider) return;
    
    if (confirm(`确定要删除供应商 "${provider.name}" 吗？此操作不可恢复。`)) {
      const { removeProvider } = useProviderConfig();
      await removeProvider(providerId);
    }
  } catch (error) {
    console.error('删除供应商失败:', error);
  }
};


// --- 预设操作 ---
const handlePresetSelect = (presetId) => {
  settingsStore.setActivePreset(presetId);
};

const handleAddPreset = async () => {
  try {
    const newId = await configStore.addPreset('新配置');
    settingsStore.setActivePreset(newId);
  } catch (error) {
    console.error('添加预设失败:', error);
  }
};

const handleRenamePreset = async (presetId, newName) => {
  try {
    await configStore.updatePreset(presetId, { name: newName });
  } catch (error) {
    console.error('重命名预设失败:', error);
  }
};

const handleDeletePreset = async (presetId) => {
  try {
    const preset = configStore.settings.presets.find(p => p.id === presetId);
    if (!preset) return;
    
    if (confirm(`确定要删除配置 "${preset.name}" 吗？`)) {
      await configStore.removePreset(presetId);
    }
  } catch (error) {
    console.error('删除预设失败:', error);
  }
};

const handlePresetsReorder = async (newList) => {
  try {
    await configStore.handlePresetsReorder(newList);
  } catch (error) {
    console.error('更新预设排序失败:', error);
  }
};


const handleReorder = async (newSimpleProviders) => {
  console.log('[SettingsModal] Received reorder event:', newSimpleProviders.map(p => p.id).join(','));
  console.log('[SettingsModal] configStore is:', configStore);
  console.log('[SettingsModal] configStore.handleReorder exists?', typeof configStore.handleReorder);
  
  try {
    if (typeof configStore.handleReorder === 'function') {
      await configStore.handleReorder(newSimpleProviders);
      console.log('[SettingsModal] Reorder completed successfully');
    } else {
      console.error('[SettingsModal] configStore.handleReorder is NOT a function!');
    }
  } catch (error) {
    console.error('[SettingsModal] 更新排序失败:', error);
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
      :active-provider-id="settingsStore.activeProviderId"
      @update:active-provider-id="handleProviderSelect"
      @toggle-status="handleStatusToggle"
      @reorder="handleReorder"
      @add="handleAddProvider"
      @rename="handleRenameProvider"
      @delete="handleDeleteProvider"
    />

    <!-- 中间预设列表 -->
    <PresetList
      v-if="settingsStore.activeCategory === 'presets'"
      :presets="configStore.settings.presets"
      :active-preset-id="settingsStore.activePresetId"
      @update:active-preset-id="handlePresetSelect"
      @reorder="handlePresetsReorder"
      @add="handleAddPreset"
      @rename="handleRenamePreset"
      @delete="handleDeletePreset"
    />


    <!-- 右侧详情面板 -->
    <main class="detail-panel modern-scroll">
      <div class="detail-container">

        <header class="detail-header">
          <div class="header-info">
            <h2>{{ 
                settingsStore.activeCategory === 'profile' ? '个人资料设置' :
                settingsStore.activeCategory === 'models' ? activeProviderName : 
                settingsStore.activeCategory === 'presets' ? activePresetName : 
                settingsStore.activeCategory === 'prompts' ? '系统提示词管理' :
                settingsStore.activeCategory === 'appearance' ? '界面外观与显示' :
                settingsStore.activeCategory === 'chatmode' ? '沉浸式聊天模式' :
                settingsStore.activeCategory === 'data' ? '数据管理' :
                '通用项设置' 
            }}</h2>
          </div>

        </header>

        <!-- 个人资料配置 -->
        <ProfileConfig
          v-if="settingsStore.activeCategory === 'profile'"
        />

        <!-- 模型配置 -->
        <ModelConfig 
          v-else-if="settingsStore.activeCategory === 'models'"
          :providerId="settingsStore.activeProviderId"
        />

        <!-- 预设配置 -->
        <PresetConfig
          v-else-if="settingsStore.activeCategory === 'presets'"
          :presetId="settingsStore.activePresetId"
        />

        <!-- 系统提示词专区 -->
        <SystemPromptConfig
          v-else-if="settingsStore.activeCategory === 'prompts'"
          :presetId="settingsStore.activePresetId"
        />


        <!-- 显示设置 -->
        <AppearanceConfig 
          v-else-if="settingsStore.activeCategory === 'appearance'"
          :themes="themes"
        />

        <!-- 聊天模式设置 -->
        <ChatModeConfig 
          v-else-if="settingsStore.activeCategory === 'chatmode'"
        />

        <!-- 通用设置 -->
        <GeneralConfig
          v-else-if="settingsStore.activeCategory === 'general'"
        />

        <!-- 数据/记忆管理 -->
        <DataConfig
            v-else-if="settingsStore.activeCategory === 'data'"
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
.settings-layout { display: flex; width: 100%; height: 100%; background: var(--bg-sidebar); color: var(--text-color); overflow: hidden; }
.detail-panel { flex: 1; background: var(--bg-chat-island); overflow-y: auto; }
.detail-container { max-width: 680px; margin: 0 auto; padding: 40px 24px; }
.detail-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 40px; }
.detail-header h2 { font-size: 20px; font-weight: 600; color: var(--text-color-white); margin: 0; }
.placeholder { display: flex; align-items: center; justify-content: center; height: 200px; color: var(--text-tertiary); }

/* 滚动条样式复原 */
.modern-scroll::-webkit-scrollbar { width: 8px; }
.modern-scroll::-webkit-scrollbar-track { background: transparent; }
.modern-scroll::-webkit-scrollbar-thumb { background: var(--bg-glass-active); border-radius: 10px; }
.modern-scroll::-webkit-scrollbar-thumb:hover { background: var(--bg-glass-hover); }
</style>
