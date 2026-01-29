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

const vFocus = {
  mounted: (el) => el.focus()
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
                v-focus
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
.preset-sidebar { width: 260px; background: #18191b; border-right: 1px solid rgba(255, 255, 255, 0.03); display: flex; flex-direction: column; }
.sidebar-header { padding: 16px; display: flex; align-items: center; gap: 8px; }
.search-box { background: #131314; border-radius: 8px; padding: 8px 12px; display: flex; align-items: center; gap: 8px; flex: 1; }
.search-box input { background: transparent; border: none; color: #fff; outline: none; flex: 1; font-size: 13px; }
.add-btn { width: 32px; height: 32px; border-radius: 8px; background: #2b2d31; border: 1px solid rgba(255,255,255,0.05); color: #fff; display: flex; align-items: center; justify-content: center; cursor: pointer; transition: all 0.2s; font-size: 16px; }
.add-btn:hover { background: #3f4148; }
.list-item { display: flex; align-items: center; justify-content: space-between; padding: 12px; border-radius: 12px; cursor: pointer; margin-bottom: 4px; transition: background 0.2s; }
.list-item:hover { background: rgba(255, 255, 255, 0.05); }
.list-item.active { background: #2b2d31; }
.item-left { display: flex; align-items: center; gap: 12px; }
.p-icon {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(40, 40, 45, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 10px;
  font-size: 16px;
}

.drag-handle { cursor: grab; color: #555; font-size: 14px; user-select: none; padding-right: 4px; }
.drag-handle:active { cursor: grabbing; }
.list-item:hover .drag-handle { color: #888; }
.p-name { font-size: 14px; font-weight: 500; color: #efefef; }

.draggable-list {
  display: flex;
  flex-direction: column;
}

.sortable-ghost {
  opacity: 0.5;
  background: #2b2d31 !important;
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

.menu-item { padding: 8px 12px; font-size: 13px; color: #d1d1d1; border-radius: 6px; cursor: pointer; user-select: none; }
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
  padding: 4px 8px;
  width: 140px;
}
</style>
