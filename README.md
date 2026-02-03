# Goge Chat 🚀

> **[简体中文](#-简体中文)** | **[English](#-english)**

---

## 简体中文

**Goge Chat：身边的本地化 AI 智能伙伴**

Goge Chat 是一款以 **“本地优先”** 为核心理念的 AI 聊天客户端。在云端 AI 遍地的时代，我们更关注你的数据主权与私密性，通过优秀的本地架构，将高性能对话与持久化记忆带到你的桌面。

### 核心特性

#### 核心优势：全本地化与隐私安全
*   **数据主权**：基于 **Rust + Tauri 2.0** 构建，你的所有聊天记录、提炼的记忆以及配置参数均 100% 存储于本地设备。
*   **无云依赖**：不依赖任何第三方云端存储，确保你的隐私数据不会被上传或用于训练，实现真正的私密对话。
*   **本地调优**：内置向量模型，所有的记忆检索与语义处理均在本地飞速运行，响应即时且稳定。

#### 专属长期记忆，提升对话连贯性
*   **静默提炼**：系统利用 **LanceDB** 向量数据库，在后台自动提炼对话中的关键信息（如偏好、习惯等）。
*   **语义复用**：开启新会话时，AI 能自动唤回相关的历史背景，无需重复介绍，让交流如同老友重逢般自然。

#### 均衡的视觉体验
*   **设计语言**：采用精心调优的玻璃拟态（Glassmorphism）美学设计，配合细腻的流体动画，营造沉浸式且稳重的交互手感。
*   **独立主题**：支持为明亮/深色模式独立配置子主题，完美契合不同光线环境下的视觉需求。

#### 灵活的工作模式
*   **标准模式**：简洁直观，适合专注的任务处理与长文对话。
*   **社交模式**：参考常用通讯软件的逻辑，提供联系人分组管理，让 AI 沟通如同日常社交般轻松。

### ️ 技术栈构成

-   **Backend**: Rust (基于 memory_processor 的局部向量引擎, social_db 关系型管理)
-   **Frontend**: Vue 3 + Pinia (通过 ConfigStore 实现多维度主题控制)
-   **Core Storage**: Tauri 2.0 + SQLite + LanceDB

### 快速开始

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

## English

**Goge Chat: A Local-First AI Companion Built for Privacy**

Goge Chat is an AI desktop client rooted in the principle of **"Local-First"**. In an era of cloud-dominant AI, we prioritize your data sovereignty and privacy, bringing high-performance interaction and persistent memory directly to your local machine.

### Key Features

*   **Core Advantage: Local-First & Privacy**: Built with **Rust & Tauri 2.0**, all your data—chats, memories, and settings—stays 100% on your device. No cloud storage, no tracking, total control.
*   **Persistent Local Memory**: Leverages **LanceDB** to silently distill and retrieve context from past interactions locally. AI that understands you better over time without ever compromising your data.
*   **Refined Visual Aesthetics**: A professional implementation of glassmorphism with smooth animations, offering an elegant and immersive user experience.
*   **Dual-Slot Theme Engine**: Truly independent theme configurations for Light and Dark modes to suit any environment.
*   **Flexible Layouts**: Toggle between a productivity-focused **Standard Mode** and a familiar, IM-style **Social Mode** for managing personal AI contacts.

### Technical Stack

-   **Frontend**: Vue 3 / Vite / Pinia
-   **Backend**: Rust (Tauri 2.0)
-   **Storage**: Hybrid system—SQLite for relational data and LanceDB for vector-based local memory.

---

## 项目结构 | Structure

```text
├── src/               # UI Layer & Layouts
│   ├── assets/        # Styles, Icons, Prompt Assets
│   └── stores/        # Configuration & Theme Engine
├── src-tauri/         # Core Logic (Rust)
│   ├── src/memory/    # Local memory processing
│   └── src/social_db.rs # Social data management
└── data/              # Local data storage
```

---

## 协议 | License

本项目基于 [MIT](LICENSE) 协议开源。
Licensed under the [MIT](LICENSE) License.
