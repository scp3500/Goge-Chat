<script setup>
import { useConfigStore } from '../../stores/config';

defineProps({
  themes: Array
});

const configStore = useConfigStore();
const { settings } = configStore;

// 更新配置
const handleUpdate = async () => {
  try {
    await configStore.updateConfig(settings);
  } catch (error) {
    console.error('保存外观设置失败:', error);
  }
};

</script>

<template>
  <div class="config-content">
    <div class="config-group">
      <div class="group-header">
        <label>界面外观与显示</label>
      </div>
      <div class="settings-card">
        <div class="control-item">
          <label>聊天字号 ({{ settings.fontSize }}px)</label>
          <input 
            type="range" 
            v-model.number="settings.fontSize" 
            min="12" 
            max="24" 
            @input="handleUpdate" 
          />
        </div>
        <div class="control-item">
          <label>主题颜色</label>
          <div class="theme-grid">
            <div 
              v-for="t in themes" 
              :key="t" 
              class="theme-item" 
              :style="{ background: t }"
              :class="{ active: settings.themeColor === t }"
              @click="settings.themeColor = t; handleUpdate()"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-group { margin-bottom: 24px; }
.group-header { margin-bottom: 12px; }
.group-header label { display: block; font-size: 13px; color: #b5bac1; }
.settings-card { background: rgba(255, 255, 255, 0.03); border-radius: 12px; padding: 20px; border: 1px solid rgba(255, 255, 255, 0.05); }
.control-item { margin-bottom: 24px; }
.control-item label { display: block; font-size: 13px; color: #b5bac1; margin-bottom: 12px; }
.theme-grid { display: flex; gap: 12px; }
.theme-item { width: 36px; height: 36px; border-radius: 10px; cursor: pointer; border: 2px solid transparent; }
.theme-item.active { border-color: #5865f2; transform: scale(1.1); }
input[type="range"] { width: 100%; height: 6px; background: #131314; border-radius: 10px; appearance: none; outline: none; }
input[type="range"]::-webkit-slider-thumb { appearance: none; width: 18px; height: 18px; background: #fff; border-radius: 50%; cursor: pointer; }
</style>
