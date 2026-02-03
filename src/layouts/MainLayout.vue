<script setup>
import { ref, watch, onMounted } from 'vue';
import SocialSidebar from '../components/layout/SocialSidebar.vue';
import SocialChatList from '../components/layout/SocialChatList.vue';
import SocialHistorySidebar from '../components/layout/SocialHistorySidebar.vue';
import SettingsModal from '../components/settings/SettingsModal.vue';
import { useSettingsStore } from '../stores/settings';
import { useChatStore } from '../stores/chat';
import { AI_EVO_SVG } from '../constants/icons';
import { invoke } from '@tauri-apps/api/core';

const settingsStore = useSettingsStore();
const chatStore = useChatStore();

const props = defineProps({
  isLeftSidebarOpen: { type: Boolean, default: true },
  isHistoryOpen: { type: Boolean, default: false },
  activeModule: { type: String, default: 'chat' }
});

const emit = defineEmits(['update:activeModule']);

const selectedContact = ref(null);

const handleSelect = (contact) => {
  selectedContact.value = contact;
  // 💾 Persist selection
  if (contact) {
    chatStore.updateSocialContactId(contact.id);
  }
  // 🚪 Auto-close settings on selection
  if (settingsStore.isModalOpen) {
    settingsStore.closeSettings();
    chatStore.setChatViewActive(true);
  }
};

const handleContactSelect = (contact) => {
  handleSelect(contact);
  // Jump to chat module is now handled by Profile page "Send Message" button
  // emit('update:activeModule', 'chat'); 
};

const handleOpenProfile = () => {
  settingsStore.openSettings('profile');
  chatStore.setChatViewActive(false);
};

const handleSessionSelect = (session) => {
    // 0. Update contact first to ensure context is correct
    if (session.contact_id && (!selectedContact.value || selectedContact.value.id !== session.contact_id)) {
        chatStore.updateSocialContactId(session.contact_id);
    }
    
    // 1. If we are in another module (like address_book), switch back to chat
    if (props.activeModule !== 'chat') {
        emit('update:activeModule', 'chat');
    }

    // 2. ⚡️ Core Fix: If settings modal is open, close it to reveal the chat/messages
    if (settingsStore.isModalOpen) {
        settingsStore.closeSettings();
        chatStore.setChatViewActive(true);
    }
    
    // 3. Update session ID (Sidebar also does this, but we force it here to be sure)
    chatStore.updateSocialSessionId(session.id);
};

// 🔄 Reactive contact resolution
const refreshContact = async () => {
    try {
        if (chatStore.activeSocialContactId) {
            const contacts = await invoke("get_social_contacts");
            const found = contacts.find(c => c.id === chatStore.activeSocialContactId);
            if (found) {
                selectedContact.value = found;
                console.log("📍 [REFRESH] Restored active contact:", found.name);
            }
        } else {
            // 💡 [Initial Fallback]: If no persisted ID, pick the one with the latest message
            const recentChats = await invoke("get_recent_social_chats");
            if (recentChats && recentChats.length > 0) {
                const latest = recentChats[0].contact;
                selectedContact.value = latest;
                chatStore.updateSocialContactId(latest.id);
                console.log("📍 [REFRESH] No persisted contact, auto-selected latest:", latest.name);
            }
        }
    } catch (e) {
        console.error("Failed to refresh contact:", e);
    }
};

// 🔄 Restore persisted contact on mount
onMounted(refreshContact);

// 🕵️ Watch for external triggers or ID changes
watch(() => chatStore.activeSocialContactId, refreshContact);
// ✨ Add a version-based trigger for when data changes but ID doesn't
watch(() => chatStore.socialSessionVersion, refreshContact);


const handleCloseSettings = () => {
  settingsStore.closeSettings();
};
</script>

<template>
  <div class="main-layout">
    <div class="sidebars-container" :class="{ 'is-collapsed': !isLeftSidebarOpen }">
      <!-- Column: Message List (Recent Chats) - Shown in 'chat' module -->
      <SocialChatList 
        v-if="activeModule === 'chat'"
        :active-contact-id="selectedContact?.id"
        @select="handleSelect" 
      />
      
      <!-- Column: Contact List (Address Book) - Shown in 'address_book' module -->
      <SocialSidebar 
        v-if="activeModule === 'address_book'"
        :active-contact-id="selectedContact?.id"
        @select="handleContactSelect"
      />
    </div>

    <!-- New Column: History Sidebar (Secondary Sidebar) -->
    <div class="history-sidebar-container" :class="{ 'is-open': isHistoryOpen }">
      <SocialHistorySidebar 
        :active-contact="selectedContact"
        @select="handleSessionSelect"
      />
    </div>

    <main class="content-view">
      <Transition name="fade-scale" mode="out-in">
        <!-- Settings Mode -->
        <SettingsModal 
          v-if="settingsStore.isModalOpen" 
          @close="settingsStore.closeSettings()" 
        />

        <!-- Chat Mode -->
        <div v-else class="chat-view-container">
          <Transition name="fade-up" mode="out-in">
            <div v-if="selectedContact" :key="selectedContact.id" class="chat-wrapper">
               <slot :active-contact="selectedContact" :active-module="activeModule"></slot>
            </div>
            <div v-else class="social-placeholder">
               <div class="placeholder-icon" v-html="AI_EVO_SVG"></div>
               <p>选择一个联系人开始对话</p>
            </div>
          </Transition>
        </div>
      </Transition>
    </main>
  </div>
</template>

<style scoped>
.main-layout {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: var(--bg-main);
}

.sidebars-container {
  display: flex;
  height: 100%;
  width: 250px; /* One sidebar width */
  transition: all 0.3s cubic-bezier(0.05, 0.7, 0.1, 1);
  flex-shrink: 0;
  border-right: 1px solid var(--border-glass);
  overflow: hidden; /* Important for width: 0 animation */
}

.sidebars-container.is-collapsed {
  width: 0;
  border-right: none;
}

.history-sidebar-container {
  display: flex;
  height: 100%;
  width: 0;
  transition: all 0.3s cubic-bezier(0.05, 0.7, 0.1, 1);
  flex-shrink: 0;
  border-left: 1px solid var(--border-glass); /* Fixed: Should be left border */
  overflow: hidden;
}

.history-sidebar-container.is-open {
  width: 250px;
}

.content-view {
  flex: 1;
  height: 100%;
  overflow: hidden;
  position: relative;
  display: flex;
  flex-direction: column;
}

.social-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-color);
  opacity: 0.1;
  gap: 16px;
}

.placeholder-icon :deep(svg) {
  width: 80px;
  height: 80px;
}

.social-placeholder p {
  font-size: 1.1rem;
  letter-spacing: 1px;
}

.chat-view-container {
  width: 100%;
  height: 100%;
  position: relative;
  display: flex;
  flex-direction: column;
}

.chat-wrapper {
  width: 100%;
  height: 100%;
}

/* Transitions */
.fade-scale-enter-active,
.fade-scale-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.fade-scale-enter-from,
.fade-scale-leave-to {
  opacity: 0;
  transform: scale(0.98);
}

.fade-up-enter-active,
.fade-up-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.fade-up-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-up-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
