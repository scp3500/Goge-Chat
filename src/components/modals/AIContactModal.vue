<script setup>
import { ref, onMounted, watch, computed } from 'vue';
import { useConfigStore } from '../../stores/config';
import { open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc } from '@tauri-apps/api/core';
import { PLUS_SVG } from '../../constants/icons';

const resolveAvatarSrc = (path) => {
  if (!path) return '';
  if (path.startsWith('data:') || path.startsWith('http')) return path;
  return convertFileSrc(path);
};
import ImageCropperModal from './ImageCropperModal.vue';
import { readFile } from '@tauri-apps/plugin-fs';


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
const avatar = ref('');
const prompt = ref('');
const model = ref('');

// Cropper State
const showCropper = ref(false);
const cropImgSrc = ref('');

// 预设头像 (使用 SVG 占位符或项目资源)
const presets = [
  'https://api.dicebear.com/7.x/adventurer/svg?seed=Felix',
  'https://api.dicebear.com/7.x/adventurer/svg?seed=Aneka',
  'https://api.dicebear.com/7.x/adventurer/svg?seed=Milo', 
  'https://api.dicebear.com/7.x/adventurer/svg?seed=Lela',
  'https://api.dicebear.com/7.x/adventurer/svg?seed=Bella',
  'https://api.dicebear.com/7.x/adventurer/svg?seed=Rocky'
];

const resetForm = () => {
    if (props.contact) {
        name.value = props.contact.name || '';
        avatar.value = props.contact.avatar || '';
        prompt.value = props.contact.prompt || '';
        model.value = props.contact.model || '';
    } else {
        name.value = '';
        avatar.value = presets[0]; // 默认选中第一个
        prompt.value = '';
        model.value = configStore.settings.selectedModelId || '';
        
        // 如果有可用模型，默认选中第一个
        if (!model.value && availableModels.value.length > 0) {
            model.value = availableModels.value[0];
        }
    }
};

onMounted(resetForm);

watch(() => props.show, (val) => {
  if (val) resetForm();
});

const handleConfirm = () => {
  if (name.value.trim()) {
    emit('confirm', {
      name: name.value.trim(),
      avatar: avatar.value.trim(),
      prompt: prompt.value.trim(),
      model: model.value
    });
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
                models.push(id);
            });
        }
    });
    return models;
});

const promptLibrary = computed(() => configStore.settings.promptLibrary || []);

const handlePromptSelect = (content) => {
    if (content) {
        prompt.value = content;
    }
};
</script>

<template>
  <Transition name="modal-fade">
    <div v-if="show" class="modal-overlay" @click.self="handleCancel">
      <div class="modal-content">
        <h3 class="modal-title">{{ contact ? '修改联系人资料' : '添加 AI 联系人' }}</h3>
        
        <div class="form-group">
          <label>头像选择</label>
          <div class="avatar-picker">
              <div 
                  v-for="(src, index) in presets" 
                  :key="index"
                  class="avatar-option"
                  :class="{ active: avatar === src }"
                  @click="avatar = src"
              >
                  <img :src="src" class="avatar-img" />
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
          <!-- 备用文本框，允许直接输入 URL -->
          <input 
              v-model="avatar" 
              placeholder="或输入图片 URL" 
              style="margin-top: 8px; font-size: 12px; padding: 6px;"
          />
        </div>

        <div class="form-group">
          <label>昵称</label>
          <input v-model="name" placeholder="为 AI 起个名字" />
        </div>

        <div class="form-group">
          <label>模型选择</label>
          <select v-model="model">
            <option v-for="m in availableModels" :key="m" :value="m">{{ m }}</option>
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

        <div class="form-group">
          <label>人设提示词 (Prompt)</label>
          <textarea v-model="prompt" rows="4" placeholder="定义这个 AI 的性格和职责..."></textarea>
        </div>
        
        <div class="modal-actions">
          <button class="cancel-btn" @click="handleCancel">取消</button>
          <button class="confirm-btn" :disabled="!name.trim()" @click="handleConfirm">确定</button>
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
  width: 440px;
  padding: 24px;
  box-shadow: var(--shadow-main);
  max-height: 90vh;
  overflow-y: auto;
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-color);
  margin: 0 0 20px 0;
}

.form-group {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 13px;
  color: var(--text-color);
  opacity: 0.7;
}

.form-group input, 
.form-group select, 
.form-group textarea {
  background: var(--bg-input-dim);
  border: 1px solid var(--border-glass);
  border-radius: 8px;
  padding: 10px 14px;
  color: var(--text-color);
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
  font-family: inherit;
}

.form-group input:focus, 
.form-group select:focus, 
.form-group textarea:focus {
  border-color: var(--theme-color);
}

/* Avatar Picker Styles */
.avatar-picker {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    margin-bottom: 4px;
}

.avatar-option {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    overflow: hidden;
    cursor: pointer;
    border: 2px solid transparent;
    transition: all 0.2s;
    background: var(--bg-input);
    display: flex;
    align-items: center;
    justify-content: center;
}

.avatar-option:hover {
    transform: scale(1.05);
    border-color: var(--bg-glass-hover);
}

.avatar-option.active {
    border-color: var(--theme-color);
    box-shadow: 0 0 0 2px var(--bg-main), 0 0 0 4px var(--theme-color);
}

.avatar-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.upload-option {
    border: 2px dashed var(--border-glass);
    color: var(--text-tertiary);
}

.upload-option:hover {
    border-color: var(--theme-color);
    color: var(--theme-color);
}

.upload-icon :deep(svg) {
    width: 20px;
    height: 20px;
}

.custom-avatar-preview {
    width: 100%;
    height: 100%;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
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
  background: var(--theme-color);
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
