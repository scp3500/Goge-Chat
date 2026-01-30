<script setup>
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { useConfigStore } from '../../stores/config';
import { useUIStore } from '../../stores/ui';
import { 
  SPARKLES_SVG, 
  CHEVRON_DOWN_SVG,
  CHECK_SVG,
  PLUS_SVG,
  SETTINGS_SVG
} from '../../constants/icons';
import NamePresetModal from '../modals/NamePresetModal.vue';
import { useSettingsStore } from '../../stores/settings';

const props = defineProps({
  menuId: {
    type: String,
    default: 'preset-selector'
  }
});

const configStore = useConfigStore();
const uiStore = useUIStore();
const settingsStore = useSettingsStore();
const isOpen = computed(() => uiStore.isMenuOpen(props.menuId));

const showNameModal = ref(false);
const handleNewPreset = () => {
  showNameModal.value = true;
};

const onCreatePreset = async (name) => {
  const newId = await configStore.addPreset(name);
  configStore.updateConfig({ defaultPresetId: newId });
  showNameModal.value = false;
  closeDropdown();
  
  // 自动跳转到设置页面进行深入配置
  settingsStore.openSettings('presets');
};

// 切换下拉框
const toggleDropdown = () => {
  uiStore.toggleMenu(props.menuId);
};

// 关闭下拉框
const closeDropdown = () => {
  uiStore.setActiveMenu(null);
};

// 点击外部关闭
const selectorRef = ref(null);
const handleClickOutside = (event) => {
  if (selectorRef.value && !selectorRef.value.contains(event.target)) {
    closeDropdown();
  }
};

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
});

// 获取当前选中的预设
const currentPreset = computed(() => {
    return configStore.settings.presets.find(p => p.id === configStore.settings.defaultPresetId) || configStore.settings.presets[0];
});

// 选择预设
const selectPreset = (presetId) => {
  configStore.updateConfig({
    defaultPresetId: presetId
  });
  closeDropdown();
};

</script>

<template>
  <div class="preset-selector" ref="selectorRef">
    <!-- 选择器按钮 -->
    <button 
      class="selector-btn" 
      :class="{ 'active': isOpen }"
      @click.stop="toggleDropdown" 
      @mousedown.stop 
    >
      <span class="icon" v-html="SPARKLES_SVG"></span>
      <span class="preset-name">{{ currentPreset?.name || '默认配置' }}</span>
      <span class="chevron" v-html="CHEVRON_DOWN_SVG"></span>
    </button>

    <!-- 下拉面板 -->
    <Transition name="fade-slide">
      <div v-if="isOpen" class="dropdown-panel pop-down" @mousedown.stop>
        <div class="panel-header">配置预设</div>
        
        <div class="presets-list custom-scrollbar">
          <div 
            v-for="preset in configStore.settings.presets" 
            :key="preset.id"
            class="preset-item"
            :class="{ 'selected': configStore.settings.defaultPresetId === preset.id }"
            @click="selectPreset(preset.id)"
          >
            <div class="preset-info">
              <span class="preset-text">{{ preset.name }}</span>
              <span class="preset-params">T: {{ preset.temperature }} | M: {{ preset.maxTokens }}</span>
            </div>
            <div class="check-icon" v-if="configStore.settings.defaultPresetId === preset.id" v-html="CHECK_SVG"></div>
          </div>
        </div>

        <div class="panel-actions">
          <button class="action-btn" @click="handleNewPreset">
             <span class="icon" v-html="PLUS_SVG"></span>
             <span>新建配置</span>
          </button>
          <button class="action-btn" @click="settingsStore.openSettings('presets'); closeDropdown()">
             <span class="icon" v-html="SETTINGS_SVG"></span>
             <span>管理配置</span>
          </button>
        </div>
      </div>
    </Transition>

    <NamePresetModal 
      :show="showNameModal" 
      @close="showNameModal = false" 
      @confirm="onCreatePreset"
    />
  </div>
</template>

<style scoped>
.preset-selector {
  position: relative;
  display: inline-block;
  -webkit-app-region: no-drag;
}

.selector-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: none;
  border-radius: 99px;
  padding: 4px 12px;
  color: var(--text-color);
  opacity: 0.8;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  height: 28px;
}

.selector-btn:hover {
  background: var(--bg-glass-hover);
  color: var(--text-color-white);
  opacity: 1;
}

.selector-btn.active {
  background: var(--bg-glass-active);
  color: var(--text-color-white);
  opacity: 1;
}

.icon :deep(svg) {
  width: 14px;
  height: 14px;
}

.preset-name {
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.chevron {
  display: flex;
  align-items: center;
  transition: transform 0.2s;
  opacity: 0.6;
}

.selector-btn.active .chevron {
  transform: rotate(180deg);
}

.dropdown-panel {
  position: absolute;
  top: calc(100% + 12px);
  right: 0;
  width: 220px;
  background: var(--bg-menu);
  backdrop-filter: blur(40px) saturate(200%);
  border: 1px solid var(--border-menu);
  border-radius: 16px;
  box-shadow: var(--shadow-main);
  z-index: 1000;
  overflow: hidden;
  padding: 8px;
}

.panel-header {
  font-size: 11px;
  color: var(--text-dim);
  padding: 8px 12px 4px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.presets-list {
  max-height: 300px;
  overflow-y: auto;
}

.preset-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  cursor: pointer;
  border-radius: 8px;
  margin: 2px 0;
  transition: all 0.2s;
}

.preset-item:hover {
  background: var(--bg-glass-hover);
}

.preset-item.selected {
  background: var(--bg-menu-active);
}

.preset-info {
  display: flex;
  flex-direction: column;
}

.preset-text {
  font-size: 13px;
  color: var(--text-color-white);
}

.preset-item.selected .preset-text {
  color: var(--color-menu-active);
  font-weight: 500;
}

.preset-params {
  font-size: 10px;
  color: #666;
  font-family: monospace;
}

.check-icon :deep(svg) {
  width: 14px;
  height: 14px;
  color: var(--color-menu-active);
}

.panel-actions {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-top: 4px;
  padding-top: 4px;
  border-top: 1px solid var(--border-menu);
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: transparent;
  border: none;
  border-radius: 8px;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: var(--bg-glass-hover);
  color: var(--text-color-white);
}

.action-btn .icon :deep(svg) {
  width: 14px;
  height: 14px;
  opacity: 0.7;
}

.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: var(--bg-glass-active); border-radius: 10px; }

.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.fade-slide-enter-from,
.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.98);
}
</style>
