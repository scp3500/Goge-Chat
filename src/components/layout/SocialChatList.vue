<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';

const resolveAvatarSrc = (path) => {
  if (!path) return '';
  if (path.startsWith('data:') || path.startsWith('http')) return path;
  return convertFileSrc(path);
};
import { SEARCH_SVG } from '../../constants/icons';

const props = defineProps({
  activeContactId: { type: Number, default: null }
});

const emit = defineEmits(['select']);

const conversations = ref([]);
const searchQuery = ref('');
let interval = null;

const filteredConversations = computed(() => {
  if (!searchQuery.value) return conversations.value;
  return conversations.value.filter(c => 
    c.contact.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
    (c.last_message && c.last_message.toLowerCase().includes(searchQuery.value.toLowerCase()))
  );
});

const loadConversations = async () => {
  try {
    conversations.value = await invoke('get_recent_social_chats');
  } catch (e) {
    console.error('Failed to load recent chats:', e);
  }
};

const formatTime = (timeStr) => {
  if (!timeStr) return '';
  const date = new Date(timeStr);
  const now = new Date();
  
  if (date.toLocaleDateString() === now.toLocaleDateString()) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: false });
  }
  return date.toLocaleDateString([], { month: 'numeric', day: 'numeric' });
};

onMounted(() => {
  loadConversations();
  interval = setInterval(loadConversations, 5000); // Polling for now
});

onUnmounted(() => {
  if (interval) clearInterval(interval);
});

const selectChat = (chat) => {
  emit('select', chat.contact);
};
</script>

<template>
  <div class="social-chat-list modern-scroll">
    <div class="search-header">
      <div class="search-box">
        <div class="search-icon" v-html="SEARCH_SVG"></div>
        <input type="text" v-model="searchQuery" placeholder="搜索消息..." />
      </div>
    </div>
    
    <div class="chat-items">
      <div v-if="filteredConversations.length === 0" class="empty-state">
        <p>暂无消息</p>
      </div>
      
      <div 
        v-for="chat in filteredConversations" 
        :key="chat.contact.id"
        class="chat-item"
        :class="{ active: activeContactId === chat.contact.id }"
        @click="selectChat(chat)"
      >
        <div class="avatar-box">
          <img v-if="chat.contact.avatar" :src="resolveAvatarSrc(chat.contact.avatar)" />
          <div v-else class="avatar-placeholder">{{ chat.contact.name[0] }}</div>
        </div>
        
        <div class="chat-info">
          <div class="chat-top">
            <span class="name">{{ chat.contact.name }}</span>
            <span class="time">{{ formatTime(chat.last_message_time) }}</span>
          </div>
          <div class="last-msg">{{ chat.last_message }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.social-chat-list {
  width: 250px;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-glass);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

:global(.app-light) .social-chat-list {
    background: #f7f7f7;
}

.header {
  padding: 10px 16px;
}

.search-header {
  padding: 10px 16px;
  display: flex;
  align-items: center;
}

.search-box {
  flex: 1;
  background: var(--bg-input);
  border-radius: 6px;
  display: flex;
  align-items: center;
  padding: 6px 10px;
  gap: 8px;
  border: 1px solid var(--border-glass);
}

.search-icon {
  width: 14px;
  height: 14px;
  opacity: 0.5;
}

.search-box input {
  background: transparent;
  border: none;
  color: var(--text-color);
  font-size: 0.85rem;
  width: 100%;
  outline: none;
}

.chat-items {
  flex: 1;
  overflow-y: auto;
}

.chat-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.2s;
  border-radius: 6px; /* 🧱 Base border radius */
}

.chat-item:hover {
  background-color: var(--bg-social-item-hover);
}

.chat-item.active {
  background-color: var(--bg-social-item-active);
  border-radius: 6px;
}

.avatar-box {
  width: 44px;
  height: 44px;
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg-active);
  flex-shrink: 0;
}

.avatar-box img { width: 100%; height: 100%; object-fit: cover; }
.avatar-placeholder { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: var(--color-primary); color: white; font-weight: 600; font-size: 1rem; }

.chat-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.chat-top {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}

.name {
  font-weight: 500;
  font-size: 0.95rem;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.time {
  font-size: 0.7rem;
  opacity: 0.4;
}

.last-msg {
  font-size: 0.8rem;
  opacity: 0.5;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-state {
  padding: 40px 20px;
  text-align: center;
  opacity: 0.4;
  font-size: 0.9rem;
}
</style>
