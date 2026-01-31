<script setup lang="ts">
import { ref, watch } from 'vue';
import { ModelInfo, ModelFeature } from '../../../types/config';

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
      <h3 class="modal-title-v3">{{ editingModel ? '编辑模型' : '新建模型' }}</h3>
      <div class="modal-form-v3">
        <div class="form-item-v3">
          <label>模型 ID (如 gemini-1.5-pro)</label>
          <input v-model="modelForm.id" placeholder="必填" :disabled="!!editingModel" />
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
    background: rgba(0,0,0,0.85);
    backdrop-filter: blur(8px);
    z-index: 2000;
    display: flex;
    align-items: center;
    justify-content: center;
}

.modal-content-v3 {
    background: #1a1a1e;
    border: 1px solid var(--border-glass-bright);
    border-radius: 16px;
    width: 400px;
    padding: 24px;
    box-shadow: 0 30px 60px rgba(0,0,0,0.8);
}

.modal-title-v3 { margin: 0 0 20px 0; color: #fff; font-size: 18px; }

.modal-form-v3 { display: flex; flex-direction: column; gap: 16px; }

.form-item-v3 { display: flex; flex-direction: column; gap: 6px; }
.form-item-v3 label { font-size: 12px; color: var(--text-secondary); }
.form-item-v3 input {
    background: var(--bg-input);
    border: 1px solid var(--border-glass);
    border-radius: 8px;
    padding: 10px;
    color: #fff;
    outline: none;
    font-size: 14px;
}
.form-item-v3 input:focus { border-color: var(--color-primary); }
.form-item-v3 input:disabled { opacity: 0.5; cursor: not-allowed; }

.features-check-v3 { display: flex; flex-wrap: wrap; gap: 12px; }
.features-check-v3 label { display: flex; align-items: center; gap: 6px; color: var(--text-secondary); cursor: pointer; font-size: 13px; }
.features-check-v3 input { width: 14px; height: 14px; cursor: pointer; }

.modal-actions-v3 { display: flex; justify-content: flex-end; gap: 12px; margin-top: 24px; }
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
.confirm-btn { background: var(--color-primary); color: #fff; }
.confirm-btn:hover { opacity: 0.9; }
</style>
