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

const setInputRef = (el) => {
  if (el) el.focus();
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
                :ref="setInputRef"
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
.provider-sidebar { width: 260px; background: var(--bg-sidebar); border-right: 1px solid var(--border-glass); display: flex; flex-direction: column; }
.sidebar-header { padding: 16px; display: flex; align-items: center; gap: 8px; }
.search-box { background: var(--bg-input); border-radius: 8px; padding: 8px 12px; display: flex; align-items: center; gap: 8px; flex: 1; border: 1px solid var(--border-glass); }
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
  background: var(--bg-model-icon);
  border: 1px solid var(--border-glass);
  border-radius: 10px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
  color: var(--color-model-icon);
  box-shadow: var(--shadow-main);
}

.list-item:hover .p-icon {
  transform: translateY(-1px);
  background: var(--bg-model-icon-hover);
  box-shadow: 0 5px 12px var(--bg-mask);
  border-color: var(--border-glass-bright);
}

.list-item.active .p-icon {
  background: var(--bg-model-icon-active);
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-primary-border), 0 4px 12px var(--bg-mask);
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
.status-tag { background: var(--bg-glass); color: var(--text-secondary); font-size: 10px; padding: 2px 6px; border-radius: 4px; }
.status-tag.on { background: var(--color-primary-bg); color: var(--color-primary); }
.status-tag.off { background: var(--bg-glass); color: var(--text-tertiary); }
.toggle-btn { width: 36px; height: 20px; background: var(--bg-glass-active); border-radius: 10px; border: none; cursor: pointer; position: relative; transition: background 0.3s; }
.toggle-btn.on { background: var(--color-primary); }
.toggle-slider { width: 16px; height: 16px; background: #fff; border-radius: 50%; position: absolute; top: 2px; left: 2px; transition: left 0.3s; }
.toggle-btn.on .toggle-slider { left: 18px; }

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
  background: var(--bg-main); 
  backdrop-filter: blur(12px); 
  border: 1px solid var(--border-glass-bright); 
  border-radius: 10px; 
  padding: 6px; 
  min-width: 150px; 
}

.menu-item { padding: 8px 12px; font-size: 13px; color: var(--text-color); border-radius: 6px; cursor: pointer; user-select: none; }
.menu-item:hover { background: var(--bg-glass-hover); color: var(--text-color-white); }
.menu-item.delete { color: #ff6b6b; }
.menu-sep { height: 1px; background: var(--border-glass); margin: 4px 0; }

.edit-input {
  background: var(--bg-input);
  border: 1px solid var(--border-glass-bright);
  color: var(--text-color-white);
  font-size: 13px;
  border-radius: 4px;
  outline: none;
  padding: 4px 8px;
  width: 140px;
}
</style>
