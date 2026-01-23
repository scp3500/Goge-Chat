<script setup>
/**
 * HistoryList.vue - 历史记录列表
 * 职责：过滤历史条目，并管理右键菜单与编辑状态。
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import HistoryItem from '../components/HistoryItem.vue';

// 🚩 核心：增加 isCollapsed 接收，用于控制窄模式显示
const props = defineProps(['list', 'active', 'filter', 'isCollapsed']);
const emit = defineEmits(['select', 'delete', 'rename', 'reorder']);

// --- [ 🩺 核心修复：搜索过滤逻辑 ] ---
const filteredList = computed(() => {
  if (!props.filter) return props.list;
  const term = props.filter.toLowerCase();
  return props.list.filter(item => 
    item.title.toLowerCase().includes(term)
  );
});

// --- [ 状态管理：编辑与右键菜单 ] ---
const editingId = ref(null);
const showMenu = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const targetId = ref(null);

const openContextMenu = (id, e) => {
  // 🚩 窄模式下禁止右键菜单，防止菜单弹出位置偏移
  if (props.isCollapsed) return;
  
  targetId.value = id;
  menuPos.value = { x: e.clientX, y: e.clientY };
  showMenu.value = true;
};

const closeMenu = () => { showMenu.value = false; };
onMounted(() => { window.addEventListener('click', closeMenu); });
onUnmounted(() => { window.removeEventListener('click', closeMenu); });
</script>

<template>
  <nav class="history-container modern-scroll">
    <HistoryItem 
      v-for="item in filteredList" 
      :key="item.id"
      :item="item"
      :is-active="active === item.id"
      :is-editing-id="editingId"
      :is-collapsed="isCollapsed" 
      @select="id => emit('select', id)"
      @contextmenu="openContextMenu"
      @enter-edit="id => editingId = id"
      @rename="(id, title) => { emit('rename', id, title); editingId = null; }"
    />

    <Teleport to="body">
      <div v-if="showMenu" class="glass-menu" :style="{ top: menuPos.y + 'px', left: menuPos.x + 'px' }">
        <div class="menu-item" @click="editingId = targetId; closeMenu()">✎ 重命名 (F2)</div>
        <div class="menu-sep"></div>
        <div class="menu-item delete" @click="emit('delete', targetId); closeMenu()">🗑 删除对话</div>
      </div>
    </Teleport>
    
    <div v-if="filteredList.length === 0 && filter && !isCollapsed" class="empty-search">
      未找到相关对话
    </div>
  </nav>
</template>

<style scoped>
.history-container { 
  flex: 1; 
  /* 🚩 容器取消左右 padding，让 Item 在窄模式下能完美居中 */
  padding: 8px 0; 
  overflow-y: auto; 
  position: relative; 
}

.empty-search { 
  text-align: center; 
  color: #555; 
  font-size: 12px; 
  margin-top: 40px; 
  padding: 0 16px;
}

/* 🚩 磨砂玻璃右键菜单样式 */
.glass-menu { 
  position: fixed; 
  z-index: 10000; 
  background: rgba(30, 31, 32, 0.95); 
  backdrop-filter: blur(12px); 
  border: 1px solid rgba(255, 255, 255, 0.1); 
  border-radius: 10px; 
  padding: 6px; 
  min-width: 150px; 
  box-shadow: 0 8px 24px rgba(0,0,0,0.5); 
}

.menu-item { padding: 8px 12px; font-size: 13px; color: #d1d1d1; border-radius: 6px; cursor: pointer; }
.menu-item:hover { background: rgba(255, 255, 255, 0.08); color: #fff; }
.menu-item.delete { color: #ff6b6b; }
.menu-sep { height: 1px; background: rgba(255, 255, 255, 0.05); margin: 4px 0; }
</style>