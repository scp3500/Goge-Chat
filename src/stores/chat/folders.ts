import { type Ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import { chatApi, type ChatSession } from '../../api/chat';
import type { Folder } from './state';

export function useFolderActions(folders: Ref<Folder[]>, historyList: Ref<ChatSession[]>) {

    const createFolder = async (name: string) => {
        try {
            const id = await invoke<string>("create_folder", { name });
            // 🚩 新建文件夹默认置顶 (unshift) 且默认折叠 (is_collapsed: true)
            folders.value.unshift({ id, name, sort_order: 0, is_collapsed: true });

            // 同步折叠状态到数据库
            try {
                await invoke("update_folder_collapsed", { id, collapsed: true });
            } catch (err) {
                console.error("同步文件夹折叠状态失败:", err);
            }
        } catch (e) {
            console.error("创建文件夹失败:", e);
        }
    };

    const deleteFolder = async (id: string) => {
        try {
            await invoke("delete_folder", { id });
            folders.value = folders.value.filter(f => f.id !== id);
            // 更新本地 session，去掉它们的 folder_id
            historyList.value.forEach(s => {
                if (s.folder_id === id) s.folder_id = null;
            });
        } catch (e) {
            console.error("删除文件夹失败:", e);
        }
    };

    const renameFolder = async (id: string, name: string) => {
        try {
            await invoke("rename_folder", { id, name });
            const folder = folders.value.find(f => f.id === id);
            if (folder) folder.name = name;
        } catch (e) {
            console.error("重命名文件夹失败:", e);
        }
    };

    const moveSessionToFolder = async (sessionId: string, folderId: string | null) => {
        try {
            await invoke("move_session_to_folder", { sessionId, folderId });
            const session = historyList.value.find(s => s.id === sessionId);
            if (session) session.folder_id = folderId;
        } catch (e) {
            console.error("移动会话失败:", e);
        }
    };

    const toggleFolder = async (id: string) => {
        const folder = folders.value.find(f => f.id === id);
        if (folder) {
            folder.is_collapsed = !folder.is_collapsed;
            try {
                await invoke("update_folder_collapsed", { id, collapsed: folder.is_collapsed });
            } catch (e) {
                console.error("更新文件夹折叠状态失败:", e);
            }
        }
    };

    const reorderFolders = async (newList: Folder[]) => {
        folders.value = newList;
        const orders: [string, number][] = newList.map((f, index) => [f.id, index]);
        try {
            await chatApi.updateFoldersOrder(orders);
        } catch (e) {
            console.error("更新文件夹排序失败:", e);
        }
    };

    return {
        createFolder,
        deleteFolder,
        renameFolder,
        moveSessionToFolder,
        toggleFolder,
        reorderFolders
    };
}
