<script setup>
import { ref, reactive } from 'vue';
import { VueCropper } from 'vue-cropper';
import 'vue-cropper/dist/index.css';

const props = defineProps({
  show: { type: Boolean, default: false },
  imgSrc: { type: String, default: '' },
  fixedBox: { type: Boolean, default: false }, // Fixed selection box size?
  borderRadius: { type: Number, default: 0 } // Custom preview radius
});

const emit = defineEmits(['close', 'confirm']);

const cropperRef = ref(null);
const option = reactive({
  img: '',
  outputSize: 1, // Full quality
  outputType: 'png',
  canMove: true,
  canMoveBox: true,
  original: false,
  autoCrop: true,
  autoCropWidth: 400,
  autoCropHeight: 400,
  centerBox: true,
  high: true,
  fixed: true, // Fixed aspect ratio
  fixedNumber: [1, 1], // 1:1 for avatar
  full: true,
  canScale: true
});

// Watch for src changes
import { watch } from 'vue';
watch(() => props.imgSrc, (val) => {
  option.img = val;
});

const handleConfirm = () => {
  if (!cropperRef.value) return;
  cropperRef.value.getCropData((data) => {
    emit('confirm', data);
  });
};

const handleZoom = (val) => {
    if(!cropperRef.value) return;
    cropperRef.value.changeScale(val);
}

import { computed } from 'vue';
const cropStyle = computed(() => {
  // Scale radius based on autoCropWidth (400) vs default avatar size (36)
  // If user has 36px avatar with 6px radius, in 400px box it should appear proportionally
  const ratio = 400 / 36; 
  return {
    borderRadius: `${props.borderRadius * ratio}px`
  };
});
</script>

<template>
  <div v-if="show" class="cropper-modal-overlay" @click.self="$emit('close')">
    <div class="cropper-modal">
      <div class="modal-header">
        <h3>裁剪头像</h3>
        <button class="close-btn" @click="$emit('close')">×</button>
      </div>
      
      <div class="cropper-content" :style="{ '--crop-radius': cropStyle.borderRadius }">
        <VueCropper
          ref="cropperRef"
          :img="option.img"
          :outputSize="option.outputSize"
          :outputType="option.outputType"
          :info="true"
          :full="option.full"
          :canMove="option.canMove"
          :canMoveBox="option.canMoveBox"
          :fixedBox="fixedBox"
          :original="option.original"
          :autoCrop="option.autoCrop"
          :autoCropWidth="option.autoCropWidth"
          :autoCropHeight="option.autoCropHeight"
          :centerBox="option.centerBox"
          :high="option.high"
          :fixed="option.fixed"
          :fixedNumber="option.fixedNumber"
        ></VueCropper>
      </div>

      <div class="modal-footer">
          <div class="zoom-controls">
              <button @click="handleZoom(1)" title="放大">+</button>
              <button @click="handleZoom(-1)" title="缩小">-</button>
          </div>
        <button class="btn-cancel" @click="$emit('close')">取消</button>
        <button class="btn-confirm" @click="handleConfirm">确定</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cropper-modal-overlay {
  position: fixed;
  top: 0; 
  left: 0;
  width: 100%; 
  height: 100%;
  background: rgba(0, 0, 0, 0.6);
  z-index: 20000;
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(5px);
}

.cropper-modal {
  width: 500px;
  background: var(--bg-card);
  border-radius: 12px;
  border: 1px solid var(--border-glass);
  box-shadow: 0 8px 32px rgba(0,0,0,0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-glass);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h3 { margin: 0; font-size: 1.1rem; color: var(--text-color); }
.close-btn { background: none; border: none; font-size: 1.5rem; color: var(--text-tertiary); cursor: pointer; }

.cropper-content {
  height: 400px;
  width: 100%;
  position: relative;
  background: #000;
}

.modal-footer {
  padding: 16px 20px;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  border-top: 1px solid var(--border-glass);
  background: var(--bg-card);
  align-items: center;
}

.zoom-controls {
    margin-right: auto;
    display: flex;
    gap: 8px;
}
.zoom-controls button {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: var(--bg-glass);
    border: 1px solid var(--border-glass);
    color: var(--text-color);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
}

.btn-cancel, .btn-confirm {
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 0.9rem;
  cursor: pointer;
}

.btn-cancel {
  background: transparent;
  border: 1px solid var(--border-glass);
  color: var(--text-color);
}

.btn-confirm {
  background: var(--theme-color, var(--color-primary, #4caf50));
  border: none;
  color: white;
  min-width: 60px;
}

/* Dynamic border radiuc sync */
:deep(.cropper-view-box) {
  outline: 2px solid var(--theme-color) !important;
  border-radius: var(--crop-radius, 0px);
}
:deep(.cropper-face) {
  background-color: transparent !important;
}
</style>
