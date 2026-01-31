<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { SEARCH_SVG } from '../../constants/icons';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
  isCollapsed: { type: Boolean, default: false }
});

const clock = ref('');
let timer = null;

const updateClock = () => {
  const now = new Date();
  clock.value = now.toLocaleTimeString('zh-CN', { hour12: false });
};

const profile = ref({
  nickname: '加载中...',
  bio: '',
  avatar: null
});

const contacts = ref([]);
const groups = ref([]);

onMounted(async () => {
  updateClock();
  timer = setInterval(updateClock, 1000);

  try {
    profile.value = await invoke('get_social_profile');
    groups.value = await invoke('get_social_groups');
    contacts.value = await invoke('get_social_contacts');
  } catch (e) {
    console.error('Failed to load social data:', e);
  }
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});
</script>

<template>
  <aside class="social-sidebar" :class="{ 'is-collapsed': isCollapsed }">
    <div class="sidebar-content">
      <!-- Search Box -->
      <div class="search-container">
        <div class="search-box">
          <div class="search-icon" v-html="SEARCH_SVG"></div>
          <input type="text" placeholder="搜索内容..." />
        </div>
      </div>

      <!-- Clock -->
      <div class="clock-display">
        {{ clock }}
      </div>

      <!-- Profile Card -->
      <div class="profile-card">
        <div class="avatar-container">
          <img v-if="profile.avatar" :src="profile.avatar" class="avatar" />
          <div v-else class="avatar-placeholder">{{ profile.nickname[0] }}</div>
        </div>
        <div class="profile-info">
          <h3 class="nickname">{{ profile.nickname }}</h3>
          <p class="bio">{{ profile.bio }}</p>
        </div>
      </div>

      <!-- Contact List -->
      <div class="contact-list">
        <div v-for="group in groups" :key="group.id" class="contact-group">
          <div class="group-header">
            <span class="group-name">{{ group.name }}</span>
          </div>
          <div class="group-items">
            <div 
              v-for="contact in contacts.filter(c => c.group_id === group.id)" 
              :key="contact.id"
              class="contact-item"
            >
              <div class="avatar-sm">
                 <img v-if="contact.avatar" :src="contact.avatar" />
                 <div v-else class="avatar-placeholder-sm">{{ contact.name[0] }}</div>
              </div>
              <div class="contact-name">{{ contact.name }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.social-sidebar {
  width: 280px;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-glass);
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
}

.social-sidebar.is-collapsed {
  width: 0;
  border-right: none;
}

.sidebar-content {
  width: 280px; /* Maintain fixed width inside for content stability during animation */
  padding: 16px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.search-box {
  background: var(--bg-input);
  border-radius: 8px;
  display: flex;
  align-items: center;
  padding: 8px 12px;
  gap: 8px;
  border: 1px solid var(--border-glass);
}

.search-icon {
  width: 16px;
  height: 16px;
  opacity: 0.5;
}

.search-box input {
  background: transparent;
  border: none;
  color: var(--text-color);
  font-size: 0.9rem;
  width: 100%;
  outline: none;
}

.clock-display {
  font-family: 'JetBrains Mono', monospace;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--theme-color);
  text-align: center;
  padding: 12px 0;
  background: var(--bg-active);
  border-radius: 8px;
}

.profile-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-hover);
  border-radius: 12px;
}

.avatar-container {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  overflow: hidden;
  background: var(--bg-active);
  flex-shrink: 0;
}

.avatar { width: 100%; height: 100%; object-fit: cover; }
.avatar-placeholder { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; font-size: 1.2rem; background: var(--theme-color); color: white; }

.profile-info {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.nickname { margin: 0; font-size: 1rem; color: var(--text-color); }
.bio { margin: 4px 0 0 0; font-size: 0.8rem; opacity: 0.6; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

.contact-list {
  flex: 1;
  overflow-y: auto;
}

.group-header {
  padding: 8px 4px;
  font-size: 0.75rem;
  font-weight: bold;
  opacity: 0.5;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.group-items {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.contact-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
}

.contact-item:hover {
  background: var(--bg-hover);
}

.avatar-sm {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  overflow: hidden;
  background: var(--bg-active);
  flex-shrink: 0;
}

.avatar-placeholder-sm { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; font-size: 0.8rem; background: var(--bg-active); }
.contact-name { font-size: 0.9rem; color: var(--text-color); }
</style>
