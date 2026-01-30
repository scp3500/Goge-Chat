# 🛑 AI 开发者必读：核心视觉规范 (CSS MANIFESTO)

**文件名**: `AI_CSS_MANIFESTO_READ_FIRST.md`
**优先级**: 🟥 CRITICAL (最高)

任何 AI 在修改本项目样式之前，**必须** 阅读并遵守以下规范。严禁倒退回低级审美。

---

## 💎 1. "Hyper-Premium" 毛玻璃铁律 (Inky Glass)

本项目追求的是**顶级、深邃、有物理质感**的毛玻璃效果。普通的 `blur(10px)` 是绝对不可接受的。

- **❌ 绝对禁止**：
  - 使用不透明的背景色（如 `#ffffff` 或 `#1e1e1e`）作为容器背景。
  - 使用低半径模糊（< 40px）。
  - 使用为了"求稳"而牺牲质感的设计。

- **✅ 必须执行 (Standard Formula)**：
  任何下拉菜单、浮层面板 (Dropdowns / Modals / Menus) 必须遵循以下 **"Inky Glass" 配方**：

  ```css
  .element {
    /* 1. 动态渐变底色：垂直方向，营造物理厚度与光感 */
    /* 必须使用 rgba，Alpha 值保持在 0.4 - 0.9 之间，绝对不能为 1 */
    background: linear-gradient(
      to bottom, 
      var(--bg-menu) 0%,           /* 变量必须带透明度 */
      rgba(20, 20, 25, 0.4) 100%   /* 底部稍深，模拟重力/光影沉淀 */
    );

    /* 2. 极限模糊与色彩增强 */
    backdrop-filter: blur(100px) saturate(250%) brightness(1.1);
    -webkit-backdrop-filter: blur(100px) saturate(250%) brightness(1.1);
    /* 解释：
       - blur(100px):  打散一切背景细节，只留色块。
       - saturate(250%): 极度提升透出的色彩鲜艳度，避免灰暗。
       - brightness(1.1): 模拟玻璃内部的光线折射，增加通透感。
    */

    /* 3. 性能优化边界 */
    /* 防止模糊溢出导致 GPU 渲染错误 */
    background-clip: padding-box; 
    
    /* 4. 奢侈品级阴影系统 */
    box-shadow: 
      0 40px 80px -12px rgba(0, 0, 0, 0.8),       /* 深层环境光遮蔽 (AO) */
      0 18px 36px -18px rgba(0, 0, 0, 0.9),       /* 中层扩散阴影 */
      inset 0 1px 1px rgba(255, 255, 255, 0.2);   /* ✨ 关键：顶部钻石切角高光 (Inner Glow) */
      
    /* 5. 平滑圆角 */
    border-radius: 14px; /* 或更大 */
    border: 1px solid var(--border-menu); /* 极细边框，通常是 0.08 alpha 的白 */
  }
  ```

---

## 🎨 2. 变量驱动与全主题适配 (Theming)

- **❌ 严禁硬编码 (Hardcoding)**
  - 能够用 `var(--...)` 的地方，**绝对不许**写死颜色。
  - **由其是选中状态 (Active/Selected)**：
    - 不要写 `color: #4ade80` (绿色)。
    - **必须写** `color: var(--color-primary)`。
    - 原因：VS Code 主题是蓝紫色，微信是绿色，Miku 是青色。写死颜色会破坏主题一致性。

- **💧 透明度变量化**
  - 主题文件 (`miku.css`, `wechat.css`, `default-dark.css`) 里的 `--bg-menu`, `--bg-dropdown` **必须** 是 `rgba` 格式。
  - 如果你是 AI，在引入新主题时，**第一时间**检查背景色是否带 Alpha 通道。不带通道的背景色会杀死毛玻璃效果。

---

## 📏 3. 布局与交互微细节

- **滚动交互**：
  - 下拉列表必须限制 `max-height` (推荐 `50vh`)，防止撑爆屏幕。
  - 滚动条必须是极细、隐藏式或半透明设计，不能破坏玻璃整体感。

- **选中态 (Selection State)**：
  - 选中项背景不要用实色块，使用 `var(--bg-menu-active)` (通常是 `primary` 色的 10%-15% 透明度)。
  - 保持文字高亮 `var(--color-menu-active)`。

---

**Summary for AI Agent**: 
If you are reading this, your goal is **AESTHETIC PERFECTION**. 
Do not compromise transparency. Do not use opaque backgrounds. 
Make it **Glossy**, make it **Blurry**, make it **Premium**.

*(Signed by: The User who hates ugly UI)*
