<script setup>
import { ref, onMounted, watch } from 'vue';

const props = defineProps({
  show: Boolean
});

const emit = defineEmits(['close', 'confirm']);

const presetName = ref('');

const handleConfirm = () => {
  if (presetName.value.trim()) {
    emit('confirm', presetName.value.trim());
    presetName.value = '';
  }
};

const handleCancel = () => {
  presetName.value = '';
  emit('close');
};

const inputRef = ref(null);

onMounted(() => {
  if (props.show) {
    setTimeout(() => inputRef.value?.focus(), 50);
  }
});

watch(() => props.show, (val) => {
  if (val) {
    setTimeout(() => inputRef.value?.focus(), 50);
  }
});
</script>

<template>
  <Transition name="modal-fade">
    <div v-if="show" class="modal-overlay" @click.self="handleCancel">
      <div class="modal-content">
        <h3 class="modal-title">新建自定义配置</h3>
        <p class="modal-desc">为你的模型配置起一个好记的名字</p>
        
        <div class="input-wrapper">
          <input 
            v-model="presetName" 
            placeholder="例如: 翻译专家, 文案助手..." 
            @keyup.enter="handleConfirm"
            @keyup.esc="handleCancel"
            ref="inputRef"
          />
        </div>
        
        <div class="modal-actions">
          <button class="cancel-btn" @click="handleCancel">取消</button>
          <button class="confirm-btn" :disabled="!presetName.trim()" @click="handleConfirm">确定</button>
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
  width: 360px;
  padding: 24px;
  box-shadow: var(--shadow-main);
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: #fff;
  margin: 0 0 8px 0;
}

.modal-desc {
  font-size: 13px;
  color: #888;
  margin: 0 0 20px 0;
}

.input-wrapper {
  background: var(--bg-input-dim);
  border: 1px solid var(--border-glass);
  border-radius: 8px;
  padding: 10px 14px;
  margin-bottom: 24px;
}

.input-wrapper input {
  width: 100%;
  background: transparent;
  border: none;
  color: #fff;
  font-size: 14px;
  outline: none;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
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
  background: var(--color-success);
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
