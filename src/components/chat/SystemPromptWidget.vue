<script setup>
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { useChatStore } from '../../stores/chat';
import { useConfigStore } from '../../stores/config';
import { invoke } from '@tauri-apps/api/core';
import { NAV_PROMPTS_SVG, CHECK_SVG, EDIT_SVG, CLOSE_SVG, PLUS_SVG, SETTINGS_SVG } from '../../constants/icons';
import NamePresetModal from '../modals/NamePresetModal.vue';
import { useSettingsStore } from '../../stores/settings';

const props = defineProps({
  menuId: {
    type: String,
    default: 'prompt-widget'
  }
});

const chatStore = useChatStore();
const configStore = useConfigStore();
const settingsStore = useSettingsStore();

const isOpen = ref(false);
const isEditing = ref(false);
const localPrompt = ref('');
const showNameModal = ref(false);
const textareaRef = ref(null);

watch(() => isEditing.value, (val) => {
    if (val) {
        setTimeout(() => {
            textareaRef.value?.focus();
        }, 100);
    }
});

const currentSession = computed(() => chatStore.activeSession);

// ...

const handleNewPrompt = () => {
    showNameModal.value = true;
};

const onCreatePrompt = async (name) => {
    const newId = await configStore.addPrompt({
        name,
        icon: '📝',
        content: '',
        category: 'custom'
    });
    showNameModal.value = false;
    isOpen.value = false;
    
    // 自动跳转到设置页面
    settingsStore.openSettings('prompts');
};

// 同步本地值
watch(() => currentSession.value?.system_prompt, (v) => {
    localPrompt.value = v || '';
}, { immediate: true });

const toggleDropdown = (e) => {
    e.stopPropagation();
    isOpen.value = !isOpen.value;
    if (isOpen.value) {
        isEditing.value = false;
    }
};

const close = () => {
    isOpen.value = false;
    isEditing.value = false;
};

const applyLibrary = async (item) => {
    await savePrompt(item.content);
};

const saveCurrentEdit = async () => {
    await savePrompt(localPrompt.value);
    isEditing.value = false;
};

const savePrompt = async (newPrompt) => {
    if (!currentSession.value) return;
    try {
        const sid = currentSession.value.id;
        const finalPrompt = newPrompt?.trim() || null;
        
        // 更新本地内存
        currentSession.value.system_prompt = finalPrompt;
        
        // 同步数据库
        await invoke("update_session_config", {
            id: sid,
            presetId: currentSession.value.preset_id || configStore.settings.defaultPresetId,
            modelId: currentSession.value.model_id || configStore.settings.selectedModelId,
            systemPrompt: finalPrompt
        });
        
        if (!isEditing.value) isOpen.value = false;
    } catch (e) {
        console.error("保存系统提示词失败:", e);
    }
};

const clearPrompt = () => savePrompt(null);

const handleClickOutside = (e) => {
    if (isOpen.value && !e.target.closest('.prompt-widget-container')) {
        close();
    }
};

onMounted(() => window.addEventListener('click', handleClickOutside));
onUnmounted(() => window.removeEventListener('click', handleClickOutside));

const isUsingCustom = computed(() => !!currentSession.value?.system_prompt);

const getMenuStyle = computed(() => {
    const inputWrapper = document.querySelector('.input-wrapper');
    if (!inputWrapper) return {};
    const wrapRect = inputWrapper.getBoundingClientRect();
    return {
        position: 'fixed',
        bottom: (window.innerHeight - wrapRect.top) + 'px',
        left: wrapRect.left + 'px',
        width: wrapRect.width + 'px',
        borderRadius: '28px 28px 0 0'
    };
});

</script>

<template>
  <div class="prompt-widget-container" @click.stop>
    <button 
      class="icon-btn tool-btn" 
      :class="{ 'active': isOpen }"
      @click="toggleDropdown"
      title="会话提示词 (System Prompt)"
    >
      <span v-html="NAV_PROMPTS_SVG"></span>
    </button>

    <Teleport to="body">
      <Transition name="fade-slide">
        <div v-if="isOpen" class="prompt-menu-popup-global modern-scroll" :style="getMenuStyle" @click.stop>
          <div v-if="!isEditing" class="menu-content">
            <div class="menu-list">
                <!-- Header label inside list -->
                <div class="section-label-item">
                    <span>角色库</span>
                    <span class="count">{{ configStore.settings.promptLibrary.length }} 个角色</span>
                </div>
                
                <div class="prompt-items-container custom-scrollbar">
                    <div 
                        v-for="p in configStore.settings.promptLibrary" 
                        :key="`widget-lib-${p.id}`"
                        class="menu-item-nested"
                        :class="{ active: currentSession?.system_prompt === p.content }"
                        @click="applyLibrary(p)"
                    >
                        <div class="item-left">
                            <span class="p-icon">{{ p.icon }}</span>
                            <div class="p-info">
                                <span class="p-name">{{ p.name }}</span>
                                <span class="p-desc" v-if="p.description">{{ p.description }}</span>
                            </div>
                        </div>
                        <div class="p-check" v-if="currentSession?.system_prompt === p.content" v-html="CHECK_SVG"></div>
                    </div>
                </div>
            </div>

            <div class="menu-footer">
              <div class="footer-left">会话设置</div>
              <div class="footer-actions">
                  <button class="footer-btn" @click="isEditing = true">
                      <span v-html="EDIT_SVG"></span>
                      <span>手动编辑</span>
                  </button>
                  <button class="footer-btn" @click="handleNewPrompt">
                      <span v-html="PLUS_SVG"></span>
                      <span>新建角色</span>
                  </button>
                  <button class="footer-btn danger" @click="clearPrompt" v-if="isUsingCustom">
                      <span v-html="CLOSE_SVG"></span>
                      <span>清除自定义</span>
                  </button>
                  <div class="v-divider"></div>
                  <button class="footer-btn" @click="settingsStore.openSettings('prompts'); isOpen = false">
                      <span v-html="SETTINGS_SVG"></span>
                      <span>管理库</span>
                  </button>
              </div>
            </div>
          </div>

          <div v-else class="wide-edit-box">
               <div class="edit-header">
                  <span>手动编辑系统提示词</span>
                  <button class="close-btn-mini" @click="isEditing = false" v-html="CLOSE_SVG"></button>
               </div>
               <textarea 
                  ref="textareaRef"
                  v-model="localPrompt" 
                  placeholder="在此输入特定的系统指令，这些指令将优先于全局设置..." 
                  class="wide-textarea modern-scroll"
               ></textarea>
               <div class="edit-footer-btns">
                  <button class="btn-cancel" @click="isEditing = false">取消</button>
                  <button class="btn-save" @click="saveCurrentEdit">保存更改</button>
               </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <NamePresetModal 
      :show="showNameModal" 
      title="新建系统提示词"
      desc="配置好后将自动跳转到库中进行编辑"
      @close="showNameModal = false" 
      @confirm="onCreatePrompt"
    />
  </div>
</template>

<style scoped>
.prompt-widget-container {
  display: flex;
  align-items: center;
}

.tool-btn {
  position: relative;
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  color: var(--text-color-white);
  opacity: 0.6;
  transition: all 0.2s ease;
}

.tool-btn:hover {
  background: var(--bg-glass-hover);
  color: var(--text-color-white);
  opacity: 1;
}

.tool-btn.active {
  background: var(--bg-glass-active);
  color: var(--text-color-white);
  opacity: 1;
}

.tool-btn :deep(svg) {
  width: 18px;
  height: 18px;
  transform: translateY(3px);
}

.active-dot {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 6px;
  height: 6px;
  background: var(--color-primary);
  border-radius: 50%;
  border: 1px solid var(--bg-main);
}

/* Full-width Popup Styles (Perfect Mirror of Search Menu) */
.prompt-menu-popup-global {
  position: fixed;
  background: var(--bg-menu);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid var(--border-menu);
  border-bottom: none;
  box-shadow: 
    0 -10px 30px rgba(0, 0, 0, 0.25),
    inset 0 1px 1px rgba(255, 255, 255, 0.1);
  z-index: 100000;
  overflow: hidden;
  padding: 8px;
  display: flex;
  flex-direction: column;
  background-clip: padding-box;
}

.menu-content {
  display: flex;
  flex-direction: column;
}

.menu-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.section-label-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 14px 4px;
  font-size: 11px;
  font-weight: 700;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-secondary);
  opacity: 1;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.section-label-item .count {
  font-weight: 400;
  text-transform: none;
  opacity: 0.6;
}

.prompt-items-container {
  max-height: 320px;
  overflow-y: auto;
  padding-right: 4px;
}

/* Mirror Search Item Styles */
.menu-item-nested {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  cursor: pointer;
  transition: all 0.2s;
  border-radius: 10px;
  border: 1px solid transparent;
}

.menu-item-nested:hover {
  background: var(--bg-glass-hover);
}

.menu-item-nested.active {
  background: var(--bg-menu-active);
  border: 1px solid var(--color-primary-border);
}

.item-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  overflow: hidden;
}

.p-icon {
  font-size: 18px;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-glass);
  border-radius: 8px;
  flex-shrink: 0;
}

.p-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  overflow: hidden;
}

.p-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-color-white);
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
}

.menu-item-nested.active .p-name {
  color: var(--color-menu-active);
}

.p-desc {
  font-size: 12px;
  color: var(--text-secondary);
  opacity: 1;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
}

.p-check :deep(svg) {
  width: 16px;
  height: 16px;
  color: var(--color-menu-active);
}

/* Footer Styles (Mirroring Search Menu Footer) */
.menu-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 14px 6px;
  border-top: 1px solid var(--border-glass);
  margin-top: 6px;
}

.footer-left {
  font-size: 12px;
  color: var(--text-dim);
  opacity: 1;
  font-weight: 500;
  text-transform: none;
}

.footer-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.footer-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--text-color);
  opacity: 0.5;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.footer-btn:hover {
  background: var(--bg-glass-hover);
  color: var(--text-color-white);
  opacity: 1;
}

.footer-btn.danger:hover {
  color: var(--color-danger);
  background: var(--color-danger-alpha-10);
}

.footer-btn :deep(svg) {
  width: 14px;
  height: 14px;
  opacity: 0.7;
}

.v-divider {
  width: 1px;
  height: 14px;
  background: var(--border-glass);
  margin: 0 4px;
}

/* Edit Box */
.wide-edit-box {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.edit-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 4px;
}

.edit-header span {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-color);
    opacity: 0.5;
}

.wide-textarea {
  width: 100%;
  height: 200px;
  background: var(--bg-input);
  border: 1px solid var(--border-glass);
  border-radius: 12px;
  color: var(--text-color-white);
  padding: 14px;
  font-size: 14px;
  line-height: 1.6;
  resize: none;
  outline: none;
  font-family: inherit;
  transition: border-color 0.2s;
}

.wide-textarea:focus {
  border-color: var(--color-primary);
  background: var(--bg-input-focus);
}

.edit-footer-btns {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn-cancel, .btn-save {
  padding: 6px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.btn-cancel {
  background: var(--bg-glass-active);
  color: var(--text-tertiary);
}

.btn-cancel:hover { background: var(--bg-glass-hover); color: var(--text-color-white); }

.btn-save {
  background: var(--color-primary);
  color: white;
}

.btn-save:hover { background: var(--color-primary-light); transform: translateY(-1px); }

.close-btn-mini {
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-tertiary);
  padding: 4px;
}

.close-btn-mini:hover { color: var(--text-color-white); }
.close-btn-mini :deep(svg) { width: 14px; height: 14px; }

/* Transitions (Match Search Menu) */
.fade-slide-enter-active, .fade-slide-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}
.fade-slide-enter-from, .fade-slide-leave-to {
  opacity: 0;
  transform: translateY(15px);
}

.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: var(--bg-glass-active); border-radius: 10px; }
</style>
