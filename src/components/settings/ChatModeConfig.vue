<script setup>
import { useConfigStore } from '../../stores/config';

const configStore = useConfigStore();
</script>

<template>
  <div class="chat-mode-config">
    <div v-if="!configStore.settings.chatMode" class="loading-state">
      正在加载配置...
    </div>
    <template v-else>
      <!-- 聊天模式 (Chat Mode) -->
      <section class="config-card">
        <div class="card-header">
           <div class="icon-wrap" style="background: rgba(16, 185, 129, 0.1); color: #10b981;">
             <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
           </div>
           <div class="title-wrap">
             <label>沉浸式聊天模式</label>
             <span class="hint">开启单人沉浸式体验，支持自动日夜主题切换</span>
           </div>
        </div>
        <div class="input-wrap">
           <div class="row-between">
              <span class="label-text">启用聊天模式</span>
              <label class="toggle-switch">
                <input type="checkbox" 
                       v-model="configStore.settings.chatMode.enabled" 
                       @change="configStore.updateConfig({ chatMode: configStore.settings.chatMode })" />
                <span class="slider"></span>
              </label>
           </div>

           <!-- Chat Mode Sub-Settings -->
           <Transition name="expand-section">
             <div v-if="configStore.settings.chatMode.enabled" class="sub-settings">
               <div class="divider"></div>
               
               <!-- Themes -->
               <div class="setting-row">
                 <div class="col-info">
                   <label>日间主题 (Light)</label>
                   <!-- Hardcoded for now as requested -->
                   <div class="read-only-val">WeChat (微信风格)</div>
                 </div>
                 <div class="col-info">
                   <label>夜间主题 (Dark)</label>
                   <!-- Hardcoded for now as requested -->
                   <div class="read-only-val">Dark++ (深色增强)</div>
                 </div>
               </div>

               <!-- Stream Toggle (Override) -->
               <div class="row-between">
                  <div class="col-info">
                    <label>流式传输</label>
                    <span class="hint-small">打字机效果 (默认关闭)</span>
                  </div>
                  <label class="toggle-switch small">
                    <input type="checkbox" 
                           v-model="configStore.settings.chatMode.enableStream" 
                           @change="configStore.updateConfig({ chatMode: configStore.settings.chatMode })" />
                    <span class="slider"></span>
                  </label>
               </div>

               <!-- Loading Bar Toggle -->
               <div class="row-between">
                  <div class="col-info">
                    <label>加载进度条</label>
                    <span class="hint-small">显示 "Thinking..." (默认隐藏)</span>
                  </div>
                  <label class="toggle-switch small">
                    <input type="checkbox" 
                           v-model="configStore.settings.chatMode.enableLoadingBar" 
                           @change="configStore.updateConfig({ chatMode: configStore.settings.chatMode })" />
                    <span class="slider"></span>
                  </label>
               </div>
             </div>
           </Transition>
        </div>
      </section>
    </template>
  </div>
</template>

<style scoped>
.chat-mode-config {
  display: flex;
  flex-direction: column;
  gap: 28px;
  animation: fadeIn 0.4s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.config-card {
  background: var(--bg-card);
  border: 1px solid var(--border-card);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  transition: all 0.3s ease;
}

.config-card:hover {
  background: var(--bg-glass-hover);
  border-color: var(--border-glass-bright);
}

.card-header {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.icon-wrap {
  width: 32px;
  height: 32px;
  background: var(--color-primary-bg);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary);
  flex-shrink: 0;
}

.icon-wrap svg {
  width: 18px;
  height: 18px;
}

.title-wrap {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}

.title-wrap label {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-color-white);
}

.hint {
  font-size: 12px;
  color: var(--text-tertiary);
  line-height: 1.4;
}

.input-wrap {
  width: 100%;
}

/* Chat Mode Sub-settings */
.row-between {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.label-text {
  font-size: 14px;
  color: var(--text-color-white);
  font-weight: 500;
}

.sub-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 12px;
  padding-top: 4px;
}

.divider {
  height: 1px;
  background: var(--border-glass);
  width: 100%;
}

.setting-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.col-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.col-info label {
  font-size: 13px;
  color: var(--text-color-white);
  font-weight: 500;
}

.hint-small {
  font-size: 11px;
  color: var(--text-tertiary);
}

.read-only-val {
  font-size: 12px;
  color: var(--text-tertiary);
  background: var(--bg-input-dim);
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-glass);
}

/* Toggle Switch Styles */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--bg-input-dim);
  transition: .4s;
  border-radius: 24px;
  border: 1px solid var(--border-glass);
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 2px;
  bottom: 2px;
  background-color: var(--text-tertiary);
  transition: .4s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: var(--color-primary-bg);
  border-color: var(--color-primary);
}

input:checked + .slider:before {
  transform: translateX(20px);
  background-color: var(--color-primary);
  box-shadow: 0 0 10px var(--color-primary);
}

/* Small Toggle Switch */
.toggle-switch.small {
  width: 36px;
  height: 20px;
}
.toggle-switch.small .slider:before {
  height: 14px;
  width: 14px;
  left: 2px;
  bottom: 2px;
}
.toggle-switch.small input:checked + .slider:before {
  transform: translateX(16px);
}

.expand-section-enter-active, .expand-section-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
  max-height: 200px;
  opacity: 1;
}
.expand-section-enter-from, .expand-section-leave-to {
  max-height: 0;
  opacity: 0;
  transform: translateY(-10px);
}
</style>
