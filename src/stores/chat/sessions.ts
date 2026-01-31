import { type Ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import { chatApi, type ChatSession } from '../../api/chat';
import { useConfigStore } from '../config';

export function useSessionActions(
    historyList: Ref<ChatSession[]>,
    activeId: Ref<string | null>,
    currentMessages: Ref<any[]>,
    switchSession: (sessionId: string) => Promise<void>
) {

    const createSession = async () => {
        try {
            // 获取配置 Store 以读取默认设置
            const configStore = useConfigStore();
            const {
                defaultSystemPrompt,
                globalModelId,
                globalPresetId
            } = configStore.settings;

            // 调用后端创建会话，并传入全局配置
            const newId = await chatApi.createSession(
                "新对话",
                globalPresetId,
                globalModelId,
                defaultSystemPrompt
            );

            historyList.value.unshift({
                id: newId,
                title: "新对话",
                last_scroll_pos: 0,
                sort_order: 0,
                preset_id: globalPresetId,
                model_id: globalModelId,
                system_prompt: defaultSystemPrompt
            });

            // 切换到新会话
            activeId.value = newId;

            // 初始化本地消息列表的系统提示词
            currentMessages.value = [{
                role: "system",
                content: defaultSystemPrompt || "你是一个简洁专业的 AI 助手。"
            }];
        } catch (e) {
            console.error("创建失败", e);
        }
    };

    const deleteSession = async (sessionId: string) => {
        try {
            // 1. 调用 Rust 后端删除数据库记录
            await invoke("delete_session", { sessionId });

            // 2. 更新本地 UI 列表
            historyList.value = historyList.value.filter(s => s.id !== sessionId);

            // 3. 自动切换逻辑
            if (activeId.value === sessionId) {
                if (historyList.value.length > 0) {
                    await switchSession(historyList.value[0].id);
                } else {
                    activeId.value = null;
                    currentMessages.value = [];
                }
            }
        } catch (e) {
            console.error("删除会话失败:", e);
        }
    };

    const renameSession = async (id: string, newTitle: string) => {
        try {
            // 1. 同步内存状态
            const session = historyList.value.find(s => s.id === id);
            if (session) {
                session.title = newTitle;
            }
            // 2. 同步数据库
            await invoke("rename_session", { id, title: newTitle });

            // 3. 强制触发响应式更新
            historyList.value = [...historyList.value];
        } catch (e) {
            console.error("重命名失败:", e);
        }
    };

    const updateSessionScroll = async (id: string, pos: number) => {
        const session = historyList.value.find(s => s.id === id);
        if (session) {
            session.last_scroll_pos = pos;
            try {
                await invoke('update_session_scroll', { id, pos });
            } catch (err) {
                console.error("滚动位置持久化失败:", err);
            }
        }
    };

    const reorderSessions = async (newList: ChatSession[]) => {
        // 使用 map 确保我们只取必要的字段，并且维持传入的物理顺序
        historyList.value = [...newList];
        const orders: [string, number][] = historyList.value.map((s, index) => [s.id, index]);
        try {
            await chatApi.updateSessionsOrder(orders);
        } catch (e) {
            console.error("更新排序失败:", e);
        }
    };

    return {
        createSession,
        deleteSession,
        renameSession,
        updateSessionScroll,
        reorderSessions
    };
}
