// src/stores/chat.ts
import { defineStore } from 'pinia'; // 必须引入
import { ref } from 'vue';           // 必须引入
import { chatApi, type ChatSession } from '../api/chat'; // 必须引入，注意路径是 ../

export const useChatStore = defineStore('chat', () => {
    // 通过 <ChatSession[]> 给 ref 一个明确的类型，解决 unshift 报错
    const historyList = ref<ChatSession[]>([]);
    const activeId = ref<string | null>(null);

    const loadData = async () => {
        try {
            const sessions = await chatApi.getSessions();
            historyList.value = sessions;
            if (sessions.length > 0 && activeId.value === null) {
                activeId.value = sessions[0].id;
            }
        } catch (e) {
            console.error("DB加载失败", e);
        }
    };

    const createSession = async () => {
        try {
            const newId = await chatApi.createSession("新对话");
            // 因为定义了 ChatSession 类型，这里就不会再报错了
            historyList.value.unshift({
                id: newId,
                title: "新对话",
                last_scroll_pos: 0
            });
            activeId.value = newId;
        } catch (e) {
            console.error("创建失败", e);
        }
    };

    const deleteSession = async (id: string) => {
        try {
            await chatApi.deleteSession(id);
            historyList.value = historyList.value.filter(item => item.id !== id);
            if (activeId.value === id) {
                activeId.value = historyList.value[0]?.id || null;
            }
        } catch (e) {
            console.error("删除失败", e);
        }
    };

    const renameSession = async (id: string, title: string) => {
        try {
            await chatApi.renameSession(id, title);
            const item = historyList.value.find(i => i.id === id);
            if (item) item.title = title;
        } catch (e) {
            console.error("重命名失败", e);
        }
    };

    const syncScroll = (id: string, pos: number) => {
        const session = historyList.value.find(s => s.id === id);
        if (session) session.last_scroll_pos = pos;
    };

    return {
        historyList,
        activeId,
        loadData,
        createSession,
        deleteSession,
        renameSession,
        syncScroll
    };
});