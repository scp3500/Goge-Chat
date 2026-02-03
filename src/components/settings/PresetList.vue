<script setup>
import draggable from 'vuedraggable';
import { computed, ref, onMounted, onUnmounted } from 'vue';

const props = defineProps({
  presets: Array,
  activePresetId: String
});

const emit = defineEmits(['update:activePresetId', 'reorder', 'add', 'rename', 'delete']);

// 右键菜单状态
const showMenu = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const targetPreset = ref(null);
const editingId = ref(null);
const editingName = ref('');

const setInputRef = (el) => {
  if (el) el.focus();
};

const openContextMenu = (preset, e) => {
  targetPreset.value = preset;
  menuPos.value = { x: e.clientX, y: e.clientY };
  showMenu.value = true;
};

const closeMenu = () => {
  showMenu.value = false;
};

const startRename = () => {
  if (targetPreset.value) {
    editingId.value = targetPreset.value.id;
    editingName.value = targetPreset.value.name;
    closeMenu();
  }
};

const handleRename = () => {
  if (editingId.value && editingName.value.trim()) {
    emit('rename', editingId.value, editingName.value.trim());
    editingId.value = null;
    editingName.value = '';
  }
};

const cancelRename = () => {
  editingId.value = null;
  editingName.value = '';
};

const handleDelete = () => {
  if (targetPreset.value) {
    if (targetPreset.value.isDefault) {
      alert('默认配置不能删除');
      closeMenu();
      return;
    }
    emit('delete', targetPreset.value.id);
    closeMenu();
  }
};

onMounted(() => { window.addEventListener('click', closeMenu); });
onUnmounted(() => { window.removeEventListener('click', closeMenu); });

const list = computed({
  get() {
    return props.presets || [];
  },
  set(newValue) {
    emit('reorder', newValue);
  }
});
</script>

<template>
  <section class="preset-sidebar">
    <div class="sidebar-header">
      <div class="search-box">
        <input type="text" placeholder="搜索配置..." />
        <span class="search-icon">🔍</span>
      </div>
      <button class="add-btn" @click="$emit('add')" title="添加自定义配置">
        <span class="plus-icon">＋</span>
      </button>
    </div>
    
    <div class="list-container modern-scroll">
      <draggable 
        v-model="list" 
        item-key="id"
        class="draggable-list"
        handle=".drag-handle"
        animation="200"
      >
        <template #item="{ element: preset }">
          <div
            :key="preset.id"
            class="list-item"
            :class="{ active: activePresetId === preset.id }"
            @click="$emit('update:activePresetId', preset.id)"
            @contextmenu.prevent="openContextMenu(preset, $event)"
          >
            <div class="item-left">
              <span class="drag-handle">⋮⋮</span>
              <span class="p-icon">⚙️</span>
              
              <!-- 重命名输入框 -->
              <input 
                v-if="editingId === preset.id"
                class="edit-input"
                v-model="editingName"
                @blur="handleRename"
                @keyup.enter="handleRename"
                @keyup.esc="cancelRename"
                @click.stop
                :ref="setInputRef"
              />
              <span v-else class="p-name">{{ preset.name }}</span>
            </div>
          </div>
        </template>
      </draggable>
    </div>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div v-if="showMenu" class="glass-menu" :style="{ top: menuPos.y + 'px', left: menuPos.x + 'px' }">
        <div class="menu-item" @click="startRename">✎ 重命名</div>
        <div class="menu-sep" v-if="!targetPreset?.isDefault"></div>
        <div class="menu-item delete" v-if="!targetPreset?.isDefault" @click="handleDelete">🗑 删除配置</div>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.preset-sidebar { width: 260px; background: var(--bg-sidebar); border-right: 1px solid var(--border-glass); display: flex; flex-direction: column; }
.sidebar-header { padding: 16px; display: flex; align-items: center; gap: 8px; }
.search-box { background: var(--bg-input); border-radius: 8px; padding: 8px 12px; display: flex; align-items: center; gap: 8px; flex: 1; }
.search-box input { background: transparent; border: none; color: var(--text-color-white); outline: none; flex: 1; font-size: 13px; }
.add-btn { width: 32px; height: 32px; border-radius: 8px; background: var(--bg-glass); border: 1px solid var(--border-glass); color: var(--text-color-white); display: flex; align-items: center; justify-content: center; cursor: pointer; transition: all 0.2s; font-size: 16px; }
.add-btn:hover { background: var(--bg-glass-hover); }
.list-item { display: flex; align-items: center; justify-content: space-between; padding: 12px; border-radius: 12px; cursor: pointer; margin-bottom: 4px; transition: background 0.2s; }
.list-item:hover { background: var(--bg-glass-hover); }
.list-item.active { background: var(--bg-glass-active); }
.item-left { display: flex; align-items: center; gap: 12px; }
.p-icon {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-glass);
  border: 1px solid var(--border-glass);
  border-radius: 10px;
  font-size: 16px;
}

.drag-handle { cursor: grab; color: var(--text-tertiary); font-size: 14px; user-select: none; padding-right: 4px; }
.drag-handle:active { cursor: grabbing; }
.list-item:hover .drag-handle { color: var(--text-secondary); }
.p-name { font-size: 14px; font-weight: 500; color: var(--text-color); }

.draggable-list {
  display: flex;
  flex-direction: column;
}

.sortable-ghost {
  opacity: 0.5;
  background: var(--bg-glass-active) !important;
}

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

.menu-item { padding: 8px 12px; font-size: 13px; color: var(--text-color); border-radius: 6px; cursor: pointer; user-select: none; }
.menu-item:hover { background: var(--bg-menu-hover); color: var(--text-color-white); }
.menu-item.delete { color: var(--color-error-light); }
.menu-sep { height: 1px; background: var(--border-menu); margin: 4px 0; }

.edit-input {
  background: var(--bg-input-focus);
  border: 1px solid var(--border-glass-bright);
  color: var(--text-color-white);
  font-size: 13px;
  border-radius: 4px;
  outline: none;
  padding: 4px 8px;
  width: 140px;
}
</style>
