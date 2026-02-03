<script setup lang="ts">
import { ref, computed } from 'vue';
import { SEARCH_SVG } from '../../../constants/icons';

const props = defineProps<{
  show: boolean;
  providerId: string;
  discoveredModels: any[];
  isModelAdded: (id: string) => boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'add', model: any): void;
}>();

const searchQuery = ref('');

const filteredModels = computed(() => {
    let list = [...props.discoveredModels];
    
    if (searchQuery.value) {
        const q = searchQuery.value.toLowerCase();
        list = list.filter(m => m.id.toLowerCase().includes(q) || m.name.toLowerCase().includes(q));
    }
    
    // Sort: New to Old (reverse chronological)
    return list.sort((a, b) => b.id.localeCompare(a.id, undefined, { numeric: true }));
});
</script>

<template>
  <div v-if="show" class="modal-overlay-v3" @click.self="emit('close')">
    <div class="modal-content-v3 discovery-modal">
        <div class="modal-header-v3">
            <h3 class="modal-title-v3">发现可用模型 ({{ providerId }})</h3>
            <div class="search-wrapper-mini">
                <span v-html="SEARCH_SVG"></span>
                <input v-model="searchQuery" placeholder="搜索可用列表..." autofocus />
            </div>
        </div>
        <div class="discovery-list modern-scroll"> <!-- Apply modern-scroll here -->
            <div v-for="m in filteredModels" :key="m.id" class="discovery-item">
                <div class="disc-info">
                    <span class="disc-name">{{ m.id }}</span>
                    <span class="disc-sub">{{ m.name }}</span>
                </div>
                <button class="add-mini-btn" @click="emit('add', m)" :disabled="isModelAdded(m.id)">
                    {{ isModelAdded(m.id) ? '已添加' : '添加' }}
                </button>
            </div>
            <div v-if="filteredModels.length === 0" class="no-disc">
                {{ discoveredModels.length === 0 ? '未发现新模型或需要有效 API Key。' : '未找到匹配的模型。' }}
            </div>
        </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay-v3 {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: var(--overlay-heavy);
    backdrop-filter: blur(8px);
    z-index: 2000;
    display: flex;
    align-items: center;
    justify-content: center;
}

.modal-content-v3 {
    background: #1a1a1e;
    border: 1px solid var(--border-glass-bright);
    border-radius: 16px;
    padding: 24px;
    box-shadow: 0 30px 60px rgba(0,0,0,0.8);
}

.discovery-modal { 
    width: 500px; 
    max-height: 80vh; 
    display: flex; 
    flex-direction: column; 
}

.modal-header-v3 { margin-bottom: 20px; }
.modal-title-v3 { margin: 0; color: #fff; font-size: 18px; }

.search-wrapper-mini {
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--bg-input-dim, rgba(0,0,0,0.2));
    border: 1px solid var(--border-glass);
    border-radius: 10px;
    padding: 8px 12px;
    margin-top: 12px;
}
.search-wrapper-mini :deep(svg) { width: 14px; height: 14px; color: var(--text-tertiary); }
.search-wrapper-mini input {
    background: transparent;
    border: none;
    color: #fff;
    font-size: 13px;
    outline: none;
    flex: 1;
}

.discovery-list { 
    overflow-y: auto; 
    display: flex; 
    flex-direction: column; 
    gap: 10px; 
    padding-right: 4px;
}

.discovery-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: linear-gradient(to right, var(--glass-white-03), transparent);
    border: 1px solid var(--glass-white-05);
    border-radius: 12px;
    transition: transform 0.2s, border-color 0.2s;
}
.discovery-item:hover { transform: translateX(4px); border-color: rgba(255,255,255,0.1); }
.disc-info { display: flex; flex-direction: column; gap: 2px; }
.disc-name { font-size: 14px; color: #fff; font-family: 'JetBrains Mono', monospace; font-weight: 500; }
.disc-sub { font-size: 11px; color: var(--text-tertiary); }

.add-mini-btn {
    background: var(--color-primary);
    color: #000;
    border: none;
    padding: 6px 16px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
}
.add-mini-btn:hover:not(:disabled) { transform: scale(1.05); opacity: 0.9; }
.add-mini-btn:disabled { background: var(--bg-glass); color: var(--text-tertiary); cursor: default; }

.no-disc { text-align: center; color: var(--text-tertiary); padding: 40px; font-size: 13px; }
</style>
