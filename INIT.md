# Mercury 跨平台复刻

## Features

- 跨平台 RSS 阅读器，支持 Windows、macOS、Linux
- 本地优先，无需账号/登录，数据存本地 SQLite
- RSS / Atom / JSON Feed 订阅与 OPML 导入导出
- 文章阅读模式，文章内容清洗 + Markdown 渲染
- AI 智能体集成，实现 OpenAI 兼容协议，支持任意云端/本地大模型
  - 文章摘要（流式输出，可配置语言与详细程度）
  - 双语翻译（段落对照，可重试失败段落）
  - AI 标签建议
- 手动标签与按标签筛选
- 多订阅源管理与同步
- 主题切换（亮色/暗色/自定义字体）

## Developing

### 已确定

- **语言**：Rust
- **UI 框架**：Tauri（与 Rust 同语言，单进程，零 IPC）
- **数据库**：SQLite（`rusqlite`），schema 参考原始 Mercury 设计
- **HTTP 客户端**：`reqwest`
- **Feed 解析**：`feed-rs`
- **HTML 清洗**：`scraper`（CSS 选择器友好，AI 辅助开发效率高）
- **Markdown 渲染**：`comrak`（GFM 完整支持，表格/任务列表/链接处理成熟）
- **AI 客户端**：实现 OpenAI 兼容协议（`/v1/chat/completions`，支持流式），覆盖 OpenAI / DeepSeek / Ollama / vLLM 等
- **前端**：React（TypeScript）

### 跨平台兼容性要求

- Windows 10+（WebView2 运行时，Win10 已内置）
- macOS 12+
- Linux（Wayland + X11，WebViewGTK 依赖需额外处理）
- 安装包格式：Windows `.msi`，macOS `.dmg`，Linux `.AppImage` / `.deb`
- 各平台路径处理用 Rust `dirs` crate，不走硬编码

### 多 AI 提供商兼容

实现 OpenAI 兼容协议，用户配置 baseURL + API Key + model name 即可接入任意兼容服务。v1 不接入非 OpenAI 协议的原生 API（如 Anthropic Messages API）。

### v1 范围

必须实现：

- Feed 订阅（增删改刷新）+ OPML 导入导出
- 文章列表（已读/未读、搜索）+ 阅读模式
- AI 摘要 + 双语翻译
- 手动标签 + AI 标签建议
- LLM 提供商/模型配置
- 跨平台打包（Windows + macOS + Linux）

推迟到 v2+：

- 批量打标签与标签库维护
- Markdown 笔记与文摘导出
- Token 用量统计
- 自定义 Prompt 模板
- 多语言 UI 切换


