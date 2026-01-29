<script setup>
import { ref, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { debounce } from '../../utils/format';
import { useChatStore } from "../../stores/chat"; 
import { useScrollRestore } from '../../composables/useScrollRestore';
import MessageItem from './MessageItem.vue';
import ModernConfirm from './ModernConfirm.vue';

const props = defineProps(['messages', 'sessionId', 'initialScrollPos']);
const emit = defineEmits(['update-pos']);

const chatStore = useChatStore();
const scrollRef = ref(null);
const isRestoring = ref(false); 
const isUserScrolledUp = ref(false); // 💡 追踪用户是否手动向上滚动
const { performRestore } = useScrollRestore();

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
    await chatStore.editMessageAction(m.id, index, editingContent.value);
    cancelEdit();
  });
};

const handleDelete = async (event, index, m) => {
  triggerConfirm(event, index, m, '删除消息', async () => {
    await chatStore.deleteMessageAction(m.id, index);
  });
};

// 💡 暴露给父组件的滚动方法
defineExpose({ 
  scrollToBottom: (behavior = 'auto') => {
    if (!isRestoring.value && scrollRef.value) {
      scrollRef.value.scrollTo({
        top: scrollRef.value.scrollHeight + 100, // 添加额外偏移确保滚到最底部
        behavior: behavior
      });
    }
  }
});

const handleScroll = debounce((e) => {
  if (!scrollRef.value) return;
  const { scrollTop, scrollHeight, clientHeight } = scrollRef.value;
  
  // 判定是否在底部 (阈值 60px)
  const isAtBottom = scrollHeight - scrollTop - clientHeight <= 60;
  isUserScrolledUp.value = !isAtBottom;

  if (isRestoring.value || !props.sessionId || chatStore.isLoading) return;
  chatStore.updateSessionScroll(props.sessionId, Math.floor(scrollTop));
  emit('update-pos', Math.floor(scrollTop));
}, 150);

// 监听消息变化，实现智能自动滚动
watch(() => props.messages, async (newVal, oldVal) => {
  // 如果是由于切换会话导致的消息变化，不要滚动 (由 restore 处理)
  // 通过检查长度变化来简单判定是否是流式更新
  // 如果没有手动向上滚动，则自动滚动到底部
  if (!isUserScrolledUp.value && !isRestoring.value) {
     await nextTick();
     if (scrollRef.value) {
       scrollRef.value.scrollTo({
         top: scrollRef.value.scrollHeight + 100, // 添加额外偏移确保滚到最底部
         behavior: 'smooth' // 丝滑滚动
       });
     }
  }
}, { deep: true });

// 💡 监听生成状态变化,确保在操作按钮渲染后滚动到底部
watch(() => chatStore.isGenerating, async (isGen, wasGen) => {
  // 当生成结束时 (从 true 变为 false),触发一次最终滚动
  if (wasGen && !isGen && !isUserScrolledUp.value) {
    // 等待操作按钮渲染完成
    await nextTick();
    // 再多等一帧确保布局完全稳定
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

// 核心监听:切换会话触发坐标恢复
watch([() => props.sessionId, () => chatStore.isLoading], async ([newId, loading]) => {
  if (!newId || loading) return;
  isRestoring.value = true;
  // 重置滚动状态
  isUserScrolledUp.value = false;
  
  if (props.messages?.length > 0) {
    await performRestore(scrollRef.value, props.initialScrollPos || 0);
  }
  
  setTimeout(() => { isRestoring.value = false; }, 500);
}, { immediate: true });

onMounted(() => {
  scrollRef.value?.addEventListener('scroll', handleScroll);
});

onUnmounted(() => scrollRef.value?.removeEventListener('scroll', handleScroll));
</script>

<template>
  <div class="message-display modern-scroll" ref="scrollRef">
    <Transition name="list-fade">
      <div v-if="!chatStore.isLoading" :key="sessionId" class="scroll-content-wrapper">
        <MessageItem 
          v-for="(m, i) in messages" 
          :key="i"
          :m="m"
          :index="i"
          :sessionId="sessionId"
          :isEditing="editingIndex === i"
          @start-edit="startEdit(i, m.content)"
          @cancel-edit="cancelEdit"
          @update-edit-content="val => editingContent = val"
          @save-edit="e => handleSaveEdit(e, i, m)"
          @delete="e => handleDelete(e, i, m)"
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
.message-display { flex: 1; padding: 40px 6% 60px 6%; display: flex; flex-direction: column; overflow-y: auto; position: relative; overflow-anchor: none !important; }
.scroll-content-wrapper { display: flex; flex-direction: column; gap: 48px; width: 100%; margin: 0 auto; backface-visibility: hidden; }

.list-fade-enter-active { transition: all 0.3s ease-out; }
.list-fade-leave-active { position: absolute; width: 100%; opacity: 0; }
.list-fade-enter-from { opacity: 0; transform: translateY(10px); filter: blur(4px); }

.modern-scroll::-webkit-scrollbar { width: 6px; }
.modern-scroll::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
.modern-scroll::-webkit-scrollbar-track { background: transparent; }
</style>