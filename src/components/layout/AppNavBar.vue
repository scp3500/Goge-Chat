<script setup>
import { ref, computed, onMounted } from 'vue';
import { 
  NAV_PROMPTS_SVG, 
  NAV_MODELS_SVG, 
  NAV_ADDRESS_BOOK_SVG,
  NAV_GENERAL_SVG,
  CHEVRON_DOWN_SVG,
  SUN_SVG,
  MOON_SVG,
  HOME_SVG
} from '../../constants/icons';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { useConfigStore } from '../../stores/config';

const resolveAvatarSrc = (path) => {
  if (!path) return '';
  if (path.startsWith('data:') || path.startsWith('http')) return path;
  return convertFileSrc(path);
};

const props = defineProps({
  activeModule: { type: String, default: 'chat' },
  isCollapsed: { type: Boolean, default: false },
  isInSettings: { type: Boolean, default: false }
});

const emit = defineEmits(['update:activeModule', 'toggleCollapse', 'openSettings', 'backHome', 'openProfile']);

const profile = ref({
  nickname: 'Guest',
  avatar: null
});

const loadProfile = async () => {
  try {
    profile.value = await invoke('get_social_profile');
  } catch (e) {
    console.warn('Failed to load profile in Nav:', e);
  }
};

onMounted(loadProfile);

const handleModuleClick = (moduleId) => {
  emit('update:activeModule', moduleId);
  if (props.isInSettings) {
    emit('backHome');
  }
};

const configStore = useConfigStore();
const isLight = computed(() => configStore.settings.theme === 'light');

const toggleTheme = () => {
  const nextTheme = isLight.value ? 'dark' : 'light';
  configStore.updateConfig({ theme: nextTheme });
};

const modules = [
  { id: 'chat', icon: NAV_PROMPTS_SVG, label: '聊天' },
  { id: 'address_book', icon: NAV_ADDRESS_BOOK_SVG, label: '通讯录' },
];

</script>

<template>
  <nav class="app-nav-bar" :class="{ 'immersive': configStore.settings.chatMode.enabled && isLight }">
    <div class="nav-top" data-tauri-drag-region>
      <div 
        class="user-avatar-container" 
        @click="emit('openProfile')" 
        title="个人资料"
        data-tauri-drag-region
      >
        <img v-if="configStore.userAvatarUrl || profile.avatar" 
             :src="configStore.userAvatarUrl || resolveAvatarSrc(profile.avatar)" 
             class="user-avatar" />
        <div v-else class="avatar-placeholder">{{ profile.nickname[0] }}</div>
      </div>
      
      <div class="nav-items">
        <button 
          v-for="mod in modules" 
          :key="mod.id"
          class="nav-item"
          :class="{ active: activeModule === mod.id && !isInSettings }"
          @click="handleModuleClick(mod.id)"
          :title="mod.label"
        >
          <div class="icon-wrapper" v-html="mod.icon"></div>
        </button>
      </div>
    </div>

    <div class="nav-bottom">
      <button 
        class="nav-item theme-toggle" 
        @click="toggleTheme" 
        :title="isLight ? '切换深色模式' : '切换浅色模式'"
      >
        <div class="icon-wrapper" v-html="isLight ? SUN_SVG : MOON_SVG"></div>
      </button>

      <button 
        class="nav-item" 
        @click="isInSettings ? emit('backHome') : emit('openSettings')" 
        :title="isInSettings ? '返回首页' : '设置'"
      >
        <div class="icon-wrapper" v-html="isInSettings ? HOME_SVG : NAV_GENERAL_SVG"></div>
      </button>

      <button 
        class="nav-item collapse-btn" 
        :class="{ collapsed: isCollapsed }"
        @click="emit('toggleCollapse')"
        title="折叠/展开"
      >
        <div class="icon-wrapper rotate-icon" v-html="CHEVRON_DOWN_SVG"></div>
      </button>
    </div>
  </nav>
</template>

<style scoped>
.app-nav-bar {
  width: 64px;
  min-width: 64px;
  height: 100%;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-glass);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
  box-sizing: border-box;
  flex-shrink: 0;
  z-index: 100;
  transition: background 0.3s;
}

.app-nav-bar.immersive {
  background: #28292c;
  border-right: none;
}

.app-nav-bar.immersive .nav-item {
  color: #999;
  opacity: 1;
}

.app-nav-bar.immersive .nav-item:hover {
  background: var(--bg-social-icon-hover); /* Visible box on hover */
  color: #fff;
}

.app-nav-bar.immersive .nav-item.active {
  color: #07c160; /* WeChat Green */
  background: var(--bg-social-icon-active); /* Visible box on active */
}

.user-avatar-container {
  margin-bottom: 24px;
  display: flex;
  justify-content: center;
  cursor: pointer;
  /* 移除侧边栏头像的偏移转换，窄侧边栏不适合此类精调 */
  transition: transform 0.2s ease;
}

.user-avatar {
  width: var(--user-avatar-size);
  height: var(--user-avatar-size);
  border-radius: var(--user-avatar-radius);
  object-fit: cover;
  transition: all 0.2s ease;
}

.avatar-placeholder {
  width: var(--user-avatar-size);
  height: var(--user-avatar-size);
  background: var(--theme-color);
  color: white;
  border-radius: var(--user-avatar-radius);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: calc(var(--user-avatar-size) * 0.45);
  transition: all 0.2s ease;
}

.nav-items {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.nav-item {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-color);
  opacity: 0.6;
  transition: all 0.2s ease;
}

.nav-item:hover {
  background: var(--bg-hover);
  opacity: 1;
}

.nav-item.active {
  background: var(--bg-active);
  color: var(--theme-color);
  opacity: 1;
}

.icon-wrapper {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

:deep(.icon-wrapper svg) {
  width: 20px;
  height: 20px;
}

.nav-bottom {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.collapse-btn {
  margin-top: 8px;
}

.rotate-icon {
  transform: rotate(90deg);
  transition: transform 0.3s ease;
}

.collapsed .rotate-icon {
  transform: rotate(-90deg);
}
</style>
