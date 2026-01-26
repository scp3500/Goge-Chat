<script setup>
defineProps({
  providers: Array,
  activeProviderId: String
});
defineEmits(['update:activeProviderId', 'toggleStatus']);
</script>

<template>
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
        :class="{ active: activeProviderId === provider.id }"
        @click="$emit('update:activeProviderId', provider.id)"
      >
        <div class="item-left">
          <span class="p-icon">{{ provider.icon }}</span>
          <span class="p-name">{{ provider.name }}</span>
        </div>
        <div class="item-right">
          <button
            class="toggle-btn"
            :class="{ on: provider.status === 'on' }"
            @click.stop="$emit('toggleStatus', provider.id)"
          >
            <span class="toggle-slider"></span>
          </button>
          <span class="status-tag" :class="provider.status">{{ provider.status === 'on' ? 'ON' : 'OFF' }}</span>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
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
</style>
