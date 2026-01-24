<script setup>
/**
 * HistoryList.vue - 历史记录列表
 * 职责：过滤历史条目，并管理右键菜单与编辑状态。
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import HistoryItem from '../components/HistoryItem.vue';
import draggable from 'vuedraggable';
import { useChatStore } from '../../../stores/chat';

const chatStore = useChatStore();
const drag = ref(false);

const props = defineProps(['list', 'active', 'filter', 'isCollapsed']);
const emit = defineEmits(['select', 'delete', 'rename', 'reorder', 'reorder-folders']);

const vFocus = {
  mounted: (el) => el.focus()
};

// 计算文件夹列表
const folderList = computed(() => {
    if (props.filter) return []; 
    return chatStore.folders;
});

// --- [ 状态管理 ] ---
const editingId = ref(null);
const editingFolderId = ref(null);
const showMenu = ref(false);
const showFolderMenu = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const targetId = ref(null);
const targetFolder = ref(null);

const openContextMenu = (id, e) => {
  if (props.isCollapsed) return;
  targetId.value = id;
  menuPos.value = { x: e.clientX, y: e.clientY };
  showMenu.value = true;
};

const openFolderMenu = (folder, e) => {
  targetFolder.value = folder;
  menuPos.value = { x: e.clientX, y: e.clientY };
  showFolderMenu.value = true;
};

const handleRenameFolder = (id, newName) => {
  chatStore.renameFolder(id, newName);
  editingFolderId.value = null;
};

const closeMenu = () => {
  showMenu.value = false;
  showFolderMenu.value = false;
};

// --- [ 拖拽逻辑：改为同步本地状态 + 后端通知 ] ---
const onSessionMove = async (evt, targetFolderId) => {
  if (evt.added) {
    const session = evt.added.element;
    const newIndex = evt.added.newIndex;
    
    // 1. 更新内存中的 folder_id
    session.folder_id = targetFolderId;
    await chatStore.moveSessionToFolder(session.id, targetFolderId);
    
    // 2. 获取当前文件夹（或未分类区域）的所有 session
    const currentSubList = props.list.filter(s => s.folder_id === targetFolderId);
    
    // 3. 将被添加的元素从原数组中过滤掉（因为它可能还在旧位置）并插入到新位置
    const otherSessionsInSubList = currentSubList.filter(s => s.id !== session.id);
    otherSessionsInSubList.splice(newIndex, 0, session);
    
    // 4. 调用内部重排序逻辑，统一更新全局列表
    handleInternalReorder(otherSessionsInSubList, targetFolderId);
  } else if (evt.moved) {
    // 处理同一列表内的移动
    const currentSubList = props.list.filter(s => s.folder_id === targetFolderId);
    handleInternalReorder(currentSubList, targetFolderId);
  }
};

const handleInternalReorder = (updatedSubList, folderId) => {
  // 确保子列表中的所有 session 都标记了正确的 folder_id
  updatedSubList.forEach(s => {
    s.folder_id = folderId;
  });

  // 构造新的完整列表顺序
  const newList = [];
  
  // 1. 遍历所有文件夹
  chatStore.folders.forEach(folder => {
    if (folder.id === folderId) {
      // 如果是当前正在排序的文件夹，使用更新后的子列表
      newList.push(...updatedSubList);
    } else {
      // 否则保持原有子列表
      const folderSessions = props.list.filter(s => s.folder_id === folder.id);
      newList.push(...folderSessions);
    }
  });
  
  // 2. 处理未分类列表
  if (folderId === null) {
    newList.push(...updatedSubList);
  } else {
    const unclassified = props.list.filter(s => !s.folder_id);
    newList.push(...unclassified);
  }

  emit('reorder', newList);
};

onMounted(() => { window.addEventListener('click', closeMenu); });
onUnmounted(() => { window.removeEventListener('click', closeMenu); });
</script>

<template>
  <nav class="history-container modern-scroll">
    <!-- 1. 文件夹区域 (仅根层级) -->
    <draggable
      v-if="!isCollapsed"
      :list="chatStore.folders"
      item-key="id"
      handle=".folder-header"
      ghost-class="drag-ghost"
      drag-class="drag-item-active"
      class="folders-area"
      :animation="200"
      @change="e => { if (e.moved) emit('reorder-folders', chatStore.folders) }"
    >
      <template #item="{ element: folder }">
        <div class="folder-group">
          <!-- 文件夹头部 -->
          <div
            class="folder-header"
            @click="chatStore.toggleFolder(folder.id)"
            @contextmenu.prevent="openFolderMenu(folder, $event)"
          >
            <span class="folder-icon" :class="{ 'is-collapsed': folder.is_collapsed }">
               <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="m6 9 6 6 6-6"/>
               </svg>
            </span>
            
            <input
              v-if="editingFolderId === folder.id"
              class="edit-input folder-edit"
              :value="folder.name"
              @blur="e => handleRenameFolder(folder.id, e.target.value)"
              @keyup.enter="e => handleRenameFolder(folder.id, e.target.value)"
              @click.stop
              v-focus
            />
            <span v-else class="folder-name">{{ folder.name.replace(/^[cr]:/g, '') }}</span>
          </div>

          <!-- 文件夹内的对话 (独立数据源：仅该文件夹的 session) -->
          <div v-if="!folder.is_collapsed" class="folder-items">
            <draggable
              :model-value="props.list.filter(s => s.folder_id === folder.id)"
              @update:model-value="val => handleInternalReorder(val, folder.id)"
              item-key="id"
              handle=".history-item"
              group="sessions"
              ghost-class="drag-ghost"
              drag-class="drag-item-active"
              class="drag-list"
              :animation="200"
              @change="e => onSessionMove(e, folder.id)"
            >
              <template #item="{ element: item }">
                <HistoryItem
                  :item="item"
                  :is-active="active === item.id"
                  :is-editing-id="editingId"
                  :is-collapsed="isCollapsed"
                  @select="id => emit('select', id)"
                  @contextmenu="openContextMenu"
                  @enter-edit="id => editingId = id"
                  @rename="(id, title) => { emit('rename', id, title); editingId = null; }"
                />
              </template>
            </draggable>
          </div>
        </div>
      </template>
    </draggable>

    <!-- 2. 外部对话区域 (独立数据源：仅 folder_id 为空的 session) -->
    <draggable
      :model-value="props.list.filter(s => !s.folder_id)"
      @update:model-value="val => handleInternalReorder(val, null)"
      item-key="id"
      handle=".history-item"
      group="sessions"
      :disabled="!!filter"
      ghost-class="drag-ghost"
      drag-class="drag-item-active"
      class="drag-list unclassified-list"
      :animation="200"
      @change="e => onSessionMove(e, null)"
    >
      <template #item="{ element: item }">
        <HistoryItem
          v-if="!filter || item.title.toLowerCase().includes(filter.toLowerCase())"
          :item="item"
          :is-active="active === item.id"
          :is-editing-id="editingId"
          :is-collapsed="isCollapsed"
          @select="id => emit('select', id)"
          @contextmenu="openContextMenu"
          @enter-edit="id => editingId = id"
          @rename="(id, title) => { emit('rename', id, title); editingId = null; }"
        />
      </template>
    </draggable>

    <Teleport to="body">
      <div v-if="showMenu" class="glass-menu" :style="{ top: menuPos.y + 'px', left: menuPos.x + 'px' }">
        <div class="menu-item" @click="editingId = targetId; closeMenu()">✎ 重命名 (F2)</div>
        <div class="menu-sep"></div>
        <div class="menu-item delete" @click="emit('delete', targetId); closeMenu()">🗑 删除对话</div>
        <div class="menu-sep"></div>
        <div class="menu-item" v-for="f in chatStore.folders" :key="f.id" @click="chatStore.moveSessionToFolder(targetId, f.id); closeMenu()">
          📁 移入: {{ f.name }}
        </div>
        <div class="menu-item" v-if="props.list.find(s => s.id === targetId)?.folder_id" @click="chatStore.moveSessionToFolder(targetId, null); closeMenu()">
          🚫 移出文件夹
        </div>
      </div>

      <div v-if="showFolderMenu" class="glass-menu" :style="{ top: menuPos.y + 'px', left: menuPos.x + 'px' }">
        <div class="menu-item" @click="editingFolderId = targetFolder.id; closeMenu()">✎ 重命名</div>
        <div class="menu-sep"></div>
        <div class="menu-item" @click="chatStore.deleteFolder(targetFolder.id); closeMenu()">🗑 删除文件夹</div>
      </div>
    </Teleport>
  </nav>
</template>

<style scoped>
.history-container { 
  flex: 1; 
  padding: 8px 0; 
  overflow-y: auto; 
  position: relative; 
  min-height: 100%;
}

.drag-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-height: 10px;
}

.unclassified-list {
  min-height: 100px;
  padding-bottom: 40px;
}

.folders-area {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.folder-group {
  display: flex;
  flex-direction: column;
}

.folder-header {
  --item-padding-left: 20px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px 6px 10px;
  color: #9aa0a6;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  user-select: none;
  transition: all 0.2s;
}

.folder-header:hover {
  color: #e8eaed;
  background: rgba(255, 255, 255, 0.05);
}

.folder-icon {
  display: flex;
  align-items: center;
  transition: transform 0.2s ease;
}

.folder-icon.is-collapsed {
  transform: rotate(-90deg);
}

.folder-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-left: 12px;
  border-left: 1px solid rgba(255, 255, 255, 0.05);
  margin-left: 17px;
  margin-top: 4px;
  margin-bottom: 8px;
}

.drag-ghost {
  opacity: 0.1;
  background: #fff !important;
}

.drag-item-active {
  opacity: 0.95 !important;
  background: rgba(45, 45, 45, 0.95) !important;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  transform: scale(1.02);
  z-index: 1000;
  cursor: grabbing !important;
}

.glass-menu { 
  position: fixed; 
  z-index: 10000; 
  background: rgba(30, 31, 32, 0.95); 
  backdrop-filter: blur(12px); 
  border: 1px solid rgba(255, 255, 255, 0.1); 
  border-radius: 10px; 
  padding: 6px; 
  min-width: 150px; 
}

.menu-item { padding: 8px 12px; font-size: 13px; color: #d1d1d1; border-radius: 6px; cursor: pointer; }
.menu-item:hover { background: rgba(255, 255, 255, 0.08); color: #fff; }
.menu-item.delete { color: #ff6b6b; }
.menu-sep { height: 1px; background: rgba(255, 255, 255, 0.05); margin: 4px 0; }

.edit-input {
  background: #2b2c2e;
  border: 1px solid #5f6368;
  color: #ffffff;
  font-size: 13px;
  border-radius: 4px;
  outline: none;
  padding: 2px 6px;
  width: 100%;
}

.folder-edit {
  height: 22px;
  font-size: 12px;
}
</style>
