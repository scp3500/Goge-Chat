// src/composables/useScrollRestore.ts
import { nextTick } from 'vue';

export function useScrollRestore() {
    const performRestore = async (scrollEl: HTMLElement, targetPos: number, retryCount = 0) => {
        if (!scrollEl || targetPos <= 0) return;

        await nextTick();
        setTimeout(() => {
            scrollEl.scrollTop = targetPos;
            // 探针逻辑：如果没滚到位，递归重试
            if (Math.abs(scrollEl.scrollTop - targetPos) > 5 && retryCount < 8) {
                performRestore(scrollEl, targetPos, retryCount + 1);
            }
        }, 50);
    };

    return { performRestore };
}