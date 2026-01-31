<script setup lang="ts">
// defineProps and defineEmits are compiler macros and don't need to be imported in <script setup>

const props = defineProps<{
  error: {
    message: string;
    type?: string;
    details?: string;
  };
}>();

const emit = defineEmits(['close', 'retry']);

const openDetails = () => {
    // 这里可以打开一个模态框或者链接显示详情
    // 暂时简单 alert 或者 consol.log，或者跳转到相关文档
    if (props.error.details) {
        console.log("Error details:", props.error.details);
        // 如果是链接，尝试打开
        if (props.error.details.startsWith('http')) {
            window.open(props.error.details, '_blank');
        }
    }
};
</script>

<template>
  <div class="error-bubble">
    <div class="error-content">
      <div class="error-text">{{ error.message }}</div>
      <div v-if="error.details" class="error-details-link" @click="openDetails">
        详情
      </div>
    </div>
    <button class="close-btn" @click="$emit('close')">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>
  </div>
</template>

<style scoped>
.error-bubble {
  background-color: rgba(60, 20, 20, 0.9); /* Dark Red background like screenshot */
  color: #ffb4b4;
  border: 1px solid rgba(255, 100, 100, 0.2);
  border-radius: 8px;
  padding: 12px 16px;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  max-width: 600px;
  margin-top: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  font-size: 14px;
  line-height: 1.5;
}

.error-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.error-text {
  word-wrap: break-word;
}

.error-details-link {
  color: #fab1a0;
  text-decoration: underline;
  cursor: pointer;
  font-size: 12px;
  opacity: 0.8;
  align-self: flex-start;
}

.error-details-link:hover {
  opacity: 1;
}

.close-btn {
  background: transparent;
  border: none;
  color: #ffb4b4;
  cursor: pointer;
  padding: 4px;
  opacity: 0.6;
  transition: opacity 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  opacity: 1;
  background-color: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
}
</style>
