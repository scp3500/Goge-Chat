<script setup>
defineProps({
  show: Boolean,
  x: Number,
  y: Number,
  actionLabel: String
});

defineEmits(['confirm', 'cancel']);
</script>

<template>
  <Teleport to="body">
    <Transition name="popup-scale">
      <div v-if="show" 
           class="modern-confirm-popup"
           :style="{ top: (y - 90) + 'px', left: (x - 90) + 'px' }">
        <div class="popup-title">{{ actionLabel }}？</div>
        <div class="popup-btns">
          <button class="pop-cancel" @click="$emit('cancel')">取消</button>
          <button class="pop-confirm" @click="$emit('confirm')">确定</button>
        </div>
        <div class="popup-arrow"></div>
      </div>
    </Transition>
    <!-- 点击外部关闭 -->
    <div v-if="show" class="popup-overlay" @click="$emit('cancel')"></div>
  </Teleport>
</template>

<style scoped>
.modern-confirm-popup {
  position: fixed;
  z-index: 10000;
  background: var(--bg-dropdown);
  backdrop-filter: blur(24px) saturate(180%);
  -webkit-backdrop-filter: blur(24px) saturate(180%);
  border: 1px solid var(--border-dropdown);
  border-radius: 12px;
  padding: 12px;
  width: 180px;
  box-shadow: var(--shadow-main);
  color: var(--text-color-white);
  pointer-events: auto;
}

.popup-title {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 12px;
  text-align: center;
  color: var(--text-color-white);
  opacity: 0.9;
  letter-spacing: 0.2px;
}

.popup-btns {
  display: flex;
  gap: 8px;
}

.popup-btns button {
  flex: 1;
  padding: 6px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border: none;
}

.pop-cancel {
  background: var(--bg-glass);
  color: var(--text-color);
  opacity: 0.7;
}

.pop-cancel:hover {
  background: var(--bg-glass-hover);
  color: var(--text-color-white);
  opacity: 1;
}

.pop-confirm {
  background: var(--color-primary);
  color: #fff;
}

.pop-confirm:hover {
  background: var(--color-primary-light);
  box-shadow: 0 0 10px var(--color-primary-border);
}

.popup-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 9999;
}

/* 箭头 */
.popup-arrow {
  position: absolute;
  bottom: -5px;
  left: 50%;
  transform: translateX(-50%) rotate(45deg);
  width: 10px;
  height: 10px;
  background: var(--bg-dropdown);
  backdrop-filter: blur(24px);
  border-right: 1px solid var(--border-dropdown);
  border-bottom: 1px solid var(--border-dropdown);
}

/* 动画 */
.popup-scale-enter-active, .popup-scale-leave-active {
  transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.popup-scale-enter-from, .popup-scale-leave-to {
  opacity: 0;
  transform: scale(0.9) translateY(8px);
}
</style>
