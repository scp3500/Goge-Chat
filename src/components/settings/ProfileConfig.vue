<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc } from '@tauri-apps/api/core';

const resolveAvatarSrc = (path) => {
  if (!path) return '';
  if (path.startsWith('data:') || path.startsWith('http')) return path;
  return convertFileSrc(path);
};

const profile = ref({
  nickname: '',
  avatar: ''
});

const isSaving = ref(false);
const saveStatus = ref('');

const loadProfile = async () => {
  try {
    profile.value = await invoke('get_social_profile');
  } catch (e) {
    console.error('Failed to load profile:', e);
  }
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
    if (selected) {
      profile.value.avatar = selected;
    }
  } catch (e) {
    console.error('Failed to select avatar:', e);
  }
};

const saveProfile = async () => {
  isSaving.value = true;
  saveStatus.value = '正在保存...';
  try {
    await invoke('update_social_profile', {
      nickname: profile.value.nickname,
      avatar: profile.value.avatar
    });
    saveStatus.value = '保存成功！';
    setTimeout(() => { saveStatus.value = ''; }, 2000);
  } catch (e) {
    console.error('Failed to save profile:', e);
    saveStatus.value = '保存失败: ' + e;
  } finally {
    isSaving.value = false;
  }
};

onMounted(loadProfile);
</script>

<template>
  <div class="profile-config">
    <div class="avatar-section">
      <div class="avatar-edit" @click="handleAvatarClick">
        <img v-if="profile.avatar" :src="resolveAvatarSrc(profile.avatar)" class="large-avatar" />
        <div v-else class="avatar-placeholder large">{{ profile.nickname ? profile.nickname[0] : 'G' }}</div>
        <div class="avatar-overlay">更换头像</div>
      </div>
      <p class="help-text">点击图片上传新头像</p>
    </div>

    <div class="form-section">
      <div class="form-group">
        <label>我的昵称</label>
        <input 
          type="text" 
          v-model="profile.nickname" 
          placeholder="请输入你的昵称"
          class="modern-input"
        />
      </div>

      <div class="actions">
        <button 
          class="save-btn" 
          @click="saveProfile" 
          :disabled="isSaving"
        >
          {{ isSaving ? '保存中...' : '提交修改' }}
        </button>
        <span v-if="saveStatus" class="status-msg" :class="{ error: saveStatus.includes('失败') }">
          {{ saveStatus }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.profile-config {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.avatar-edit {
  position: relative;
  width: 100px;
  height: 100px;
  border-radius: 12px;
  overflow: hidden;
  cursor: pointer;
  background: var(--bg-active);
  border: 2px solid var(--border-glass);
  transition: all 0.2s ease;
}

.avatar-edit:hover {
  border-color: var(--theme-color);
  transform: scale(1.02);
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
  font-size: 2.5rem;
  font-weight: bold;
  color: white;
  background: var(--theme-color);
}

.avatar-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  font-size: 11px;
  padding: 4px 0;
  text-align: center;
  opacity: 0;
  transition: opacity 0.2s;
}

.avatar-edit:hover .avatar-overlay {
  opacity: 1;
}

.help-text {
  font-size: 12px;
  color: var(--text-dim);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 24px;
}

.form-group label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-tertiary);
}

.modern-input {
  background: var(--bg-input);
  border: 1px solid var(--border-glass);
  border-radius: 8px;
  padding: 12px 14px;
  color: var(--text-color-white);
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.modern-input:focus {
  border-color: var(--theme-color);
}

.actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.save-btn {
  background: var(--theme-color);
  color: white;
  border: none;
  border-radius: 8px;
  padding: 10px 24px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.2s;
}

.save-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.status-msg {
  font-size: 13px;
  color: #07c160;
}

.status-msg.error {
  color: #ff4d4f;
}
</style>
