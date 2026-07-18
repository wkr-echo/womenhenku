# v1.0.0 — Mercury 跨平台复刻

这是 Mercury RSS 阅读器的跨平台复刻版本，支持 Windows 10+、macOS 12+、Linux（Wayland + X11）。

## 📦 核心功能

### Feed 订阅与管理

- RSS 0.91/0.92/1.0/2.0、Atom 1.0、JSON Feed 全格式支持
- 添加/删除/刷新订阅源，支持自动同步（可配置间隔）
- OPML 导入导出，方便迁移
- 文章列表（已读/未读标记、分页浏览）

### 阅读体验

- Readability 内容提取（自动去除广告、导航等干扰）
- HTML 清洗与 Markdown 转换（GFM 完整支持）
- 亮色/暗色主题切换
- 字体定制（系统字体列表）
- 全文搜索（FTS5）

### AI 功能

- AI 摘要（流式输出，可配置语言和详细程度）
- AI 双语翻译（段落级对照布局，多段落并发翻译）
- 支持 OpenAI 兼容协议（DeepSeek/Ollama/vLLM 等）
- 多 Provider 配置（baseURL/API Key/model）

### 笔记与导出

- Markdown 笔记编辑
- 文摘导出（原文/摘要/笔记，支持 Markdown/HTML 格式）
- 单篇/多篇批量导出

## 🛠️ 技术栈

- **后端**: Rust 1.80 + Tauri 2 + SQLite（rusqlite）
- **前端**: React + TypeScript + Vite + Tailwind CSS + shadcn/ui
- **数据库**: SQLite（WAL 模式，本地优先）
- **打包**: .msi（Windows）/ .dmg（macOS）/ .deb（Linux）

## 🔒 安全特性

- API Key 使用平台原生安全存储（Keychain / Credential Manager / libsecret）
- 本地优先，数据存储在本地 SQLite，不主动收集用户数据

## 📁 安装包下载

| 平台 | 文件 |
|---|---|
| Windows | `Platinum_1.0.0_x64.msi` |
| macOS | `Platinum-1.0.0.dmg` |
| Linux | `Platinum_1.0.0_amd64.deb` |

## 🚀 使用说明

1. 安装并启动应用
2. 点击左侧侧边栏「+」添加 RSS 订阅源
3. 等待订阅源刷新，点击文章阅读
4. 使用右上角工具栏导出原文/摘要/笔记

## 📝 更新日志

### Stage 1: 基础阅读器原型

- 实现 Feed 解析与存储
- 文章列表与阅读器视图
- 已读/未读标记

### Stage 2: 阅读体验增强

- Readability 内容提取管线
- HTML 清洗与 Markdown 转换
- 主题切换与字体定制
- 全文搜索

### Stage 3: AI 功能接入

- OpenAI 兼容协议客户端
- AI 摘要 Agent（流式输出）
- AI 双语翻译 Agent（段落级并发）
- Provider 管理

### Stage 4: 信息整理与导出

- Markdown 笔记编辑
- 文摘导出功能
- 三平台打包配置
