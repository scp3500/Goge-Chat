<script setup>
import { 
  NAV_MODELS_SVG, 
  NAV_PRESETS_SVG, 
  NAV_PROMPTS_SVG, 
  NAV_APPEARANCE_SVG, 
  NAV_GENERAL_SVG, 
  NAV_DATA_SVG 
} from '../../constants/icons';

defineProps({
  activeCategory: String
});
defineEmits(['update:activeCategory']);
</script>

<template>
  <aside class="nav-sidebar">
    <div class="nav-group">
      <div class="nav-item" :class="{ active: activeCategory === 'models' }" @click="$emit('update:activeCategory', 'models')">
        <span class="icon" v-html="NAV_MODELS_SVG"></span>
        <span class="label">模型服务</span>
      </div>
      <div class="nav-item" :class="{ active: activeCategory === 'presets' }" @click="$emit('update:activeCategory', 'presets')">
        <span class="icon" v-html="NAV_PRESETS_SVG"></span>
        <span class="label">模型配置</span>
      </div>
      <div class="nav-item" :class="{ active: activeCategory === 'prompts' }" @click="$emit('update:activeCategory', 'prompts')">
        <span class="icon" v-html="NAV_PROMPTS_SVG"></span>
        <span class="label">系统提示词</span>
      </div>
      <div class="nav-item" :class="{ active: activeCategory === 'appearance' }" @click="$emit('update:activeCategory', 'appearance')">
        <span class="icon" v-html="NAV_APPEARANCE_SVG"></span>
        <span class="label">显示设置</span>
      </div>
      <div class="nav-item" :class="{ active: activeCategory === 'chatmode' }" @click="$emit('update:activeCategory', 'chatmode')">
        <span class="icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
        </span>
        <span class="label">聊天模式</span>
      </div>
    </div>
    
    <div class="nav-divider"></div>

    <div class="nav-group">
      <div class="nav-item" :class="{ active: activeCategory === 'general' }" @click="$emit('update:activeCategory', 'general')">
        <span class="icon" v-html="NAV_GENERAL_SVG"></span>
        <span class="label">通用设置</span>
      </div>
      <div class="nav-item" :class="{ active: activeCategory === 'data' }" @click="$emit('update:activeCategory', 'data')">
        <span class="icon" v-html="NAV_DATA_SVG"></span>
        <span class="label">数据管理</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.nav-sidebar { width: 180px; background: var(--bg-sidebar); padding: 12px 8px; border-right: none; }
.nav-item { display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-radius: 10px; cursor: pointer; margin-bottom: 2px; color: var(--text-color); opacity: 0.7; font-size: 14px; transition: all 0.2s; }
.nav-item.active { background: var(--bg-glass-active); color: var(--color-menu-active); opacity: 1; }
.nav-item:hover:not(.active) { background: var(--bg-glass-hover); color: var(--color-sidebar-text-hover); opacity: 0.9; }
.icon { display: flex; align-items: center; justify-content: center; width: 18px; height: 18px; opacity: 0.7; }
.nav-item.active .icon { opacity: 1; }
.icon :deep(svg) { width: 100%; height: 100%; }
.nav-divider { height: 1px; background: var(--border-glass); margin: 12px 8px; }
</style>
