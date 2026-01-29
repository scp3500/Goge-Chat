<script setup>
defineProps({
  activeCategory: String
});
defineEmits(['update:activeCategory']);
</script>

<template>
  <aside class="nav-sidebar">
    <div class="nav-group">
      <div class="nav-item" :class="{ active: activeCategory === 'models' }" @click="$emit('update:activeCategory', 'models')">
        <span class="icon">☁️</span>
        <span class="label">模型服务</span>
      </div>
      <div class="nav-item" :class="{ active: activeCategory === 'presets' }" @click="$emit('update:activeCategory', 'presets')">
        <span class="icon">⚙️</span>
        <span class="label">模型配置</span>
      </div>
      <div class="nav-item" :class="{ active: activeCategory === 'appearance' }" @click="$emit('update:activeCategory', 'appearance')">

        <span class="icon">🎨</span>
        <span class="label">显示设置</span>
      </div>
    </div>
    
    <div class="nav-divider"></div>

    <div class="nav-group">
      <div class="nav-item" :class="{ active: activeCategory === 'general' }" @click="$emit('update:activeCategory', 'general')">
        <span class="icon">⚙️</span>
        <span class="label">通用设置</span>
      </div>
      <div class="nav-item" :class="{ active: activeCategory === 'data' }" @click="$emit('update:activeCategory', 'data')">
        <span class="icon">💾</span>
        <span class="label">数据管理</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.nav-sidebar { width: 180px; background: #131314; padding: 12px 8px; border-right: 1px solid rgba(255, 255, 255, 0.03); }
.nav-item { display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-radius: 10px; cursor: pointer; margin-bottom: 2px; color: #999; font-size: 14px; }
.nav-item.active { background: #2b2d31; color: #fff; }
.nav-divider { height: 1px; background: rgba(255, 255, 255, 0.05); margin: 12px 8px; }
</style>
