<script setup>
import { computed, ref, watch } from 'vue';
import { useChatStore } from '../../stores/chat';
import { useConfigStore } from '../../stores/config';
import { invoke } from '@tauri-apps/api/core';
import { BRAIN_SVG, EDIT_SVG, CHECK_SVG } from '../../constants/icons';

const chatStore = useChatStore();
const configStore = useConfigStore();

const isEditing = ref(false);
const localPrompt = ref('');

const currentSession = computed(() => chatStore.activeSession);

// 当会话切换时，同步本地编辑值
watch(() => currentSession.value?.id, () => {
  localPrompt.value = currentSession.value?.system_prompt || '';
  isEditing.value = false;
}, { immediate: true });

const toggleEdit = () => {
  if (isEditing.value) {
    savePrompt();
  } else {
    localPrompt.value = currentSession.value?.system_prompt || '';
    isEditing.value = true;
  }
};

const savePrompt = async () => {
  if (!currentSession.value) return;
  
  try {
    const sid = currentSession.value.id;
    const newPrompt = localPrompt.value.trim() || null;
    
    // 更新本地内存项（保证 UI 实时更新）
    currentSession.value.system_prompt = newPrompt;
    
    // 同步到数据库
    await invoke("update_session_config", {
      id: sid,
      presetId: currentSession.value.preset_id || configStore.settings.defaultPresetId,
      modelId: currentSession.value.model_id || configStore.settings.selectedModelId,
      systemPrompt: newPrompt
    });
    
    isEditing.value = false;
  } catch (e) {
    console.error("保存系统提示词失败:", e);
  }
};

const cancelEdit = () => {
  localPrompt.value = currentSession.value?.system_prompt || '';
  isEditing.value = false;
};

const displayedPrompt = computed(() => {
    return currentSession.value?.system_prompt || configStore.settings.defaultSystemPrompt;
});

const isUsingDefault = computed(() => !currentSession.value?.system_prompt);

</script>

<template>
  <div class="system-prompt-bar" :class="{ 'editing': isEditing }" v-if="currentSession">
    <div class="bar-header" @click="!isEditing && (isEditing = true)">
        <div class="header-left">
            <span class="icon" v-html="BRAIN_SVG"></span>
            <span class="label">系统提示词</span>
            <span class="tag" v-if="isUsingDefault">默认</span>
            <span class="tag custom" v-else>自定义</span>
        </div>
        <div class="header-right">
            <button v-if="!isEditing" class="action-btn" @click.stop="isEditing = true" v-html="EDIT_SVG"></button>
            <template v-else>
                <button class="action-btn cancel" @click.stop="cancelEdit">取消</button>
                <button class="action-btn save" @click.stop="savePrompt" v-html="CHECK_SVG"></button>
            </template>
        </div>
    </div>
    
    <div class="prompt-body" v-if="isEditing">
        <textarea 
            v-model="localPrompt" 
            placeholder="输入此对话特定的系统指令... 留空则使用默认值。" 
            class="prompt-textarea modern-scroll"
            auto-focus
        ></textarea>
        <div class="hint">此配置仅对当前会话生效</div>
    </div>
    <div class="prompt-preview" v-else @click="isEditing = true">
        <p class="preview-text">{{ displayedPrompt }}</p>
    </div>
  </div>
</template>

<style scoped>
.system-prompt-bar {
  margin: 12px 16px 8px;
  background: var(--bg-glass);
  border: 1px solid var(--border-glass);
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.system-prompt-bar:hover {
  background: var(--bg-glass-hover);
  border-color: var(--border-glass-bright);
}

.system-prompt-bar.editing {
  background: var(--bg-input-focus);
  border-color: var(--color-primary-alpha-30);
  box-shadow: var(--shadow-main);
}

.bar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  cursor: pointer;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-left .icon :deep(svg) {
  width: 14px;
  height: 14px;
  color: var(--color-primary);
}

.label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
}

.tag {
  font-size: 10px;
  padding: 1px 6px;
  background: var(--bg-glass);
  border-radius: 4px;
  color: var(--text-tertiary);
}

.tag.custom {
  background: var(--bg-glass-active);
  color: var(--color-primary);
}

.header-right {
  display: flex;
  gap: 8px;
}

.action-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.action-btn:hover {
  color: var(--text-color-white);
}

.action-btn :deep(svg) {
  width: 14px;
  height: 14px;
}

.action-btn.save { color: var(--color-success); }
.action-btn.cancel { font-size: 11px; padding: 0 4px; }

.prompt-body {
  padding: 0 12px 12px;
}

.prompt-textarea {
  width: 100%;
  height: 100px;
  background: var(--bg-input);
  border: 1px solid var(--border-glass-bright);
  border-radius: 8px;
  color: #eee;
  padding: 10px;
  font-size: 13px;
  line-height: 1.5;
  resize: vertical;
  outline: none;
  font-family: inherit;
}

.prompt-textarea:focus {
  border-color: var(--color-primary);
}

.hint {
  font-size: 10px;
  color: var(--text-tertiary);
  margin-top: 4px;
  text-align: right;
}

.prompt-preview {
  padding: 0 12px 10px;
  cursor: pointer;
}

.preview-text {
  font-size: 12px;
  color: var(--text-tertiary);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}
</style>
