<script setup>
import { ref, watch, onMounted, onUnmounted, onBeforeUnmount, nextTick, computed } from 'vue';
import { debounce } from '../../utils/format';
import { useChatStore } from "../../stores/chat"; 
import { useScrollRestore } from '../../composables/useScrollRestore';
import MessageItem from './MessageItem.vue';
import ModernConfirm from './ModernConfirm.vue';
import SystemPromptBanner from './SystemPromptBanner.vue';

const props = defineProps(['messages', 'sessionId', 'initialScrollPos', 'themeOverride', 'showSystemPrompt', 'assistantAvatar', 'assistantName', 'loadingMore']);
const emit = defineEmits(['update-pos', 'delete', 'regenerate', 'save-edit', 'load-more']);

const chatStore = useChatStore();
const scrollRef = ref(null);
const isRestoring = ref(false); 
const isUserScrolledUp = ref(false); // 💡 追踪用户是否手动向上滚动

const displayMessages = computed(() => (props.messages || []).filter(m => m.role !== 'system'));

// 💡 优化滚动：只监听最后一条消息内容的长度变化 (用于流式渲染时自动滚动)
const lastMsgLen = computed(() => {
  if (!props.messages || props.messages.length === 0) return 0;
  return props.messages[props.messages.length - 1].content?.length || 0;
});

watch(lastMsgLen, (newLen) => {
  if (chatStore.isGenerating && chatStore.generatingSessionId === props.sessionId && !isUserScrolledUp.value) {
    scrollToBottomDefault();
  }
});

// 💡 Simplified Scroll Logic: Always to Bottom
const scrollToBottomDefault = async () => {
   await nextTick();
   setTimeout(() => {
     if (scrollRef.value) {
       scrollRef.value.scrollTo({
         top: scrollRef.value.scrollHeight,
         behavior: 'auto' // Instant jump
       });
     }
   }, 20); 
};

const saveScrollPosition = () => {
   if (!scrollRef.value || !props.sessionId) return;
   const { scrollTop } = scrollRef.value;
   chatStore.updateSessionScroll(props.sessionId, Math.floor(scrollTop));
};

// 💡 编辑状态
const editingIndex = ref(-1);
const editingContent = ref('');

const startEdit = (index, content) => {
  editingIndex.value = index;
  editingContent.value = content;
};

const cancelEdit = () => {
  editingIndex.value = -1;
  editingContent.value = '';
};

// 💡 现代化确认弹窗状态
const confirmState = ref({
  show: false,
  x: 0,
  y: 0,
  index: -1,
  m: null,
  actionLabel: '',
  onConfirm: null
});

const triggerConfirm = (event, index, m, actionLabel, onConfirm) => {
  const rect = event.currentTarget.getBoundingClientRect();
  confirmState.value = {
    show: true,
    x: rect.left,
    y: rect.top,
    index,
    m,
    actionLabel,
    onConfirm
  };
};

const executeConfirm = async () => {
  if (confirmState.value.onConfirm) {
    await confirmState.value.onConfirm();
  }
  confirmState.value.show = false;
};

const handleSaveEdit = async (event, index, m) => {
  triggerConfirm(event, index, m, '修改并重新生成', async () => {
    if (props.themeOverride) {
      emit('save-edit', m.id, index, editingContent.value);
    } else {
      await chatStore.editMessageAction(m.id, index, editingContent.value);
    }
    cancelEdit();
  });
};

const handleDelete = async (m, event) => {
  // Find the message index in the FULL array
  const index = props.messages.indexOf(m);
  if (index === -1) return;

  triggerConfirm(event, index, m, '删除消息', async () => {
    if (props.themeOverride) {
      // For social mode, we pass ID and the full index
      emit('delete', m.id, index);
    } else {
      // Normal mode
      await chatStore.deleteMessageAction(m.id, index);
    }
  });
};

const handleRegenerate = (messageId, event) => {
  const index = props.messages.findIndex(msg => msg.id === messageId);
  if (index === -1) return;

  if (props.themeOverride) {
    emit('regenerate', messageId, index);
  } else {
    chatStore.regenerateAction(index);
  }
};

// 💡 暴露给父组件的滚动方法
defineExpose({ 
  scrollToBottom: (behavior = 'auto') => {
    if (!isRestoring.value && scrollRef.value) {
      scrollRef.value.scrollTo({
        top: scrollRef.value.scrollHeight + 100, 
        behavior: behavior
      });
    }
  }
});

const { performRestore } = useScrollRestore();

const handleScroll = debounce((e) => {
  if (!scrollRef.value) return;
  const { scrollTop, scrollHeight, clientHeight } = scrollRef.value;
  
  // 1. Detect Top Reach (Load More)
  if (scrollTop < 50) {
    emit('load-more');
  }

  // 判定是否在底部 (阈值 60px)
  const isAtBottom = scrollHeight - scrollTop - clientHeight <= 60;
  isUserScrolledUp.value = !isAtBottom;

  if (isRestoring.value || !props.sessionId || chatStore.isLoading) return;
  chatStore.updateSessionScroll(props.sessionId, Math.floor(scrollTop));
  emit('update-pos', Math.floor(scrollTop));
}, 150);

const restorePosition = async () => {
    if (!scrollRef.value || isRestoring.value) return;
    
    // Only force to bottom in social mode (wechat theme)
    if (props.themeOverride === 'wechat') {
        scrollToBottomDefault();
        return;
    }

    // Normal mode: Restore last position if it exists
    if (props.initialScrollPos > 0) {
        isRestoring.value = true;
        await performRestore(scrollRef.value, props.initialScrollPos);
        isRestoring.value = false;
    }
};

// 💡 监听 sessionId 切换，恢复状态
watch(() => props.sessionId, async (newId) => {
  if (newId) {
    isUserScrolledUp.value = false;
    await nextTick();
    restorePosition();
  }
}, { immediate: true });

// 💡 监听生成状态变化,确保在操作按钮渲染后滚动到底部
watch(() => chatStore.isGenerating, async (isGen, wasGen) => {
  // 当生成结束时 (从 true 变为 false),触发一次最终滚动
  // 🟢 修复：必须确保是当前会话结束生成
  if (wasGen && !isGen && chatStore.generatingSessionId === props.sessionId && !isUserScrolledUp.value) {
    // 等待操作按钮渲染完成
    await nextTick();
    setTimeout(() => {
      if (scrollRef.value) {
        scrollRef.value.scrollTo({
          top: scrollRef.value.scrollHeight + 100,
          behavior: 'smooth'
        });
      }
    }, 100);
  }
});


onMounted(() => {
  scrollRef.value?.addEventListener('scroll', handleScroll);
  // restorePosition() 被 watch sessionId immediate: true 覆盖了，这里不需要重复调用
});

onBeforeUnmount(() => {
  saveScrollPosition(); // 💾 Save BEFORE unmounting (when DOM is still valid)
  scrollRef.value?.removeEventListener('scroll', handleScroll);
});
</script>

<template>
  <div class="message-display modern-scroll" ref="scrollRef">
    <Transition name="list-fade">
      <div v-if="!chatStore.isLoading" :key="sessionId" class="scroll-content-wrapper">
        <!-- 🌀 PULL TO LOAD SPINNER -->
        <div v-if="loadingMore" class="pagination-loader">
           <svg class="spinner" viewBox="0 0 50 50">
             <circle class="path" cx="25" cy="25" r="20" fill="none" stroke-width="4"></circle>
           </svg>
        </div>

        <SystemPromptBanner v-if="showSystemPrompt !== false" />
        
        <MessageItem 
          v-for="(m, i) in displayMessages" 
          :key="i"
          :m="m"
          :index="i"
          :sessionId="sessionId"
          :isEditing="editingIndex === i"
          :themeOverride="themeOverride"
          :assistantAvatar="assistantAvatar"
          :assistantName="assistantName"
          @start-edit="startEdit(i, m.content)"
          @cancel-edit="cancelEdit"
          @update-edit-content="val => editingContent = val"
          @save-edit="e => handleSaveEdit(e, i, m)"
          @delete="(id, event) => handleDelete(m, event)"
          @regenerate="(id, event) => handleRegenerate(id, event)"
        />
      </div>
    </Transition>
    <ModernConfirm 
      :show="confirmState.show"
      :x="confirmState.x"
      :y="confirmState.y"
      :actionLabel="confirmState.actionLabel"
      @confirm="executeConfirm"
      @cancel="confirmState.show = false"
    />
  </div>
</template>

<style scoped>
.message-display { flex: 1; padding: 40px 6% 80px 6%; display: flex; flex-direction: column; overflow-y: auto; position: relative; overflow-anchor: none !important; }
.scroll-content-wrapper { display: flex; flex-direction: column; gap: 48px; width: 100%; margin: 0 auto; backface-visibility: hidden; }

/* 🕊️ 优雅淡入淡出 */
.list-fade-enter-active { transition: all 0.3s ease-out; }
.list-fade-leave-active { position: absolute; width: 100%; opacity: 0; }
.list-fade-enter-from { opacity: 0; transform: translateY(10px); filter: blur(4px); }
.list-fade-leave-to { opacity: 0; }

.modern-scroll::-webkit-scrollbar { width: 6px; }
.modern-scroll::-webkit-scrollbar-thumb { background: rgba(0, 0, 0, 0.25); border-radius: 10px; }
.modern-scroll::-webkit-scrollbar-track { background: transparent; }

/* 🌀 Pagination Spinner */
.pagination-loader {
  display: flex;
  justify-content: center;
  padding: 10px 0;
  width: 100%;
  animation: fadeIn 0.3s ease;
}

.spinner {
  animation: rotate 2s linear infinite;
  width: 24px;
  height: 24px;
}

.path {
  stroke: var(--color-primary);
  stroke-linecap: round;
  animation: dash 1.5s ease-in-out infinite;
}

@keyframes rotate {
  100% { transform: rotate(360deg); }
}

@keyframes dash {
  0% { stroke-dasharray: 1, 150; stroke-dashoffset: 0; }
  50% { stroke-dasharray: 90, 150; stroke-dashoffset: -35; }
  100% { stroke-dasharray: 90, 150; stroke-dashoffset: -124; }
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-5px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>