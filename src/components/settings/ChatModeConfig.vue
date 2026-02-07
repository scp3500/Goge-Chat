<script setup>
import { useConfigStore } from '../../stores/config';

const configStore = useConfigStore();

// Toggle typo correction feature
const toggleTypoCorrection = (e) => {
  if (e.target.checked) {
    configStore.settings.immersiveMode.behaviors.typoCorrection = {
      triggerRate: 0.02,
      fixDelayMs: 1500
    };
  } else {
    configStore.settings.immersiveMode.behaviors.typoCorrection = null;
  }
  configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode });
};

// Toggle proactive initiation feature
const toggleProactive = (e) => {
  if (e.target.checked) {
    configStore.settings.immersiveMode.behaviors.proactiveInitiation = {
      idleThresholdRange: [120, 600], // 2m - 10m
      successRate: 0.7,
      cooldownRange: [600, 3600],     // 10m - 60m
      idleCheckIntervalRange: [30, 90]
    };
  } else {
    configStore.settings.immersiveMode.behaviors.proactiveInitiation = null;
  }
  configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode });
};
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
           <div class="icon-wrap" style="background: var(--color-success-alpha-10); color: var(--color-success);">
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

      <!-- 沉浸式行为模拟 (Immersive Behavior Simulation) -->
      <section class="config-card" v-if="configStore.settings.chatMode && configStore.settings.chatMode.enabled">
        <div class="card-header">
           <div class="icon-wrap" style="background: var(--color-warning-alpha-10); color: var(--color-warning);">
             <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
               <circle cx="12" cy="12" r="10"></circle>
               <path d="M12 6v6l4 2"></path>
             </svg>
           </div>
           <div class="title-wrap">
             <label>沉浸式行为模拟</label>
             <span class="hint">让 AI 的回复更加人性化和真实</span>
           </div>
        </div>
        <div class="input-wrap">
           <div class="row-between">
              <span class="label-text">启用行为模拟</span>
              <label class="toggle-switch">
                <input type="checkbox" 
                       v-model="configStore.settings.immersiveMode.enabled" 
                       @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                <span class="slider"></span>
              </label>
           </div>

           <!-- 详细设置 -->
           <Transition name="expand-section">
             <div v-if="configStore.settings.immersiveMode && configStore.settings.immersiveMode.enabled" class="sub-settings">
               <div class="divider"></div>
               
               <!-- 回复延迟 -->
               <div class="setting-item">
                 <label class="setting-label">回复延迟 (毫秒)</label>
                 <div class="range-inputs">
                   <input type="number" 
                          class="number-input"
                          v-model.number="configStore.settings.immersiveMode.behaviors.replyDelay[0]"
                          placeholder="最小"
                          @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                   <span class="range-separator">-</span>
                   <input type="number" 
                          class="number-input"
                          v-model.number="configStore.settings.immersiveMode.behaviors.replyDelay[1]"
                          placeholder="最大"
                          @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                 </div>
               </div>
                
                <!-- 消息拆分 -->
                <div class="row-pair">
                  <div class="setting-item half">
                    <label class="setting-label">最大拆分段数</label>
                    <input type="number" 
                           class="number-input full-width"
                           v-model.number="configStore.settings.immersiveMode.behaviors.multiSegment"
                           min="1" max="10"
                           @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                  </div>
                  
                  <div class="setting-item half">
                     <label class="setting-label">拆分阈值 (字符范围)</label>
                     <div class="range-inputs">
                       <input type="number" class="number-input" placeholder="Min"
                              v-model.number="configStore.settings.immersiveMode.behaviors.segmentationThresholdRange[0]"
                              @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                       <span class="range-separator">-</span>
                       <input type="number" class="number-input" placeholder="Max"
                              v-model.number="configStore.settings.immersiveMode.behaviors.segmentationThresholdRange[1]"
                              @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                     </div>
                  </div>
                </div>

                <!-- 模拟输入速度 -->
                <div class="setting-item">
                  <label class="setting-label">模拟输入速度 (字符/秒)</label>
                  <div class="range-inputs">
                    <input type="number" class="number-input" placeholder="Min"
                           v-model.number="configStore.settings.immersiveMode.behaviors.typingSpeedRange[0]"
                           @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                    <span class="range-separator">-</span>
                    <input type="number" class="number-input" placeholder="Max"
                           v-model.number="configStore.settings.immersiveMode.behaviors.typingSpeedRange[1]"
                           @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                  </div>
                </div>
                
                <!-- 段间延迟系数 -->
                <div class="setting-item">
                  <label class="setting-label">段间延迟系数范围 (0.0 - 1.0)</label>
                  <div class="range-inputs">
                    <input type="number" class="number-input" placeholder="Min"
                           v-model.number="configStore.settings.immersiveMode.behaviors.segmentDelayFactor[0]"
                           step="0.05" min="0" max="1"
                           @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                    <span class="range-separator">-</span>
                    <input type="number" class="number-input" placeholder="Max"
                           v-model.number="configStore.settings.immersiveMode.behaviors.segmentDelayFactor[1]"
                           step="0.05" min="0" max="1"
                           @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                  </div>
                  <span class="hint-small">相对于主延迟的百分比</span>
                </div>
                
               <!-- 已读不回概率 -->
               <div class="setting-item">
                 <label class="setting-label">
                   已读不回概率 ({{ (configStore.settings.immersiveMode.behaviors.ignoreRate * 100).toFixed(0) }}%)
                 </label>
                 <input type="range" 
                        class="range-slider"
                        v-model.number="configStore.settings.immersiveMode.behaviors.ignoreRate"
                        min="0" max="1" step="0.01"
                        @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
               </div>
               
               <!-- 撤回修正 -->
               <div class="row-between">
                 <div class="col-info">
                   <label>模拟"手滑"撤回</label>
                   <span class="hint-small" v-if="configStore.settings.immersiveMode.behaviors.typoCorrection">
                     触发概率: {{ (configStore.settings.immersiveMode.behaviors.typoCorrection.triggerRate * 100).toFixed(1) }}%
                   </span>
                   <span class="hint-small" v-else>已禁用</span>
                 </div>
                 <label class="toggle-switch small">
                   <input type="checkbox" 
                          :checked="!!configStore.settings.immersiveMode.behaviors.typoCorrection"
                          @change="toggleTypoCorrection" />
                   <span class="slider"></span>
                 </label>
               </div>
               
               <!-- 主动开启话题 -->
               <div class="setting-group">
                 <div class="row-between">
                   <div class="col-info">
                     <label>主动开启话题</label>
                     <span class="hint-small" v-if="configStore.settings.immersiveMode.behaviors.proactiveInitiation">
                       当前配置: {{ configStore.settings.immersiveMode.behaviors.proactiveInitiation.idleThresholdMin }}分 / {{ (configStore.settings.immersiveMode.behaviors.proactiveInitiation.successRate * 100).toFixed(0) }}%
                     </span>
                     <span class="hint-small" v-else>已禁用</span>
                   </div>
                   <label class="toggle-switch small">
                     <input type="checkbox" 
                            :checked="!!configStore.settings.immersiveMode.behaviors.proactiveInitiation"
                            @change="toggleProactive" />
                     <span class="slider"></span>
                   </label>
                 </div>
                 
                 <Transition name="expand-section">
                   <div v-if="configStore.settings.immersiveMode.behaviors.proactiveInitiation" class="nested-settings">
                      <!-- 空闲阈值范围 -->
                      <div class="setting-item">
                        <label class="setting-label">
                          空闲触发阈值范围 (秒)
                        </label>
                        <div class="range-inputs">
                          <input type="number" class="number-input" placeholder="Min"
                                 v-model.number="configStore.settings.immersiveMode.behaviors.proactiveInitiation.idleThresholdRange[0]"
                                 @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                          <span class="range-separator">-</span>
                          <input type="number" class="number-input" placeholder="Max"
                                 v-model.number="configStore.settings.immersiveMode.behaviors.proactiveInitiation.idleThresholdRange[1]"
                                 @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                        </div>
                        <span class="hint-small">多长时间没说话后触发 (秒)</span>
                      </div>
                      
                      <!-- 冷却范围 -->
                      <div class="setting-item">
                        <label class="setting-label">
                          冷却时长范围 (秒)
                        </label>
                        <div class="range-inputs">
                          <input type="number" class="number-input" placeholder="Min"
                                 v-model.number="configStore.settings.immersiveMode.behaviors.proactiveInitiation.cooldownRange[0]"
                                 @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                          <span class="range-separator">-</span>
                          <input type="number" class="number-input" placeholder="Max"
                                 v-model.number="configStore.settings.immersiveMode.behaviors.proactiveInitiation.cooldownRange[1]"
                                 @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                        </div>
                        <span class="hint-small">防止短时间连续主动打扰 (秒)</span>
                      </div>

                      <!-- 检查间隔 -->
                      <div class="setting-item">
                        <label class="setting-label">后台检查间隔 (秒)</label>
                        <div class="range-inputs">
                           <input type="number" class="number-input" placeholder="Min"
                                  v-if="configStore.settings.immersiveMode.behaviors.proactiveInitiation.idleCheckIntervalRange"
                                  v-model.number="configStore.settings.immersiveMode.behaviors.proactiveInitiation.idleCheckIntervalRange[0]"
                                  @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                           <span class="range-separator">-</span>
                           <input type="number" class="number-input" placeholder="Max"
                                  v-if="configStore.settings.immersiveMode.behaviors.proactiveInitiation.idleCheckIntervalRange"
                                  v-model.number="configStore.settings.immersiveMode.behaviors.proactiveInitiation.idleCheckIntervalRange[1]"
                                  @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                        </div>
                         <span class="hint-small">后台线程多久醒来检查一次 (影响响应及时性)</span>
                      </div>

                      <div class="setting-item">
                        <label class="setting-label">
                          触发成功率 ({{ (configStore.settings.immersiveMode.behaviors.proactiveInitiation.successRate * 100).toFixed(0) }}%)
                        </label>
                        <input type="range" 
                               class="range-slider"
                               v-model.number="configStore.settings.immersiveMode.behaviors.proactiveInitiation.successRate"
                               min="0" max="1" step="0.05"
                               @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                      </div>
                     
                     <div class="dynamic-badge" v-if="configStore.settings.immersiveMode.behaviors.character_state_config?.enabled">
                       <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v4m0 12v4M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83M2 12h4m12 0h4M4.93 19.07l2.83-2.83m8.48-8.48l2.83-2.83"></path></svg>
                       <span>受角色心情动态调节: 兴趣高更积极, 忙碌时更安静</span>
                     </div>
                   </div>
                 </Transition>
               </div>
                
                <!-- 打字状态抖动 -->
                <div class="row-between">
                  <div class="col-info">
                    <label>打字状态抖动</label>
                    <span class="hint-small">模拟断断续续的输入</span>
                  </div>
                  <label class="toggle-switch small">
                    <input type="checkbox" 
                           v-model="configStore.settings.immersiveMode.behaviors.typingJitter"
                           @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                    <span class="slider"></span>
                  </label>
                </div>
                
                <div class="divider"></div>
                
                <!-- 角色状态追踪 -->
                <div v-if="configStore.settings.immersiveMode.behaviors.characterStateConfig" class="setting-group">
                  <div class="row-between">
                    <div class="col-info">
                      <label>🧠 角色状态追踪</label>
                      <span class="hint-small">通过AI分析角色心情和状态</span>
                    </div>
                    <label class="toggle-switch small">
                      <input type="checkbox" 
                             v-model="configStore.settings.immersiveMode.behaviors.characterStateConfig.enabled"
                             @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                      <span class="slider"></span>
                    </label>
                  </div>
                  
                  <Transition name="expand-section">
                    <div v-if="configStore.settings.immersiveMode.behaviors.characterStateConfig.enabled" class="nested-settings">
                      <div class="setting-item">
                        <label class="setting-label">状态分析频率 (每N条消息)</label>
                        <input type="number" 
                               class="number-input full-width"
                               v-model.number="configStore.settings.immersiveMode.behaviors.characterStateConfig.analysisFrequency"
                               min="1" max="100"
                               @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                        <span class="hint-small">⚠️ 减少频次可降低API成本</span>
                      </div>
                      
                      <div class="setting-item">
                        <label class="setting-label">状态缓存时长 (分钟)</label>
                        <input type="number" 
                               class="number-input full-width"
                               v-model.number="configStore.settings.immersiveMode.behaviors.characterStateConfig.cacheDurationMin"
                               min="1" max="120"
                               @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                        <span class="hint-small">避免短时间内重复分析</span>
                      </div>
                      
                      <div class="row-between">
                        <div class="col-info">
                          <label>主动发言时分析状态</label>
                          <span class="hint-small">空闲触发时是否分析</span>
                        </div>
                        <label class="toggle-switch small">
                          <input type="checkbox" 
                                 v-model="configStore.settings.immersiveMode.behaviors.characterStateConfig.analysisOnProactive"
                                 @change="configStore.updateConfig({ immersiveMode: configStore.settings.immersiveMode })" />
                          <span class="slider"></span>
                        </label>
                      </div>
                    </div>
                  </Transition>
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

/* Immersive Mode Settings Styles */
.setting-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.setting-label {
  font-size: 13px;
  color: var(--text-color-white);
  font-weight: 500;
}

.range-inputs {
  display: flex;
  align-items: center;
  gap: 8px;
}

.range-separator {
  color: var(--text-tertiary);
  font-size: 14px;
}

.number-input {
  background: var(--bg-input-dim);
  border: 1px solid var(--border-glass);
  border-radius: 8px;
  padding: 8px 12px;
  color: var(--text-color-white);
  font-size: 13px;
  outline: none;
  width: 100px;
  transition: all 0.2s;
}

.number-input.full-width {
  width: 100%;
}

.number-input:focus {
  border-color: var(--color-primary);
  background: var(--bg-input-focus);
}

.row-pair {
  display: flex;
  gap: 12px;
  width: 100%;
}

.half {
  flex: 1;
}

.range-slider {
  width: 100%;
  height: 6px;
  border-radius: 3px;
  background: var(--bg-input-dim);
  outline: none;
  appearance: none;
  -webkit-appearance: none;
}

.range-slider::-webkit-slider-thumb {
  appearance: none;
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-primary);
  cursor: pointer;
  transition: all 0.2s;
}

.range-slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
  box-shadow: 0 0 10px var(--color-primary);
}

.range-slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-primary);
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.range-slider::-moz-range-thumb:hover {
  transform: scale(1.2);
  box-shadow: 0 0 10px var(--color-primary);
}

/* New Settings Styles */
.setting-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.nested-settings {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-left: 16px;
  margin-top: 8px;
  border-left: 2px solid var(--border-glass);
}

.hint-small {
  font-size: 11px;
  color: var(--text-tertiary);
  display: block;
  margin-top: 4px;
}

.dynamic-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--color-primary-alpha-10);
  border: 1px solid var(--color-primary-alpha-20);
  padding: 8px 12px;
  border-radius: 8px;
  margin-top: 12px;
  color: var(--color-primary);
  font-size: 11px;
}

.dynamic-badge svg {
  width: 14px;
  height: 14px;
}
</style>
