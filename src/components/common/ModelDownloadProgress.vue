<script setup>
import { computed } from 'vue';

const props = defineProps({
  visible: { type: Boolean, required: true },
  progress: { type: Number, default: 0 },
  statusText: { type: String, default: '' },
  detailText: { type: String, default: '' }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="download-overlay">
        <div class="download-modal">
          
          <!-- 背景装饰 -->
          <div class="modal-decoration"></div>
          
          <h3 class="modal-title">首次运行配置</h3>
          <p class="modal-status">{{ statusText }}</p>
          
          <!-- 进度条容器 -->
          <div class="progress-track">
             <!-- 动态流光进度条 -->
            <div class="progress-fill" :style="{ width: progress + '%' }">
                 <div class="shimmer-effect"></div>
            </div>
          </div>

          <!-- 底部信息 -->
          <div class="modal-footer">
            <span class="detail-text">{{ detailText }}</span>
            <span class="progress-text">{{ progress }}%</span>
          </div>

        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* 核心容器 */
.download-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
}

.download-modal {
  position: relative;
  background-color: rgba(17, 24, 39, 0.95); /* bg-gray-900/95 */
  border: 1px solid #374151; /* border-gray-700 */
  padding: 2rem;
  border-radius: 1rem; /* rounded-2xl */
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  max-width: 28rem; /* max-w-md */
  width: 100%;
  text-align: center;
  overflow: hidden;
  color: white;
  font-family: sans-serif;
}

/* 顶部装饰条 */
.modal-decoration {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 4px;
  background: linear-gradient(to right, #3b82f6, #a855f7, #ec4899);
}

.modal-title {
  font-size: 1.25rem;
  font-weight: 700;
  margin-bottom: 0.5rem;
  letter-spacing: 0.025em;
  color: white;
}

.modal-status {
  color: #9ca3af; /* text-gray-400 */
  font-size: 0.875rem;
  margin-bottom: 2rem;
}

/* 进度条 */
.progress-track {
  position: relative;
  width: 100%;
  background-color: #1f2937; /* bg-gray-800 */
  border-radius: 9999px;
  height: 1rem;
  margin-bottom: 0.75rem;
  overflow: hidden;
  box-shadow: inset 0 2px 4px 0 rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.progress-fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background: linear-gradient(to right, #3b82f6, #6366f1, #9333ea);
  transition: width 0.3s ease-out;
  border-radius: 9999px;
}

/* 流光动画 */
.shimmer-effect {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.2) 50%,
    rgba(255, 255, 255, 0) 100%
  );
  animation: shimmer 1.5s infinite linear;
}

@keyframes shimmer {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}

/* 底部文字 */
.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.75rem;
  font-family: monospace;
  color: #6b7280;
  margin-top: 0.5rem;
}

.detail-text {
  color: #9ca3af;
}

.progress-text {
  color: #60a5fa; /* text-blue-400 */
  font-weight: 700;
  font-size: 0.875rem;
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
