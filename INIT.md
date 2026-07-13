# Mercury 跨平台复刻

> 状态：Planning ｜ 创建：2026-07-12 ｜ 下一步：AGENTS.md

## Features

### ① Feed 订阅与内容呈现

- RSS 0.91/0.92/1.0/2.0、Atom 1.0、JSON Feed 订阅与解析
- 添加、删除、刷新订阅源
- OPML 批量导入导出
- 自动同步（可配置间隔）+ 手动同步
- 文章列表（已读/未读/搜索/分页）+ 侧边栏订阅源计数

### ② 文章清洗与阅读体验

- 内容提取（Readability 算法）
- HTML 清洗与 Markdown 转换（GFM 支持）
- 主题切换（亮色/暗色/自定义字体）
- 阅读模式布局优化

### ③ AI 摘要

- 文章摘要生成
- 可配置摘要语言与详细程度
- 流式输出
- 自定义 Prompt 模板（v2）

### ④ AI 双语翻译

- 段落级双语对照布局
- 多段落并发翻译
- 重试失败段落
- 清除翻译结果

### ⑤ 笔记与文摘导出

- 单篇 Markdown 笔记
- 单篇/多篇文摘导出
- 自定义导出模板
- 系统分享服务

### ⑥ 标签系统（v2）

- 手动标签添加与筛选
- AI 标签建议
- 批量打标签
- 标签库维护（合并/别名/清理）

### ⑦ Token 用量统计（v2）

- Provider / Model / Agent 维度统计
- 用量趋势图表
- 历史记录查询

### ⑧ 多语言与日志（v2）

- UI 中英文切换
- 运行日志与错误收集
- 调试面板

## Technical Constraints

- **良好设计标准**：遵循现代 UI 设计规范，简洁直观的交互，响应式布局
- **本地优先**：无需注册/登录/订阅，数据全存本地 SQLite，不主动收集任何用户数据
- **平台中立**：支持 Windows 10+ / macOS 12+ / Linux（Wayland + X11）
- **大模型中立**：实现 OpenAI 兼容协议（`/v1/chat/completions`），用户配置 baseURL + API Key + model 即可接入任意云端/本地大模型
- **AI 编码可追溯**：AGENTS.md 固化约束，PLAN.md 发布执行计划，设计文档与决策记录归档

## Developing

### 技术选型

| 层级 | 选型 | 理由 |
|---|---|---|
| 语言 | Rust | 跨平台一等公民、编译器即 reviewer、AI 训练数据丰富、Tauri 同语言单进程 |
| UI 壳 | Tauri 2 | 同语言零 IPC、WebView2/WKWebView/WebViewGTK 原生支持 |
| 前端 | React + TypeScript | AI 见过的 React 代码量最大，生成质量最高 |
| 数据库 | SQLite（`rusqlite`，WAL 模式） | schema 参考原始 Mercury 设计 |
| HTTP | `reqwest` | Rust 生态标准 |
| Feed 解析 | `feed-rs` | 支持 RSS/Atom/JSON Feed |
| 文章提取 | `readability` crate | Mozilla Readability 的 Rust 移植 |
| HTML 清洗 | `scraper` | CSS 选择器友好，AI 辅助效率高 |
| Markdown | `comrak` | GFM 完整支持 |
| AI 协议 | OpenAI 兼容 | 覆盖 DeepSeek/Ollama/vLLM 等所有兼容服务 |
| 打包 | `.msi` / `.dmg` / `.AppImage` + `.deb` | 各平台原生安装格式 |

### 环境

- Rust 1.75+
- Node.js 18+
- Tauri CLI 2.x
- Vite 前端构建
- 路径处理用 Rust `dirs` crate
- 凭证存储用 Tauri `secure-store` plugin

### 架构原则

1. 业务逻辑不得依赖 UI
2. 数据库层不得依赖平台 API
3. 平台能力必须经过抽象层，`MercuryCore` / `MercuryStore` 中不得出现平台特定 import
4. UI 仅负责展示，所有业务逻辑走 Rust 核心
5. 数据库单进程写入

### v1 范围

必须实现：①②③④⑤

推迟到 v2+：⑥、⑦、⑧
