# Platinum — Mercury RSS 阅读器跨平台复刻

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-blue)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-61dafb)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.x-3178c6)](https://www.typescriptlang.org/)

基于 **Rust + Tauri 2 + React** 的跨平台 RSS 阅读器，复刻 iOS/macOS 应用 Mercury 的核心体验。

> **本项目不是设计新的 RSS 阅读器。本项目是 Mercury 的跨平台复刻。**
>
> 目标：在 Windows 10+ / macOS 12+ / Linux 上尽可能复现 Mercury 的功能与体验。

---

## 特性

- **本地优先** — 无需注册/登录，数据存本地 SQLite
- **Reader 管线** — Mozilla Readability 内容提取 + HTML 清洗 + GFM Markdown 渲染
- **AI 摘要** — OpenAI 兼容协议，流式输出，语言与详细程度可配置
- **AI 双语翻译** — 段落级双语对照，多段落并发
- **笔记与导出** — Markdown/HTML 格式导出，单篇/多篇文摘
- **标签系统** — AI 自动推荐标签 + NLP 回退 + 标签库管理
- **多 Provider** — 多模型配置，API Key 加密存储
- **i18n** — 中/英文界面切换

## 技术栈

| 层 | 技术 |
|---|---|
| 语言 | Rust 1.80+ |
| 框架 | Tauri 2 |
| 前端 | React + TypeScript + Vite + Tailwind CSS + shadcn/ui |
| 数据库 | SQLite（rusqlite, WAL 模式） |
| Feed 解析 | feed-rs |
| 内容提取 | readability crate |
| Markdown | comrak（GFM） |
| AI 协议 | OpenAI 兼容（SSE 流式） |

## 快速开始

### 环境要求

- Rust 1.80+（stable）
- Node.js 20 LTS
- Tauri CLI 2.x：`cargo install tauri-cli --version "^2"`

### 开发运行

```bash
git clone https://github.com/wkr-echo/womenhenku.git
cd womenhenku
cargo tauri dev
```

### 构建

```bash
cargo tauri build
```

## 项目结构

```
womenhenku/
├── src-tauri/          # Rust 核心 + Tauri 后端
│   ├── src/
│   │   ├── db/         # 数据库层（模型、迁移、Repository）
│   │   ├── feed/       # Feed 解析与同步
│   │   ├── reader/     # 阅读器管线（提取→清洗→Markdown→渲染）
│   │   ├── agent/      # AI Agent（摘要、翻译、Provider 管理）
│   │   └── platform/   # 平台抽象层
│   └── resources/      # Prompt 模板
├── src-ui/             # React 前端
│   └── src/
│       ├── api/        # Tauri invoke 封装
│       ├── components/ # UI 组件
│       └── locales/    # i18n 翻译文件
└── docs/               # 项目文档
    ├── PLAN.md         # 执行计划
    ├── adr/            # 架构决策记录
    └── reference/      # 参考文档
```

## 文档

- [AGENTS.md](AGENTS.md) — AI 编码助手的完整操作手册
- [PLAN.md](docs/PLAN.md) — 项目执行计划
- [SESSION_CONTEXT.md](docs/SESSION_CONTEXT.md) — 会话上下文速查
- [ai-assisted-dev-retrospective.md](docs/ai-assisted-dev-retrospective.md) — AI 辅助开发心得体会

## License

MIT
