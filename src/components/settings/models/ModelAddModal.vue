<script setup lang="ts">
import { ref, watch } from 'vue';
import { ModelInfo, ModelFeature } from '../../../types/config';
import { CLOSE_SVG } from '../../../constants/icons';

const props = defineProps<{
  show: boolean;
  editingModel: ModelInfo | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', model: ModelInfo): void;
}>();

const modelForm = ref<ModelInfo>({
    id: '',
    name: '',
    group: '',
    features: []
});

const firstInput = ref<HTMLInputElement | null>(null);

watch(() => props.show, (show) => {
    if (show) {
        setTimeout(() => {
            firstInput.value?.focus();
        }, 100);
    }
});

watch(() => props.editingModel, (newVal) => {
    if (newVal) {
        modelForm.value = { ...newVal, features: [...(newVal.features || [])] };
    } else {
        modelForm.value = { id: '', name: '', group: '', features: [] };
    }
}, { immediate: true });

const handleSave = () => {
    if (!modelForm.value.id || !modelForm.value.name) {
        alert('请输入模型 ID 和名称');
        return;
    }
    emit('save', { ...modelForm.value });
};

const toggleFeature = (feat: ModelFeature) => {
    const idx = modelForm.value.features?.indexOf(feat);
    if (idx === undefined) {
        modelForm.value.features = [feat];
    } else if (idx > -1) {
        modelForm.value.features?.splice(idx, 1);
    } else {
        modelForm.value.features?.push(feat);
    }
};
</script>

<template>
  <div v-if="show" class="modal-overlay-v3" @click.self="emit('close')">
    <div class="modal-content-v3">
      <div class="modal-header-v3">
        <h3 class="modal-title-v3">{{ editingModel ? '编辑模型' : '手动新建模型' }}</h3>
        <button class="close-modal-btn-v3" @click="emit('close')" v-html="CLOSE_SVG"></button>
      </div>
      <div class="modal-form-v3">
        <div class="form-item-v3">
          <label>模型 ID (如 gemini-1.5-pro)</label>
          <input ref="firstInput" v-model="modelForm.id" placeholder="必填" :disabled="!!editingModel" />
        </div>
        <div class="form-item-v3">
          <label>显示名称</label>
          <input v-model="modelForm.name" placeholder="如 Gemini 1.5 Pro" />
        </div>
        <div class="form-item-v3">
          <label>分组名称</label>
          <input v-model="modelForm.group" placeholder="如 Gemini/OpenAI" />
        </div>
        <div class="form-item-v3">
          <label>支持特性</label>
          <div class="features-check-v3">
            <label><input type="checkbox" :checked="modelForm.features?.includes('vision')" @change="toggleFeature('vision')" /> 视觉</label>
            <label><input type="checkbox" :checked="modelForm.features?.includes('web')" @change="toggleFeature('web')" /> 搜索</label>
            <label><input type="checkbox" :checked="modelForm.features?.includes('reasoning')" @change="toggleFeature('reasoning')" /> 推理</label>
            <label><input type="checkbox" :checked="modelForm.features?.includes('tools')" @change="toggleFeature('tools')" /> 工具</label>
          </div>
        </div>
      </div>
      <div class="modal-actions-v3">
        <button class="cancel-btn" @click="emit('close')">取消</button>
        <button class="confirm-btn" @click="handleSave">确定</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay-v3 {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: var(--overlay-heavy);
    backdrop-filter: blur(8px);
    z-index: 2000;
    display: flex;
    align-items: center;
    justify-content: center;
}

.modal-content-v3 {
    background: var(--bg-main);
    border: 1px solid var(--border-glass-bright);
    border-radius: 20px;
    width: 500px;
    max-height: 85vh;
    overflow-y: auto;
    padding: 0;
    box-shadow: var(--shadow-2xl);
}

/* Custom scrollbar */
.modal-content-v3::-webkit-scrollbar { width: 6px; }
.modal-content-v3::-webkit-scrollbar-track { background: transparent; }
.modal-content-v3::-webkit-scrollbar-thumb { background: var(--border-glass-bright); border-radius: 10px; }
.modal-content-v3::-webkit-scrollbar-thumb:hover { background: var(--color-primary-alpha-30); }

.modal-header-v3 {
    padding: 20px 24px;
    border-bottom: 1px solid var(--border-glass);
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.modal-title-v3 { margin: 0; color: var(--text-color-white); font-size: 18px; }

.close-modal-btn-v3 {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 4px;
    display: flex;
}
.close-modal-btn-v3:hover { color: #fff; }

.modal-form-v3 { display: flex; flex-direction: column; gap: 16px; padding: 24px; }

.form-item-v3 { display: flex; flex-direction: column; gap: 6px; }
.form-item-v3 label { font-size: 12px; color: var(--text-secondary); }
.form-item-v3 input {
    background: var(--bg-input-dim);
    border: 1px solid var(--border-glass);
    border-radius: 12px;
    padding: 12px 16px;
    color: var(--text-color-white);
    outline: none;
    font-size: 14px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.form-item-v3 input:focus { 
    border-color: var(--color-primary); 
    background: var(--bg-input-focus);
    box-shadow: 0 0 0 3px var(--color-primary-alpha-10);
}
.form-item-v3 input:disabled { opacity: 0.5; cursor: not-allowed; background: var(--bg-input-dim); }

.features-check-v3 { 
    display: grid; 
    grid-template-columns: repeat(2, 1fr); 
    gap: 12px; 
    background: var(--bg-input-dim);
    padding: 16px;
    border-radius: 12px;
    border: 1px solid var(--border-glass);
}
.features-check-v3 label { 
    display: flex; 
    align-items: center; 
    gap: 8px; 
    color: var(--text-color-white); 
    cursor: pointer; 
    font-size: 14px; 
    padding: 8px;
    border-radius: 8px;
    transition: background 0.2s;
}
.features-check-v3 label:hover {
    background: var(--bg-glass-hover);
}
.features-check-v3 input { 
    width: 16px; 
    height: 16px; 
    cursor: pointer; 
    accent-color: var(--color-primary);
}

.modal-actions-v3 { display: flex; justify-content: flex-end; gap: 12px; margin-top: 0; padding: 16px 24px 24px; }
.modal-actions-v3 button {
    padding: 8px 20px;
    border-radius: 8px;
    cursor: pointer;
    font-weight: 500;
    border: none;
    transition: all 0.2s;
}
.cancel-btn { background: var(--bg-glass); color: var(--text-color); }
.cancel-btn:hover { background: var(--bg-glass-hover); }
.confirm-btn { background: var(--color-primary); color: var(--color-white); }
.confirm-btn:hover { opacity: 0.9; }
</style>
