<script setup>
import { computed, ref } from 'vue';
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
  if (!configStore.settings) return [];
  return designStyles.filter(s => s.mode === configStore.settings.theme);
});

// 头像上传逻辑
const fileInput = ref(null);

const triggerUpload = () => {
  fileInput.value?.click();
};

const handleFileUpload = async (event) => {
  const file = event.target.files?.[0];
  if (!file) return;

  // 简单的文件验证
  if (!file.type.startsWith('image/')) {
    alert('请上传图片文件');
    return;
  }

  // 由于浏览器安全限制，我们不能直接获取全路径传给后端上传
  // 但我们是 Tauri 环境，input type=file 拿到的是 File 对象
  // 我们需要一个方法把 File 对象传给 Rust，或者读取 ArrayBuffer 传给 Rust
  // 我之前实现的 `upload_user_avatar` 是接受 file_path string。
  // Tauri 的 fs API 或 invoke 怎么传文件？
  // 如果是 drag drop 或 dialog open，我们可以拿到 path。
  // 但是 web input file 拿不到 path。
  // 变更计划：使用 Tauri Dialog 打开文件，而不是 HTML input。
  
  // 修正：使用 open Dialog
};

// 使用 Tauri API 选择文件
import { open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';
import ImageCropperModal from '../modals/ImageCropperModal.vue';

const showEditor = ref(false);
const editingImg = ref('');

const selectAndUploadAvatar = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Images',
        extensions: ['png', 'jpg', 'jpeg', 'webp']
      }]
    });
    
    if (selected && typeof selected === 'string') { // string (path)
       // 🟢 Fix: Read file as base64 for vue-cropper compatibility
       const content = await readFile(selected);
       const base64 = btoa(
           new Uint8Array(content)
             .reduce((data, byte) => data + String.fromCharCode(byte), '')
       );
       const mimeType = selected.toLowerCase().endsWith('.png') ? 'image/png' : 
                        selected.toLowerCase().endsWith('.gif') ? 'image/gif' : 
                        selected.toLowerCase().endsWith('.webp') ? 'image/webp' :
                        'image/jpeg';

       editingImg.value = `data:${mimeType};base64,${base64}`;
       showEditor.value = true;
    }
  } catch (err) {
    console.error('选择头像失败:', err);
  }
};



const handleAvatarSave = async (croppedData) => {
  try {
    // croppedData is a base64 data-URL
    const savedPath = await configStore.uploadAvatar(croppedData);
    showEditor.value = false;
    handleUpdate();
  } catch (err) {
    console.error('保存裁剪头像失败:', err);
  }
};

const selectFontFile = async (type) => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Fonts',
        extensions: ['ttf', 'otf', 'woff', 'woff2']
      }]
    });

    if (selected && typeof selected === 'string') {
       if (type === 'english') {
           configStore.settings.fontFamilyEnglish = selected;
       } else {
           configStore.settings.fontFamilyChinese = selected;
       }
       handleUpdate();
    }
  } catch (err) {
    console.error('选择字体失败:', err);
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
          <label>UI 密度 / 缩放 ({{ (configStore.settings.globalScale * 100).toFixed(0) }}%)</label>
          <div class="scale-control">
            <input 
              type="range" 
              v-model.number="configStore.settings.globalScale" 
              min="0.5" 
              max="1.5" 
              step="0.05"
              @input="handleUpdate" 
            />
            <div class="scale-presets">
               <button class="preset-tag" @click="configStore.settings.globalScale = 1.0; handleUpdate()">100% (标准)</button>
               <button class="preset-tag" @click="configStore.settings.globalScale = 0.85; handleUpdate()">85% (高密)</button>
               <button class="preset-tag" @click="configStore.settings.globalScale = 0.75; handleUpdate()">75% (紧凑)</button>
            </div>
          </div>
          <span class="sub-label">缩放 UI 元素大小，同时保持文字清晰可读</span>
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

        <!-- 🅰️ Global Typography -->
        <div class="control-item">
          <label>全局字体设置 <span class="sub-label">支持直接选择本地 .ttf / .otf 文件</span></label>
          <div class="font-settings-group">
             <div class="font-row">
                <label>英文 / 数字字体</label>
                <div class="input-with-btn">
                   <input 
                     type="text" 
                     v-model="configStore.settings.fontFamilyEnglish" 
                     placeholder="例如: Segoe UI, Inter, 或选择文件"
                     @change="handleUpdate"
                   />
                   <button class="icon-btn-small" @click="selectFontFile('english')" title="选择字体文件">📂</button>
                </div>
             </div>
             <div class="font-row">
                <label>中文字体</label>
                <div class="input-with-btn">
                   <input 
                     type="text" 
                     v-model="configStore.settings.fontFamilyChinese" 
                     placeholder="例如: Microsoft YaHei, PingFang SC"
                     @change="handleUpdate"
                   />
                   <button class="icon-btn-small" @click="selectFontFile('chinese')" title="选择字体文件">📂</button>
                </div>
             </div>
          </div>
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


        <div class="control-item">
          <label>气泡模式</label>
          <div class="toggle-row">
             <label class="toggle-switch">
               <input type="checkbox" v-model="configStore.settings.enableBubble" @change="handleUpdate" />
               <span class="slider"></span>
             </label>
             <span class="sub-label">为消息添加气泡背景和圆角</span>
          </div>
        </div>

        <div class="control-item">
          <label>用户头像</label>
          <div class="avatar-settings">
             <div class="toggle-row">
                <label class="toggle-switch">
                  <input type="checkbox" v-model="configStore.settings.showUserAvatar" @change="handleUpdate" />
                  <span class="slider"></span>
                </label>
                <span class="sub-label">显示用户头像 (将在侧边栏或标题栏展示)</span>
             </div>
             
             <div v-if="configStore.settings.showUserAvatar" class="avatar-adjustments">
                <div v-if="configStore.settings.showUserAvatar" class="upload-row">
                <div class="avatar-preview" 
                     :style="{ backgroundImage: configStore.userAvatarUrl ? `url('${configStore.userAvatarUrl}')` : 'none' }">
                   <span v-if="!configStore.userAvatarUrl" class="placeholder">+</span>
                </div>
                   <button class="upload-btn" @click="selectAndUploadAvatar">更换头像</button>
                </div>

                <!-- 头像细调 -->
                <div class="adjustment-grid">
                  <div class="adj-item">
                    <label>大小: {{ configStore.settings.userAvatarSize }}px</label>
                    <input type="range" v-model.number="configStore.settings.userAvatarSize" min="20" max="80" @input="handleUpdate" />
                  </div>
                  <div class="adj-item">
                    <label>圆角: {{ configStore.settings.userAvatarBorderRadius }}px</label>
                    <input type="range" v-model.number="configStore.settings.userAvatarBorderRadius" min="0" max="40" @input="handleUpdate" />
                  </div>
                  <div class="adj-item">
                    <label>水平偏移: {{ configStore.settings.userAvatarOffsetX }}px</label>
                    <input type="range" v-model.number="configStore.settings.userAvatarOffsetX" min="-50" max="50" @input="handleUpdate" />
                  </div>
                  <div class="adj-item">
                    <label>垂直偏移: {{ configStore.settings.userAvatarOffsetY }}px</label>
                    <input type="range" v-model.number="configStore.settings.userAvatarOffsetY" min="-50" max="50" @input="handleUpdate" />
                  </div>
                </div>
             </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Avatar Editor Modal (Reused) -->
    <ImageCropperModal 
      :show="showEditor"
      :img-src="editingImg"
      :border-radius="configStore.settings.userAvatarBorderRadius"
      @confirm="handleAvatarSave"
      @close="showEditor = false"
    />
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

/* Toggle Switch */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
  flex-shrink: 0;
}
.toggle-switch input { opacity: 0; width: 0; height: 0; }
.slider {
  position: absolute;
  cursor: pointer;
  top: 0; left: 0; right: 0; bottom: 0;
  background-color: var(--bg-input-dim);
  transition: .4s;
  border-radius: 22px;
  border: 1px solid var(--border-glass);
}
.slider:before {
  position: absolute;
  content: "";
  height: 16px; width: 16px;
  left: 2px; bottom: 2px;
  background-color: var(--text-tertiary);
  transition: .4s;
  border-radius: 50%;
}
input:checked + .slider { background-color: var(--color-primary-bg); border-color: var(--color-primary); }
input:checked + .slider:before { transform: translateX(18px); background-color: var(--color-primary); }

.toggle-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* Avatar Upload */
.upload-row {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-top: 12px;
  padding-left: 4px;
}
.avatar-preview {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background-color: var(--bg-input-dim);
  border: 1px solid var(--border-glass);
  background-size: cover;
  background-position: center;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  font-size: 20px;
}
.upload-btn {
  padding: 6px 12px;
  border-radius: 6px;
  background: var(--bg-glass-active);
  border: 1px solid var(--border-glass);
  color: var(--text-color);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}
.upload-btn:hover {
  background: var(--bg-glass-hover);
  border-color: var(--color-primary);
  color: var(--text-color-white);
}

.avatar-adjustments {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.adjustment-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  padding: 16px;
  background: var(--bg-input-dim);
  border-radius: 8px;
  border: 1px solid var(--border-glass);
}

.adj-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.adj-item label {
  font-size: 11px;
  opacity: 0.8;
  margin-bottom: 0px !important;
}

.scale-control {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.scale-presets {
  display: flex;
  gap: 8px;
}

.preset-tag {
  padding: 4px 10px;
  background: var(--bg-input-dim);
  border: 1px solid var(--border-glass);
  border-radius: 4px;
  font-size: 11px;
  color: var(--text-color);
  cursor: pointer;
  transition: all 0.2s;
}

.preset-tag:hover {
  background: var(--bg-glass-hover);
  border-color: var(--color-primary);
}

.font-settings-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: var(--bg-input-dim);
    padding: 12px;
    border-radius: 8px;
    border: 1px solid var(--border-glass);
}

.font-row label {
    font-size: 11px;
    margin-bottom: 4px !important;
    opacity: 0.8;
}

.input-with-btn {
    display: flex;
    gap: 8px;
}

.input-with-btn input {
    flex: 1;
    background: var(--bg-main);
    border: 1px solid var(--border-glass);
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 13px;
    color: var(--text-color);
}

.input-with-btn input:focus {
    border-color: var(--color-primary);
}

.icon-btn-small {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    border-radius: 6px;
    background: var(--bg-glass-active);
    border: 1px solid var(--border-glass);
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
    color: var(--text-color);
}

.icon-btn-small:hover {
    background: var(--bg-glass-hover);
    border-color: var(--color-primary);
}
</style>
