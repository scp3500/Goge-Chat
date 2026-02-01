<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { SEARCH_SVG, PLUS_SVG } from '../../constants/icons';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { useConfigStore } from '../../stores/config';
import AIContactModal from '../modals/AIContactModal.vue';

import { resolveSocialAvatar } from '../../utils/social';
const resolveAvatarSrc = resolveSocialAvatar;

const props = defineProps({
  isCollapsed: { type: Boolean, default: false }
});

const emit = defineEmits(['select']);

const configStore = useConfigStore();
const searchQuery = ref('');
const activeContactId = ref(null);
const clock = ref('');
let timer = null;

const showClock = computed(() => configStore.settings.chatMode.showSocialClock);

const showModal = ref(false);
const editingContact = ref(null);

const showMenu = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const targetContact = ref(null);

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

const loadData = async () => {
  // Load Profile
  try {
    profile.value = await invoke('get_social_profile');
  } catch (e) {
    console.error('⚠️ Failed to load profile (using default):', e);
    // Keep default placeholder
  }

  // Load Groups
  try {
    groups.value = await invoke('get_social_groups');
  } catch (e) {
    console.error('❌ Failed to load groups:', e);
  }

  // Load Contacts
  try {
    contacts.value = await invoke('get_social_contacts');
  } catch (e) {
    console.error('❌ Failed to load contacts:', e);
  }
};

onMounted(async () => {
  updateClock();
  timer = setInterval(updateClock, 1000);
  await loadData();
  window.addEventListener('click', closeMenu);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
  window.removeEventListener('click', closeMenu);
});

// 过滤后的联系人列表
const filteredContacts = computed(() => {
  if (!searchQuery.value) return contacts.value;
  return contacts.value.filter(c => 
    c.name.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});

const handleAddContact = () => {
  editingContact.value = null;
  showModal.value = true;
};

const selectContact = (contact) => {
  activeContactId.value = contact.id;
  emit('select', contact);
};

const openContextMenu = (contact, e) => {
  targetContact.value = contact;
  menuPos.value = { x: e.clientX, y: e.clientY };
  showMenu.value = true;
};

const closeMenu = () => {
  showMenu.value = false;
};

const handleEdit = () => {
  editingContact.value = targetContact.value;
  showModal.value = true;
  closeMenu();
};

const handleDelete = async () => {
  if (confirm(`确定要删除联系人 ${targetContact.value.name} 吗？`)) {
    try {
      await invoke('delete_social_contact', { id: targetContact.value.id });
      await loadData();
    } catch (e) {
      alert('删除失败: ' + e);
    }
  }
  closeMenu();
};

  const handleConfirmModal = async (data) => {
  try {
    // Ensure we have a valid group ID
    let targetGroupId = null;
    if (groups.value.length > 0) {
      targetGroupId = groups.value[0].id;
    } else {
      console.warn('⚠️ No groups found! Contact will have NULL group_id.');
    }

    if (editingContact.value) {
      await invoke('update_social_contact', { 
        id: editingContact.value.id,
        ...data
      });
    } else {
      // Use snake_case 'group_id' to match Rust argument strictly if auto-mapping fails
      // Tauri usually maps camelCase -> snake_case, but explicit snake_case is safer here if inconsistent
      await invoke('add_social_contact', { 
        ...data,
        group_id: targetGroupId 
      });
    }
    
    await loadData(); // Refresh list
    showModal.value = false;
  } catch (e) {
    console.error('❌ Failed to save contact:', e);
    alert('保存失败: ' + e);
  }
};
</script>

<template>
  <aside class="social-sidebar" :class="{ 'is-collapsed': isCollapsed }">
    <div class="sidebar-content">
      <!-- Search Box & Add Button -->
      <div class="search-header">
        <div class="search-box">
          <div class="search-icon" v-html="SEARCH_SVG"></div>
          <input type="text" v-model="searchQuery" placeholder="搜索联系人..." />
        </div>
        <button class="add-btn" @click="handleAddContact" title="添加联系人">
           <div class="icon-sm" v-html="PLUS_SVG"></div>
        </button>
      </div>

      <!-- Clock -->
      <div v-if="showClock" class="clock-display">
        {{ clock }}
      </div>

      <!-- Contact List -->
      <div class="contact-list modern-scroll">
        <!-- Loading / Empty State -->
        <div v-if="groups.length === 0 && contacts.length === 0" class="empty-state">
             <p>暂无联系人</p>
        </div>

        <div v-for="group in groups" :key="group.id" class="contact-group">
          <div class="group-header">
            <span class="group-name">{{ group.name }}</span>
          </div>
          <div class="group-items">
            <div 
              v-for="contact in filteredContacts.filter(c => c.group_id === group.id)" 
              :key="contact.id"
              class="contact-item"
              :class="{ 'active': activeContactId === contact.id }"
              @click="selectContact(contact)"
              @contextmenu.prevent="openContextMenu(contact, $event)"
            >
              <div class="avatar-sm">
                     <img v-if="contact.avatar" :src="resolveAvatarSrc(contact.avatar)" />
                 <div v-else class="avatar-placeholder-sm">{{ contact.name[0] }}</div>
              </div>
              <div class="contact-name">{{ contact.name }}</div>
            </div>
          </div>
        </div>

        <!-- Fallback for Ungrouped Contacts -->
        <div v-if="filteredContacts.some(c => !c.group_id || !groups.find(g => g.id === c.group_id))" class="contact-group">
            <div class="group-header">未分组 / 其他</div>
            <div class="group-items">
                <div 
                  v-for="contact in filteredContacts.filter(c => !c.group_id || !groups.find(g => g.id === c.group_id))" 
                  :key="contact.id"
                  class="contact-item"
                  :class="{ 'active': activeContactId === contact.id }"
                  @click="selectContact(contact)"
                  @contextmenu.prevent="openContextMenu(contact, $event)"
                >
                  <div class="avatar-sm">
                        <img v-if="contact.avatar" :src="resolveAvatarSrc(contact.avatar)" />
                     <div v-else class="avatar-placeholder-sm">{{ contact.name[0] }}</div>
                  </div>
                  <div class="contact-name">{{ contact.name }}</div>
                </div>
            </div>
        </div>
      </div>
    </div>

    <!-- AI Contact Modal -->
    <AIContactModal 
      :show="showModal" 
      :contact="editingContact"
      @close="showModal = false"
      @confirm="handleConfirmModal"
    />

    <!-- Context Menu -->
    <Teleport to="body">
      <div v-if="showMenu" class="glass-menu" :style="{ top: menuPos.y + 'px', left: menuPos.x + 'px' }" @click.stop>
        <div class="menu-item" @click="handleEdit">✎ 修改资料</div>
        <div class="menu-sep"></div>
        <div class="menu-item delete" @click="handleDelete">🗑 删除好友</div>
      </div>
    </Teleport>
  </aside>
</template>

<style scoped>
.empty-state {
    padding: 20px;
    text-align: center;
    opacity: 0.5;
    font-size: 14px;
}

.social-sidebar {
  width: 250px;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-glass);
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: width 0.3s;
  flex-shrink: 0;
}

:global(.app-light) .social-sidebar {
    background: #ebebeb;
}

.social-sidebar.is-collapsed {
  width: 0;
  border-right: none;
}

.sidebar-content {
  width: 100%;
  padding: 10px 12px 16px 12px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.search-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 4px;
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

.add-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-input-dim);
  border: 1px solid var(--border-glass-bright);
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  flex-shrink: 0;
}

.add-btn:hover {
  background: var(--color-primary-bg);
  border-color: var(--color-primary);
  color: var(--color-primary);
  transform: translateY(-1px);
}

.icon-sm :deep(svg) {
  width: 16px;
  height: 16px;
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
  border-radius: 8px;
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
  background: var(--bg-social-item-hover);
}

.contact-item.active {
  background: var(--bg-social-item-active);
}

.contact-item.active .contact-name {
  color: var(--theme-color);
  font-weight: 600;
}

.avatar-sm {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  overflow: hidden;
  background: var(--bg-active);
  flex-shrink: 0;
}

.avatar-sm img { width: 100%; height: 100%; object-fit: cover; }

.avatar-placeholder-sm { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; font-size: 0.85rem; background: var(--color-primary); color: white; font-weight: 600; }
.contact-name { font-size: 0.9rem; color: var(--text-color); }

/* Context Menu */
.glass-menu { 
  position: fixed; 
  z-index: 10000; 
  background: var(--bg-menu); 
  backdrop-filter: blur(12px); 
  border: 1px solid var(--border-menu); 
  border-radius: 10px; 
  padding: 6px; 
  min-width: 150px;
  box-shadow: var(--shadow-main);
}

.menu-item { 
    padding: 8px 12px; 
    font-size: 13px; 
    color: var(--text-color); 
    border-radius: 6px; 
    cursor: pointer; 
    display: flex;
    align-items: center;
    gap: 8px;
}

.menu-item:hover { background: var(--bg-menu-hover); color: var(--text-color-white); }
.menu-item.delete { color: #ff6b6b; }
.menu-sep { height: 1px; background: var(--border-menu); margin: 4px 0; }
</style>
