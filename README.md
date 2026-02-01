# Goge Chat 🚀

> **[简体中文](#-简体中文)** | **[English](#-english)**

---

## 🏮 简体中文

**Goge Chat** 是一款基于 **Tauri 2.0**、**Vue 3** 和 **Rust** 构建的现代化、高性能 AI 聊天客户端。它旨在提供极致的响应速度与“奢华级”的视觉体验，同时确保您的数据隐私始终处于本地管控之下。

### ✨ 核心特性

*   **💎 Inky Glass 视觉规范**：独创的高饱和度毛玻璃特效，模拟奢侈品级的物理通透感。
*   **🎭 双态切换**：拥有“标准模式”与“社交模式（WeChat Style）”，兼顾沉浸式生产力与轻量化对话。
*   **🌈 双槽主题引擎**：支持为深色/浅色模式独立配置子主题，从 *Nord*、*One Dark* 到 *Sakura*、*Cyberpunk*，一键切换。
*   **📦 零云存储，本地优先**：聊天历史与配置参数通过 Rust 后端加密存储在本地，支持便携式使用。
*   **⚙️ 深度调优**：内置提示词库、预设管理、模型切换，提供超越浏览器端的使用体验。

### 🛠️ 技术架构

-   **Frontend**: Vue 3 (Composition API) + Pinia + Vite
-   **Backend**: Rust + Tauri 2.0 (Mobile Ready)
-   **Storage**: Shared SQLite 核心 + Social Social DB
-   **Styling**: Vanilla CSS (CSS Variables Driven)

### 🚀 快速开始

1.  **安装依赖**
    ```bash
    npm install
    ```
2.  **启动开发环境**
    ```bash
    npm run tauri dev
    ```
3.  **构建生产版本**
    ```bash
    npm run tauri build
    ```

---

## 🌐 English

**Goge Chat** is a premium, high-performance AI chat client built with **Tauri 2.0**, **Vue 3**, and **Rust**. It combines lightning-fast responsiveness with a "Hyper-Premium" glassmorphism UI, ensuring your data remains private and local.

### ✨ Key Features

*   **💎 Inky Glass Visuals**: Bespoke high-saturation blur effects and light refraction, delivering a premium "glass" tactile feel.
*   **🎭 Dual-Mode Layout**: Toggle between "Standard Mode" (Classic) and "Social Mode" (IM-style), optimized for both focus and casual chat.
*   **🌈 Dual-Slot Theme Engine**: Configure independent themes for Dark and Light modes (e.g., *Cyberpunk* for night, *Sakura* for day).
*   **📦 Local-First Persistence**: Powered by Rust & SQLite, all your data is stored offline. No cloud, no tracking.
*   **⚙️ Power-User Tools**: Built-in prompt library, system instruction presets, and instant model switching.

### 🏗️ Technical Stack

-   **UI Layer**: Vue 3 / Vite / Pinia
-   **Core Engine**: Rust (Tauri 2.0)
-   **Database**: Dual SQLite architecture (Core + Social)
-   **Aesthetics**: 100% Theme-variable driven CSS

### 🏎️ Development

1.  **Dependencies**
    ```bash
    npm install
    ```
2.  **Dev Mode**
    ```bash
    npm run tauri dev
    ```
3.  **Build Production**
    ```bash
    npm run tauri build
    ```

---

## 📂 项目结构 | Structure

```text
├── src/               # Vue Frontend
│   ├── assets/        # Styles (Manifesto), icons, prompts
│   ├── components/    # Reusable UI islands
│   ├── layouts/       # Mode-specific layouts (Standard/Main)
│   └── stores/        # Pinia state management
├── src-tauri/         # Rust Backend
│   └── src/           # Commands, DB logic, app entry
└── data/              # Local storage (Generated at runtime)
```

---

## 📄 协议 | License

本项目基于 [MIT](LICENSE) 协议开源。
Licensed under the [MIT](LICENSE) License.
