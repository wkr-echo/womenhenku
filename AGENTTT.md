# Mercury 跨平台复刻 — AGENTTT.md

> 项目标准与约束 | 基于图片模板 + 三版 AGENTS.md 整合 + PLAN.md 执行计划

---

## 1. Project Goal — 一句话目标

将 macOS 原生 RSS 阅读器 **Mercury** 复刻为跨平台桌面应用，在 **Windows 10+ / macOS 12+ / Linux** 上实现功能对等，保持本地优先、大模型中立、AI 编码可追溯的核心原则。

---

## 2. Architecture — 架构原则

### 2.1 三层解耦

```
React UI（纯展示）
    ↓  invoke()
Tauri Command（参数校验、错误转换，不含业务逻辑）
    ↓
Rust Core（所有业务逻辑、数据库、网络、AI 协议）
```

### 2.2 核心原则

- **业务逻辑不得依赖 UI** — Rust Core 零 UI 导入
- **数据库层不得依赖平台 API** — SQLite 唯一事实来源
- **平台能力必须经过抽象层** — 禁止平台特定 import 进入核心
- **单进程架构** — Tauri 2 单进程，零 IPC 开销
- **数据库单进程写入** — SQLite WAL 模式支持并发读

### 2.3 阶段隔离（崩盘可回滚）

每个阶段代码在独立目录，禁止修改上一阶段已有文件：

```
Stage 1: src-tauri/src/feed/     src-tauri/src/db/
Stage 2: src-tauri/src/reader/                            ← 不改 feed/, db/
Stage 3: src-tauri/src/agent/                             ← 不改 reader/, feed/
Stage 4: src-tauri/src/digest/   src-tauri/src/notes/     ← 不改 agent/, reader/
```

每阶段完成打 Git tag：`v0.1-stage1` ~ `v0.4-stage4`。崩盘时 `git reset --hard <上一阶段tag>` 回滚。

### 2.4 Reader 管线（固定流程）

```
Feed Entry → Raw HTML → Readability → HTML Sanitization → Markdown Conversion → Rendered HTML → Reader View
```

所有步骤在 **Rust Core** 完成，React 禁止执行任何内容处理。

### 2.5 AI 协议引擎

实现 **OpenAI 兼容协议**（`/v1/chat/completions` + SSE 流式），用户配置 baseURL + API Key + model 即可接入任意云端/本地大模型（OpenAI / DeepSeek / Ollama / vLLM 等）。

---

## 3. Tech Stack — 技术选型

| 层级 | 选型 | 版本/说明 |
|---|---|---|
| **语言** | Rust | 1.80+ stable，跨平台一等公民 |
| **UI 壳** | Tauri 2 | WebView2 / WKWebView / WebViewGTK |
| **前端** | React + TypeScript + Vite | AI 训练数据最丰富 |
| **CSS** | Tailwind CSS | 与 shadcn/ui 共生 |
| **组件库** | shadcn/ui | 按需复制，错误在自己文件 |
| **状态管理** | React Context + useReducer | 官方方案，够用 |
| **数据库** | SQLite（`rusqlite`，WAL 模式） | 版本化迁移，内存 SQLite 测试 |
| **HTTP** | `reqwest` | Rust 生态标准 |
| **Feed 解析** | `feed-rs` | RSS 0.91/0.92/1.0/2.0、Atom 1.0、JSON Feed |
| **文章提取** | `readability` crate | Mozilla Readability Rust 移植 |
| **HTML 清洗** | `scraper` | CSS 选择器友好 |
| **Markdown** | `comrak` | GFM 完整支持 |
| **AI 协议** | OpenAI 兼容 | `/v1/chat/completions` + SSE |
| **日志** | `tracing` | 结构化日志，禁止 `println!` |
| **错误处理** | `thiserror`（库层）+ `anyhow`（应用层） | 分层策略 |
| **打包** | `.msi` / `.dmg` / `.AppImage` + `.deb` | 各平台原生格式 |

---

## 4. Key Features — 功能清单

### v1 必须实现

| # | 功能 | 要点 |
|---|---|---|
| ① | **Feed 订阅与内容呈现** | RSS/Atom/JSON Feed 解析、OPML 导入导出、自动同步、文章列表（已读/未读/搜索/分页） |
| ② | **文章清洗与阅读体验** | Readability 提取、HTML 清洗、Markdown 转换（GFM）、主题切换（亮色/暗色）、字体定制 |
| ③ | **AI 摘要** | 可配置语言/详细程度、流式输出、1 秒防抖串行、latest-only 队列 |
| ④ | **AI 双语翻译** | 段落级双语对照布局、并发控制（Semaphore 1~5）、重试/恢复/清除 |
| ⑤ | **笔记与文摘导出** | 单篇 Markdown 笔记、单篇/多篇文摘导出、自定义模板 |

### v2 推迟

| # | 功能 |
|---|---|
| ⑥ | 标签系统（手动/AI 建议/批量/标签库维护） |
| ⑦ | Token 用量统计（Provider/Model/Agent 维度图表） |
| ⑧ | 多语言 UI 切换与运行日志 |

---

## 5. Coding Conventions — 编码约定

### 通用

- 与用户用 **中文** 沟通；代码注释与文档用 **英文**
- 禁用 emoji 于代码注释与文档
- Markdown 中用 backticks 包裹代码引用

### Rust 侧

- 模块名 `snake_case`，类型/trait `CamelCase`
- 数据库迁移不可回退（只增不改），每次迁移需对应测试
- Tauri Command 签名即契约，变更需同步更新前端
- 错误传递：`Result<T, String>`，`.map_err(|e| e.to_string())` 在边界转换
- 默认数值类型 `i32` / `f64`
- 日志用 `tracing`：Command 入口 `info!`，数据库操作 `debug!`，错误 `error!`

### React 侧

- 组件文件 `*View.tsx`，hooks `use*.ts`，类型定义 `types.ts`
- CSS 使用 Tailwind CSS，禁止额外 CSS 框架
- 组件库 shadcn/ui，按需复制到 `src-ui/src/components/ui/`
- 状态管理 React Context + `useReducer`，不引入 Redux/Zustand
- v1 仅中文，用户可见字符串通过 `t()` 包裹，v2 升级 react-i18next

### Repository 分层

```
Tauri Command → Service（业务逻辑） → Repository（数据访问） → SQLite
```

禁止在 Command 中直接拼接 SQL。

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

### 7.1 Rust over Swift

- **方案**：Rust + Tauri 2 单进程替代 Swift + SwiftUI
- **理由**：Swift on Windows 不成熟；Rust 单进程零 IPC
- **验证**：三平台 `cargo build` 通过

### 7.2 OpenAI 兼容协议

- **方案**：实现 `/v1/chat/completions` + SSE 流式
- **理由**：行业事实标准，覆盖 DeepSeek/Ollama/vLLM 等
- **验证**：mock HTTP server 测试流式解析、断连恢复

### 7.3 Tauri 单进程架构

- **方案**：Rust 核心直接嵌入 Tauri 2，零 IPC
- **理由**：同语言无开销，三平台原生 WebView
- **验证**：`cargo tauri dev` 正常运行

### 7.4 React + TypeScript 前端

- **方案**：React + Vite + Tailwind CSS + shadcn/ui
- **理由**：AI 训练数据最丰富，生成质量最高
- **验证**：组件渲染测试，`npm run lint` 零 error

### 7.5 SQLite Schema 自行设计

- **方案**：功能语义参考原始 Mercury，内部结构自行设计
- **理由**：原始 GRDB（macOS 专属）无法直接复用
- **验证**：迁移测试 + CRUD 测试

### 7.6 AI 流式 IPC 传递

- **方案**：Tauri Event API（非前端直连 HTTP）
- **理由**：原生支持，不暴露 API Key 到前端
- **验证**：前端实时接收流式事件，数据完整

### 7.7 Agent 统一状态机

- **方案**：所有 Agent 实现 Idle / Running / Succeeded / Failed / Cancelled 五状态
- **理由**：与 Mercury 原版行为严格一致
- **验证**：状态转换正确性测试

### 7.8 后台 Feed 同步

- **方案**：Tokio 异步任务 + Semaphore 并发控制
- **理由**：高性能、内存安全
- **验证**：后台同步不阻塞 UI，并发数正确限制

---

## 8. Recent Notes — 近期实现细节

### 文档整合（2026-07-13 ~ 2026-07-14）

三版 AGENTS.md 已整合完成：

| 版本 | 作者 | 侧重点 |
|---|---|---|
| 初版 | 王康睿 | AI 操作手册、文件结构、四阶段路线图 |
| 第二版 | 刘欣慧 | 8 个设计决策（方案/理由/实现要点/验证方式） |
| 第三版 | 杜偲妍 | 复刻优先原则、Feature Parity、状态机合约、Definition of Done |

### 新增 ADR

| ADR | 决策 | 来源 |
|---|---|---|
| 005 | Repository 分层架构（Command → Service → Repository → SQLite） | 杜偲妍提案 |
| 006 | Reader 管线固定流程（7 步不可变） | 杜偲妍提案 |
| 007 | Agent 统一状态机（5 状态 + 转换规则） | 杜偲妍提案 |

### 团队分工

| 功能模块 | 负责人 | Rust 侧 | React 侧 |
|---|---|---|---|
| Feed 与内容管线（①②） | 王康睿 | Feed 解析/同步、OPML、SQLite schema、Readability、Markdown | 订阅源管理、文章列表、阅读器 |
| AI 功能（③④） | 刘欣慧 | OpenAI 协议、SSE 流式、Summary/Translation Agent | Provider 配置、摘要面板、翻译面板 |
| 笔记与导出（⑤） | 杜偲妍 | 笔记 CRUD、文摘导出、模板引擎 | 笔记编辑器、设置页、通用组件 |

### 并行开发策略

```
数据库 Schema 落定（三人一起）
    ├─→ 王康睿：Feed → Reader（Stage 1→2）
    ├─→ 刘欣慧：mock 开发 AI 协议 → 对接真实文章（Stage 3）
    └─→ 杜偲妍：mock 开发笔记 → 对接真实数据（Stage 4）
```

---

## 9. Roadmap — 路线图

### 第一阶段：基础阅读器原型

> 目标：跑通 RSS → 存储 → 列表 → 阅读的最小闭环

| 任务 | 产出 |
|---|---|
| 1.1 项目骨架搭建 | Rust + Tauri + React 骨架，CI 三平台构建 |
| 1.2 数据库 Schema | 自行设计 SQLite schema，版本化迁移系统 |
| 1.3 Feed 订阅 | RSS/Atom/JSON Feed 解析、CRUD、同步 |
| 1.4 基础阅读 + OPML | 文章阅读页、OPML 导入导出 |

### 第二阶段：阅读体验增强

> 目标：文章内容清洗 + 舒适阅读体验

| 任务 | 产出 |
|---|---|
| 2.1 Readability 管线 | 内容提取、HTML 清洗、Markdown 转换 |
| 2.2 主题与字体 | 亮色/暗色主题、自定义字体 |
| 2.3 搜索与离线缓存 | FTS5 全文搜索、离线阅读 |

### 第三阶段：AI 功能接入

> 目标：OpenAI 兼容协议 + 摘要 + 双语翻译

| 任务 | 产出 |
|---|---|
| 3.1 Provider 管理 | Provider 配置、OpenAI 协议封装 |
| 3.2 Summary Agent | 文章摘要、流式输出、防抖串行 |
| 3.3 Translation Agent | 段落级双语翻译、并发控制、重试 |

### 第四阶段：信息整理与导出

> 目标：笔记 + 文摘导出 + 打磨发布

| 任务 | 产出 |
|---|---|
| 4.1 笔记系统 | Markdown 笔记 CRUD |
| 4.2 文摘导出 | 单篇/多篇导出、自定义模板 |
| 4.3 打磨与发布 | 快捷键、通知、三平台打包、设置页 |

---

## 10. Known Issues — 已知问题与风险

| # | 问题 | 严重程度 | 缓解措施 |
|---|---|---|---|
| 1 | AI 生成的 Rust 代码质量依赖训练数据 | 高 | 优先使用主流 crate，避免小众方案 |
| 2 | 非标准 RSS Feed 兼容性 | 中 | `feed-rs` 兜底，解析失败记录 warn 而非崩溃 |
| 3 | Readability 中文提取质量 | 中 | 用多种中文 HTML fixture 测试验证 |
| 4 | Linux WebViewGTK 依赖复杂性 | 中 | Linux 为次要目标，打包时处理 |
| 5 | 团队成员为大一新生，全部依赖 AI | 高 | 技术选型以「AI 见过最多」为第一原则，避免复杂架构 |
| 6 | 数据库 Schema 是唯一不可并行环节 | 中 | 三人共同评审后各自独立推进 |
| 7 | SSE 流式解析网络波动 | 低 | `warn!` 并重试，不打 `error!` |
| 8 | 多 crate 循环引用导致编译错误难定位 | 中 | 优先单体结构，禁止多 crate 循环依赖 |
