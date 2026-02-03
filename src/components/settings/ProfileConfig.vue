<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';
import { useConfigStore } from '../../stores/config';
import ImageCropperModal from '../modals/ImageCropperModal.vue';

const configStore = useConfigStore();

const showEditor = ref(false);
const editingImg = ref('');

const resolveAvatarSrc = (path) => {
  if (!path) return '';
  if (path.startsWith('data:') || path.startsWith('http')) return path;
  return convertFileSrc(path);
};

// Local values for easier editing before sync
const nickname = ref('');
const isSaving = ref(false);
const saveStatus = ref('');

const loadProfile = async () => {
  nickname.value = configStore.settings.nickname;
};

const handleAvatarClick = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Images',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp']
      }]
    });
    if (selected && typeof selected === 'string') {
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
  } catch (e) {
    console.error('Failed to select avatar:', e);
  }
};

const handleAvatarSave = async (croppedData) => {
  try {
    const savedPath = await configStore.uploadAvatar(croppedData);
    // Profile SYNC is now handled automatically by configStore.updateConfig
    showEditor.value = false;
  } catch (err) {
    console.error('保存裁剪头像失败:', err);
  }
};

const saveProfile = async (silent = false) => {
  if (nickname.value === configStore.settings.nickname) return;
  
  if (!silent) {
    isSaving.value = true;
    saveStatus.value = '正在同步...';
  }
  try {
    await configStore.updateConfig({ nickname: nickname.value });
    if (!silent) {
      saveStatus.value = '已自动保存';
      setTimeout(() => { saveStatus.value = ''; }, 2000);
    }
  } catch (e) {
    console.error('Failed to save profile:', e);
    if (!silent) saveStatus.value = '保存失败';
  } finally {
    isSaving.value = false;
  }
};

const resetAvatarSettings = async () => {
    const defaults = {
        userAvatarSize: 36,
        userAvatarBorderRadius: 6,
        userAvatarOffsetX: 0,
        userAvatarOffsetY: 0
    };
    await configStore.updateConfig(defaults);
    saveStatus.value = '已恢复默认值';
    setTimeout(() => { saveStatus.value = ''; }, 2000);
};

onMounted(loadProfile);
</script>

<template>
  <div class="profile-config-container modern-scroll">
    <div class="config-card">
      <div class="header-decoration"></div>
      
      <div class="save-indicator" :class="{ visible: saveStatus }">
        <span class="dot" :class="{ loading: isSaving }"></span>
        {{ saveStatus }}
      </div>

      <div class="avatar-section">
        <div class="avatar-edit-wrapper">
            <div class="avatar-edit" @click="handleAvatarClick">
                <img v-if="configStore.userAvatarUrl" :src="configStore.userAvatarUrl" class="large-avatar" />
                <div v-else class="avatar-placeholder large">{{ nickname ? nickname[0] : (configStore.settings.nickname ? configStore.settings.nickname[0] : 'G') }}</div>
                <div class="avatar-overlay">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"></path>
                        <circle cx="12" cy="13" r="4"></circle>
                    </svg>
                    <span>更换头像</span>
                </div>
            </div>
        </div>
        <p class="help-text">JPG, PNG 或 WebP · 建议正方形</p>
      </div>

      <div class="config-body">
        <div class="form-group floating-group">
          <label>我的昵称</label>
          <div class="input-wrapper">
              <input 
                type="text" 
                v-model="nickname" 
                @blur="saveProfile()"
                @change="saveProfile()"
                placeholder="设置你的社交昵称..."
                class="premium-input"
              />
              <div class="input-focus-line"></div>
          </div>
        </div>

        <div class="divider">
            <span>头像精调</span>
            <button class="reset-link" @click="resetAvatarSettings">恢复默认</button>
        </div>

        <div class="adjustment-panel">
          <div class="adj-grid">
            <div class="adj-control">
              <div class="adj-header">
                  <span class="adj-label">大小</span>
                  <span class="adj-value">{{ configStore.settings.userAvatarSize }}px</span>
              </div>
              <input type="range" v-model.number="configStore.settings.userAvatarSize" min="20" max="80" @input="configStore.updateConfig({ userAvatarSize: configStore.settings.userAvatarSize })" />
            </div>

            <div class="adj-control">
                <div class="adj-header">
                    <span class="adj-label">圆角</span>
                    <span class="adj-value">{{ configStore.settings.userAvatarBorderRadius }}px</span>
                </div>
              <input type="range" v-model.number="configStore.settings.userAvatarBorderRadius" min="0" max="40" @input="configStore.updateConfig({ userAvatarBorderRadius: configStore.settings.userAvatarBorderRadius })" />
            </div>

            <div class="adj-control">
                <div class="adj-header">
                    <span class="adj-label">水平偏移</span>
                    <span class="adj-value">{{ configStore.settings.userAvatarOffsetX }}px</span>
                </div>
              <input type="range" v-model.number="configStore.settings.userAvatarOffsetX" min="-50" max="50" @input="configStore.updateConfig({ userAvatarOffsetX: configStore.settings.userAvatarOffsetX })" />
            </div>

            <div class="adj-control">
                <div class="adj-header">
                    <span class="adj-label">垂直偏移</span>
                    <span class="adj-value">{{ configStore.settings.userAvatarOffsetY }}px</span>
                </div>
              <input type="range" v-model.number="configStore.settings.userAvatarOffsetY" min="-50" max="50" @input="configStore.updateConfig({ userAvatarOffsetY: configStore.settings.userAvatarOffsetY })" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Avatar Editor Modal -->
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
.profile-config-container {
  padding: 8px;
  animation: slideUp 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.config-card {
  position: relative;
  background: var(--bg-main-light, rgba(255,255,255,0.02));
  border: 1px solid var(--border-glass);
  border-radius: 20px;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0,0,0,0.1);
  backdrop-filter: blur(20px);
}

.header-decoration {
  height: 60px;
  background: linear-gradient(90deg, var(--theme-color-alpha-20, rgba(25,118,210,0.1)), transparent);
  border-bottom: 1px solid var(--border-glass);
}

.save-indicator {
  position: absolute;
  top: 16px;
  right: 20px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-dim);
  padding: 6px 12px;
  background: var(--bg-glass);
  border-radius: 20px;
  border: 1px solid var(--border-glass);
  opacity: 0;
  transform: translateX(10px);
  transition: all 0.3s ease;
}

.save-indicator.visible {
  opacity: 1;
  transform: translateX(0);
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-wechat-green);
}

.dot.loading {
  background: var(--theme-color);
  animation: pulse 1s infinite alternate;
}

@keyframes pulse {
  from { opacity: 0.4; transform: scale(0.8); }
  to { opacity: 1; transform: scale(1.2); }
}

.avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-top: -50px;
  padding-bottom: 24px;
}

.avatar-edit-wrapper {
    padding: 6px;
    background: var(--bg-main);
    border-radius: 28px;
    box-shadow: 0 4px 15px rgba(0,0,0,0.1);
}

.avatar-edit {
  position: relative;
  width: 110px;
  height: 110px;
  border-radius: 24px;
  overflow: hidden;
  cursor: pointer;
  background: var(--bg-active);
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.avatar-edit:hover {
  transform: scale(1.05) rotate(2deg);
}

.large-avatar {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-placeholder.large {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 3rem;
  font-weight: bold;
  color: white;
  background: linear-gradient(135deg, var(--theme-color), var(--theme-color-dark, #1565C0));
}

.avatar-overlay {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  color: white;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  opacity: 0;
  transition: all 0.3s ease;
  backdrop-filter: blur(2px);
}

.avatar-edit:hover .avatar-overlay {
  opacity: 1;
}

.avatar-overlay span {
    font-size: 12px;
    font-weight: 500;
}

.help-text {
  margin-top: 12px;
  font-size: 11px;
  color: var(--text-tertiary);
  letter-spacing: 0.5px;
}

.config-body {
    padding: 0 32px 32px;
}

.floating-group {
    margin-bottom: 40px;
}

.floating-group label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    color: var(--theme-color);
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 1px;
}

.input-wrapper {
    position: relative;
}

.premium-input {
  width: 100%;
  background: transparent;
  border: none;
  border-bottom: 2px solid var(--border-glass-bright);
  padding: 12px 0;
  color: var(--text-color-white);
  font-size: 18px;
  font-weight: 500;
  outline: none;
  transition: all 0.3s ease;
}

.premium-input:focus ~ .input-focus-line {
    width: 100%;
}

.input-focus-line {
    position: absolute;
    bottom: 0;
    left: 0;
    width: 0;
    height: 2px;
    background: var(--theme-color);
    transition: width 0.3s ease;
}

.premium-input:focus ~ .input-focus-line {
    width: 100%;
}

.divider {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
}

.divider span {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 1px;
}

.reset-link {
    background: transparent;
    border: none;
    color: var(--theme-color);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: background 0.2s;
}

.reset-link:hover {
    background: var(--theme-color-alpha-10);
    text-decoration: underline;
}

.adjustment-panel {
    background: var(--bg-glass-heavy, rgba(0,0,0,0.1));
    border-radius: 16px;
    padding: 24px;
    border: 1px solid var(--border-glass);
}

.adj-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 24px;
}

.adj-control {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.adj-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.adj-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.adj-value {
    font-size: 12px;
    font-family: 'Roboto Mono', monospace;
    color: var(--theme-color);
    font-weight: 600;
}

input[type="range"] { 
    width: 100%; height: 4px; background: var(--border-glass-bright); border-radius: 10px; appearance: none; outline: none; 
}

input[type="range"]::-webkit-slider-thumb { 
    appearance: none; width: 16px; height: 16px; 
    background: #fff; 
    border: 2px solid var(--theme-color);
    border-radius: 50%; cursor: pointer; 
    transition: all 0.2s ease;
    box-shadow: 0 2px 6px rgba(0,0,0,0.2);
}

input[type="range"]:active::-webkit-slider-thumb {
    transform: scale(1.3);
    background: var(--theme-color);
}
</style>
