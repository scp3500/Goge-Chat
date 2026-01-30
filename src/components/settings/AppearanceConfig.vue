<script setup>
import { computed } from 'vue';
import { useConfigStore } from '../../stores/config';

defineProps({
  themes: Array
});

const configStore = useConfigStore();

// 更新配置
const handleUpdate = async () => {
  try {
    await configStore.updateConfig(configStore.settings);
  } catch (error) {
    console.error('保存外观设置失败:', error);
  }
};

const designStyles = [
  // Dark Themes
  { id: 'dark', name: 'VS Code Dark+', bg: '#1E1F20', mode: 'dark', defaultFor: 'dark' },
  { id: 'nord', name: '北欧冰雪 (Nord)', bg: '#2E3440', mode: 'dark' },
  { id: 'one-dark', name: 'One Dark', bg: '#282C34', mode: 'dark' },
  { id: 'catppuccin', name: '莫卡 (Catppuccin)', bg: '#1E1E2E', mode: 'dark' },
  { id: 'midnight', name: '极昼 (Midnight)', bg: '#000000', mode: 'dark' },
  { id: 'ocean', name: '深海蓝 (Ocean)', bg: '#1E293B', mode: 'dark' },
  
  // Light Themes
  { id: 'light', name: '亮洁模式', bg: '#FFFFFF', mode: 'light', defaultFor: 'light' },
  { id: 'wechat', name: '微信经典 (WeChat)', bg: '#f3f3f3', mode: 'light' },
  { id: 'solarized-light', name: '象牙白 (Solarized)', bg: '#FDF6E3', mode: 'light' },
  { id: 'sakura', name: '樱花粉 (Sakura)', bg: '#FFF5F7', mode: 'light' },
  { id: 'frieren', name: '芙莉莲 (Frieren)', bg: '#f9fbfd', mode: 'light' },
  { id: 'twilight', name: '黄昏 (Twilight)', bg: '#fffdfc', mode: 'light' },
  { id: 'ghibli', name: '吉卜力 (Ghibli)', bg: '#fcfaf2', mode: 'light' },
  { id: 'violet', name: '薇尔莉特 (Violet)', bg: '#fdfdfb', mode: 'light' },
  { id: 'miku', name: '初音未来 (Miku)', bg: '#f5fafd', mode: 'light' },
  { id: 'clannad', name: '大团子 (Clannad)', bg: '#f0f7ff', mode: 'light' },
  { id: 'wa2', name: '冬马和纱 (WA2)', bg: '#f5f8fa', mode: 'light' },

  // Dark/Gaming Themes
  { id: 'eva-01', name: '初号机 (Eva-01)', bg: '#191421', mode: 'dark' },
  { id: 'cyberpunk', name: '夜之城 (Cyberpunk)', bg: '#0b0c15', mode: 'dark' },
  { id: 'terraria', name: '泰拉瑞亚 (Terraria)', bg: '#2d1e12', mode: 'dark' },
  { id: 'rdr2', name: '大镖客2 (RDR2)', bg: '#1a1512', mode: 'dark' },
  { id: 'sts-ironclad', name: '铁甲战士 (StS)', bg: '#1a0a0a', mode: 'dark' },
  { id: 'sts-silent', name: '静默猎手 (StS)', bg: '#0d1a0d', mode: 'dark' },
  { id: 'sts-defect', name: '故障机器 (StS)', bg: '#0a0d1a', mode: 'dark' },
  { id: 'sts-watcher', name: '观者 (StS)', bg: '#1a0a1a', mode: 'dark' },
  { id: 'hollow-knight', name: '空洞骑士 (Hallownest)', bg: '#0c0f12', mode: 'dark' },
  { id: 'wukong', name: '天命人 (Wukong)', bg: '#0f0d0b', mode: 'dark' },
  { id: 'steins-gate', name: '凤凰院凶真 (S;G)', bg: '#1c1c1c', mode: 'dark' },
  { id: 'fate', name: '圣杯战争 (Fate)', bg: '#0d1117', mode: 'dark' },
  { id: 'danganronpa', name: '希望与绝望 (Danganronpa)', bg: '#000000', mode: 'dark' },
  { id: 'miku-dark', name: '深夜未来 (Miku Dark)', bg: '#0a1a1a', mode: 'dark' }
];

const applyThemePreset = (styleId) => {
  // 直接通过 Store 访问，确保最准确的响应性
  if (configStore.settings.theme === 'light') {
    configStore.settings.lightThemeId = styleId;
  } else {
    configStore.settings.darkThemeId = styleId;
  }
  handleUpdate();
};

const filteredStyles = computed(() => {
  // 确保 configStore.settings 存在
  if (!configStore.settings) return [];
  return designStyles.filter(s => s.mode === configStore.settings.theme);
});

</script>

<template>
  <div class="config-content">
    <div class="config-group">
      <div class="group-header">
        <label>界面外观与显示</label>
      </div>
      <div class="settings-card">
        <!-- 1. 模式切换器 -->
        <div class="control-item">
          <label>当前模式</label>
          <div class="theme-mode-switch">
             <button 
               class="mode-btn" 
               :class="{ active: configStore.settings.theme === 'dark' }"
               @click="configStore.settings.theme = 'dark'; handleUpdate()"
             >深色模式 (Dark)</button>
             <button 
               class="mode-btn" 
               :class="{ active: configStore.settings.theme === 'light' }"
               @click="configStore.settings.theme = 'light'; handleUpdate()"
             >浅色模式 (Light)</button>
          </div>
        </div>

        <!-- 2. 主题选择器 (上下文感知) -->
        <div class="control-item">
          <label>
            为 {{ configStore.settings.theme === 'light' ? '浅色模式' : '深色模式' }} 选择一个喜欢的风格
            <span class="sub-label">（每个模式都可以独立保存一个预设）</span>
          </label>
          <div class="design-grid">
             <!-- 只显示当前模式适用的主题 -->
             <div 
               v-for="style in filteredStyles" 
               :key="style.id" 
               class="design-card"
               :class="{ active: (configStore.settings.theme === 'light' ? configStore.settings.lightThemeId : configStore.settings.darkThemeId) === style.id }"
               @click="applyThemePreset(style.id)"
             >
               <div class="design-preview" :style="{ background: style.bg }">
                 <div class="preview-dot" v-if="(configStore.settings.theme === 'light' ? configStore.settings.lightThemeId : configStore.settings.darkThemeId) === style.id"></div>
               </div>
               <span class="design-name">{{ style.name }}</span>
               <span class="design-tag" v-if="style.defaultFor === configStore.settings.theme">默认</span>
             </div>
          </div>
        </div>

        <div class="control-item">
          <label>聊天字号 ({{ configStore.settings.fontSize }}px)</label>
          <input 
            type="range" 
            v-model.number="configStore.settings.fontSize" 
            min="12" 
            max="24" 
            @input="handleUpdate" 
          />
        </div>
        <div class="control-item">
          <label>行高比例 ({{ configStore.settings.lineRatio }})</label>
          <input 
            type="range" 
            v-model.number="configStore.settings.lineRatio" 
            min="1.2" 
            max="2.5" 
            step="0.1"
            @input="handleUpdate" 
          />
        </div>
        <div class="control-item">
          <label>滚动条宽度 ({{ configStore.settings.scrollbarWidth }}px)</label>
          <input 
            type="range" 
            v-model.number="configStore.settings.scrollbarWidth" 
            min="4" 
            max="16" 
            step="1"
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
              :class="{ active: configStore.settings.themeColor === t }"
              @click="configStore.settings.themeColor = t; handleUpdate()"
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
.group-header label { display: block; font-size: 13px; color: var(--text-secondary); opacity: 0.7; }
.settings-card { background: var(--bg-card); border-radius: 12px; padding: 20px; border: 1px solid var(--border-card); }
.control-item { margin-bottom: 24px; }
.control-item label { display: block; font-size: 13px; color: var(--text-color); opacity: 0.7; margin-bottom: 12px; }

.theme-mode-switch {
    display: flex;
    background: var(--bg-input-dim);
    padding: 4px;
    border-radius: 8px;
    gap: 4px;
    border: 1px solid var(--border-glass);
}
.mode-btn {
    flex: 1;
    padding: 6px;
    font-size: 13px;
    border: none;
    background: transparent;
    color: var(--text-color);
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.2s;
}
.mode-btn.active {
    color: var(--text-color-white);
    background: var(--bg-glass-active);
    box-shadow: var(--shadow-main);
}

.theme-grid { display: flex; gap: 12px; }
.theme-item { width: 36px; height: 36px; border-radius: 10px; cursor: pointer; border: 2px solid transparent; transition: all 0.2s; }
.theme-item.active { border-color: var(--color-primary); transform: scale(1.1); }

.design-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
}

.design-card {
    background: var(--bg-input-dim);
    padding: 10px;
    border-radius: 10px;
    border: 1px solid var(--border-card);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    transition: all 0.2s;
}

.design-card:hover {
    background: var(--bg-glass-hover);
    border-color: var(--border-glass-bright);
}

.design-card.active {
    border-color: var(--color-primary);
    background: var(--color-primary-bg);
}

.design-preview {
    width: 100%;
    height: 48px;
    border-radius: 6px;
    border: 1px solid var(--border-glass);
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
}

.preview-dot {
    width: 12px;
    height: 12px;
    background: var(--color-primary);
    border-radius: 50%;
    box-shadow: 0 0 8px var(--color-primary);
}

.design-name {
    font-size: 11px;
    color: var(--text-color);
    text-align: center;
}

.sub-label {
    font-size: 12px;
    color: var(--text-tertiary);
    font-weight: normal;
    margin-left: 8px;
}

.design-tag {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--bg-glass-active);
    border-radius: 4px;
    color: var(--text-color);
}

input[type="range"] { 
    width: 100%; height: 6px; background: var(--bg-input-dim); border-radius: 10px; appearance: none; outline: none; 
    border: 1px solid var(--border-glass);
}
input[type="range"]::-webkit-slider-thumb { 
    appearance: none; width: 18px; height: 18px; background: var(--text-color-white); border-radius: 50%; cursor: pointer; 
    box-shadow: var(--shadow-main);
    border: 1px solid var(--border-glass);
}
</style>
