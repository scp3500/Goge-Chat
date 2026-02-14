<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen, emit as tauriEmit } from '@tauri-apps/api/event';
import { useChatStore } from '../../stores/chat';
import { useConfigStore } from '../../stores/config';
import * as PIXI from 'pixi.js';
import { Live2DModel, config } from 'pixi-live2d-display';

// 🎨 引入极简模式专属样式表 (你可以修改 assets/css/minimalist.css)
import '../../assets/css/minimalist.css';
// 🟢 【唯一控制按钮】 - 改这个数，Alice 就会变大变小
// ========================================================
const SIZE = 1.25;
const ALICE_ZOOM = 1;  
const Y_OFFSET = 0.5;
// ========================================================

// 核心配置 (根据 Alice 模型的 77 个遮罩进行修正)
// 全局注入 PIXI (在任何 Live2D 操作之前)
if (typeof window !== 'undefined') {
  window.PIXI = PIXI;
}

// 核心配置 (根据 Alice 模型的 77 个遮罩进行修正)
config.cubism4.maskSize = 4096;
config.cubism4.renderTextureCount = 1; // 🎭 必须为 1，配合魔改库使用
if (Live2DModel.config) {
    Live2DModel.config.maxMasks = 256; // 🎭 进一步提高上限 try 256
}

// 性能模式完全同步
PIXI.Program.defaultFragmentPrecision = PIXI.PRECISION.MEDIUM;
PIXI.Ticker.shared.maxFPS = 240;
PIXI.Filter.defaultResolution = window.devicePixelRatio || 1;

const props = defineProps({
  visible: { type: Boolean, default: false }
});

const emit = defineEmits(['close', 'send']);

const chatStore = useChatStore();
const configStore = useConfigStore();
const inputText = ref('');
const inputRef = ref(null);
const isSending = ref(false);
const isRecording = ref(false);
const mediaRecorder = ref(null);
const audioChunks = ref([]);

// 字幕相关状态
const subtitleText = ref('');
const isTyping = ref(false);

// 拖拽窗口位置状态
const windowPos = ref({ x: 0, y: 0 });
const isDragging = ref(false);
const dragOffset = ref({ x: 0, y: 0 });
const inputWidth = ref(1200);

// Live2D 相关状态
const live2dApp = ref(null);
const live2dModel = ref(null);
const isSpeaking = ref(false);
const currentLipValue = ref(0);
// 🚀 [核心修复]：模型位置独立化，不再绑定窗口
const modelPos = ref({ x: 0, y: 0 });
const isModelDragging = ref(false);

// 🎤 TTS 流式播放相关状态
const audioQueue = ref([]);
const isPlayingAudio = ref(false);
const currentAudioElement = ref(null);
const accumulatedText = ref(''); // 累积的文本,用于句子切分

// 🚀 [V3] 序列化播放控制
const nextAssignIndex = ref(0); // 🚀 [重构] 下一个要分配给文段的序号
const nextToDeliverIndex = ref(0); // 🚀 [重构] 期望入队的音频序号
const pendingAudioMap = ref(new Map()); // 存储乱序到达的音频 {sequenceIndex: audioItem}
const sentenceBuffer = ref(''); // 🚀 [V3] 短句合并缓冲区
const MIN_SENTENCE_LENGTH = 15; // 🚀 [优化] 降低阈值，让短语也能及时触发 TTS
const MAX_FORCE_SPLIT_LENGTH = 35; // 🚀 [新增] 强制切分长度，防止无标点长句憋死

// 🚀 [V2] 任务锁定与请求 ID
const currentRequestId = ref(0);
const currentSessionToken = ref(''); 
const isWaitingForResponse = ref(false); // 🚀 [V2] 用于过滤旧消息的流

// 🚀 [V5] 字节流播放核心状态
const audioCtx = ref(null);
const nextChunkTime = ref(0);
const residualBuffer = ref(null); // 🚀 [修复] 用于处理 PCM 字节对齐的残留缓冲区
const currentStreamingSequence = ref(-1);
const isStreamPlaying = ref(false);

// 🚀 [性能监测] 测量打火延迟
const perfMetrics = ref({
    sendTime: 0,
    llmFirstTokenTime: 0,
    ttsTriggerStartTime: 0, // LLM 出第一个词的时间
    ttsRequestStartTime: 0, // 真正发给 Rust 后端的时间
    ttsFirstAudioTime: 0,
    hasTrackedLLM: false,
    hasTrackedTTS: false
});

const initAudioContext = async () => {
    if (!audioCtx.value) {
        audioCtx.value = new (window.AudioContext || window.webkitAudioContext)({ sampleRate: 32000 });
        console.log('[TTS] AudioContext 已初始化 (32000Hz), State:', audioCtx.value.state);
    }
    if (audioCtx.value.state === 'suspended') {
        return audioCtx.value.resume().then(() => {
            console.log('[TTS] AudioContext 已恢复');
        });
    }
    return Promise.resolve();
};

// 复刻独立版窗口拖拽
const onWindowMouseDown = (e) => {
  if (e.button === 0) {
    startDragging(e);
  }
};

const initPosition = () => {
    const width = window.innerWidth;
    const height = window.innerHeight;

    // 1. 输入框位置：居中底部
    const maxWidth = Math.min(1600, width * 0.80);
    inputWidth.value = maxWidth;
    windowPos.value = {
        x: (width - maxWidth) / 2,
        y: height - 50 - 100
    };

    // 2. 模型初始位置：居中 (或你喜欢的任何位置)
    modelPos.value = {
        x: width * 0.5,
        y: height * 0.5
    };
};

const updatePassthroughMonitor = async () => {
    if (!props.visible) return;
    try {
        const regions = [];
        // 🚀 [性能核心] 拖拽中判定区锁定为全屏，且不随位置更新触发 IPC
        if (isDragging.value || isModelDragging.value) {
            regions.push({ x: 0, y: 0, w: window.innerWidth, h: window.innerHeight });
        } else {
            // 1. 输入框判定区
            regions.push({ x: windowPos.value.x, y: windowPos.value.y, w: inputWidth.value, h: 100 });
            // 2. 模型判定区
            if (live2dModel.value) {
                const model = live2dModel.value;
                regions.push({
                    x: model.x - (model.width * 0.4),
                    y: model.y - (model.height * 0.5),
                    w: model.width * 0.8,
                    h: model.height * 0.9
                });
            }
        }

        const regionsStr = JSON.stringify(regions);
        if (window._lastPassthroughRegions === regionsStr) return;
        window._lastPassthroughRegions = regionsStr;

        await invoke('start_passthrough_monitor', { regions });
    } catch (e) {
        console.error('Failed to update passthrough regions:', e);
    }
};

// 探测是否正在拖拽
const isAnyDragging = computed(() => isDragging.value || isModelDragging.value);

// 🚀 [性能核心] 仅监听状态切换，不监听具体坐标
watch([isAnyDragging, () => props.visible, live2dModel], () => {
    updatePassthroughMonitor();
}, { immediate: true });

// 监听全局鼠标坐标 (仅用于看向鼠标)
let unlistenMouseMove = null;
onMounted(async () => {
    unlistenMouseMove = await listen('global-mouse-move', (event) => {
        window.mouseX = event.payload.x;
        window.mouseY = event.payload.y;
    });
});

// 移到顶层同步注册
onUnmounted(() => {
    if (unlistenMouseMove) unlistenMouseMove();
});

// 监听可见性
watch(() => props.visible, (newVal) => {
  if (newVal) {
    if (windowPos.value.y === 0) initPosition();
    setTimeout(() => {
        inputRef.value?.focus();
        updatePassthroughMonitor();
        initLive2D();
    }, 400);
  } else {
    invoke('set_window_ignore_cursor_events', { ignore: false });
    
    // 🎤 关闭简约模式时停止 TTS
    stopAllTTS();
    
    if (live2dApp.value) {
        console.log('[MinimalistOverlay] 销毁 Live2D App');
        live2dApp.value.destroy(false, { children: true, texture: true, baseTexture: true });
        live2dApp.value = null;
        live2dModel.value = null;
    }
  }
});

// 🚀 [修复] 移除对 live2dModel 的 deep watch，防止 Vue 深度遍历复杂 Pixi 对象导致递归溢出
watch([windowPos, isDragging, isModelDragging, live2dModel], () => {
    if (props.visible) updatePassthroughMonitor();
});

// Live2D 初始化
const initLive2D = async () => {
  try {
    const canvas = document.getElementById('live2d-canvas');
    if (!canvas) return;

    const app = new PIXI.Application({
      view: canvas,
      autoStart: true,
      backgroundAlpha: 0,
      resizeTo: window,
      antialias: true,
      premultipliedAlpha: true,
      powerPreference: 'default',
      resolution: window.devicePixelRatio || 1,
      autoDensity: true,
      hello: false
    });
    live2dApp.value = app;
    app.ticker.maxFPS = 240;
    PIXI.Ticker.shared.maxFPS = 240;

    const modelUrl = '/live2d/alice/alice_model3.json';
    console.log('[MinimalistOverlay] 开始初始化 Live2D...', modelUrl);
    const model = await Live2DModel.from(modelUrl, {
      autoInteract: true,
      idleMotionGroup: 'Idle'
    });

    if (!model) {
        console.error('[MinimalistOverlay] 模型加载失败!');
        throw new Error('模型解析失败');
    }
    console.log('[MinimalistOverlay] 模型加载成功');

    live2dModel.value = model;
    app.stage.addChild(model);
    model.anchor.set(0.5, 0.5);
    model.eventMode = 'static';
    
    // 🚀 [关键修复] 初始位置同步
    model.x = modelPos.value.x;
    model.y = modelPos.value.y;

    // 🏆 [模型独立拖拽监听]
    const onModelMouseDown = (e) => {
        if (e.button !== 0) {
            if (e.button === 2) isSpeaking.value = !isSpeaking.value;
            return;
        }
        
        isModelDragging.value = true;
        dragOffset.value = {
            x: e.clientX - model.x,
            y: e.clientY - model.y
        };
        
        // 🚀 使用原生 mousemove + requestAnimationFrame 确保丝滑跟手
        let rafId = null;
        const onMove = (me) => {
            if (rafId) return;
            rafId = requestAnimationFrame(() => {
                const nx = me.clientX - dragOffset.value.x;
                const ny = me.clientY - dragOffset.value.y;
                modelPos.value = { x: nx, y: ny };
                model.x = nx;
                model.y = ny;
                rafId = null;
            });
        };
        
        const onUp = () => {
            isModelDragging.value = false;
            document.removeEventListener('mousemove', onMove);
            window.removeEventListener('mouseup', onUp);
            if (rafId) cancelAnimationFrame(rafId);
        };
        
        document.addEventListener('mousemove', onMove);
        window.addEventListener('mouseup', onUp);
    };
    canvas.addEventListener('mousedown', onModelMouseDown);

    model.internalModel.on('beforeModelUpdate', () => {
      const coreModel = model.internalModel.coreModel;
      if (coreModel) {
        const now = Date.now() / 1000;
        const set = (id, value) => coreModel.setParameterValueById(id, value, 1.0);

        set('Param66', 1.0); set('Param61', 0.0); set('Param62', 1.0);
        const stickX = Math.sin(now * 3) * 0.2;
        const stickY = Math.sin(now * 4) * 0.2;
        set('LeftStickX', stickX); set('RightStickX', -stickX);
        set('LeftStickY', stickY); set('RightStickY', stickY);
        set('ParamBreath', Math.sin(now * 1.5) * 0.5 + 0.5);

        const light = Math.sin(now * 0.8) * 0.5 + 0.5;
        set('light', light); set('Param65', light);

        if (isSpeaking.value) {
          const speed = 8;
          const noise = Math.sin(now * speed) * Math.sin(now * speed * 0.5);
          let targetOpenness = (noise + 1) / 2 * 0.8 + 0.2;
          if (Math.random() > 0.95) targetOpenness = 0;
          currentLipValue.value += (targetOpenness - currentLipValue.value) * 0.1;
        } else {
          currentLipValue.value += (0 - currentLipValue.value) * 0.1;
        }
        set('ParamMouthOpenY', currentLipValue.value);

        if (window.mouseX !== undefined && !isModelDragging.value) {
             model.focus(window.mouseX, window.mouseY);
        }
      }
    });

    const updateLayout = () => {
      app.renderer.resize(window.innerWidth, window.innerHeight);
      
      // 使用独立的模型位置坐标
      model.x = modelPos.value.x;
      model.y = modelPos.value.y;

      const coreModel = model.internalModel.coreModel;
      let s = 1.0;
      if (coreModel && coreModel.canvasHeight) {
        s = (window.innerHeight / coreModel.canvasHeight) * ALICE_ZOOM;
      } else {
        model.scale.set(1);
        s = (window.innerHeight / model.height) * ALICE_ZOOM;
      }
      model.scale.set(s);
    };

    updateLayout();
    // 🚀 [关键修复] 加载完成后立即刷新一次点击判定区
    updatePassthroughMonitor();
    
    app.renderer.on('destroy', () => {
        canvas.removeEventListener('mousedown', onModelMouseDown);
    });

  } catch (e) {
    console.error('渲染异常:', e);
  }
};

// 监听器引用(用于清理)
let unlistenMessage = null;
let unlistenTyping = null;
let unlistenStreaming = null;
let unlistenAudioChunk = null;

onMounted(async () => {
  initPosition(); 
  // 移除窗口自动重置，保留用户最后的位置
  // window.addEventListener('resize', initPosition);
  
  // 仅监听 AI 的消息来更新字幕显示
  unlistenMessage = await listen('new-social-message', (event) => {
    if (props.visible && event.payload.role === 'assistant') {
      subtitleText.value = event.payload.content;
    }
  });

  unlistenTyping = await listen('typing-status', (event) => {
    if (props.visible) {
      isTyping.value = event.payload.isTyping;
      // 只有在没有流式内容时，才显示“正在输入”
      if (isTyping.value && !subtitleText.value) subtitleText.value = '正在输入...';
      if (!isTyping.value && subtitleText.value === '正在输入...') subtitleText.value = '';
    }
  });

  // 🚀 [流式传输监听 + TTS 生成]
  unlistenStreaming = await listen('social-streaming-chunk', (event) => {
    if (props.visible) {
      // 🚀 [V2 核心修复] 如果我们刚刚发送了消息，但在等第一个 chunk 时收到了旧 chunk，则丢弃
      if (isWaitingForResponse.value && !event.payload.isFirst) {
        console.log('🚮 [TTS] 丢弃上一条消息的延迟 Chunk');
        return;
      }

      if (event.payload.isFirst) {
        // 🚀 [性能监测] 记录 LLM 首字耗时
        if (!perfMetrics.value.hasTrackedLLM) {
            perfMetrics.value.llmFirstTokenTime = performance.now();
            const latency = perfMetrics.value.llmFirstTokenTime - perfMetrics.value.sendTime;
            console.log(`%c[PERF] ⚡ LLM 首字延迟: ${latency.toFixed(2)}ms`, "color: #00ff00; font-weight: bold;");
            perfMetrics.value.hasTrackedLLM = true;
        }
        isWaitingForResponse.value = false; // 收到第一个 chunk，正式开始
        subtitleText.value = event.payload.content;
        accumulatedText.value = event.payload.content;
      } else {
        subtitleText.value += event.payload.content;
        accumulatedText.value += event.payload.content;
      }
      
      // 🚀 [V3] 如果流式传输结束,强制刷新缓冲区
      if (event.payload.isDone) {
        console.log('[TTS] 流式传输结束,刷新缓冲区');
        flushSentenceBuffer();
      } else {
        // 🎤 检测句子结束,尝试合并或触发 TTS 生成
        checkAndGenerateTTS();
      }
    }
  });

  // 🚀 [V5] 接收字节流 Chunk 并调度播放
  unlistenAudioChunk = await listen('tts-audio-chunk', (event) => {
    const { requestId, sequenceId, chunk, isDone, backendPrepMs } = event.payload;
    
    // 🚀 [性能监测] 记录 TTS 首声耗时
    if (!perfMetrics.value.hasTrackedTTS && chunk && sequenceId === 0) {
        perfMetrics.value.ttsFirstAudioTime = performance.now();
        const totalLatency = perfMetrics.value.ttsFirstAudioTime - perfMetrics.value.sendTime;
        const llmWait = perfMetrics.value.llmFirstTokenTime - perfMetrics.value.sendTime;
        const ttsSentenceWait = perfMetrics.value.ttsRequestStartTime - perfMetrics.value.llmFirstTokenTime;
        const ttsFullWait = perfMetrics.value.ttsFirstAudioTime - perfMetrics.value.ttsRequestStartTime;
        
        console.log(`%c[PERF-ANALYSIS] 🛠️ 全链路延迟分析 (总计: ${totalLatency.toFixed(0)}ms)`, "background: #222; color: #ffeb3b; padding: 4px; border-radius: 4px; font-weight: bold;");
        console.log(`  1. LLM 打火 (问->答): ${llmWait.toFixed(0)}ms`);
        console.log(`  2. TTS 缓冲等待 (答->送): ${ttsSentenceWait.toFixed(0)}ms (等待标点符号)`);
        console.log(`  3. TTS 后端处理 (送->响): ${ttsFullWait.toFixed(0)}ms (含网络+推理)`);
        if (backendPrepMs) {
            console.log(`     └─ 其中 GPT-SoVITS 推理耗时: ${backendPrepMs}ms`);
        }
        perfMetrics.value.hasTrackedTTS = true;
    }

    // 找到对应的音频项 (优先从主队列找，找不到再去暂存区找)
    let audioItem = audioQueue.value.find(i => Number(i.sequenceIndex) === Number(sequenceId));
    if (!audioItem) {
        audioItem = pendingAudioMap.value.get(Number(sequenceId));
    }

    if (audioItem && audioItem.audioData === 'STREAMING') {
        if (chunk) {
            let raw = base64ToArrayBuffer(chunk);
            
            // 🚀 [修复] 如果是第一波数据且包含 RIFF (WAV) 头，裁掉前 44 字节
            if (audioItem.chunks.length === 0 && raw.byteLength >= 44) {
                const u8 = new Uint8Array(raw);
                if (u8[0] === 0x52 && u8[1] === 0x49 && u8[2] === 0x46 && u8[3] === 0x46) {
                    console.log('[V5-CHUNK] 检测到 WAV 容器头，已自动剥离 PCM 数据');
                    raw = raw.slice(44);
                }
            }

            const isMatch = isPlayingAudio.value && Number(currentStreamingSequence.value) === Number(sequenceId);
            
            if (isMatch) {
                schedulePCMChunk(raw);
            } else {
                audioItem.chunks.push(raw);
            }
        }
        if (isDone) {
            audioItem.isDone = true;
            console.log(`[V5] 序列 ${sequenceId} 流结束`);
        }
    }
  });
});

onUnmounted(() => {
  window.removeEventListener('resize', initPosition);
  if (unlistenMessage) unlistenMessage();
  if (unlistenTyping) unlistenTyping();
  if (unlistenStreaming) unlistenStreaming();
  
  // 清理 Live2D 资源
  if (live2dApp.value) {
    live2dApp.value.destroy(true, { children: true, texture: true });
    live2dApp.value = null;
  }
});

// 处理输入框逻辑
const handleKeyDown = (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    handleSend();
  } else if (e.key === 'Escape') {
    emit('close');
  } else if (e.key === 'v' && e.altKey) {
    e.preventDefault();
    if (isRecording.value) {
      stopRecording();
    } else {
      startRecording();
    }
  }
};

const handleSend = async () => {
  const text = inputText.value.trim();
  if (!text || isSending.value) return;

  // 🚀 [性能监测] 记录开始发送时间
  perfMetrics.value.sendTime = performance.now();
  perfMetrics.value.hasTrackedLLM = false;
  perfMetrics.value.hasTrackedTTS = false;
  
  // 🚀 清空之前的字幕内容，准备接收新流
  subtitleText.value = '';
  
  // 🎤 清空 TTS 队列和累积文本
  stopAllTTS();
  accumulatedText.value = '';
  
  // 🚀 [V3] 重置序列索引与缓冲区
  nextAssignIndex.value = 0;
  nextToDeliverIndex.value = 0;
  pendingAudioMap.value.clear();
  sentenceBuffer.value = '';
  
  // 🚀 [V2] 产生新的请求 ID,让旧的后台任务失效
  try {
    currentRequestId.value = await invoke('next_tts_request_id');
    currentSessionToken.value = Math.random().toString(36).substring(7);
    isWaitingForResponse.value = true; // 开始等待新的一轮
  } catch(e) {
    console.warn('获取请求 ID 失败:', e);
  }
  
  try {
    isSending.value = true;
    
    // 1. 发送消息到后端并获取真正的数据库 ID
    const msgId = await invoke('save_social_message', {
      contactId: chatStore.activeSocialContactId,
      sessionId: chatStore.activeSocialSessionId,
      role: 'user',
      content: text,
      fileMetadata: null
    });

    // 2. 全局同步：确保字段与 SocialChatContainer.vue 里的监听器完全匹配
    const msgData = {
        messageId: msgId, // 必须包含 ID，否则主界面会因为找不到 ID 而同步失败
        contactId: chatStore.activeSocialContactId,
        sessionId: chatStore.activeSocialSessionId,
        role: 'user',
        content: text,
        createdAt: new Date().toISOString() // 这里的变量名也要注意
    };

    // 发射给父组件
    emit('send', msgData);
    
    // 关键修复：发射全局事件，确保字段名符合 SocialChatContainer 的解构逻辑
    await tauriEmit('new-social-message', {
        ...msgData,
        created_at: msgData.createdAt // 主界面用的是 created_at
    });

    // 3. 触发沉浸式回复逻辑
    if (configStore.settings.chatMode?.enabled) {
      // 🚀 [性能监测] 记录开始请求 AI 的时间
      perfMetrics.value.ttsRequestStartTime = performance.now(); // 初步记录，会被后面的覆盖
      
      await invoke('send_social_message_immersive', {
        sessionId: chatStore.activeSocialSessionId,
        contactId: chatStore.activeSocialContactId,
        content: text
      });
    }

    // 4. 清空输入框并确保焦点保持
    inputText.value = '';
    await nextTick();
    inputRef.value?.focus();
  } catch (e) {
    console.error('Failed to send:', e);
  } finally {
    isSending.value = false;
  }
};

// 语音录制逻辑
const startRecording = async () => {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    audioChunks.value = [];
    
    mediaRecorder.value = new MediaRecorder(stream);
    mediaRecorder.value.ondataavailable = (event) => {
      if (event.data.size > 0) {
        audioChunks.value.push(event.data);
      }
    };
    
    mediaRecorder.value.onstop = async () => {
      const audioBlob = new Blob(audioChunks.value, { type: 'audio/webm' });
      await processAudio(audioBlob);
      stream.getTracks().forEach(track => track.stop());
    };
    
    mediaRecorder.value.start();
    isRecording.value = true;
    console.log('[MinimalistOverlay] 开始录音');
  } catch (e) {
    console.error('录音失败:', e);
  }
};

const stopRecording = () => {
  if (mediaRecorder.value && isRecording.value) {
    mediaRecorder.value.stop();
    isRecording.value = false;
    console.log('[MinimalistOverlay] 停止录音');
  }
};

const processAudio = async (audioBlob) => {
  try {
    const arrayBuffer = await audioBlob.arrayBuffer();
    const audioContext = new AudioContext({ sampleRate: 16000 });
    const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
    
    // 转换为单声道 PCM
    const pcmData = audioBuffer.getChannelData(0);
    const samples = Array.from(pcmData);
    
    console.log('[MinimalistOverlay] 发送音频到ASR, 样本数:', samples.length);
    
    const text = await invoke('transcribe_pcm', {
      samples,
      sampleRate: 16000
    });
    
    if (text && text.trim()) {
      inputText.value = text;
      await nextTick();
      inputRef.value?.focus();
    }
  } catch (e) {
    console.error('音频处理失败:', e);
  }
};

// 🚀 [性能优化] 窗口拖拽同样使用原生 + RAF
const startDragging = (e) => {
  isDragging.value = true;
  dragOffset.value = {
    x: e.clientX - windowPos.value.x,
    y: e.clientY - windowPos.value.y
  };
  
  let rafId = null;
  const onMove = (me) => {
    if (rafId) return;
    rafId = requestAnimationFrame(() => {
      windowPos.value = {
        x: me.clientX - dragOffset.value.x,
        y: me.clientY - dragOffset.value.y
      };
      rafId = null;
    });
  };
  
  const stopDragging = () => {
    isDragging.value = false;
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', stopDragging);
    if (rafId) cancelAnimationFrame(rafId);
  };
  
  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', stopDragging);
};

// 🚀 [V3] 强制刷新并发送缓冲区中的文本
const flushSentenceBuffer = () => {
    // 处理 accumulatedText 中剩余的文本(不一定带标点)
    if (accumulatedText.value.trim()) {
        sentenceBuffer.value += accumulatedText.value.trim();
        accumulatedText.value = '';
    }
    
    if (sentenceBuffer.value.trim()) {
        console.log('[TTS] 强制刷新缓冲区:', sentenceBuffer.value);
        const sequenceIndex = nextAssignIndex.value;
        nextAssignIndex.value++;
        generateTTSForSentence(sentenceBuffer.value.trim(), sequenceIndex);
        sentenceBuffer.value = '';
    }
};

// 🎤 [TTS 功能] 检测句子结束并生成 TTS
const checkAndGenerateTTS = () => {
  let text = accumulatedText.value;
  // 🚀 [重构] 加入逗号支持，AI 读到一个短句或逗号就发起请求，显著降低前几秒的等待感
  const sentenceEndRegex = /[。！？!?；，, \n]/;
  
  let match;
  while ((match = text.match(sentenceEndRegex)) !== null) {
    const mark = match[0];
    const endIndex = text.indexOf(mark) + 1;
    const sentence = text.substring(0, endIndex).trim();
    
    if (sentence.length > 0) {
      sentenceBuffer.value += sentence;
      
      // 🚀 [逻辑优化] 只要有内容就累积，遇到标点或长度足够就开始 TTS
      if (sentenceBuffer.value.length >= MIN_SENTENCE_LENGTH || /[。！？!?；,，\n]/.test(mark)) {
          console.log('[TTS] 触发请求 (Len: ' + sentenceBuffer.value.length + ', Mark: ' + mark + '):', sentenceBuffer.value);
          const sequenceIndex = nextAssignIndex.value;
          nextAssignIndex.value++;
          generateTTSForSentence(sentenceBuffer.value, sequenceIndex);
          sentenceBuffer.value = '';
      } else {
          console.log('[TTS] 已缓冲片段, 当前总长:', sentenceBuffer.value.length);
      }
    }
    
    text = text.substring(endIndex);
  }
  
  
  // 🚀 [新增] 强制长句切分逻辑：如果剩余文本太长且没有标点，强制切出一段来播放
  if (text.length >= MAX_FORCE_SPLIT_LENGTH) {
      console.log('[TTS] 触发强制切分 (无标点长句):', text);
      const sequenceIndex = nextAssignIndex.value;
      nextAssignIndex.value++;
      generateTTSForSentence(text, sequenceIndex);
      text = '';
  }
  
  accumulatedText.value = text;
};

// 🎤 [TTS 功能] 为单个句子生成 TTS
const generateTTSForSentence = async (sentence, sequenceIndex) => {
  const taskId = currentRequestId.value;
  const sessionToken = currentSessionToken.value;

  // 🚀 [性能监测] 记录向后端发起请求的时间 (仅针对第一段)
  if (Number(sequenceIndex) === 0) {
      perfMetrics.value.ttsRequestStartTime = performance.now();
  }

  try {
    console.log(`[V4-CORE] 申请本地生成 (ID: ${taskId}, Seq: ${sequenceIndex}):`, sentence);
    
    // 🚀 [核心] 调用后端本地 TTS
    // 🚀 [关键修复] 先创建并在队列中预位，再执行 invoke
    // 否则 invoke 阻塞期间到达的 chunk 会因为找不到 audioItem 而被丢弃
    const audioItem = {
      sentence,
      audioData: 'STREAMING', 
      taskId,
      sequenceIndex,
      chunks: [],
      isDone: false
    };

    // 🚀 [严格交付] 仅在序号对齐时入队
    if (sequenceIndex === nextToDeliverIndex.value) {
      console.log(`[V5-STREAM] 序列 ${sequenceIndex} 对齐, 预入队并开始生成`);
      audioQueue.value.push(audioItem);
      nextToDeliverIndex.value++;
      
      // 级联处理暂存区
      while (pendingAudioMap.value.has(nextToDeliverIndex.value)) {
        const nextItem = pendingAudioMap.value.get(nextToDeliverIndex.value);
        audioQueue.value.push(nextItem);
        pendingAudioMap.value.delete(nextToDeliverIndex.value);
        nextToDeliverIndex.value++;
      }
      
      if (!isPlayingAudio.value) {
        playNextAudio();
      }
    } else {
      console.log(`[V5-STREAM] 序列 ${sequenceIndex} 乱序到达, 暂存入库`);
      pendingAudioMap.value.set(sequenceIndex, audioItem);
    }

    // 现在执行 invoke，它会阻塞直到后端流式推送结束
    const result = await invoke('generate_tts', { 
        text: sentence,
        requestId: taskId,
        sequenceId: sequenceIndex
    });
    
    // 🚀 [关键修复] Invoke 返回意味着流已经结束。即使没收到 Chunk (比如只有标点)，也要标为 Done。
    audioItem.isDone = true;
    console.log(`[V5-STREAM] 序列 ${sequenceIndex} Invoke 返回并标记完毕: ${result}`);

  } catch (e) {
    console.error('[V5-STREAM] TTS 生成触发失败:', e);
    // 这里如果失败了，理想情况下应该把预占的 item 标记为 done 以跳过
  }
};

// 🚀 [V5] 记录当前正在流式播放的序号
// const currentStreamingSequence = ref(-1); // 已移动到上方顶层

// 🎤 [TTS 功能] 播放队列中的下一个音频
const playNextAudio = async () => {
  // 🚀 [修复] 如果已经在播放，不允许多次触发，防止逻辑冲突
  if (isPlayingAudio.value && currentStreamingSequence.value !== -1) {
      console.log('[V4-CORE] 已经在播放流式音频，跳过触发');
      return;
  }

  if (audioQueue.value.length === 0) {
    isPlayingAudio.value = false;
    isSpeaking.value = false;
    currentAudioElement.value = null;
    currentStreamingSequence.value = -1;
    return;
  }
  
  // 安全校验：清除过期任务
  if (audioQueue.value[0].taskId !== currentRequestId.value) {
      console.warn('[V4-CORE] 清理过期队列任务');
      audioQueue.value = [];
      isPlayingAudio.value = false;
      isSpeaking.value = false;
      return;
  }

  const audioItem = audioQueue.value[0]; 
  isPlayingAudio.value = true;
  
  try {
    if (audioItem.audioData === 'STREAMING') {
        console.log('[V5] 启动流式播放:', audioItem.sentence);
        
        // 🚀 [关键修复] 先发布状态，后 Await 硬件。防止 Await 期间来的 Chunk 匹配失败。
        currentStreamingSequence.value = Number(audioItem.sequenceIndex);
        isPlayingAudio.value = true;
        isSpeaking.value = true;

        await initAudioContext();
        
        // 🚀 [优化] 衔接逻辑：如果当前时间离上一次调度结束还没超过 0.5 秒，则继续追加调度，实现无缝衔接。
        const now = audioCtx.value.currentTime;
        if (nextChunkTime.value < now || nextChunkTime.value > now + 2.0) {
            console.log('[V5] 重新校准音频调度时间轴');
            nextChunkTime.value = now + 0.05;
        } else {
            console.log(`[V5] 沿用现有时间轴，偏移: ${(nextChunkTime.value - now).toFixed(3)}s`);
        }

        // 播放现有的缓存 chunks
        console.log(`[V5] 补播缓存 Chunks: ${audioItem.chunks.length} 个`);
        while (audioItem.chunks.length > 0) {
            schedulePCMChunk(audioItem.chunks.shift());
        }

        // 持续检测是否结束
        let lastCheckTime = Date.now();
        let lastLength = -1;
        
        const checkEnd = () => {
            // 🚀 [安全补偿] 消费可能错过的 chunks (防止竞态)
            while (audioItem.chunks.length > 0) {
                console.log('[V5] checkEnd 补播错过的项');
                schedulePCMChunk(audioItem.chunks.shift());
            }

            const isDone = audioItem.isDone && audioItem.chunks.length === 0;
            const now = Date.now();
            
            // 🚀 [安全退出] 如果 5 秒钟没有新数据且已标记 Done，强行结束，防止因为逻辑 Bug 卡死
            const isTimeout = audioItem.isDone && (now - lastCheckTime > 5000);
            
            if (isDone || isTimeout) {
                if (isTimeout) console.warn('[V5] 检测到播放超时，强行切换下一段');
                
                // 等待最后一段播完
                const waitTime = Math.max(0, (nextChunkTime.value - audioCtx.value.currentTime) * 1000 + 50);
                setTimeout(() => {
                    console.log('[V5] 流式段结束，切换下一段');
                    audioQueue.value.shift(); // 消费掉
                    isPlayingAudio.value = false; 
                    playNextAudio();
                }, waitTime);
            } else {
                if (audioItem.chunks.length !== lastLength) {
                    lastCheckTime = now;
                    lastLength = audioItem.chunks.length;
                }
                setTimeout(checkEnd, 100);
            }
        };
        checkEnd();

    } else {
        audioQueue.value.shift(); // 普通模式先消费
        let url;
        const isRemote = audioItem.audioData.startsWith('URL:');
        
        // 🚀 [修复] 检测是否为本地文件路径
        const isLocalFile = /^[a-zA-Z]:\\/.test(audioItem.audioData) || audioItem.audioData.includes('\\') || audioItem.audioData.startsWith('/');

        if (isRemote) {
          url = audioItem.audioData.substring(4);
          console.log('[V4-CORE] 启动远程流播放:', audioItem.sentence, url);
        } else if (isLocalFile) {
          url = convertFileSrc(audioItem.audioData);
          console.log('[V4-CORE] 启动本地文件播放:', audioItem.sentence, url);
        } else {
          const blob = base64ToBlob(audioItem.audioData, 'audio/wav');
          url = URL.createObjectURL(blob);
          console.log('[V4-CORE] 启动本地二进制播放:', audioItem.sentence);
        }
        
        const audio = new Audio(url);
        audio.volume = 1.0; 
        currentAudioElement.value = audio;
        
        audio.onplay = () => {
          console.log('[V4-CORE] 音频开始输出，触发动嘴动画');
          isSpeaking.value = true;
        };

        audio.onended = () => {
          console.log('[V4-CORE] 播放完成:', audioItem.sentence);
          if (!isRemote) URL.revokeObjectURL(url);
          isSpeaking.value = false;
          setTimeout(playNextAudio, 50); 
        };
        
        audio.onerror = (e) => {
          console.error('[V4-CORE] 音频加载/播放失败:', e, url);
          if (!isRemote) URL.revokeObjectURL(url);
          isSpeaking.value = false;
          playNextAudio();
        };
        
        audio.play().catch(e => {
          console.error('[V4-CORE] 自动播放被拦截或失败:', e);
          isSpeaking.value = false;
          playNextAudio();
        });
    }
  } catch (err) {
    console.error('[V4-CORE] 播放初始化异常:', err);
    playNextAudio();
  }
};


// 🚀 [V5] Base64 转 ArrayBuffer
const base64ToArrayBuffer = (base64) => {
    const binary = window.atob(base64);
    const len = binary.length;
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    return bytes.buffer;
};

// 🚀 [V5] 调度 PCM 采样播放
const schedulePCMChunk = (arrayBuffer) => {
    if (!audioCtx.value) {
        console.warn('[TTS] AudioContext 未初始化，放弃调度 Chunk');
        return;
    }
    
    // 自动恢复上下文 (防止某些浏览器由于非点击触发而被挂起)
    if (audioCtx.value.state === 'suspended') {
        audioCtx.value.resume();
    }
    
    // 🚀 [修复核心] 字节对齐处理
    let currentData = new Uint8Array(arrayBuffer);
    if (residualBuffer.value) {
        const combined = new Uint8Array(residualBuffer.value.length + currentData.length);
        combined.set(residualBuffer.value);
        combined.set(currentData, residualBuffer.value.length);
        currentData = combined;
        residualBuffer.value = null;
    }
    
    // 如果长度是奇数，保留最后一个字节到下一次处理
    if (currentData.length % 2 !== 0) {
        residualBuffer.value = currentData.slice(-1);
        currentData = currentData.slice(0, -1);
    }
    
    if (currentData.length === 0) return;

    // 将 Int16（2字节）转换为 Float32
    // 诊断日志：打印前几个采样值
    const int16Array = new Int16Array(currentData.buffer, currentData.byteOffset, currentData.byteLength / 2);
    
    // 🚀 加强日志：如果采样全为 0，特别指出
    const isSilent = int16Array.every(v => v === 0);
    console.log(`[TTS] Chunk 调度: ${int16Array.length} samples, 第一个采样: ${int16Array[0]}${isSilent ? ' (⚠️ 全程静音!)' : ''}`);

    const float32Array = new Float32Array(int16Array.length);
    for (let i = 0; i < int16Array.length; i++) {
        float32Array[i] = int16Array[i] / 32768.0;
    }
    
    const audioBuffer = audioCtx.value.createBuffer(1, float32Array.length, 32000);
    audioBuffer.getChannelData(0).set(float32Array);
    
    const source = audioCtx.value.createBufferSource();
    source.buffer = audioBuffer;
    
    // 🚀 [优化] 显式增加 GainNode 控制音量
    const gainNode = audioCtx.value.createGain();
    gainNode.gain.value = 1.0; 
    
    source.connect(gainNode);
    gainNode.connect(audioCtx.value.destination);
    
    // 增加一个极小的平滑偏置，防止由于调度精度导致的爆音
    const startTime = Math.max(audioCtx.value.currentTime, nextChunkTime.value);
    source.start(startTime);
    nextChunkTime.value = startTime + audioBuffer.duration;
};

// 🎤 [TTS 功能] 停止所有 TTS 播放
const stopAllTTS = () => {
  // 停止当前播放
  if (currentAudioElement.value) {
    currentAudioElement.value.pause();
    currentAudioElement.value = null;
  }
  
  // 清空队列
  audioQueue.value = [];
  isPlayingAudio.value = false;
  isSpeaking.value = false;
  
  // 🚀 [V3] 清空序列状态
  nextAssignIndex.value = 0;
  nextToDeliverIndex.value = 0;
  pendingAudioMap.value.clear();
  sentenceBuffer.value = '';
  residualBuffer.value = null; // 清空残留缓冲
  currentStreamingSequence.value = -1;
  console.log('[TTS] 已停止所有播放');
};

// 🎤 [工具函数] Base64 转 Blob (恢复被意外删除的函数)
const base64ToBlob = (base64, type) => {
  const binaryString = window.atob(base64);
  const len = binaryString.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return new Blob([bytes], { type });
};
</script>

<template>
  <div v-if="visible" class="minimalist-root">
    <!-- Live2D Canvas 层 -->
    <canvas id="live2d-canvas" class="live2d-canvas"></canvas>
    
    <!-- 字幕显示区域 (复刻 ShowText) -->
    <div class="subtitle-area">
      <div class="subtitle-text" :class="{ 'is-typing': isTyping }">
        {{ subtitleText }}
      </div>
    </div>

    <!-- 文本输入窗口 (复刻 TextWindow) -->
    <div 
        class="text-window" 
        :style="{ 
            left: windowPos.x + 'px', 
            top: windowPos.y + 'px',
            width: inputWidth + 'px',
            cursor: isDragging ? 'grabbing' : 'default'
        }"
    >
      <div class="h-layout">
        <input
          ref="inputRef"
          v-model="inputText"
          class="q-line-edit"
          placeholder="输入文字后按回车..."
          @keydown="handleKeyDown"
          @mousedown.stop
          @click="() => console.log('[MinimalistOverlay] 输入框被点击')"
          @focus="() => console.log('[MinimalistOverlay] 输入框获得焦点')"
          :disabled="isSending"
        />
        <div 
            class="voice-button" 
            :class="{ 'recording': isRecording }"
            @mousedown.stop="startRecording"
            @mouseup="stopRecording"
            @mouseleave="stopRecording"
            title="按住说话 (Alt+V)"
        >
          {{ isRecording ? '🔴' : '🎤' }}
        </div>
        <div 
            class="drag-button" 
            @mousedown.stop="onWindowMouseDown"
            title="拖动位置"
        >
          ⋮⋮
        </div>
      </div>
    </div>

    <!-- 退出提示 -->
    <div class="exit-hint">
        按 Esc 或点击导航栏按钮退出极简模式
    </div>
  </div>
</template>


