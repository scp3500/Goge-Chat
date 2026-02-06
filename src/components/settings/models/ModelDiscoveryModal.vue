<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { SEARCH_SVG, CLOSE_SVG } from '../../../constants/icons';

const props = defineProps<{
  show: boolean;
  providerId: string;
  discoveredModels: any[];
  isModelAdded: (id: string) => boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'add', model: any): void;
  (e: 'add-all'): void;
}>();

const searchInput = ref<HTMLInputElement | null>(null);

watch(() => props.show, (show: boolean) => {
    if (show) {
        setTimeout(() => {
            searchInput.value?.focus();
        }, 100);
    }
});

const remainingToAdd = computed(() => {
    return props.discoveredModels.filter(m => !props.isModelAdded(m.id)).length;
});

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
            <div class="header-top">
                <h3 class="modal-title-v3">发现可用模型 ({{ providerId }})</h3>
                <button class="close-modal-btn-v3" @click="emit('close')" v-html="CLOSE_SVG"></button>
            </div>
            <div class="header-actions-mini">
                <div class="search-wrapper-mini" @click="searchInput?.focus()">
                    <span class="search-icon" v-html="SEARCH_SVG"></span>
                    <input 
                        ref="searchInput" 
                        v-model="searchQuery" 
                        placeholder="在可用列表中搜索..." 
                        @click.stop
                    />
                </div>
                <button v-if="remainingToAdd > 0" class="add-all-btn-v3" @click="emit('add-all')">
                    批量添加 ({{ remainingToAdd }})
                </button>
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
    background: var(--bg-main);
    border: 1px solid var(--border-glass-bright);
    border-radius: 16px;
    padding: 24px;
    box-shadow: var(--shadow-2xl);
}

.discovery-modal { 
    width: 560px; 
    max-height: 85vh; 
    display: flex; 
    flex-direction: column;
    gap: 0;
}

.modal-header-v3 { margin-bottom: 20px; display: flex; flex-direction: column; gap: 12px; }
.header-top { display: flex; justify-content: space-between; align-items: center; }
.modal-title-v3 { margin: 0; color: var(--text-color-white); font-size: 18px; }

.close-modal-btn-v3 {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 4px;
    display: flex;
}
.close-modal-btn-v3:hover { color: #fff; }

.header-actions-mini {
    display: flex;
    align-items: center;
    gap: 12px;
}

.add-all-btn-v3 {
    background: var(--color-primary-alpha-10);
    border: 1px solid var(--color-primary-border);
    color: var(--color-primary);
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
}
.add-all-btn-v3:hover { background: var(--color-primary-alpha-20); transform: translateY(-1px); }

.search-wrapper-mini {
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--bg-input-dim);
    border: 1px solid var(--border-glass);
    border-radius: 12px;
    padding: 10px 14px;
    flex: 1;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    cursor: text;
}
.search-wrapper-mini:focus-within {
    background: var(--bg-input-focus);
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-alpha-10);
}
.search-icon { 
    display: flex;
    align-items: center;
    opacity: 0.5;
    pointer-events: none; /* Ensure icon doesn't block clicks */
}
.search-wrapper-mini :deep(svg) { width: 16px; height: 16px; color: var(--text-tertiary); }
.search-wrapper-mini input {
    background: transparent;
    border: none;
    color: var(--text-color-white);
    font-size: 14px;
    outline: none;
    flex: 1;
    width: 100%;
    cursor: text;
}

.discovery-list { 
    overflow-y: auto; 
    display: flex; 
    flex-direction: column; 
    gap: 8px; 
    padding: 10px;
    max-height: 420px;
    min-height: 200px;
    scrollbar-gutter: stable; /* Reserve space for scrollbar */
}

/* Custom scrollbar for discovery list */
.discovery-list::-webkit-scrollbar { width: 6px; }
.discovery-list::-webkit-scrollbar-track { background: transparent; }
.discovery-list::-webkit-scrollbar-thumb { background: var(--bg-glass-active); border-radius: 10px; border: 1px solid transparent; background-clip: content-box; }
.discovery-list::-webkit-scrollbar-thumb:hover { background: var(--color-primary); }

.discovery-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 18px;
    background: var(--bg-input-dim);
    border: 1px solid var(--border-glass);
    border-radius: 10px;
    transition: all 0.2s ease;
    cursor: pointer;
}
.discovery-item:hover { 
    transform: translateY(-2px); 
    border-color: var(--color-primary-alpha-30);
    background: var(--bg-input-focus);
    box-shadow: 0 4px 12px rgba(0,0,0,0.1);
}
.disc-info { display: flex; flex-direction: column; gap: 2px; }
.disc-name { font-size: 14px; color: var(--text-color-white); font-family: 'JetBrains Mono', monospace; font-weight: 500; }
.disc-sub { font-size: 11px; color: var(--text-tertiary); }

.add-mini-btn {
    background: var(--color-primary);
    color: var(--color-white);
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
