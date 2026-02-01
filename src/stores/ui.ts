import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export const useUIStore = defineStore('ui', () => {
    /** 
     * 当前激活的菜单 ID
     * 'model-selector' | 'search-menu' | null
     */
    const activeMenuId = ref<string | null>(null);

    // 📱 Layout State Persistence
    const isHistoryOpen = ref(localStorage.getItem('ui_history_open') === 'true');
    const isLeftSidebarOpen = ref(localStorage.getItem('ui_left_sidebar_open') !== 'false'); // Default true

    // Watchers for persistence
    watch(isHistoryOpen, (val) => {
        localStorage.setItem('ui_history_open', val.toString());
    });

    watch(isLeftSidebarOpen, (val) => {
        localStorage.setItem('ui_left_sidebar_open', val.toString());
    });

    /**
     * 设置当前激活的菜单
     * @param id 菜单 ID，传 null 则关闭所有菜单
     */
    const setActiveMenu = (id: string | null) => {
        console.log('🖥️ UI Store: setActiveMenu', id);
        activeMenuId.value = id;
    };

    /**
     * 切换菜单状态
     * @param id 菜单 ID
     */
    const toggleMenu = (id: string) => {
        if (activeMenuId.value === id) {
            activeMenuId.value = null;
        } else {
            activeMenuId.value = id;
        }
        console.log('🖥️ UI Store: toggleMenu', id, '->', activeMenuId.value);
    };

    /**
     * 检查某个菜单是否处于激活状态
     * @param id 菜单 ID
     */
    const isMenuOpen = (id: string) => activeMenuId.value === id;

    return {
        activeMenuId,
        isHistoryOpen,
        isLeftSidebarOpen,
        setActiveMenu,
        toggleMenu,
        isMenuOpen
    };
});
