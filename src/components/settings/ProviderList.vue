<script setup>
import draggable from 'vuedraggable';
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { getProviderIcon } from '../../assets/icons';

const props = defineProps({
  providers: Array,
  activeProviderId: String
});

const emit = defineEmits(['update:activeProviderId', 'toggleStatus', 'reorder', 'add', 'rename', 'delete']);

// 右键菜单状态
const showMenu = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const targetProvider = ref(null);
const editingId = ref(null);
const editingName = ref('');

const vFocus = {
  mounted: (el) => el.focus()
};

const openContextMenu = (provider, e) => {
  targetProvider.value = provider;
  menuPos.value = { x: e.clientX, y: e.clientY };
  showMenu.value = true;
};

const closeMenu = () => {
  showMenu.value = false;
};

const startRename = () => {
  if (targetProvider.value) {
    editingId.value = targetProvider.value.id;
    editingName.value = targetProvider.value.name;
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
  if (targetProvider.value) {
    emit('delete', targetProvider.value.id);
    closeMenu();
  }
};

onMounted(() => { window.addEventListener('click', closeMenu); });
onUnmounted(() => { window.removeEventListener('click', closeMenu); });

// 使用 computed 的 getter/setter 实现双向绑定
// 这样可以直接使用 props 数据，同时在拖拽时正确触发 reorder 事件
const list = computed({
  get() {
    console.log('[ProviderList] list getter, current props order:', props.providers?.map(p => p.id).join(','));
    return props.providers || [];
  },
  set(newValue) {
    console.log('[ProviderList] list setter, new order:', newValue.map(p => p.id).join(','));
    emit('reorder', newValue);
  }
});
</script>

<template>
  <section class="provider-sidebar">
    <div class="sidebar-header">
      <div class="search-box">
        <input type="text" placeholder="搜索模型平台..." />
        <span class="search-icon">🔍</span>
      </div>
      <button class="add-btn" @click="$emit('add')" title="添加自定义供应商">
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
        <template #item="{ element: provider }">
          <div
            :key="provider.id"
            class="list-item"
            :class="{ active: activeProviderId === provider.id }"
            @click="$emit('update:activeProviderId', provider.id)"
            @contextmenu.prevent="openContextMenu(provider, $event)"
          >
            <div class="item-left">
              <span class="drag-handle">⋮⋮</span>
              <span v-html="getProviderIcon(provider.icon)" class="p-icon"></span>
              
              <!-- 重命名输入框 -->
              <input 
                v-if="editingId === provider.id"
                class="edit-input"
                v-model="editingName"
                @blur="handleRename"
                @keyup.enter="handleRename"
                @keyup.esc="cancelRename"
                @click.stop
                v-focus
              />
              <span v-else class="p-name">{{ provider.name }}</span>
            </div>
            <div class="item-right">
              <button
                class="toggle-btn"
                :class="{ on: provider.status === 'on' }"
                @click.stop="$emit('toggleStatus', provider.id)"
              >
                <span class="toggle-slider"></span>
              </button>
              <span class="status-tag" :class="provider.status">{{ provider.status === 'on' ? 'ON' : 'OFF' }}</span>
            </div>
          </div>
        </template>
      </draggable>
    </div>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div v-if="showMenu" class="glass-menu" :style="{ top: menuPos.y + 'px', left: menuPos.x + 'px' }">
        <div class="menu-item" @click="startRename">✎ 重命名</div>
        <div class="menu-sep" v-if="targetProvider?.isCustom"></div>
        <div class="menu-item delete" v-if="targetProvider?.isCustom" @click="handleDelete">🗑 删除供应商</div>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.provider-sidebar { width: 260px; background: #18191b; border-right: 1px solid rgba(255, 255, 255, 0.03); display: flex; flex-direction: column; }
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
  background: rgba(220, 220, 225, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 10px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
  color: #1a1a1b;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12), inset 0 1px 1px rgba(255, 255, 255, 0.2);
}

.list-item:hover .p-icon {
  transform: translateY(-1px);
  background: rgba(235, 235, 240, 0.95);
  box-shadow: 0 5px 12px rgba(0, 0, 0, 0.18);
  border-color: rgba(255, 255, 255, 0.3);
}

.list-item.active .p-icon {
  background: #f0f0f5;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.25), 0 4px 12px rgba(0, 0, 0, 0.2);
}

.p-icon :deep(svg) {
  width: 20px;
  height: 20px;
  object-fit: contain;
}
.drag-handle { cursor: grab; color: #555; font-size: 14px; user-select: none; padding-right: 4px; }
.drag-handle:active { cursor: grabbing; }
.list-item:hover .drag-handle { color: #888; }
.item-right { display: flex; align-items: center; gap: 8px; }
.p-name { font-size: 14px; font-weight: 500; }
.status-tag.on { background: rgba(59, 130, 246, 0.15); color: #3b82f6; font-size: 10px; padding: 2px 6px; border-radius: 4px; }
.status-tag.off { background: rgba(255, 255, 255, 0.05); color: #888; font-size: 10px; padding: 2px 6px; border-radius: 4px; }
.toggle-btn { width: 36px; height: 20px; background: #555; border-radius: 10px; border: none; cursor: pointer; position: relative; transition: background 0.3s; }
.toggle-btn.on { background: #3b82f6; }
.toggle-slider { width: 16px; height: 16px; background: #fff; border-radius: 50%; position: absolute; top: 2px; left: 2px; transition: left 0.3s; }
.toggle-btn.on .toggle-slider { left: 18px; }

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
