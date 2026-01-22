<script setup>
import { ref, onMounted, onUnmounted } from "vue";

const props = defineProps(['active', 'list']);
const emit = defineEmits(['create', 'select', 'delete', 'rename', 'reorder']); // 🩺 新增 reorder 用于同步顺序

// --- [核心状态 - 严格保留] ---
const editingId = ref(null);
const tempTitle = ref("");

const vFocusSelect = {
  mounted(el) {
    el.focus();
    el.select();
    const handler = (e) => {
      e.preventDefault();
      el.removeEventListener('mouseup', handler);
    };
    el.addEventListener('mouseup', handler);
  }
};

const enterEdit = (id, currentTitle) => {
  editingId.value = id;
  tempTitle.value = currentTitle;
  showMenu.value = false;
};

const saveRename = (id) => {
  if (editingId.value === null) return;
  const trimmed = tempTitle.value.trim();
  if (trimmed) emit('rename', id, trimmed);
  editingId.value = null;
};

// --- [🩺 新增：原生极致拖拽逻辑] ---
const dragIndex = ref(null);
const dropTargetIndex = ref(null);

const handleDragStart = (index, e) => {
  if (editingId.value !== null) {
    e.preventDefault();
    return;
  }
  dragIndex.value = index;
  // 🩺 核心修复：必须设置 Data，否则某些内核会显示红色禁止图标 [cite: 2026-01-22]
  e.dataTransfer.setData("text/plain", index.toString());
  e.dataTransfer.effectAllowed = "move";
  // 视觉反馈：稍微变淡
  e.target.style.opacity = "0.4";
};

const handleDragOver = (index, e) => {
  // 🩺 核心修复：必须阻止默认行为，否则浏览器会显示禁止图标 [cite: 2026-01-22]
  e.preventDefault(); 
  e.dataTransfer.dropEffect = "move";
  dropTargetIndex.value = index;
};

const handleDragEnter = (e) => {
  // 🩺 核心修复：进入区域也必须拦截默认行为
  e.preventDefault();
};

const handleDragEnd = (e) => {
  dragIndex.value = null;
  dropTargetIndex.value = null;
  if (e.target) e.target.style.opacity = "";
};

const handleDrop = (index, e) => {
  e.preventDefault();
  e.stopPropagation();
  
  if (dragIndex.value === null || dragIndex.value === index) return;
  
  // 生成新顺序并推送给父组件
  const newList = [...props.list];
  const [movedItem] = newList.splice(dragIndex.value, 1);
  newList.splice(index, 0, movedItem);
  
  emit('reorder', newList);
  handleDragEnd(e);
};

// --- [菜单逻辑 - 严格保留] ---
const showMenu = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const targetId = ref(null);

const handleContextMenu = (id, e) => {
  e.preventDefault();
  e.stopPropagation();
  targetId.value = id;
  menuPos.value = { x: e.clientX, y: e.clientY };
  showMenu.value = true;
};

const closeMenu = () => { showMenu.value = false; };

const handleGlobalKey = (e) => {
  if (e.key === 'F2' && props.active !== null) {
    const item = props.list.find(i => i.id === props.active);
    if (item) enterEdit(item.id, item.title);
  }
  if (e.key === 'Escape') {
    editingId.value = null;
    closeMenu();
  }
};

onMounted(() => {
  window.addEventListener('click', closeMenu);
  window.addEventListener('keydown', handleGlobalKey);
});

onUnmounted(() => {
  window.removeEventListener('click', closeMenu);
  window.removeEventListener('keydown', handleGlobalKey);
});
</script>

<template>
  <aside class="sidebar">
    <header class="sidebar-header">
      <button class="new-chat-btn" @click="emit('create')">发起新对话</button>
    </header>
    
    <nav class="history-container modern-scroll">
      <div 
        v-for="(item, index) in props.list" 
        :key="item.id" 
        class="history-item"
        :class="{ 
          'active': props.active === item.id,
          'is-dragging': dragIndex === index,
          'drop-hint': dropTargetIndex === index && dragIndex !== index
        }"
        :draggable="editingId === null" 
        @dragstart="handleDragStart(index, $event)"
        @dragover="handleDragOver(index, $event)"
        @dragenter="handleDragEnter"
        @dragend="handleDragEnd"
        @drop="handleDrop(index, $event)"
        @click="emit('select', item.id)"
        @contextmenu="handleContextMenu(item.id, $event)" 
      >
        <input 
          v-if="editingId === item.id"
          v-focus-select
          v-model="tempTitle"
          class="inline-edit-input"
          @keyup.enter.stop="saveRename(item.id)"
          @blur="saveRename(item.id)"
          @click.stop
        />
        <span v-else class="title" @dblclick="enterEdit(item.id, item.title)">
          {{ item.title }}
        </span>

        <button 
          v-if="editingId !== item.id"
          class="more-action-btn" 
          @click.stop="handleContextMenu(item.id, $event)"
        >⋯</button>
      </div>
    </nav>

    <Teleport to="body">
      <div v-if="showMenu" class="glass-menu" :style="{ top: menuPos.y + 'px', left: menuPos.x + 'px' }" @click.stop>
        <div class="menu-item" @click="enterEdit(targetId, props.list.find(i => i.id === targetId)?.title)">✎ 重命名 (F2)</div>
        <div class="menu-sep"></div>
        <div class="menu-item delete" @click="emit('delete', targetId); closeMenu()">🗑 删除对话</div>
      </div>
    </Teleport>

    <footer class="sidebar-footer">
      <div class="user-info">DEEPSEEK V3</div>
    </footer>
  </aside>
</template>

<style scoped>
.sidebar { 
  width: 260px; 
  flex-shrink: 0; 
  background: #111214; /* 稍微调深，增加现代感感 */
  height: 100vh; 
  display: flex; 
  flex-direction: column; 
  border-right: 1px solid #2a2b2d; 
  user-select: none; 
}
.sidebar-header { padding: 16px; flex-shrink: 0; }
.new-chat-btn { width: 100%; background: #2b2c2e; color: #ececec; border: 1px solid #3d3e40; padding: 10px; border-radius: 6px; cursor: pointer; font-size: 13px; font-weight: 500; transition: all 0.2s; }
.new-chat-btn:hover { background: #3a3b3d; }

.history-container { flex: 1; padding: 0 8px; overflow-y: auto; }

/* 🩺 现代 UI 增强：强制 Pointer 指针 */
.history-item { 
  display: flex; 
  align-items: center; 
  padding: 12px 16px; 
  margin-bottom: 2px; 
  font-size: 13px; 
  border-radius: 8px; 
  /* ✨ 核心：强制 Pointer 形状，拒绝握拳 */
  cursor: pointer !important; 
  color: #999; 
  position: relative; 
  transition: background 0.2s;
  /* 解决拖拽兼容性 */
  -webkit-user-drag: element;
}

.history-item:hover { background: rgba(255, 255, 255, 0.05); }
.history-item.active { background: #2b2c2e; color: #fff; }
.history-item.active::before { content: ""; position: absolute; left: 0; width: 3px; height: 14px; background: #ececec; border-radius: 0 4px 4px 0; }

/* 拖拽时的蓝线指示 (编辑器风格) */
.history-item.drop-hint::after {
  content: "";
  position: absolute;
  left: 8px;
  right: 8px;
  bottom: -2px;
  height: 2px;
  background: #0078d4;
  border-radius: 2px;
  z-index: 10;
}

.title {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding-right: 20px;
}

.more-action-btn { position: absolute; right: 8px; background: transparent; border: none; color: #666; font-size: 16px; cursor: pointer; opacity: 0; }
.history-item:hover .more-action-btn { opacity: 1; }

.inline-edit-input { flex: 1; background: #3a3c3e; border: none; color: #fff; font-size: 13px; padding: 2px 4px; outline: none; width: 100%; border-radius: 4px; cursor: text !important; }

.glass-menu { position: fixed; z-index: 10000; background: rgba(30, 31, 32, 0.95); backdrop-filter: blur(12px); border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 10px; padding: 6px; min-width: 150px; box-shadow: 0 8px 24px rgba(0,0,0,0.3); }
.menu-item { padding: 8px 12px; font-size: 12px; color: #d1d1d1; border-radius: 6px; cursor: pointer; display: flex; align-items: center; }
.menu-item:hover { background: rgba(255, 255, 255, 0.08); color: #fff; }
.menu-item.delete { color: #ff6b6b; }
.menu-sep { height: 1px; background: rgba(255, 255, 255, 0.05); margin: 4px 0; }

.sidebar-footer { padding: 16px; border-top: 1px solid #2a2b2d; font-size: 10px; color: #555; text-align: center; flex-shrink: 0; }
</style>