<script setup>
import { ref, onMounted, watch } from 'vue';
import { useConfigStore } from '../../stores/config';

const props = defineProps({
  show: Boolean,
  contact: {
    type: Object,
    default: null
  }
});

const emit = defineEmits(['close', 'confirm']);

const configStore = useConfigStore();
const name = ref('');
const avatar = ref('');
const prompt = ref('');
const model = ref('');

const resetForm = () => {
    if (props.contact) {
        name.value = props.contact.name || '';
        avatar.value = props.contact.avatar || '';
        prompt.value = props.contact.prompt || '';
        model.value = props.contact.model || '';
    } else {
        name.value = '';
        avatar.value = '';
        prompt.value = '';
        model.value = configStore.settings.selectedModelId || '';
    }
};

onMounted(resetForm);

watch(() => props.show, (val) => {
  if (val) resetForm();
});

const handleConfirm = () => {
  if (name.value.trim()) {
    emit('confirm', {
      name: name.value.trim(),
      avatar: avatar.value.trim(),
      prompt: prompt.value.trim(),
      model: model.value
    });
  }
};

const handleCancel = () => {
  emit('close');
};

const availableModels = computed(() => {
    const models = [];
    configStore.settings.providers.forEach(p => {
        if (p.enabled) {
            p.models.forEach(m => {
                const id = typeof m === 'string' ? m : m.id;
                models.push(id);
            });
        }
    });
    return models;
});

import { computed } from 'vue';
</script>

<template>
  <Transition name="modal-fade">
    <div v-if="show" class="modal-overlay" @click.self="handleCancel">
      <div class="modal-content">
        <h3 class="modal-title">{{ contact ? '修改联系人资料' : '添加 AI 联系人' }}</h3>
        
        <div class="form-group">
          <label>昵称</label>
          <input v-model="name" placeholder="为 AI 起个名字" />
        </div>

        <div class="form-group">
          <label>头像 (URL/路径)</label>
          <input v-model="avatar" placeholder="图片链接或本地路径" />
        </div>

        <div class="form-group">
          <label>模型选择</label>
          <select v-model="model">
            <option v-for="m in availableModels" :key="m" :value="m">{{ m }}</option>
          </select>
        </div>

        <div class="form-group">
          <label>人设提示词 (Prompt)</label>
          <textarea v-model="prompt" rows="4" placeholder="定义这个 AI 的性格和职责..."></textarea>
        </div>
        
        <div class="modal-actions">
          <button class="cancel-btn" @click="handleCancel">取消</button>
          <button class="confirm-btn" :disabled="!name.trim()" @click="handleConfirm">确定</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: var(--bg-mask);
  backdrop-filter: blur(8px);
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-content {
  background: var(--bg-main);
  border: 1px solid var(--border-glass);
  border-radius: 16px;
  width: 420px;
  padding: 24px;
  box-shadow: var(--shadow-main);
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-color);
  margin: 0 0 20px 0;
}

.form-group {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 13px;
  color: var(--text-color);
  opacity: 0.7;
}

.form-group input, 
.form-group select, 
.form-group textarea {
  background: var(--bg-input-dim);
  border: 1px solid var(--border-glass);
  border-radius: 8px;
  padding: 10px 14px;
  color: var(--text-color);
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.form-group input:focus, 
.form-group select:focus, 
.form-group textarea:focus {
  border-color: var(--theme-color);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}

.cancel-btn {
  background: transparent;
  border: none;
  color: #888;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  padding: 8px 16px;
  border-radius: 6px;
}

.cancel-btn:hover { background: var(--bg-glass-hover); color: var(--text-color-white); }

.confirm-btn {
  background: var(--theme-color);
  border: none;
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  padding: 8px 24px;
  border-radius: 6px;
}

.confirm-btn:hover:not(:disabled) { opacity: 0.9; }
.confirm-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.modal-fade-enter-active, .modal-fade-leave-active { transition: all 0.2s ease; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; transform: scale(0.95); }
</style>
