<script setup>
import { ref, onMounted, watch, computed } from 'vue';
import { useConfigStore } from '../../stores/config';
import { open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc } from '@tauri-apps/api/core';
import { PLUS_SVG } from '../../constants/icons';

import { resolveSocialAvatar } from '../../utils/social';
import ImageCropperModal from './ImageCropperModal.vue';
import { readFile } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core'; // Ensure invoke is imported

const resolveAvatarSrc = resolveSocialAvatar; // 💡 Re-use centralized logic


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
const remark = ref('');
const avatar = ref('');
const prompt = ref('');
const model = ref('');
const provider = ref('');

// 🛠️ Database Maintenance State
const isDevelopmentMode = ref(import.meta.env.DEV);
const diagnosticReport = ref('');

// Cropper State
const showCropper = ref(false);
const cropImgSrc = ref('');

// 🖼️ Local Avatars (Use IDs for selection)
import { PRESET_AVATARS_MAP } from '../../utils/social';

const presets = [
  'avatar_1', 'avatar_2', 'avatar_3', 'avatar_4', 
  'avatar_5', 'avatar_6', 'avatar_7', 'avatar_8'
];

const resetForm = () => {
    if (props.contact) {
        name.value = props.contact.name || '';
        remark.value = props.contact.remark || '';
        avatar.value = props.contact.avatar || '';
        prompt.value = props.contact.prompt || '';
        model.value = props.contact.model || '';
        provider.value = props.contact.provider || '';

        // ✨ 自动修复：如果模型存在但供应商缺失，尝试从可用列表中查找
        if (model.value && !provider.value) {
            const found = availableModels.value.find(m => m.id === model.value);
            if (found) {
                provider.value = found.providerId;
                console.log(`[AIContactModal] Auto-resolved provider for ${model.value}: ${provider.value}`);
            }
        }
    } else {
        name.value = '';
        remark.value = '';
        avatar.value = presets[0]; // 默认选中第一个
        prompt.value = '';
        model.value = configStore.settings.selectedModelId || '';
        provider.value = configStore.settings.defaultProviderId || '';
        
        // 如果有可用模型且没有选中的，默认选中第一个
        if (!model.value && availableModels.value.length > 0) {
            model.value = availableModels.value[0].id;
            provider.value = availableModels.value[0].providerId;
        }
    }
};

onMounted(resetForm);

watch(() => props.show, (val) => {
  if (val) resetForm();
});

const handleConfirm = () => {
  if (name.value.trim()) {
    const contactData = {
      name: name.value.trim(),
      remark: remark.value.trim() || null,
      avatar: avatar.value.trim(),
      prompt: prompt.value.trim(),
      model: model.value,
      provider: provider.value
    };
    emit('confirm', contactData);
  }
};

const handleCancel = () => {
  emit('close');
};

const handleUploadAvatar = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'Images',
                extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif']
            }]
        });
        
        if (selected) {
            // Read file as binary
            const content = await readFile(selected);
            // Convert to base64
            const base64 = btoa(
                new Uint8Array(content)
                  .reduce((data, byte) => data + String.fromCharCode(byte), '')
            );
            const mimeType = selected.toLowerCase().endsWith('.png') ? 'image/png' : 
                             selected.toLowerCase().endsWith('.gif') ? 'image/gif' : 
                             'image/jpeg';
            
            cropImgSrc.value = `data:${mimeType};base64,${base64}`;
            showCropper.value = true;
        }
    } catch (e) {
        console.error('Failed to open file dialog:', e);
    }
};

const handleCropConfirm = (data) => {
    // data is base64 string
    avatar.value = data;
    showCropper.value = false;
};

const availableModels = computed(() => {
    const models = [];
    configStore.settings.providers.forEach(p => {
        if (p.enabled) {
            p.models.forEach(m => {
                const id = typeof m === 'string' ? m : m.id;
                models.push({
                    id,
                    value: `${p.id}:${id}`, // 复合 ID，处理不同平台同名模型
                    name: typeof m === 'string' ? m : (m.name || m.id),
                    providerId: p.id,
                    providerName: p.name
                });
            });
        }
    });
    return models;
});

const selectedModelValue = computed({
    get: () => provider.value && model.value ? `${provider.value}:${model.value}` : '',
    set: (val) => {
        if (!val) return;
        const [pId, ...mIdParts] = val.split(':');
        const mId = mIdParts.join(':');
        provider.value = pId;
        model.value = mId;
    }
});

const promptLibrary = computed(() => configStore.settings.promptLibrary || []);

const handlePromptSelect = (content) => {
    if (content) {
        prompt.value = content;
    }
};

// 🛠️ Maintenance Methods
async function diagnoseDatabaseIssue() {
  try {
    const report = await invoke('diagnose_database');
    // Format the structured report
    diagnosticReport.value = JSON.stringify(report, null, 2);
    console.log('诊断报告:', report);
  } catch (error) {
    console.error('诊断失败:', error);
    alert(`诊断失败: ${error}`);
  }
}

async function forceCleanup() {
  if (!confirm('确定要执行强制清理吗？\n这会压缩数据库并清理已删除的记录。')) {
    return;
  }
  
  try {
    const result = await invoke('force_cleanup_database');
    alert(result);
    // Refresh diagnostic report
    await diagnoseDatabaseIssue();
  } catch (error) {
    console.error('清理失败:', error);
    alert(`清理失败: ${error}`);
  }
}

async function rebuildDatabase() {
  if (!confirm('⚠️ 警告：这将清空所有记忆数据！\n确定要继续吗？')) {
    return;
  }
  
  const confirmCode = prompt('请输入确认码 "REBUILD" 以继续:');
  if (confirmCode !== 'REBUILD') {
    alert('操作已取消');
    return;
  }
  
  try {
    const result = await invoke('rebuild_database', { confirmationCode: confirmCode });
    alert(result);
    diagnosticReport.value = '';
    // Refresh the whole UI to clear any stale data
    location.reload();
  } catch (error) {
    console.error('重建失败:', error);
    alert(`重建失败: ${error}`);
  }
}
</script>

<template>
  <Transition name="modal-fade">
    <div v-if="show" class="modal-overlay" @click.self="handleCancel">
      <div class="modal-card">
        <!-- 🟢 Fixed Header -->
        <div class="modal-header">
           <h3 class="modal-title">{{ contact ? '修改联系人资料' : '添加 AI 联系人' }}</h3>
           <button class="close-icon-btn" @click="handleCancel">
             <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
               <path d="M18 6L6 18M6 6l12 12"></path>
             </svg>
           </button>
        </div>

        <!-- 🟡 Scrollable Body -->
        <div class="modal-body modern-scroll">
            <div class="form-section">
              <label class="section-label">头像选择</label>
              <div class="avatar-picker">
                  <div 
                      v-for="(src, index) in presets" 
                      :key="index"
                      class="avatar-option"
                      :class="{ active: avatar === src }"
                      @click="avatar = src"
                  >
                      <img :src="resolveAvatarSrc(src)" class="avatar-img" />
                  </div>
                  
                  <div 
                      class="avatar-option upload-option" 
                      :class="{ active: avatar && !presets.includes(avatar) }"
                      @click="handleUploadAvatar"
                      title="上传本地图片"
                  >
                      <div v-if="avatar && !presets.includes(avatar)" class="custom-avatar-preview">
                          <img :src="resolveAvatarSrc(avatar)" class="avatar-img" />
                      </div>
                      <div v-else class="upload-icon" v-html="PLUS_SVG"></div>
                  </div>
              </div>
              <input 
                  v-model="avatar" 
                  class="url-input"
                  placeholder="或粘贴图片 URL..." 
              />
            </div>

            <div class="form-group">
              <label>昵称</label>
              <input v-model="name" placeholder="为 AI 起个名字..." />
            </div>

            <div class="form-group">
              <label>备注名</label>
              <input v-model="remark" placeholder="设置备注名 (可选)..." />
            </div>

            <div class="form-group">
              <label>模型选择</label>
              <select v-model="selectedModelValue">
                <option value="" disabled>选择模型...</option>
                <option v-for="m in availableModels" :key="m.value" :value="m.value">
                    [{{ m.providerName }}] {{ m.name }}
                </option>
              </select>
            </div>

            <div class="form-group" v-if="promptLibrary.length > 0">
              <label>角色快捷模板</label>
              <select @change="handlePromptSelect($event.target.value)" class="preset-select">
                <option value="" disabled selected>从您的提示词库中快速填充...</option>
                <option v-for="item in promptLibrary" :key="item.id" :value="item.content">
                  {{ item.icon || '💬' }} {{ item.name }}
                </option>
              </select>
            </div>

            <div class="form-group hover-grow">
              <label>人设提示词 (Prompt)</label>
              <textarea 
                v-model="prompt" 
                rows="6" 
                placeholder="定义这个 AI 的性格和职责..."
              ></textarea>
            </div>

            <!-- 🛠️ Database Maintenance Section (Dev Mode Only) -->
            <div class="database-maintenance" v-if="isDevelopmentMode">
              <div class="maintenance-header">
                <h4>🛠️ 数据库维护工具</h4>
                <span class="debug-badge">DEBUG</span>
              </div>
              
              <div class="maintenance-actions">
                <button @click="diagnoseDatabaseIssue" class="btn-diagnose">
                  📊 诊断
                </button>
                <button @click="forceCleanup" class="btn-cleanup">
                  🧹 强制清理
                </button>
                <button @click="rebuildDatabase" class="btn-rebuild danger">
                  ⚠️ 重建数据库
                </button>
              </div>
              
              <!-- 诊断结果显示区 -->
              <div v-if="diagnosticReport" class="diagnostic-report modern-scroll">
                <pre>{{ diagnosticReport }}</pre>
              </div>
            </div>
        </div>
        
        <!-- 🔵 Fixed Footer -->
        <div class="modal-footer">
          <button class="cancel-btn" @click="handleCancel">取消</button>
          <button class="confirm-btn" :disabled="!name.trim()" @click="handleConfirm">
             {{ contact ? '保存修改' : '创建联系人' }}
          </button>
        </div>
      </div>
    </div>
  </Transition>

  <!-- Image Cropper -->
  <ImageCropperModal 
    :show="showCropper"
    :imgSrc="cropImgSrc"
    :fixedBox="false"
    @close="showCropper = false"
    @confirm="handleCropConfirm"
  />
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.modal-card {
  background: var(--bg-main);
  border: 1px solid var(--border-glass);
  border-radius: 20px;
  width: 480px;
  max-width: 100%;
  max-height: 85vh;
  box-shadow: 0 20px 40px rgba(0,0,0,0.2);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: modalSlideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

/* 🟢 Header */
.modal-header {
    padding: 20px 24px;
    border-bottom: 1px solid var(--border-glass);
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--bg-main);
    flex-shrink: 0;
}

.modal-title {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-color);
  margin: 0;
}

.close-icon-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 4px;
    border-radius: 6px;
    transition: all 0.2s;
    display: flex;
}
.close-icon-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--text-color);
}

/* 🟡 Body */
.modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 24px;
}

.form-grid {
    display: grid;
    grid-template-columns: 1.5fr 1fr;
    gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-weight: 600;
  color: var(--text-secondary);
  margin-left: 2px;
}

.form-group input, 
.form-group select, 
.form-group textarea,
.url-input {
  background: var(--bg-input-dim);
  border: 1.5px solid var(--border-glass-bright);
  border-radius: 12px;
  padding: 12px 14px;
  color: var(--text-color);
  font-size: 14px;
  outline: none;
  transition: all 0.2s ease;
  font-family: inherit;
  width: 100%;
  box-sizing: border-box;
}

.form-group input:focus, 
.form-group select:focus, 
.form-group textarea:focus,
.url-input:focus {
  background: var(--bg-input);
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-10);
}

.form-group textarea {
    resize: vertical;
    min-height: 100px;
    line-height: 1.6;
}

/* Avatar Section */
.section-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 10px;
    display: block;
    margin-left: 2px;
}

.avatar-picker {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
    margin-bottom: 12px;
}

.avatar-option {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    overflow: hidden;
    cursor: pointer;
    border: 1.5px solid var(--border-glass-bright);
    transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
    background: var(--bg-input-dim);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    box-shadow: 0 2px 5px rgba(0,0,0,0.05);
}

.avatar-option:hover {
    transform: translateY(-2px);
    border-color: var(--color-primary);
    box-shadow: 0 4px 12px rgba(0,0,0,0.1);
}

.avatar-option.active {
    border-color: var(--color-primary);
    border-width: 2px;
    box-shadow: 0 0 0 3px var(--color-primary-alpha-30), 0 4px 12px rgba(0,0,0,0.1);
    transform: scale(1.05);
}

.avatar-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.upload-option {
    border: 1.5px dashed var(--border-glass-bright);
    color: var(--text-tertiary);
    background: var(--bg-input-dim);
}

.upload-option:hover {
    border-style: solid;
    border-color: var(--color-primary);
    color: var(--color-primary);
    background: var(--color-primary-alpha-5);
    transform: translateY(-2px);
}

.upload-option.active {
    border-style: solid;
}

.url-input {
    font-size: 13px;
    padding: 8px 12px;
    opacity: 0.8;
}
.url-input:focus { opacity: 1; }

/* 🔵 Footer */
.modal-footer {
  padding: 16px 24px;
  background: var(--bg-main);
  border-top: 1px solid var(--border-glass);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  flex-shrink: 0;
}

.cancel-btn {
  background: transparent;
  border: 1px solid transparent;
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  padding: 10px 20px;
  border-radius: 10px;
  transition: all 0.2s;
}

.cancel-btn:hover { 
    background: var(--bg-glass-hover); 
    color: var(--text-color); 
}

.confirm-btn {
  background: var(--color-primary); /* ⚡️ Fix: Replaced theme-color with color-primary */
  border: none;
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  padding: 10px 24px;
  border-radius: 10px;
  box-shadow: 0 4px 12px var(--color-primary-alpha-30);
  transition: all 0.2s;
}

.confirm-btn:hover:not(:disabled) { 
    transform: translateY(-1px);
    box-shadow: 0 6px 16px var(--color-primary-alpha-50);
}
.confirm-btn:active:not(:disabled) {
    transform: translateY(0);
}

.confirm-btn:disabled { 
    opacity: 0.5; 
    cursor: not-allowed; 
    box-shadow: none;
}

@keyframes modalSlideUp {
  from { opacity: 0; transform: translateY(20px) scale(0.96); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

.modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 0.2s; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }

/* 🛠️ Maintenance Styles */
.database-maintenance {
  margin-top: 10px;
  padding: 16px;
  border: 1.5px dashed var(--color-danger-alpha-30);
  border-radius: 14px;
  background: var(--color-danger-alpha-5);
}

.maintenance-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.maintenance-header h4 {
  margin: 0;
  font-size: 14px;
  color: var(--text-color);
}

.debug-badge {
  background: var(--color-danger);
  color: white;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.5px;
}

.maintenance-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.maintenance-actions button {
  padding: 6px 12px;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.2s;
  color: white;
}

.btn-diagnose {
  background: #4dabf7;
}

.btn-cleanup {
  background: #51cf66;
}

.btn-rebuild {
  background: var(--color-danger);
}

.maintenance-actions button:hover {
  filter: brightness(1.1);
  transform: translateY(-1px);
}

.diagnostic-report {
  margin-top: 12px;
  padding: 12px;
  background: var(--bg-input-dim);
  border-radius: 10px;
  border: 1px solid var(--border-glass-bright);
  max-height: 200px;
  overflow-y: auto;
}

.diagnostic-report pre {
  margin: 0;
  white-space: pre-wrap;
  font-family: 'Cascadia Code', 'Fira Code', monospace;
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
