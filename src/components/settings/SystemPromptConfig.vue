<script setup>
import { computed, ref, onMounted } from 'vue';
import { useConfigStore } from '../../stores/config';
import { BRAIN_SVG, EDIT_SVG, TRASH_SVG, DRAG_SVG, PLUS_SVG } from '../../constants/icons';
import draggable from 'vuedraggable';

const props = defineProps(['presetId']);
const configStore = useConfigStore();

const preset = computed(() => 
  configStore.settings.presets.find(p => p.id === props.presetId)
);

// 状态
const editingItem = ref(null); // null means adding
const showEditModal = ref(false);

const promptLibrary = computed({
    get: () => configStore.settings.promptLibrary,
    set: (val) => configStore.handlePromptsReorder(val.map(p => ({ id: p.id })))
});

// 编辑表单内容
const form = ref({
    name: '',
    icon: '💬',
    content: '',
    description: ''
});

// 库管理方法
const startAdd = () => {
    editingItem.value = null;
    form.value = { name: '', icon: '💬', content: '', description: '' };
    showEditModal.value = true;
};

const startEdit = (item) => {
    editingItem.value = item;
    form.value = { ...item };
    showEditModal.value = true;
};

const saveItem = async () => {
    if (!form.value.name || !form.value.content) return;
    
    if (editingItem.value) {
        await configStore.updatePrompt(editingItem.value.id, form.value);
    } else {
        await configStore.addPrompt(form.value);
    }
    showEditModal.value = false;
};

const deleteItem = async (id) => {
    if (confirm('确定要删除此提示词吗？')) {
        await configStore.removePrompt(id);
    }
};

const applyLibraryItem = (item) => {
  if (preset.value) {
    configStore.updatePreset(props.presetId, { systemPrompt: item.content });
  }
};
</script>

<template>
  <div class="prompt-library-config" v-if="preset">
    <div class="library-header">
        <div class="header-info">
            <h2>提示词库管理</h2>
            <p class="subtitle">管理并排列你的 AI 角色库。点击卡片可快速应用到当前预设。</p>
        </div>
        <button class="add-btn-premium" @click="startAdd">
            <span class="icon" v-html="PLUS_SVG"></span>
            <span>新增提示词</span>
        </button>
    </div>

    <!-- 拖拽列表 -->
    <draggable 
        v-model="promptLibrary" 
        item-key="id" 
        handle=".drag-handle"
        class="library-grid modern-scroll"
        ghost-class="ghost-card"
        animation="300"
    >
        <template #item="{ element: item }">
            <div class="library-card-premium" @click="applyLibraryItem(item)">
                <div class="card-glow"></div>
                <div class="drag-handle" @click.stop v-html="DRAG_SVG"></div>
                
                <div class="card-icon-wrapper">
                    <span class="emoji-icon">{{ item.icon || '💬' }}</span>
                </div>
                
                <div class="card-title">{{ item.name }}</div>

                <div class="card-actions">
                    <button class="mini-action-btn edit" @click.stop="startEdit(item)" title="编辑" v-html="EDIT_SVG"></button>
                    <button class="mini-action-btn delete" @click.stop="deleteItem(item.id)" title="删除" v-html="TRASH_SVG"></button>
                </div>
            </div>
        </template>
    </draggable>

    <!-- Empty State -->
    <div v-if="promptLibrary.length === 0" class="empty-library">
        <div class="empty-icon" v-html="BRAIN_SVG"></div>
        <div class="empty-text">提示词库还是空的</div>
        <button class="add-btn-premium" @click="startAdd">立即创建第一个</button>
    </div>

    <!-- 编辑/新增 Modal -->
    <Teleport to="body">
        <Transition name="modal-fade">
            <div v-if="showEditModal" key="library-edit-modal-beautified" class="modal-overlay-premium" @click="showEditModal = false">
                <div class="edit-modal-premium" @click.stop>
                    <div class="modal-header">
                        <h3>{{ editingItem ? '重塑 AI 角色' : '缔造新角色' }}</h3>
                        <div class="modal-subtitle">{{ editingItem ? '对现有人设进行精细化调整' : '定义一个新的 AI 行为范式' }}</div>
                    </div>

                    <div class="form-body">
                        <div class="form-row split">
                            <div class="form-group icon">
                                <label>图标</label>
                                <input v-model="form.icon" placeholder="Emoji" class="premium-input-mini" />
                            </div>
                            <div class="form-group name">
                                <label>名称</label>
                                <input v-model="form.name" placeholder="例如：架构大师" class="premium-input" />
                            </div>
                        </div>
                        <div class="form-group">
                            <label>核心描述</label>
                            <input v-model="form.description" placeholder="一句话描述这个角色的专长..." class="premium-input" />
                        </div>
                        <div class="form-group content">
                            <label>System Prompt 指令</label>
                            <textarea v-model="form.content" rows="8" placeholder="在这里输入详细的系统指令..." class="premium-textarea modern-scroll"></textarea>
                        </div>
                    </div>

                    <div class="modal-footer">
                        <button class="cancel-btn-premium" @click="showEditModal = false">取消</button>
                        <button class="confirm-btn-premium" @click="saveItem">
                            {{ editingItem ? '保存更改' : '加入库中' }}
                        </button>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.prompt-library-config {
  display: flex;
  flex-direction: column;
  gap: 24px;
  height: 100%;
}

.library-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-glass);
}

.header-info h2 { margin: 0; font-size: 18px; color: var(--text-color-white); font-weight: 700; letter-spacing: -0.5px; }
.subtitle { margin: 4px 0 0; font-size: 13px; color: var(--text-tertiary); }

.add-btn-premium {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  background: var(--color-primary);
  border: none;
  border-radius: 12px;
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 12px var(--color-primary-alpha-30);
}

.add-btn-premium:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px var(--color-primary-alpha-50);
}

.add-btn-premium .icon :deep(svg) { width: 14px; height: 14px; }

/* 列表网格 */
.library-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-height: 60vh;
  overflow-y: auto;
  padding: 4px;
}

.library-card-premium {
  position: relative;
  background: var(--bg-glass);
  border: 1px solid var(--border-glass);
  border-radius: 16px;
  padding: 12px 20px;
  cursor: pointer;
  transition: all 0.3s ease;
  overflow: hidden;
  display: flex;
  align-items: center;
  gap: 16px;
}

.library-card-premium:hover {
  background: var(--bg-glass-hover);
  border-color: var(--color-primary-border);
  transform: translateX(4px);
}

.card-glow {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, var(--color-primary-alpha-5) 0%, transparent 100%);
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.3s ease;
}
.library-card-premium:hover .card-glow { opacity: 1; }

.drag-handle {
  color: var(--text-tertiary);
  cursor: grab;
  z-index: 5;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.drag-handle:hover { color: var(--text-color-white); opacity: 0.6; }
.drag-handle :deep(svg) { width: 16px; height: 16px; }

.card-icon-wrapper {
  width: 36px;
  height: 36px;
  background: var(--color-primary-bg);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  flex-shrink: 0;
}

.card-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-color-white);
  flex: 1;
}

.card-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}
.mini-action-btn {
  background: var(--bg-glass);
  border: 1px solid var(--border-glass);
  border-radius: 8px;
  color: var(--text-tertiary);
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
}
.mini-action-btn:hover { background: var(--bg-glass-hover); color: var(--text-color-white); }
.mini-action-btn.edit:hover { color: var(--color-primary); border-color: var(--color-primary-border); }
.mini-action-btn.delete:hover { color: var(--color-danger); border-color: var(--color-danger-alpha-30); }
.mini-action-btn :deep(svg) { width: 14px; height: 14px; }

.ghost-card { opacity: 0.3; transform: scale(0.95); border: 2px dashed var(--color-primary); }

/* Empty State */
.empty-library {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 60px 0;
  opacity: 0.5;
}
.empty-icon :deep(svg) { width: 48px; height: 48px; color: var(--color-primary); }
.empty-text { font-size: 14px; color: var(--text-secondary); }

/* Modal Premium Styles */
.modal-overlay-premium {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--bg-mask);
  backdrop-filter: blur(12px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.edit-modal-premium {
  background: var(--bg-main);
  width: 500px;
  max-width: 90vw;
  border-radius: 24px;
  border: 1px solid var(--border-glass-bright);
  padding: 32px;
  box-shadow: var(--shadow-main);
}

.modal-header { margin-bottom: 24px; }
.modal-header h3 { margin: 0; font-size: 20px; color: var(--text-color-white); font-weight: 700; }
.modal-subtitle { font-size: 13px; color: var(--text-tertiary); margin-top: 4px; }

.form-body { display: flex; flex-direction: column; gap: 20px; }
.form-row.split { display: grid; grid-template-columns: 80px 1fr; gap: 16px; }
.form-group { display: flex; flex-direction: column; gap: 8px; }
.form-group label { font-size: 12px; color: var(--text-tertiary); font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; }

.premium-input, .premium-input-mini, .premium-textarea {
  background: var(--bg-input);
  border: 1px solid var(--border-glass);
  border-radius: 12px;
  padding: 12px 16px;
  color: var(--text-color);
  font-size: 14px;
  outline: none;
  transition: all 0.2s;
}

.premium-input:focus, .premium-input-mini:focus, .premium-textarea:focus {
  background: var(--bg-input-focus);
  border-color: var(--color-primary);
}

.premium-textarea { resize: vertical; line-height: 1.6; }

.modal-footer { display: flex; justify-content: flex-end; gap: 16px; margin-top: 32px; }
.cancel-btn-premium { background: transparent; border: none; color: var(--text-tertiary); font-weight: 600; cursor: pointer; padding: 10px 20px; }
.confirm-btn-premium {
  background: var(--text-color-white);
  border: none;
  border-radius: 12px;
  color: var(--bg-main);
  padding: 10px 24px;
  font-weight: 700;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s;
}
.confirm-btn-premium:hover { background: var(--color-primary); color: #fff; transform: scale(1.02); }

/* Transitions */
.modal-fade-enter-active, .modal-fade-leave-active { transition: all 0.3s ease; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; transform: scale(0.95); }

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
