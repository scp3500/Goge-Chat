<script setup>
import { ref, computed, onMounted, nextTick, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useChatStore } from '../../stores/chat';
import { useUIStore } from '../../stores/ui';
import { 
    SEARCH_SVG, 
    PLUS_SVG, 
    EDIT_SVG, 
    TRASH_SVG, 
    CHECK_SVG, 
    CLOSE_SVG 
} from '../../constants/icons';

const props = defineProps({
  activeContact: { type: Object, default: null }
});

const chatStore = useChatStore();
const uiStore = useUIStore();
const sessions = ref([]);
const loading = ref(false);
const searchQuery = ref("");
const editingSessionId = ref(null);
const editTitleInput = ref("");
const editInputRef = ref(null);


const loadSessions = async () => {
  if (!props.activeContact?.id) return;
  
  try {
    loading.value = true;
    const res = await invoke("get_social_sessions", { contactId: props.activeContact.id });
    sessions.value = res;
  } catch (e) {
    console.error("Failed to load sessions:", e);
  } finally {
    loading.value = false;
  }
};

const filteredSessions = computed(() => {
    if (!searchQuery.value.trim()) return sessions.value;
    return sessions.value.filter(s => 
        s.title.toLowerCase().includes(searchQuery.value.toLowerCase())
    );
});

const handleCreateSession = async () => {
    if (!props.activeContact?.id) return;
    try {
        const newId = await invoke("create_social_session", { 
            contactId: props.activeContact.id, 
            title: "新对话" 
        });
        await loadSessions();
        chatStore.updateSocialSessionId(newId);
    } catch(e) {
        console.error("Create session failed:", e);
    }
};

const handleEditSession = (session, event) => {
    event.stopPropagation(); // Prevent selection
    editingSessionId.value = session.id;
    editTitleInput.value = session.title;
    
    nextTick(() => {
        const inputEl = Array.isArray(editInputRef.value) ? editInputRef.value[0] : editInputRef.value;
        if (inputEl) {
            inputEl.focus();
        }
    });
};

const handleSaveEdit = async (session) => {
    if (!editTitleInput.value.trim()) return;
    try {
        await invoke("update_social_session_title", { 
            id: session.id, 
            title: editTitleInput.value.trim() 
        });
        editingSessionId.value = null;
        await loadSessions();
    } catch(e) {
        console.error("Update title failed:", e);
    }
};

const handleCancelEdit = () => {
    editingSessionId.value = null;
};

const handleDeleteSession = async (session, event) => {
    event.stopPropagation();
    if (!confirm("确定要删除这个会话吗？")) return;
    
    try {
        await invoke("delete_social_session", { id: session.id });
        const isCurrent = chatStore.activeSocialSessionId === session.id;
        await loadSessions();
        
        // If deleted current session, select another one or create new
        if (isCurrent) {
            if (sessions.value.length > 0) {
                chatStore.updateSocialSessionId(sessions.value[0].id);
            } else {
                handleCreateSession();
            }
        }
    } catch(e) {
        console.error("Delete session failed:", e);
    }
};

const formatTime = (timeStr) => {
  if (!timeStr) return '';
  // 🛡️ Fix: Handle both SQLite "YYYY-MM-DD HH:MM:SS" (needs Z) and ISO "YYYY-MM-DDTHH:MM:SSZ"
  let normalizedTime = timeStr;
  if (!normalizedTime.includes('Z') && !normalizedTime.includes('+') && !normalizedTime.includes('T')) {
      // If it looks like SQLite raw string (e.g. 2024-02-01 12:00:00), treat as UTC
     normalizedTime = normalizedTime.replace(' ', 'T') + 'Z';
  } else if (!normalizedTime.includes('Z') && !normalizedTime.includes('+')) {
      normalizedTime += 'Z';
  }
  
  const date = new Date(normalizedTime);
  const now = new Date();
  
  if (date.toLocaleDateString() === now.toLocaleDateString()) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: false });
  }
  return date.toLocaleDateString([], { month: 'numeric', day: 'numeric' });
};

const selectSession = (session) => {
  if (editingSessionId.value === session.id) return;
  chatStore.updateSocialSessionId(session.id);
};

// Reload when contact changes or title is auto-updated
watch(() => [props.activeContact?.id, chatStore.socialSessionVersion], () => {
    loadSessions();
});

// Also watch for external trigger to reload (if needed)
watch(() => chatStore.activeSocialSessionId, (newId) => {
    // Optional: could reload to refresh 'updated_at' time
    // loadSessions();
});

onMounted(() => {
  loadSessions();
  // ⚡️ Auto-refresh when window regains focus to keep timestamps fresh
  window.addEventListener('focus', loadSessions);
});

// Clean up
import { onUnmounted } from 'vue';
onUnmounted(() => {
  window.removeEventListener('focus', loadSessions);
});
</script>

<template>
  <div class="social-history-sidebar">
    <div class="history-header">
      <h2>历史会话</h2>
      <button class="add-btn" @click="handleCreateSession" title="新建会话" v-html="PLUS_SVG"></button>
    </div>
    
    <div class="search-box">
        <div class="search-input-wrapper">
             <i class="search-icon" v-html="SEARCH_SVG"></i>
            <input 
                v-model="searchQuery" 
                placeholder="搜索消息..." 
                class="search-input"
            />
        </div>
    </div>
    
    <div class="history-list modern-scroll">
      <div v-if="loading" class="loading-state">
          加载中...
      </div>
      <div v-else-if="filteredSessions.length === 0" class="empty-state">
          {{ searchQuery ? '未找到相关会话' : '暂无历史会话' }}
      </div>
      <TransitionGroup v-else name="list-slide-up" tag="div" class="list-container">
        <div 
          v-for="session in filteredSessions" 
          :key="session.id" 
          class="history-item"
          :class="{ active: chatStore.activeSocialSessionId === session.id, editing: editingSessionId === session.id }"
          @click="selectSession(session)"
          @dblclick="uiStore.isHistoryOpen = false"
        >
          <!-- Editing Mode -->
          <div v-if="editingSessionId === session.id" class="edit-mode" @click.stop>
              <input 
                  ref="editInputRef"
                  v-model="editTitleInput" 
                  class="edit-input"
                  @keyup.enter="handleSaveEdit(session)"
                  @keyup.esc="handleCancelEdit"
                  @blur="handleSaveEdit(session)" 
              />
              <!-- Blur auto-save is usually enough, buttons optional if space limited -->
          </div>

          <!-- Display Mode -->
          <div v-else class="item-content">
            <div class="item-main">
              <div class="topic-title">{{ session.title }}</div>
              <div class="topic-meta">{{ formatTime(session.updated_at || session.created_at) }}</div>
            </div>
            
            <div class="item-actions">
                <button class="action-btn edit" @click="handleEditSession(session, $event)" v-html="EDIT_SVG"></button>
                <button class="action-btn delete" @click="handleDeleteSession(session, $event)" v-html="TRASH_SVG"></button>
            </div>
          </div>
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<style scoped>
.social-history-sidebar {
  width: 260px; /* Slightly wider for actions */
  height: 100%;
  background: var(--bg-sidebar); /* Use standard sidebar bg */
  border-left: 1px solid var(--border-color); /* Fixed: Should be left border */
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.history-header {
  padding: 16px 16px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.history-header h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-color);
}

.add-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-secondary);
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
}

.add-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--color-primary);
}

.search-box {
    padding: 0 16px 12px;
}

.search-input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
}

.search-icon {
    position: absolute;
    left: 10px;
    width: 14px;
    height: 14px;
    color: var(--text-tertiary);
    pointer-events: none;
    display: flex; /* Ensure SVG renders correctly */
}
/* Ensure deep SVG styling */
.search-icon :deep(svg) {
    width: 100%;
    height: 100%;
}

.search-input {
    width: 100%;
    padding: 8px 12px 8px 32px;
    border-radius: 6px;
    border: 1px solid var(--border-color);
    background: var(--bg-input);
    color: var(--text-color);
    font-size: 0.9rem;
    outline: none;
    transition: all 0.2s;
}

.search-input:focus {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px var(--color-primary-alpha-30);
}

.history-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 12px;
}

.history-item {
  padding: 10px 12px;
  margin-bottom: 4px;
  cursor: pointer;
  border-radius: 8px;
  transition: all 0.2s ease;
  position: relative;
}

.history-item:hover {
  background: var(--bg-social-item-hover);
}

.history-item.active {
  background: var(--bg-social-item-active); /* Slightly darker/distinct from hover */
}

.item-content {
  display: flex;
  justify-content: space-between;
  align-items: center; /* Center vertically to stabilize height */
  gap: 8px;
  min-height: 44px; /* Ensure a stable height */
}

.item-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.topic-title {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.topic-meta {
  font-size: 0.75rem;
  color: var(--text-tertiary);
}

/* Actions appear on hover, but we don't hide the meta anymore */
.item-actions {
    visibility: hidden;
    opacity: 0;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: all 0.2s;
}

.history-item:hover .item-actions,
.history-item.active .item-actions {
    visibility: visible;
    opacity: 1;
}

/* Ensure meta and actions can coexist or meta stays visible */
.history-item:hover .topic-meta,
.history-item.active .topic-meta {
    display: block; 
}

.action-btn {
    background: none;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: var(--text-tertiary);
    border-radius: 4px;
    display: flex;
}

.action-btn:hover {
    background: var(--bg-glass-active);
    color: var(--text-color);
}

.action-btn.delete:hover {
    color: var(--color-danger);
    background: var(--color-danger-alpha-5);
}

.action-btn :deep(svg) {
    width: 14px;
    height: 14px;
}

/* Edit Mode */
.edit-mode {
    width: 100%;
}

.edit-input {
    width: 100%;
    padding: 4px 6px;
    border-radius: 4px;
    border: 1px solid var(--color-primary);
    outline: none;
    background: var(--bg-input);
    color: var(--text-color);
    font-size: 0.9rem;
}

.loading-state, .empty-state {
    padding: 20px;
    text-align: center;
    color: var(--text-tertiary);
    font-size: 0.9rem;
}

/* Dark Mode Support handled via vars, but explicit overrides if needed */
.history-item.active {
  background: var(--bg-social-item-active);
  color: var(--color-social-item-active-text);
}

.history-item.active .topic-title,
.history-item.active .topic-meta {
    color: var(--color-social-item-active-text);
}

/* --- Transition Animations --- */
.list-slide-up-enter-active,
.list-slide-up-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.list-slide-up-enter-from {
  opacity: 0;
  transform: translateY(12px);
  filter: blur(4px);
}

.list-slide-up-leave-to {
  opacity: 0;
  transform: scale(0.95);
}

/* Ensure list items move smoothly when others are added/removed */
.list-slide-up-move {
  transition: transform 0.3s ease;
}
</style>
