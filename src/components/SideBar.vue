<script setup>
import { ref, onMounted, onUnmounted } from "vue";

const props = defineProps(['active', 'list']);
const emit = defineEmits(['create', 'select', 'delete', 'rename']);

// --- [核心状态] ---
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

// --- [菜单逻辑] ---
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

// 处理全局按键逻辑
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
        v-for="item in props.list" 
        :key="item.id" 
        class="history-item"
        :class="{ 'active': props.active === item.id }"
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
  flex-shrink: 0; /* 【关键修复】：禁止在 Flex 布局中被压缩，防止变窄 [cite: 2026-01-20] */
  background: #1e1f20; 
  height: 100vh; 
  display: flex; 
  flex-direction: column; 
  border-right: 1px solid #2a2b2d; 
  user-select: none; 
}
.sidebar-header { padding: 16px; flex-shrink: 0; }
.new-chat-btn { width: 100%; background: #2b2c2e; color: #ececec; border: 1px solid #3d3e40; padding: 10px; border-radius: 6px; cursor: pointer; font-size: 13px; font-weight: 500; transition: all 0.2s; }
.new-chat-btn:hover { background: #3a3b3d; }

.history-container { 
  flex: 1; 
  padding: 0 8px; 
  overflow-y: auto; /* 【关键修复】：确保即使没有 modern-scroll 类，自身也能滚动 [cite: 2026-01-20] */
}

.history-item { display: flex; align-items: center; padding: 12px 16px; margin-bottom: 2px; font-size: 13px; border-radius: 6px; cursor: pointer; color: #999; position: relative; transition: background 0.2s; }
.history-item:hover { background: rgba(255, 255, 255, 0.05); }
.history-item.active { background: #2b2c2e; color: #fff; }
.history-item.active::before { content: ""; position: absolute; left: 0; width: 3px; height: 14px; background: #ececec; border-radius: 0 4px 4px 0; }

.title {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding-right: 20px;
}

.more-action-btn { position: absolute; right: 8px; background: transparent; border: none; color: #666; font-size: 16px; cursor: pointer; opacity: 0; }
.history-item:hover .more-action-btn { opacity: 1; }

.inline-edit-input { flex: 1; background: #3a3c3e; border: none; color: #fff; font-size: 13px; padding: 2px 4px; outline: none; width: 100%; border-radius: 4px; }

.glass-menu { position: fixed; z-index: 10000; background: rgba(30, 31, 32, 0.95); backdrop-filter: blur(12px); border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 10px; padding: 6px; min-width: 150px; box-shadow: 0 8px 24px rgba(0,0,0,0.3); }
.menu-item { padding: 8px 12px; font-size: 12px; color: #d1d1d1; border-radius: 6px; cursor: pointer; display: flex; align-items: center; }
.menu-item:hover { background: rgba(255, 255, 255, 0.08); color: #fff; }
.menu-item.delete { color: #ff6b6b; }
.menu-sep { height: 1px; background: rgba(255, 255, 255, 0.05); margin: 4px 0; }

.sidebar-footer { padding: 16px; border-top: 1px solid #2a2b2d; font-size: 10px; color: #555; text-align: center; flex-shrink: 0; }
</style>