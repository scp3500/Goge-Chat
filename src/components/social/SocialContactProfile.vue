<script setup>
import { ref, computed } from 'vue';
import { useConfigStore } from '../../stores/config';
import { resolveSocialAvatar } from '../../utils/social';
import { invoke } from '@tauri-apps/api/core';
import AIContactModal from '../modals/AIContactModal.vue';
import { useChatStore } from '../../stores/chat';
import { useSettingsStore } from '../../stores/settings';

const props = defineProps({
  activeContact: {
    type: Object,
    required: true
  }
});

const emit = defineEmits(['startChat']);

const configStore = useConfigStore();
const chatStore = useChatStore();
const settingsStore = useSettingsStore();
const showEditModal = ref(false);

const handleStartChat = () => {
  emit('startChat');
};

const handleEditProfile = () => {
  showEditModal.value = true;
};

const handleManageMemories = () => {
    // 1. 设置过滤器：使用物理 ID 确保唯一性
    settingsStore.dataFilterRoleId = props.activeContact.id;
    // 2. 打开设置页
    settingsStore.openSettings('data');
};

const handleConfirmEdit = async (data) => {
  try {
    await invoke('update_social_contact', { 
      id: props.activeContact.id,
      ...data
    });
    showEditModal.value = false;
    chatStore.triggerSocialSessionRefresh(); // Signal refresh
  } catch (e) {
    console.error('Failed to update contact:', e);
    alert('保存失败: ' + e);
  }
};

const displayName = computed(() => props.activeContact.remark || props.activeContact.name);
const hasRemark = computed(() => !!props.activeContact.remark);

</script>

<template>
  <div class="social-contact-profile">
    <div class="profile-card">
      <div class="profile-main">
        <div class="avatar-large">
          <img v-if="activeContact.avatar" :src="resolveSocialAvatar(activeContact.avatar)" />
          <div v-else class="avatar-placeholder-lg">{{ activeContact.name[0] }}</div>
        </div>
        
        <div class="info-section">
          <h2 class="contact-name">
            {{ displayName }}
          </h2>
          <div v-if="hasRemark" class="original-name">
            昵称：{{ activeContact.name }}
          </div>
        </div>
      </div>

      <div class="divider"></div>

      <div class="actions">
        <!-- Main Message Action -->
        <button class="action-btn message-btn" @click="handleStartChat">
          发消息
        </button>

        <!-- Edit Remark Action -->
        <button class="action-btn edit-btn" @click="handleEditProfile">
          个人资料
        </button>

        <!-- Manage Memories Action -->
        <button class="action-btn memory-btn" @click="handleManageMemories">
          记忆管理
        </button>
      </div>
    </div>

    <!-- AI Contact Modal for Editing -->
    <AIContactModal 
      :show="showEditModal" 
      :contact="activeContact"
      @close="showEditModal = false"
      @confirm="handleConfirmEdit"
    />
  </div>
</template>

<style scoped>
.social-contact-profile {
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-main);
  position: relative;
}

.profile-card {
  flex: 1;
  width: 100%;
  max-width: 400px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding-bottom: 20px;
}

.profile-main {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 32px;
  margin-bottom: 40px;
}

.avatar-large {
  width: 100px;
  height: 100px;
  border-radius: 16px;
  overflow: hidden;
  background: var(--bg-active);
  flex-shrink: 0;
  box-shadow: 0 8px 24px rgba(0,0,0,0.12);
}

.avatar-large img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-placeholder-lg {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 2.5rem;
  background: var(--color-primary);
  color: white;
  font-weight: 600;
}

.info-section {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.contact-name {
  margin: 0;
  font-size: 1.75rem;
  font-weight: 600;
  color: var(--text-color);
}

.original-name {
  font-size: 0.9rem;
  color: var(--text-tertiary);
  opacity: 0.5;
}

.divider {
  width: 100%;
  height: 1px;
  background: var(--border-glass);
  margin-bottom: 60px;
}

.actions {
  display: flex;
  flex-direction: column;
  gap: 16px;
  width: 100%;
  max-width: 280px;
}

.action-btn {
  width: 100%;
  padding: 14px;
  border-radius: 10px;
  font-size: 1.05rem;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
  justify-content: center;
}

.message-btn {
  background: var(--color-wechat-green); /* WeChat Green Fixed */
  color: var(--color-white);
  padding: 10px 24px;
  border-radius: 8px;
  cursor: pointer;
  box-shadow: var(--shadow-wechat);
}

.message-btn:hover {
  background: var(--color-wechat-green-hover);
  box-shadow: var(--shadow-wechat-hover);
  transform: translateY(-2px);
}

.edit-btn {
  background: var(--bg-glass);
  color: var(--text-color);
  border: 1px solid var(--border-glass-bright);
}

.edit-btn:hover {
  background: var(--bg-glass-hover);
  transform: translateY(-1px);
}

.memory-btn {
  background: transparent;
  color: var(--text-tertiary);
  border: 1px solid var(--border-glass);
  font-size: 0.9rem;
  padding: 10px;
  margin-top: 8px;
}

.memory-btn:hover {
  background: var(--bg-glass-hover);
  color: var(--color-primary);
  border-color: var(--color-primary);
}

/* 适配不同模式的主题变量 */
:global(.app-light) .social-contact-profile {
  background: #f7f7f7; /* 这里保持 light mode 的灰白背景，模拟微信 */
}

:global(.app-light) .edit-btn {
    background: var(--color-white);
    border-color: #ddd;
}
</style>
