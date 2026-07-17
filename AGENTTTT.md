# Mercury 跨平台复刻 — Project Standards & Constraints

> AI Agent 项目入口 | 基于图片模板 + 三版 AGENTS.md 整合 | 版本 v1.1

---

## 1. Project Goal — 一句话目标

将 macOS 原生 RSS 阅读器 **Mercury** 复刻为跨平台桌面应用，在 **Windows 10+ / macOS 12+ / Linux** 上实现功能对等，保持本地优先、大模型中立、AI 编码可追溯的核心原则。

---

## 2. Architecture — 架构原则

**三层解耦**：React UI（纯展示）→ Tauri Command（参数校验）→ Rust Core（所有业务逻辑）

**核心原则**：
- 业务逻辑不得依赖 UI
- 数据库层不得依赖平台 API
- 平台能力必须经过抽象层
- 单进程架构，零 IPC 开销
- SQLite WAL 模式支持并发读

**阶段隔离**：每个阶段代码在独立目录，禁止修改上一阶段文件。每阶段打 Git tag，崩盘可 `git reset --hard` 回滚。

**Reader 管线**：Feed Entry → Raw HTML → Readability → HTML Sanitization → Markdown Conversion → Rendered HTML → Reader View（全部 Rust Core 完成）

**AI 协议引擎**：OpenAI 兼容协议 `/v1/chat/completions` + SSE 流式，支持 OpenAI / DeepSeek / Ollama / vLLM 等。

---

## 3. Tech Stack — 技术选型

| 层级 | 选型 | 说明 |
|---|---|---|
| 语言 | Rust 1.80+ stable | 跨平台一等公民 |
| UI 壳 | Tauri 2 | WebView2 / WKWebView / WebViewGTK |
| 前端 | React + TypeScript + Vite | AI 训练数据最丰富 |
| CSS | Tailwind CSS | 与 shadcn/ui 共生 |
| 组件库 | shadcn/ui | 按需复制 |
| 状态管理 | React Context + useReducer | 官方方案 |
| 数据库 | SQLite（rusqlite，WAL 模式） | 版本化迁移 |
| Feed 解析 | feed-rs | RSS/Atom/JSON Feed |
| 文章提取 | readability crate | Mozilla Readability Rust 移植 |
| HTML 清洗 | scraper | CSS 选择器友好 |
| Markdown | comrak | GFM 完整支持 |
| AI 协议 | OpenAI 兼容 | /v1/chat/completions + SSE |
| 日志 | tracing | 结构化日志 |
| 错误处理 | thiserror（库层）+ anyhow（应用层） | 分层策略 |
| 打包 | .msi / .dmg / .AppImage + .deb | 各平台原生格式 |

---

## 4. Key Features — 功能清单

### v1 必须实现
| # | 功能 | 要点 |
|---|---|---|
| ① | Feed 订阅与内容呈现 | RSS/Atom/JSON Feed 解析、OPML 导入导出、自动同步、文章列表（已读/未读/搜索/分页） |
| ② | 文章清洗与阅读体验 | Readability 提取、HTML 清洗、Markdown 转换（GFM）、主题切换、字体定制 |
| ③ | AI 摘要 | 可配置语言/详细程度、流式输出、1 秒防抖串行、latest-only 队列 |
| ④ | AI 双语翻译 | 段落级双语对照布局、并发控制（Semaphore 1~5）、重试/恢复/清除 |
| ⑤ | 笔记与文摘导出 | 单篇 Markdown 笔记、单篇/多篇文摘导出、自定义模板 |

### v2 推迟
| # | 功能 |
|---|---|
| ⑥ | 标签系统（手动/AI 建议/批量/标签库维护） |
| ⑦ | Token 用量统计（Provider/Model/Agent 维度图表） |
| ⑧ | 多语言 UI 切换与运行日志 |

---

## 5. Coding Conventions — 编码约定

- 与用户用中文沟通；代码注释与文档用英文；禁用 emoji
- Rust：snake_case 模块名、CamelCase 类型名；thiserror 定义错误；tracing 日志（禁止 println!）
- React：*View.tsx 组件、use*.ts hooks、types.ts 类型定义；Tailwind CSS；Context + useReducer
- Repository 分层：Command → Service → Repository → SQLite，禁止 Command 直接拼接 SQL
- 数据库迁移不可回退（只增不改）
- Tauri Command 签名即契约，变更需同步前端
- 错误传递：`Result<T, String>`，`.map_err(|e| e.to_string())` 在边界转换

---

## 6. Current Status — 当前状态

| 维度 | 状态 |
|---|---|
| 项目文档 | ✅ INIT.md / 决策记录 / ADR×4 / AGENTS.md / PLAN.md |
| Rust 核心 | 🔲 待搭建 Cargo workspace |
| Tauri 壳 | 🔲 待 `cargo tauri init` |
| React 前端 | 🔲 待 `npm create vite` |
| 数据库 Schema | 🔲 待自行设计 |
| CI/CD | 🔲 目标 Day 1 三平台构建 |

---

## 7. Design Decisions — 关键设计决策

| 决策 | 方案 | 理由 | 验证方式 |
|---|---|---|---|
| Rust over Swift | Rust + Tauri 2 | Swift on Windows 不成熟；单进程零 IPC | 三平台 cargo build 通过 |
| OpenAI 兼容协议 | /v1/chat/completions + SSE | 行业事实标准 | mock HTTP server 测试流式解析 |
| Tauri 单进程 | Rust 直接嵌入 Tauri 2 | 同语言无开销 | cargo tauri dev 正常运行 |
| React + TypeScript | React + Vite + Tailwind + shadcn/ui | AI 训练数据最丰富 | npm run lint 零 error |
| SQLite 自行设计 | rusqlite + WAL + 版本化迁移 | GRDB 无法复用 | 迁移测试 + CRUD 测试 |
| AI 流式 IPC | Tauri Event API | 原生支持，不暴露 API Key | 前端实时接收流式事件 |
| Agent 统一状态机 | 5 状态（Idle/Running/Succeeded/Failed/Cancelled） | 与 Mercury 原版一致 | 状态转换正确性测试 |
| 后台 Feed 同步 | Tokio + Semaphore 并发控制 | 高性能、内存安全 | 后台同步不阻塞 UI |

---

## 8. Recent Notes — 近期实现细节

**文档整合（2026-07-13 ~ 2026-07-14）**：三版 AGENTS.md 已整合完成，融合王康睿（AI 操作手册/路线图）、刘欣慧（8 设计决策）、杜偲妍（复刻原则/状态机/DoD）三版内容。

**新增 ADR**：ADR-005 Repository 分层架构、ADR-006 Reader 管线固定流程、ADR-007 Agent 统一状态机。

**团队分工**：
| 模块 | 负责人 | Rust 侧 | React 侧 |
|---|---|---|---|
| Feed 与内容管线（①②） | 王康睿 | Feed 解析/同步、OPML、SQLite schema、Readability、Markdown | 订阅源管理、文章列表、阅读器 |
| AI 功能（③④） | 刘欣慧 | OpenAI 协议、SSE 流式、Summary/Translation Agent | Provider 配置、摘要面板、翻译面板 |
| 笔记与导出（⑤） | 杜偲妍 | 笔记 CRUD、文摘导出、模板引擎 | 笔记编辑器、设置页、通用组件 |

**并行策略**：数据库 Schema 落定后三人可 mock 数据独立开发，刘欣慧可用 mock HTML 提前开发 AI 协议。

---

## 9. Roadmap — 未来规划

| 阶段 | 目标 | 主导 | 产出 |
|---|---|---|---|
| Stage 1 | 基础阅读器原型 | 王康睿 | 项目骨架、数据库、Feed 订阅、基础阅读 + OPML |
| Stage 2 | 阅读体验增强 | 王康睿 | Readability 管线、主题字体、FTS5 搜索 |
| Stage 3 | AI 功能接入 | 刘欣慧 | Provider 管理、Summary Agent、Translation Agent |
| Stage 4 | 信息整理与导出 | 杜偲妍 | 笔记系统、文摘导出、三平台打包发布 |

**回滚机制**：每阶段验收后打 tag（`v0.1-stage1` ~ `v0.4-stage4`），崩盘可 `git reset --hard <tag>`。

**Definition of Done**：编译通过 / 测试通过 / clippy 零 error / lint 零 error / 无 panic / 数据持久化 / 三平台兼容 / 无回归 / Git tag。

---

## 10. Known Issues — 已知问题与风险

| # | 问题 | 严重程度 | 缓解措施 |
|---|---|---|---|
| 1 | AI 生成的 Rust 代码质量依赖训练数据 | 高 | 优先使用主流 crate，避免小众方案 |
| 2 | 非标准 RSS Feed 兼容性 | 中 | feed-rs 兜底，解析失败记录 warn |
| 3 | Readability 中文提取质量 | 中 | 多种中文 HTML fixture 测试验证 |
| 4 | Linux WebViewGTK 依赖复杂性 | 中 | Linux 为次要目标，打包时处理 |
| 5 | 团队成员为大一新生，全部依赖 AI | 高 | 技术选型以「AI 见过最多」为第一原则 |
| 6 | 数据库 Schema 是唯一不可并行环节 | 中 | 三人共同评审后各自独立推进 |
| 7 | SSE 流式解析网络波动 | 低 | warn 并重试，不打 error |
| 8 | 多 crate 循环引用导致编译错误难定位 | 中 | 优先单体结构，禁止多 crate 循环依赖 |
