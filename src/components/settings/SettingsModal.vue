<script setup>
import { ref } from 'vue'; // 【致命缺失：必须补上这行】
import { useConfigService } from '../../services/configService';
import { EYE_OPEN_SVG, EYE_CLOSED_SVG } from '../../constants/icons.ts';

// 1. 注入配置服务
const configStore = useConfigService();
// settings 在 store 中已经是 ref，这里直接引用
const { settings } = configStore;

const emit = defineEmits(['close']);

// 2. 主题列表
const themes = ['#202124', '#1b1b1f', '#0f0f10', '#2c2c32'];

// 3. 布局控制变量
const activeCategory = ref('models');
const activeProvider = ref('deepseek');

// 4. API Key 显示控制
const showApiKey = ref(false);

// 5. 供应商状态管理
const providers = ref([
  { id: 'deepseek', name: 'DeepSeek', icon: '🐋', status: 'on' },
  { id: 'openai', name: 'OpenAI', icon: '🤖', status: 'off' }
]);

// 6. 切换供应商状态
const toggleProviderStatus = (providerId) => {
  const provider = providers.value.find(p => p.id === providerId);
  if (provider) {
    provider.status = provider.status === 'on' ? 'off' : 'on';
  }
};

// 7. DeepSeek 下的常见供应商列表
const deepseekProviders = [
  { id: 'deepseek-chat', name: 'DeepSeek Chat', icon: '🧠', status: 'on' },
  { id: 'claude', name: 'Claude', icon: '🦜', status: 'off' },
  { id: 'gemini', name: 'Gemini', icon: '💎', status: 'off' },
  { id: 'gpt4', name: 'GPT-4', icon: '🤖', status: 'off' },
  { id: 'llama', name: 'Llama', icon: '🦙', status: 'off' },
  { id: 'qwen', name: 'Qwen', icon: '🐑', status: 'off' }
];


</script>

<template>
  <div class="settings-layout">
    <aside class="nav-sidebar">
      <div class="nav-group">
        <div class="nav-item" :class="{ active: activeCategory === 'models' }" @click="activeCategory = 'models'">
          <span class="icon">☁️</span>
          <span class="label">模型服务</span>
        </div>
        <div class="nav-item" :class="{ active: activeCategory === 'appearance' }" @click="activeCategory = 'appearance'">
          <span class="icon">🎨</span>
          <span class="label">显示设置</span>
        </div>
      </div>
      
      <div class="nav-divider"></div>

      <div class="nav-group">
        <div class="nav-item" :class="{ active: activeCategory === 'general' }" @click="activeCategory = 'general'">
          <span class="icon">⚙️</span>
          <span class="label">通用设置</span>
        </div>
        <div class="nav-item" :class="{ active: activeCategory === 'data' }" @click="activeCategory = 'data'">
          <span class="icon">💾</span>
          <span class="label">数据管理</span>
        </div>
      </div>
    </aside>

    <section class="provider-sidebar">
      <div class="sidebar-header">
        <div class="search-box">
          <input type="text" placeholder="搜索模型平台..." />
          <span class="search-icon">🔍</span>
        </div>
      </div>
      
      <div class="list-container modern-scroll">
        <div
          v-for="provider in providers"
          :key="provider.id"
          class="list-item"
          :class="{ active: activeProvider === provider.id }"
          @click="activeProvider = provider.id"
        >
          <div class="item-left">
            <span class="p-icon">{{ provider.icon }}</span>
            <span class="p-name">{{ provider.name }}</span>
          </div>
          <div class="item-right">
            <button
              class="toggle-btn"
              :class="{ on: provider.status === 'on' }"
              @click.stop="toggleProviderStatus(provider.id)"
            >
              <span class="toggle-slider"></span>
            </button>
            <span class="status-tag" :class="provider.status">{{ provider.status === 'on' ? 'ON' : 'OFF' }}</span>
          </div>
        </div>

      </div>
    </section>

    <main class="detail-panel modern-scroll">
      <div class="detail-container">
        
        <header class="detail-header">
          <div class="header-info">
            <h2>{{
              activeProvider === 'deepseek' ? 'DeepSeek' :
              activeProvider === 'openai' ? 'OpenAI' :
              '配置详情'
            }}</h2>
          </div>
        </header>

        <div class="config-content">
          <div class="config-group" v-if="activeCategory === 'models'">
            <div class="group-header">
              <label>DeepSeek API Key</label>
            </div>
            <div class="input-row">
              <div class="input-box">
                <input
                  :type="showApiKey ? 'text' : 'password'"
                  v-model="settings.apiKey"
                  placeholder="在此输入您的 sk-..."
                  @change="configStore.updateConfig(settings)"
                />
                <button
                  class="eye-btn"
                  @click="showApiKey = !showApiKey"
                  v-html="showApiKey ? EYE_OPEN_SVG : EYE_CLOSED_SVG"
                ></button>
              </div>
            </div>
            <p class="hint">点击眼睛图标切换显示/隐藏密钥。</p>
          </div>

          <div class="config-group" v-if="activeCategory === 'appearance'">
            <div class="group-header">
              <label>界面外观与显示</label>
            </div>
            <div class="settings-card">
              <div class="control-item">
                <label>聊天字号 ({{ settings.fontSize }}px)</label>
                <input type="range" v-model="settings.fontSize" min="12" max="24" @input="configStore.updateConfig(settings)" />
              </div>
              <div class="control-item">
                <label>主题颜色</label>
                <div class="theme-grid">
                  <div 
                    v-for="t in themes" :key="t" 
                    class="theme-item" 
                    :style="{ background: t }"
                    :class="{ active: settings.themeColor === t }"
                    @click="settings.themeColor = t; configStore.updateConfig(settings)"
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* 此处保留之前的 CSS 样式即可 */
.settings-layout { display: flex; width: 100%; height: 100%; background: #131314; color: #e3e3e3; overflow: hidden; }
.nav-sidebar { width: 180px; background: #131314; padding: 12px 8px; border-right: 1px solid rgba(255, 255, 255, 0.03); }
.nav-item { display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-radius: 10px; cursor: pointer; margin-bottom: 2px; color: #999; font-size: 14px; }
.nav-item.active { background: #2b2d31; color: #fff; }
.nav-divider { height: 1px; background: rgba(255, 255, 255, 0.05); margin: 12px 8px; }
.provider-sidebar { width: 260px; background: #18191b; border-right: 1px solid rgba(255, 255, 255, 0.03); display: flex; flex-direction: column; }
.sidebar-header { padding: 16px; }
.search-box { background: #131314; border-radius: 8px; padding: 8px 12px; display: flex; align-items: center; gap: 8px; }
.search-box input { background: transparent; border: none; color: #fff; outline: none; flex: 1; font-size: 13px; }
.list-item { display: flex; align-items: center; justify-content: space-between; padding: 12px; border-radius: 12px; cursor: pointer; margin-bottom: 4px; }
.list-item.active { background: #2b2d31; }
.item-left { display: flex; align-items: center; gap: 12px; }
.item-right { display: flex; align-items: center; gap: 8px; }
.p-name { font-size: 14px; font-weight: 500; }
.status-tag.on { background: rgba(46, 204, 113, 0.15); color: #2ecc71; font-size: 10px; padding: 2px 6px; border-radius: 4px; }
.status-tag.off { background: rgba(255, 255, 255, 0.05); color: #888; font-size: 10px; padding: 2px 6px; border-radius: 4px; }
.toggle-btn { width: 36px; height: 20px; background: #555; border-radius: 10px; border: none; cursor: pointer; position: relative; transition: background 0.3s; }
.toggle-btn.on { background: #2ecc71; }
.toggle-slider { width: 16px; height: 16px; background: #fff; border-radius: 50%; position: absolute; top: 2px; left: 2px; transition: left 0.3s; }
.toggle-btn.on .toggle-slider { left: 18px; }
.detail-panel { flex: 1; background: #1e1f22; overflow-y: auto; }
.detail-container { max-width: 680px; margin: 0 auto; padding: 40px 24px; }
.detail-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 40px; }
.settings-card { background: rgba(255, 255, 255, 0.03); border-radius: 12px; padding: 20px; border: 1px solid rgba(255, 255, 255, 0.05); }
.control-item { margin-bottom: 24px; }
.control-item label { display: block; font-size: 13px; color: #b5bac1; margin-bottom: 12px; }
.input-box { background: #131314; border: 1px solid rgba(255, 255, 255, 0.05); border-radius: 8px; padding: 12px 16px; display: flex; align-items: center; gap: 8px; }
.input-box input { background: transparent; border: none; color: #fff; outline: none; flex: 1; font-family: monospace; }
.eye-btn { background: transparent; border: none; color: #888; cursor: pointer; padding: 4px; border-radius: 4px; display: flex; align-items: center; justify-content: center; transition: color 0.2s; }
.eye-btn:hover { color: #fff; }
.theme-grid { display: flex; gap: 12px; }
.theme-item { width: 36px; height: 36px; border-radius: 10px; cursor: pointer; border: 2px solid transparent; }
.theme-item.active { border-color: #5865f2; transform: scale(1.1); }
input[type="range"] { width: 100%; height: 6px; background: #131314; border-radius: 10px; appearance: none; outline: none; }
input[type="range"]::-webkit-slider-thumb { appearance: none; width: 18px; height: 18px; background: #fff; border-radius: 50%; cursor: pointer; }
</style>