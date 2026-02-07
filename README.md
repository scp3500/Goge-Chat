# Goge Chat

> **[简体中文](#-简体中文)** | **[English](#-english)**

---

## 简体中文

**Goge Chat：你的沉浸式本地 AI 社交伙伴**

Goge Chat 不仅仅是一个 AI 聊天客户端，它是一个致力于 **“打破第四面墙”** 的智能伴侣。我们坚持 **“本地优先 (Local-First)”** 理念，利用 Rust 的高性能与 Tauri 的轻量化，为你打造一个既私密又充满“人情味”的桌面伙伴。

它不再是冷冰冰的问答机器，而是具备**模拟行为**、**长期记忆**与**主动交互**能力的数字好友。

### 核心特性

#### 沉浸式行为引擎 (Behavior Engine)
*   **拟人化交互**：拒绝机械式的秒回。Goge Chat 内置 Rust 驱动的行为引擎，能根据回复长度动态模拟真实的“思考”与“打字”延迟，带来如同真人般的即时通讯体验。
*   **主动社交 (Immersive Mode)**：在沉浸模式下，AI 能够感知上下文的静默，在适当时机主动发起话题，打破冷场，让交流不再单向。

#### 绝对的数据主权
*   **全本地存储**：基于 **LanceDB** (向量记忆) + **SQLite** (关系数据)，你的所有聊天记录、记忆片段、偏好设置均 100% 存储于本地。
*   **隐私无忧**：我们不上传任何数据。你的数字生活完全属于你，彻底隔离云端窥探。

#### 社交化体验 (Social Mode)
*   **多重人格管理**：像管理微信好友一样管理你的 AI 角色。支持为不同 AI 设定独立的头像、性格与记忆库。
*   **无缝切换**：在“标准办公模式”与“社交闲聊模式”间自由切换，满足从高效生产力 to 情感陪伴的全场景需求。
*   **自动记忆提炼**：系统会在后台静默提炼对话中的关键信息，构建专属的长期记忆库。下次聊天时，它依然记得你的喜好。

#### 极致的视觉美学
*   **玻璃拟态设计**：精心调优的 Glassmorphism UI，配合丝滑的流体动画，视觉体验通透且现代。
*   **双模独立主题**：支持为明亮/深色模式分别配置独立的主题色与背景，完美契合昼夜变化。

#### 极客级工具箱
*   **智能维护**：内置数据库诊断与压缩工具（Compact & Vaccum），确保本地记忆库在大规模数据下依然飞快。
*   **模型自动发现**：支持自动扫描并适配兼容的 AI 模型服务（如 SiliconFlow 等）。
*   **高性能架构**：底层采用 Rust 编写，针对 TTFT (Time To First Token) 进行了极致优化，响应速度极快。

### 技术栈

-   **Frontend**: Vue 3 + Pinia + TypeScript (Tailwind CSS / Custom Glass UI)
-   **Backend**: Rust (Tauri 2.0)
-   **Core**: 
    -   `behavior_engine`: 拟人化行为调度
    -   `memory_processor`: 向量记忆与 RAG 检索
-   **Storage**: LanceDB (Vector) + SQLite (Relational)

### 快速开始

1.  **安装依赖**
    ```bash
    npm install
    ```
2.  **启动开发环境 (Rust + Vue)**
    ```bash
    npm run tauri dev
    ```
3.  **构建生产版本**
    ```bash
    npm run tauri build
    ```

---

## English

**Goge Chat: Your Immersive, Local-First AI Companion**

Goge Chat is more than just an AI client—it's an intelligent companion designed to **"break the fourth wall."** Built on the **"Local-First"** philosophy using Rust and Tauri, it delivers a private, high-performance, and deeply "human" experience right on your desktop.

It's not just a Q&A bot; it's a digital friend with **simulated behaviors**, **long-term memory**, and **proactive interaction**.

### Key Features

#### Immersive Behavior Engine
*   **Human-Like Interaction**: Say goodbye to robotic, instant replies. Goge Chat's Rust-based behavior engine dynamically simulates real "thinking" and "typing" delays based on response complexity, creating a natural feel.
*   **Proactive Engagement (Immersive Mode)**: When enabled, the AI can sense silence in the ongoing context and proactively initiate conversations, making interactions feel alive and two-way.

#### Absolute Data Sovereignty
*   **100% Local Storage**: Powered by **LanceDB** (Vector Memory) and **SQLite** (Relational Data), all your chats, memories, and settings act locally.
*   **Privacy First**: No cloud dependencies. Your digital life stays on your device, completely isolated from external tracking.

#### Social Experience
*   **Persona Management**: Manage AI characters like contacts in a messaging app. Customize avatars, personalities, and memory banks for each AI.
*   **Dual Modes**: Seamlessly switch between "Standard Mode" for productivity and "Social Mode" for casual, emotional connection.
*   **Silent Memory Distillation**: The system quietly extracts key details from conversations in the background, building a persistent memory bank. It remembers your preferences without you having to repeat them.

#### Refined Aesthetics
*   **Glassmorphism UI**: A polished, modern interface featuring glassmorphism and fluid animations.
*   **Independent Theming**: Configure distinct themes and backgrounds for Light and Dark modes independently.

#### Power User Tools
*   **Smart Maintenance**: Built-in database diagnostics and compaction tools ensure your local memory bank remains lightning-fast efficiently.
*   **Model Discovery**: Automatically scans and adapts to compatible AI model services (e.g., SiliconFlow).
*   **High Performance**: The Rust backend is optimized for low latency and minimal TTFT (Time To First Token).

### Tech Stack

-   **Frontend**: Vue 3 + Pinia + TypeScript
-   **Backend**: Rust (Tauri 2.0)
-   **Core Modules**: 
    -   `behavior_engine`: Simulates human-like typing and thinking.
    -   `memory_processor`: Handles vector memory and RAG retrieval.
-   **Storage**: LanceDB + SQLite

### Quick Start

1.  **Install Dependencies**
    ```bash
    npm install
    ```
2.  **Run Development Environment**
    ```bash
    npm run tauri dev
    ```
3.  **Build for Production**
    ```bash
    npm run tauri build
    ```

---

## 协议 | License

Licensed under the [MIT](LICENSE) License.
