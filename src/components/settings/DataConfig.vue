<script setup>
import { ref, onMounted, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../../stores/settings';
import { useChatStore } from '../../stores/chat';
import { useConfigStore } from '../../stores/config';

const settingsStore = useSettingsStore();
const chatStore = useChatStore();
const configStore = useConfigStore();
const socialContacts = ref([]);

const memories = ref([]);
const isLoading = ref(false);
const filterMode = ref('all'); // all, Social, Standard
const filterRole = ref('all'); 

// --- Edit State ---
const editingId = ref(null);
const editForm = ref({ content: '', mode: 'Standard', role_id: 'global' });

// --- Add State ---
const showAddForm = ref(false);
const addForm = ref({ content: '', mode: 'Standard', role_id: 'global' });

const loadMemories = async () => {
    isLoading.value = true;
    try {
        const all = await invoke('get_memories', { query: null });
        console.log(`📥 [DataConfig] 已加载 ${all.length} 条记忆数据`, all);
        memories.value = all;
    } catch (e) {
        console.error("Failed to load memories:", e);
    } finally {
        isLoading.value = false;
    }
};

const loadSocialContacts = async () => {
    try {
        socialContacts.value = await invoke('get_social_contacts');
    } catch (e) {
        console.error("Failed to load contacts:", e);
    }
};

const handleAdd = async () => {
    console.log("按下了提交按钮", addForm.value);
    if (!addForm.value.content.trim()) return;
    isLoading.value = true;
    try {
        await invoke('insert_memory', { 
            content: addForm.value.content,
            mode: addForm.value.mode,
            role_id: addForm.value.role_id,
            is_instruction: false // 显式传递防止后端反序列化等待
        });
        console.log("新增成功");
        showAddForm.value = false;
        addForm.value.content = '';
        await loadMemories();
    } catch (e) {
        console.error("添加失败:", e);
        alert("添加失败: " + e);
    } finally {
        isLoading.value = false;
    }
};

// The provided snippet seems to be a new function or a block of code intended to be added elsewhere,
// as it introduces variables (activeSyncSessions, sessionId, contactSnapshot) not defined in this scope,
// and replaces the core logic of handleAdd with a different operation (trigger_fact_sync vs insert_memory).
// To faithfully apply the change as given, I will insert it as a new, separate function.
// If this was intended to modify handleAdd, the provided snippet structure was misleading.
const activeSyncSessions = new Set(); // Assuming this is defined globally or in a store
const triggerMemorySync = async (sessionId, contactSnapshot) => {
    if (activeSyncSessions.has(sessionId)) {
        console.warn(`⏳ [Memory] 同步已在进行中，跳过重复触发。Session: ${sessionId}`);
        return;
    }
    activeSyncSessions.add(sessionId);
    try {
        const strRoleId = String(contactSnapshot.id);
        console.warn(`🚀 [Memory] 正在执行记忆同步... | 角色: ${contactSnapshot.name} (ID: ${strRoleId}) | Session: ${sessionId}`);
        
        await invoke("trigger_fact_sync", {
            session_id: parseInt(sessionId, 10),
            role_id: strRoleId,
            mode: "Social"
        });
        
        console.log(`✅ [Memory] 同步成功: ${contactSnapshot.name}`);
    } catch (e) {
        console.error(`❌ [Memory] 同步异常:`, e);
    } finally {
        activeSyncSessions.delete(sessionId);
    }
};

const handleEdit = (item) => {
    editingId.value = item.id;
    editForm.value = { ...item };
};

const cancelEdit = () => {
    editingId.value = null;
};

const saveEdit = async () => {
    try {
        await invoke('update_memory', { 
            id: editingId.value,
            content: editForm.value.content,
            mode: editForm.value.mode,
            role_id: editForm.value.role_id
        });
        editingId.value = null;
        await loadMemories();
    } catch (e) {
        alert("保存失败: " + e);
    }
};

const deleteMemory = async (item) => {
    if (!confirm("确定要删除这条记忆吗？AI 将不再记得这件事。")) return;
    try {
        // 传递 content 以便后端执行“全量清理”，防止重复的幽灵记录复活
        await invoke('delete_memory', { id: item.id, content: item.content });
        memories.value = memories.value.filter(m => m.id !== item.id);
    } catch (e) {
        console.error("Delete failed:", e);
    }
};

const clearAll = async () => {
    if (!confirm("⚠️ 警告：这将彻底清空所有 AI 记忆！此操作不可恢复。确定吗？")) return;
    try {
        await invoke('clear_memories');
        memories.value = [];
    } catch (e) {
        console.error("Clear failed:", e);
    }
};

const optimizeDatabase = async () => {
    isLoading.value = true;
    try {
        await invoke('optimize_memories');
        alert("✨ 优化完成！已合并磁盘分片并回收多余文件。");
        await loadMemories();
    } catch (e) {
        alert("优化失败: " + e);
    } finally {
        isLoading.value = false;
    }
};

const filteredMemories = computed(() => {
    return memories.value.filter(m => {
        // 1. Mode Filter
        const modeMatch = filterMode.value === 'all' || m.mode === filterMode.value;
        
        // 2. Role Filter (High compatibility)
        let roleMatch = filterRole.value === 'all';
        if (!roleMatch) {
            const fRole = String(filterRole.value);
            const mRole = String(m.role_id);
            
            // Match by ID, exactly as stored (String normalized)
            if (mRole === fRole) {
                roleMatch = true;
            } else {
                // Secondary Match: If we are filtering by ID, also show entries stored by name for that contact
                const contact = socialContacts.value.find(c => String(c.id) === fRole);
                if (contact && mRole === String(contact.name)) roleMatch = true;
                
                // Tertiary Match: If we are filtering by name, also show entries stored by ID for that contact
                const contactByName = socialContacts.value.find(c => String(c.name) === fRole);
                if (contactByName && mRole === String(contactByName.id)) roleMatch = true;
            }
        }
        return modeMatch && roleMatch;
    });
});

// 严格受控的下拉选项：只显示正式联系人和全局选项，杜绝“未知角色”或重复项
const roleOptions = computed(() => {
    const options = [
        { id: 'all', label: '所有归属' }, 
        { id: 'global', label: '全局共同记忆' }
    ];
    
    // 仅基于当前加载的正式社交联系人生成选项
    socialContacts.value.forEach(c => {
        const fullLabel = c.remark ? `${c.name} [${c.remark}]` : c.name;
        // 保证 ID 匹配唯一（强制转字符串比较）
        if (!options.some(o => String(o.id) === String(c.id))) {
            options.push({ id: c.id, label: fullLabel });
        }
    });

    return options;
});

const getRoleLabel = (roleId) => {
    if (roleId === 'global') return '全局共同记忆';
    // 关键修正：将 ID 转为字符串后进行比较，处理 SQLite(Number) 与 LanceDB(String) 的类型差异
    const contact = socialContacts.value.find(c => String(c.id) === String(roleId));
    if (contact) return contact.remark ? `${contact.name} [${contact.remark}]` : contact.name;
    
    // 兼容性搜索：如果 roleId 存的是名字而非 ID
    const contactByName = socialContacts.value.find(c => c.name === roleId);
    if (contactByName) return contactByName.remark ? `${contactByName.name} [${contactByName.remark}]` : contactByName.name;
    
    return roleId; // 最后的保底
};

onMounted(async () => {
    console.log("🚀 [DataConfig] Mounting...");
    await Promise.all([loadMemories(), loadSocialContacts()]);

    const explicitRole = settingsStore.dataFilterRoleId;
    console.log("📦 [DataConfig] Initial explicit role:", explicitRole);
    
    if (explicitRole) {
        console.log("🎯 [DataConfig] Activating explicit filter for ID:", explicitRole);
        filterMode.value = 'Social';
        filterRole.value = explicitRole;
        addForm.value.mode = 'Social';
        addForm.value.role_id = explicitRole;
        settingsStore.dataFilterRoleId = null;
    } 
    else if (configStore.settings.chatMode?.enabled && chatStore.activeSocialContactId) {
        const currentRoleId = chatStore.activeSocialContactId;
        console.log("📍 [DataConfig] Auto-locking current chat ID:", currentRoleId);
        filterMode.value = 'Social';
        filterRole.value = currentRoleId;
        addForm.value.mode = 'Social';
        addForm.value.role_id = currentRoleId;
    }
    else {
        console.log("🌐 [DataConfig] Defaulting to all memories");
        filterMode.value = 'all';
        filterRole.value = 'all';
    }
});

// 监听模式切换，自动调整默认角色
watch(() => addForm.value.mode, (newMode) => {
    if (newMode === 'Standard') {
        addForm.value.role_id = 'global';
    } else if (newMode === 'Social') {
        // 如果是从 Standard 切换到 Social，且没有锁定角色，默认选第一个
        if (addForm.value.role_id === 'global' && socialContacts.value.length > 0) {
            addForm.value.role_id = socialContacts.value[0].id;
        }
    }
});

const getModeLabel = (mode) => {
    if (mode === 'Social') return '社交模式';
    if (mode === 'Standard') return '普通模式';
    return mode;
};
</script>

<template>
    <div class="data-config">
        <!-- Header Controls -->
        <div class="memory-header">
            <div class="header-main">
                <div class="title-area">
                    <h3>🧠 Antigravity 记忆库</h3>
                    <p class="subtitle">管理 AI 提取的长期事实和个人偏好</p>
                </div>
                <div class="header-actions">
                    <button class="btn-secondary" @click="optimizeDatabase" :disabled="isLoading" title="合并磁盘冗余文件">
                        优化清理
                    </button>
                    <button class="btn-secondary" @click="showAddForm = !showAddForm">
                        {{ showAddForm ? '取消添加' : '手动新增' }}
                    </button>
                    <button class="btn-danger-outline" @click="clearAll">清空全部</button>
                </div>
            </div>

            <!-- Add Form Panel -->
            <Transition name="fade-slide">
                <div v-if="showAddForm" class="add-panel">
                    <textarea v-model="addForm.content" placeholder="输入你想让 AI 记住的事实..."></textarea>
                    <div class="form-row">
                        <div class="input-group">
                            <label>模式</label>
                            <select v-model="addForm.mode">
                                <option value="Standard">普通模式 (通用)</option>
                                <option value="Social">社交模式 (角色)</option>
                            </select>
                        </div>
                        <div class="input-group">
                            <label>归属角色</label>
                            <!-- 标准模式下固定为 global -->
                            <input v-if="addForm.mode === 'Standard'" value="全局 (global)" disabled />
                            
                            <!-- 社交模式下使用下拉框 -->
                            <select v-else v-model="addForm.role_id">
                                <option value="global">全局共同记忆 (global)</option>
                                <option v-for="c in socialContacts" :key="c.id" :value="c.id">
                                    {{ c.remark ? `${c.name} [${c.remark}]` : c.name }}
                                </option>
                            </select>
                        </div>
                        <button class="btn-primary" @click="handleAdd" :disabled="!addForm.content.trim()">提交记忆</button>
                    </div>
                </div>
            </Transition>

            <!-- Filters Bar -->
            <div class="filters-bar">
                <div class="filter-group">
                    <span class="filter-label">过滤显示:</span>
                    <div class="segmented-control">
                        <button :class="{ active: filterMode === 'all' }" @click="filterMode = 'all'">全部</button>
                        <button :class="{ active: filterMode === 'Standard' }" @click="filterMode = 'Standard'">普通</button>
                        <button :class="{ active: filterMode === 'Social' }" @click="filterMode = 'Social'">社交</button>
                    </div>
                </div>
                
                <div class="role-filter">
                    <select v-model="filterRole">
                        <option v-for="opt in roleOptions" :key="opt.id" :value="opt.id">
                            {{ opt.label }}
                        </option>
                    </select>
                </div>

                <button class="icon-btn-refresh" @click="loadMemories" :disabled="isLoading" title="刷新数据">
                    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" :class="{ 'spinning': isLoading }">
                        <path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
                    </svg>
                </button>
            </div>
        </div>

        <!-- Memory List Content -->
        <div class="memory-container">
            <div v-if="isLoading && memories.length === 0" class="loading-state">
                <div class="loader"></div>
                <span>能量场同步中...</span>
            </div>

            <div v-else-if="filteredMemories.length === 0" class="empty-placeholder">
                <div class="empty-icon">📂</div>
                <p>暂时没有相关记忆，快去聊天吧！</p>
            </div>

            <div v-else class="memory-grid">
                <div v-for="item in filteredMemories" :key="item.id" class="memory-card" :class="{ 'is-editing': editingId === item.id }">
                    <!-- NORMAL MODE -->
                    <template v-if="editingId !== item.id">
                        <div class="card-header">
                            <div class="badges">
                                <span class="badge mode" :class="item.mode.toLowerCase()">{{ getModeLabel(item.mode) }}</span>
                                <span class="badge role">{{ getRoleLabel(item.role_id) }}</span>
                            </div>
                            <div class="card-actions">
                                <button class="action-btn" @click="handleEdit(item)" title="编辑">
                                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>
                                </button>
                                <button class="action-btn delete" @click="deleteMemory(item)" title="删除">
                                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                                </button>
                            </div>
                        </div>
                        <div class="card-body">
                            {{ item.content }}
                        </div>
                    </template>

                    <!-- EDIT MODE -->
                    <template v-else>
                        <div class="edit-fields">
                            <textarea v-model="editForm.content" class="edit-textarea"></textarea>
                            <div class="edit-meta">
                                <select v-model="editForm.mode">
                                    <option value="Standard">普通</option>
                                    <option value="Social">社交</option>
                                </select>
                                <input v-model="editForm.role_id" placeholder="角色ID" />
                            </div>
                            <div class="edit-btns">
                                <button class="btn-text" @click="cancelEdit">取消</button>
                                <button class="btn-primary-small" @click="saveEdit">保存修改</button>
                            </div>
                        </div>
                    </template>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.data-config {
    display: flex;
    flex-direction: column;
    gap: 20px;
    animation: fadeIn 0.4s ease-out;
}

/* Header & Panels */
.memory-header {
    background: var(--bg-sidebar);
    border: 1px solid var(--border-glass);
    border-radius: 16px;
    padding: 20px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.1);
    flex-shrink: 0; /* 防止被下方列表挤压 */
    position: relative;
    z-index: 10;
}

.header-main {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 20px;
}

.title-area h3 { margin: 0; font-size: 18px; color: var(--text-color-white); }
.subtitle { margin: 4px 0 0; font-size: 12px; color: var(--text-tertiary); }

.header-actions { display: flex; gap: 10px; }

.add-panel {
    background: var(--bg-chat-island);
    border: 1px solid var(--color-primary);
    border-radius: 12px;
    padding: 16px;
    margin-bottom: 20px;
}

.add-panel textarea {
    width: 100%;
    height: 80px;
    background: transparent;
    border: 1px solid var(--border-glass);
    border-radius: 8px;
    color: var(--text-color);
    padding: 10px;
    margin-bottom: 12px;
    resize: none;
    outline: none;
}

.form-row { display: flex; gap: 12px; align-items: flex-end; }
.input-group { display: flex; flex-direction: column; gap: 4px; flex: 1; }
.input-group label { font-size: 11px; color: var(--text-tertiary); }
.input-group select, .input-group input {
    background: var(--bg-sidebar);
    border: 1px solid var(--border-glass);
    color: var(--text-color);
    padding: 6px 10px;
    border-radius: 6px;
}

/* Filters bar */
.filters-bar {
    display: flex;
    align-items: center;
    gap: 20px;
    padding-top: 15px;
    border-top: 1px solid var(--border-glass);
}

.segmented-control {
    display: flex;
    background: var(--bg-chat-island);
    padding: 3px;
    border-radius: 8px;
    gap: 2px;
}

.segmented-control button {
    background: transparent;
    border: none;
    padding: 4px 12px;
    font-size: 12px;
    border-radius: 6px;
    color: var(--text-tertiary);
    cursor: pointer;
}

.segmented-control button.active {
    background: var(--color-primary);
    color: white;
}

.role-filter select {
    background: var(--bg-chat-island);
    border: 1px solid var(--border-glass);
    color: var(--text-color);
    padding: 5px 12px;
    border-radius: 8px;
}

/* Grid & Cards */
.memory-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
}

.memory-card {
    background: var(--bg-sidebar);
    border: 1px solid var(--border-glass);
    border-radius: 14px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    border-left: 4px solid transparent;
}

.memory-card:hover {
    transform: translateY(-2px);
    border-color: var(--color-primary);
    box-shadow: 0 6px 24px rgba(0,0,0,0.2);
}

.memory-card.is-editing {
    border-color: var(--color-primary);
    background: var(--bg-chat-island);
}

.card-header { display: flex; justify-content: space-between; align-items: center; }
.badges { display: flex; gap: 6px; }

.badge { font-size: 10px; padding: 2px 8px; border-radius: 20px; font-weight: 500; }
.badge.mode.social { background: rgba(139, 92, 246, 0.2); color: #a78bfa; }
.badge.mode.standard { background: rgba(59, 130, 246, 0.2); color: #60a5fa; }
.badge.role { background: rgba(255, 255, 255, 0.05); color: var(--text-tertiary); border: 1px solid var(--border-glass); }

.card-actions { display: flex; gap: 4px; opacity: 0; transition: opacity 0.2s; }
.memory-card:hover .card-actions { opacity: 1; }

.action-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    padding: 6px;
    border-radius: 8px;
    cursor: pointer;
}
.action-btn:hover { background: var(--bg-glass-hover); color: var(--text-color-white); }
.action-btn.delete:hover { color: #f87171; background: rgba(239, 68, 68, 0.1); }

.card-body {
    font-size: 14px;
    line-height: 1.6;
    color: var(--text-color);
    word-break: break-all;
}

/* Edit Form inside card */
.edit-textarea {
    width: 100%;
    min-height: 60px;
    background: var(--bg-sidebar);
    border: 1px solid var(--color-primary);
    border-radius: 8px;
    color: var(--text-color);
    padding: 8px;
    margin-bottom: 8px;
    outline: none;
}

.edit-meta { display: flex; gap: 8px; margin-bottom: 12px; }
.edit-meta select, .edit-meta input {
    background: var(--bg-sidebar);
    border: 1px solid var(--border-glass);
    color: var(--text-color);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    flex: 1;
}

.edit-btns { display: flex; justify-content: flex-end; gap: 10px; }

/* Buttons */
.btn-primary { background: var(--color-primary); color: white; border: none; padding: 8px 20px; border-radius: 8px; font-weight: 600; cursor: pointer; }
.btn-primary-small { background: var(--color-primary); color: white; border: none; padding: 5px 12px; border-radius: 6px; font-size: 12px; font-weight: 600; cursor: pointer; }
.btn-secondary { background: var(--bg-glass-hover); color: var(--text-color-white); border: 1px solid var(--border-glass); padding: 8px 16px; border-radius: 8px; cursor: pointer; }
.btn-danger-outline { background: transparent; border: 1px solid #f87171; color: #f87171; padding: 8px 16px; border-radius: 8px; cursor: pointer; }
.btn-text { background: transparent; border: none; color: var(--text-tertiary); font-size: 12px; cursor: pointer; }

.icon-btn-refresh {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 5px;
}
.spinning { animation: spin 1s linear infinite; }

@keyframes spin { 100% { transform: rotate(360deg); } }
@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }

.fade-slide-enter-active, .fade-slide-leave-active { transition: all 0.3s ease; }
.fade-slide-enter-from, .fade-slide-leave-to { opacity: 0; transform: translateY(-10px); }

/* States */
.loading-state, .empty-placeholder {
    padding: 60px;
    text-align: center;
    color: var(--text-tertiary);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 15px;
}
.loader { width: 30px; height: 30px; border: 3px solid var(--border-glass); border-top-color: var(--color-primary); border-radius: 50%; animation: spin 1s infinite; }
.empty-icon { font-size: 40px; margin-bottom: 10px; opacity: 0.5; }
</style>
