# Mercury 跨平台复刻 — AGENT.md

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
│   ├── src/
│   │   ├── api/              # Tauri invoke 封装层
│   │   ├── components/       # UI 组件
│   │   ├── contexts/         # React Context
│   │   └── styles/           # 主题样式
│   ├── package.json
│   └── vite.config.ts
├── scripts/                  # 构建/测试脚本
└── .github/workflows/
    └── ci.yml                # CI 三平台构建
```

---

## 1. 项目使命与复刻原则

### 1.1 项目使命

**本项目不是设计新的 RSS 阅读器。本项目是 Mercury 的跨平台复刻。**

目标：在 **Windows 10+** / **macOS 12+** / **Linux（Wayland + X11）** 上尽可能复现 Mercury 的功能与体验。

### 1.2 允许与不允许

| 允许 | 不允许 |
|---|---|
| 编程语言不同（Swift → Rust） | 用户行为改变 |
| UI 框架不同（SwiftUI → React） | 功能语义改变 |
| 数据库实现不同（GRDB → rusqlite） | 工作流改变 |
| 内部架构不同 | Agent 行为改变 |
| UI 展示风格优化 | 数据语义改变 |

### 1.3 复刻原则

开发任何功能前必须先回答：

> "Mercury 原版是如何工作的？"

然后再开始实现。优先级：

```
Mercury 原版行为
> 当前实现
> 开发者个人偏好
```

如果发现 Mercury 原版设计存在缺陷：**创建 ADR，提交人工评审**。禁止 AI 自行优化产品行为。

### 1.4 技术约束

- **本地优先**：无需注册/登录，数据存本地 SQLite，不主动收集用户数据
- **平台中立**：三平台统一代码库
- **大模型中立**：OpenAI 兼容协议，用户自行配置 baseURL + API Key + model
- **良好设计标准**：遵循现代 UI 设计规范

---

## 2. 架构与管线

### 2.1 三层架构

```
React UI（纯展示与交互）
    │ invoke()
Tauri Command（参数校验、错误转换，不含业务逻辑）
    │
Rust Core（所有业务逻辑、数据库、网络、AI 协议）
```

**职责划分**：

| 层 | 职责 | 禁止 |
|---|---|---|
| **Rust Core** | Feed、Reader、Agent Runtime、Database、Search、Notes、Export、Settings | 无 UI 依赖 |
| **Tauri Command** | 参数校验、错误转换 | 包含业务逻辑、直接拼接 SQL |
| **React 前端** | UI 渲染、用户交互 | 访问文件系统/数据库/网络、执行 Readability/Markdown 转换/Feed 解析 |

**业务逻辑必须位于 Rust Core。**

### 2.2 Reader 管线（固定流程）

文章处理必须遵循以下管线，**所有步骤在 Rust Core 完成**：

```
Feed Entry
    ↓
Raw HTML
    ↓
Readability（内容提取）
    ↓
HTML Sanitization（scraper 清洗）
    ↓
Markdown Conversion（comrak，GFM 支持）
    ↓
Rendered HTML（注入阅读器主题样式）
    ↓
Reader View（React 纯展示）
```

**React 禁止执行**：Readability 提取、Markdown 转换、Feed 解析、HTML 清洗。

### 2.3 阶段隔离（崩盘可回滚）

每个阶段的新代码在独立目录下，**禁止修改上一阶段的已有文件**：

```
Stage 1: src-tauri/src/feed/     src-tauri/src/db/
Stage 2: src-tauri/src/reader/                               ← 不改 feed/, db/
Stage 3: src-tauri/src/agent/                                ← 不改 reader/, feed/
Stage 4: src-tauri/src/digest/   src-tauri/src/notes/        ← 不改 agent/, reader/
```

前端同理：

```
Stage 1: EntryList.tsx, ReaderView.tsx, Sidebar.tsx
Stage 2: CleanedReaderView.tsx    ← 新建，不改 ReaderView.tsx
Stage 3: SummaryPanel.tsx, TranslationPanel.tsx
Stage 4: NoteEditor.tsx, SettingsPage.tsx
```

如需增强上一阶段功能，**新建文件而非原地修改**。每个阶段验收后打 Git tag：

| Tag | 阶段 |
|---|---|
| `v0.1-stage1` | 基础阅读器原型 |
| `v0.2-stage2` | 阅读体验增强 |
| `v0.3-stage3` | AI 功能接入 |
| `v0.4-stage4` | 信息整理与导出 |

崩盘时 `git reset --hard <上一阶段tag>` 回到已知良好状态。

---

## 3. 技术选型

| 层级 | 选型 | 说明 |
|---|---|---|
| 语言 | Rust 1.80+（stable） | 跨平台一等公民，Tauri 同语言单进程 |
| UI 壳 | Tauri 2 | WebView2 / WKWebView / WebViewGTK |
| 前端 | React + TypeScript + Vite | AI 训练数据最丰富，生成质量最高 |
| CSS | Tailwind CSS | AI 见过最多，与 shadcn/ui 共生 |
| 组件库 | shadcn/ui | 代码复制到项目内，报错在自己文件里 |
| 状态管理 | React Context + useReducer | 官方方案，够用，报错简单 |
| 数据库 | SQLite（`rusqlite`，features: bundled，WAL 模式） | 本地优先 |
| HTTP | `reqwest` | Rust 生态标准 |
| Feed 解析 | `feed-rs` | RSS 0.91/0.92/1.0/2.0、Atom 1.0、JSON Feed |
| 文章提取 | `readability` crate | Mozilla Readability 的 Rust 移植 |
| HTML 清洗 | `scraper` | CSS 选择器友好 |
| Markdown | `comrak` | GFM 完整支持 |
| AI 协议 | OpenAI 兼容（`/v1/chat/completions` + SSE 流式） | 覆盖 DeepSeek/Ollama/vLLM 等 |
| 日志 | `tracing` | 结构化日志，禁止 `println!` |
| 错误处理 | `thiserror`（库层）+ `anyhow`（应用层） | 分层错误策略 |
| 连接池 | `r2d2` + `r2d2_sqlite` | 数据库并发访问 |
| 打包 | `.msi` / `.dmg` / `.AppImage` + `.deb` | 各平台原生安装格式 |

---

## 4. 构建与运行

### 环境要求

| 工具 | 版本 | 说明 |
|---|---|---|
| Rust | 1.80+（stable） | 2024 年 7 月稳定版 |
| Node.js | 20 LTS | LTS 线 |
| Tauri CLI | 2.x（latest stable） | `cargo install tauri-cli --version "^2"` |

### 常用命令

| 命令 | 用途 |
|---|---|
| `cargo tauri dev` | 开发运行（同时启动 Rust 后端 + React 前端） |
| `cargo tauri build` | 生产构建 |
| `cargo test` | 运行 Rust 侧测试 |
| `cargo clippy` | Rust 代码检查 |
| `cd src-ui && npm run lint` | 前端代码检查 |
| `cd src-ui && npm run dev` | 前端独立开发 |

---

## 5. v1 功能清单

### v1 必须实现

| # | 模块 | 要点 |
|---|---|---|
| ① | Feed 订阅与内容呈现 | RSS/Atom/JSON Feed 解析、添加/删除/刷新、OPML 导入导出、自动同步（可配置间隔）、手动同步、文章列表（已读/未读/分页） |
| ② | 文章清洗与阅读体验 | Readability 提取、HTML 清洗、Markdown 转换（GFM）、主题切换（亮色/暗色）、字体定制、响应式布局 |
| ③ | AI 摘要 | 语言与详细程度可配置、流式输出、Prompt 模板可自定义 |
| ④ | AI 双语翻译 | 段落级双语对照布局、多段落并发（默认 3）、重试失败段落、清除翻译 |
| ⑤ | 笔记与文摘导出 | 单篇 Markdown 笔记、单篇/多篇文摘导出（Markdown/HTML） |
| ⑥ | Provider 管理 | 多 Provider 配置（baseURL/API Key/model）、加密存储、连接验证 |

### v2 推迟实现

| # | 模块 |
|---|---|
| ⑥ | 标签系统（手动/AI 建议/批量/标签库维护） |
| ⑦ | Token 用量统计（Provider/Model/Agent 维度图表） |
| ⑧ | 多语言 UI 切换（i18n）与运行日志/调试面板 |

---

## 6. 编码约定

### 6.1 语言与文档

- **沟通语言**：中文（与用户交流）
- **代码注释**：英文
- **文档正文**：中文优先
- **禁止**：emoji 于代码注释与文档中

### 6.2 Rust 代码

- 模块名 `snake_case`，类型/trait `CamelCase`
- 使用 `thiserror` 定义错误类型
- 使用 `tracing` 日志（`info!`, `warn!`, `error!`），禁止 `println!` / `eprintln!`
- 所有公开函数返回 `Result<T, AppError>`
- 使用 `async`/`await`，`reqwest` 是异步
- Repository 方法命名：`find_by_*`、`insert`、`update`、`delete`
- Command 命名：动词开头（`add_feed`、`list_entries`）
- 默认数值类型 `i32` / `f64`；仅在平台 API 要求时使用 `usize` / `c_float`
- Tauri Command 错误传递：`Result<T, String>`，用 `.map_err(|e| e.to_string())` 在边界转换

### 6.3 React 代码

- TypeScript 严格模式
- 组件使用函数式组件 + Hooks
- 组件文件 `*View.tsx`，hooks 文件 `use*.ts`，类型定义 `types.ts`
- API 调用封装在 `src-ui/src/api/` 目录下，一个模块对应一类 Command 域
- 样式使用 Tailwind 类名
- 使用 `@tauri-apps/api` 的 `invoke()` 调用 Rust 侧
- v1 仅中文，所有用户可见字符串通过伪 i18n 函数 `t()` 包裹，v2 升级 react-i18next

### 6.4 通用规范

- 跨平台路径用 Rust `dirs` crate，禁止硬编码 `/` 或 `\`
- 异步操作用 `async/await`，禁止阻塞主线程
- 数据库迁移不可回退（只增不改）
- 单次 AI 调用错误连续重试不超过 3 次

---

## 7. 设计决策与实现要点

以下为 8 个关键技术决策，每项含方案、理由、实现要点与验证方式。

### 决策 1：Tauri 2 + React 方案

- **方案**：Rust 后端（Tauri 2）+ React + TypeScript + Tailwind CSS + shadcn/ui
- **理由**：跨平台一等公民、单进程零 IPC、AI 生态丰富、AI 见过最多正确样本
- **实现要点**：
  - `cargo tauri init` 初始化 Tauri 2
  - React + Vite + Tailwind + shadcn/ui
  - 所有业务逻辑在 Rust Core
- **验证**：`cargo tauri dev` 三平台弹出窗口

### 决策 2：SQLite + rusqlite（WAL 模式）

- **方案**：`rusqlite`（features: bundled）+ WAL 模式 + 版本化迁移
- **理由**：本地优先、无需数据库服务器、AI 训练数据多
- **实现要点**：
  - `PRAGMA journal_mode=WAL;`
  - `PRAGMA foreign_keys=ON;`
  - 迁移目录 `db/migrations/`，编号递增
  - 连接池使用 `r2d2`
- **验证**：单元测试验证迁移执行、表创建、CRUD

### 决策 3：Repository 分层架构（ADR 005）

- **方案**：Command → Service → Repository → SQLite
- **理由**：分离关注点、可测试、AI 辅助开发效率高
- **实现要点**：
  - Repository 持有 `r2d2::Pool<SqliteConnectionManager>` 引用
  - `RepositoryError` 用 `thiserror` 派生
  - Command 不做业务逻辑，不直接拼接 SQL
- **验证**：每个 Repository 的单元测试（内存 SQLite）

### 决策 4：Reader 固定管线（ADR 006）

- **方案**：7 步固定流程，Rust Core 完成
- **理由**：确保一致性和可测试性
- **实现要点**：
  - `readability` → `scraper`（清洗白名单）→ `comrak`
  - 结果缓存到 `contents` 表
  - 渲染 HTML 缓存 key：`theme_id + entry_id + reader_render_version`
- **验证**：多种 HTML fixture 测试（新闻/博客/中文/嵌套表格/畸形标记）

### 决策 5：OpenAI 兼容协议

- **方案**：OpenAI 兼容协议（`/v1/chat/completions` + SSE 流式）
- **理由**：行业事实标准，覆盖 DeepSeek/Ollama/vLLM 等
- **实现要点**：
  - 用户配置 baseURL + API Key + model
  - SSE 流式：`data: {"choices":[{"delta":{"content":"..."}}]}`
  - 通过 Tauri Event 推送到前端：`app_handle.emit("ai-stream", {task_id, content, is_done, agent_type})`
  - 前端 `listen("ai-stream", callback)` 监听
  - 超时：连接 30s、请求 120s
- **验证**：mock HTTP server 测试流式解析、断连恢复、错误处理

### 决策 6：Agent 统一状态机（ADR 007）

- **方案**：5 状态（Idle/Running/Succeeded/Failed/Cancelled），严格转换规则
- **理由**：确保 Agent 行为可预测，与 Mercury 原版一致
- **实现要点**：
  - Idle → Running（用户触发）
  - Running → Succeeded（流完成）/ Failed（API 错误）/ Cancelled（用户取消）
  - Failed 不可直接 → Running，需创建新 Run
  - 等待队列：latest-only 替换策略
  - 当前每类 Agent 限制：active slot 1 个 + waiting slot 1 个
- **验证**：状态机转换正确性测试

### 决策 7：Feed 解析

- **方案**：`feed-rs` 解析 + `reqwest` 抓取
- **理由**：支持 RSS/Atom/JSON Feed 全格式
- **实现要点**：
  - 超时 30s
  - 解析失败不崩溃，记录 warn 日志，返回原始 HTML 作降级
  - 去重：按 `entry.link` 或 `entry.id`
  - 并发刷新：Semaphore 限制 5 并发
- **验证**：fixture 文件验证多格式解析正确性

### 决策 8：API Key 安全存储

- **方案**：Tauri secure-store plugin（平台原生：Keychain / Credential Manager / libsecret）
- **理由**：安全存储敏感信息
- **实现要点**：
  - Provider 配置存数据库，API Key 加密存 secure store
  - 验证：发送最小请求 `{model, messages: [{role:"user", content:"hi"}], max_tokens:1}`
- **验证**：验证 200/401 响应时 UI 正确提示

---

## 8. 数据库优先规则

### 8.1 SQLite 是唯一事实来源

- 所有业务状态必须持久化到 SQLite
- 应用状态启动时从数据库重建
- 不依赖文件系统缓存

**禁止**：LocalStorage 存业务数据、React State 存业务数据、内存对象作为唯一状态。

**允许**：UI 状态（当前选中的标签页）、临时缓存、查询缓存。

### 8.2 迁移只增不改

每个阶段新增 migration 只创建新表/新列（带 DEFAULT），**绝不修改已有表结构**：

```
Stage 1: 001_initial_schema.sql  → feeds, entries, contents
Stage 2: 002_fts_search.sql      → FTS5 虚拟表（不碰原表）
Stage 3: 003_agent_tables.sql    → summaries, translations, providers（新表）
Stage 4: 004_notes_digest.sql    → notes, digest_templates（新表）
```

回滚时后续阶段的空表留在数据库中无害，不需要回滚 migration，数据不丢失。

### 8.3 核心表结构

#### feeds
| 字段 | 类型 | 约束 | 说明 |
|---|---|---|---|
| id | INTEGER | PK, AUTO | |
| url | TEXT | UNIQUE, NOT NULL | |
| title | TEXT | | |
| description | TEXT | | |
| link | TEXT | | |
| last_synced_at | TEXT | | |
| created_at | TEXT | DEFAULT CURRENT_TIMESTAMP | |

#### entries
| 字段 | 类型 | 约束 | 说明 |
|---|---|---|---|
| id | INTEGER | PK, AUTO | |
| feed_id | INTEGER | FK→feeds.id | |
| title | TEXT | NOT NULL | |
| author | TEXT | | |
| link | TEXT | UNIQUE | |
| published_at | TEXT | | |
| updated_at | TEXT | | |
| is_read | INTEGER | DEFAULT 0 | |
| is_starred | INTEGER | DEFAULT 0 | |
| created_at | TEXT | DEFAULT CURRENT_TIMESTAMP | |

#### contents
| 字段 | 类型 | 约束 | 说明 |
|---|---|---|---|
| id | INTEGER | PK, AUTO | |
| entry_id | INTEGER | FK, UNIQUE | 一对一关联 |
| raw_html | TEXT | | 原始 HTML |
| cleaned_html | TEXT | | Readability 提取后 |
| cleaned_markdown | TEXT | | 转 Markdown |
| rendered_html | TEXT | | 最终渲染 HTML |
| readability_version | TEXT | | 缓存失效判断 |
| created_at | TEXT | DEFAULT CURRENT_TIMESTAMP | |

---

## 9. Agent 运行时合约

### 9.1 统一状态机

所有 Agent（Summary、Translation）必须实现统一状态机：

```
                 ┌─────────────────┐
                 │      Idle       │ ◄────┐
                 └────────┬────────┘      │
                          │ start         │ cancel / 清除
                          ▼               │
                 ┌─────────────────┐      │
                 │    Running      │──────┘
                 └──┬────────┬────┘
          ┌─────────┘        └─────────┐
          ▼                             ▼
   ┌──────────────┐            ┌──────────────┐
   │  Succeeded   │            │   Failed     │
   └──────────────┘            └──────────────┘
```

**转换规则**：
| 转换 | 触发条件 |
|---|---|
| Idle → Running | 用户触发 |
| Running → Succeeded | 流式完成 |
| Running → Failed | API 错误 |
| Running → Cancelled | 用户取消 |
| Succeeded → Idle | 用户清除结果 |
| Failed → Cancelled | 用户取消 |

**禁止的转换**：Idle → Succeeded（跳过执行）、Failed → Running（直接恢复失败任务）。重试必须创建新的 Run。

### 9.2 全局执行策略

- 不自动取消进行中的后台任务
- 取消只能来自用户显式操作
- 等待队列为 latest-only 替换策略（新请求覆盖旧等待项）
- 当前每类 Agent 限制：active slot 1 个 + waiting slot 1 个

### 9.3 Summary Agent 合约

| 配置 key | 说明 | 默认值 |
|---|---|---|
| Agent.Summary.DefaultTargetLanguage | 目标语言 | "中文" |
| Agent.Summary.DefaultDetailLevel | 详细程度 | "balanced" |
| Agent.Summary.PrimaryModelId | 模型 ID | 用户配置 |

- 输入：`cleaned_markdown`
- 输出：流式文本 → 存入 `summaries` 表
- 1 秒防抖、串行执行、不自动重试、latest-only 队列
- Prompt 模板从 `resources/prompts/summary.default.yaml` 加载

### 9.4 Translation Agent 合约

| 配置 key | 说明 | 默认值/范围 |
|---|---|---|
| Agent.Translation.DefaultTargetLanguage | 目标语言 | "中文" |
| Agent.Translation.concurrencyDegree | 并发度 | 3（1~5） |

- 段落切分：按 `<p>`、`<ul>`、`<ol>` 切分 `cleaned_html`
- 合成前置段落 `seg_meta_title_author` 保持标题对齐
- 有界并发：`tokio::sync::Semaphore` 限制
- 结果持久化到 `translations` 表
- 支持：恢复、重试失败段落、清除所有翻译、回到原文

### 9.5 Prompt 管理

- Prompt 属于资源文件，内置在 `src-tauri/resources/prompts/` 下
- 内置 Prompt 永远**只读**
- 用户可通过文件覆盖或新增 Prompt
- **禁止**在代码中硬编码 Prompt 文本

---

## 10. AI 使用指南

### 10.1 工作流程

1. **先读文档**：每次任务开始前，读取 `AGENT.md` 和 `PLAN.md`，确认项目状态和当前阶段
2. **Tauri Command 契约先行**：新增或修改前后端交互接口时，先协定 Command 签名，再分别实现 Rust 和 React 侧
3. **变更后更新本文件**：任何影响项目级约束的变更需同步更新 AGENT.md

### 10.2 技术选型硬原则

团队成员为大一新生，全部依赖 AI 辅助编码，无法处理复杂编译器报错。所有技术决策必须遵循：

1. **优先使用 AI 训练数据最丰富的方案**：官方模板、社区主流用法、Tauri 默认配置。偏离主流意味着 AI 生成的代码正确率断崖下降。
2. **编译器报错必须能直指问题位置**：新手能复制错误信息发给 AI，AI 一次修正成功。禁止多 crate 循环引用、多层 trait bound 连锁错误等新手无法定位的架构。
3. **优先单体结构，推迟拆分**：一个 `Cargo.toml`、一个 `package.json`、目录隔离满足需求。crate 拆分是优化手段，不是必需品。
4. **AI 一次生成对的概率 = 一切**：选型不看「技术先进性」，看「AI 见过多少正确样本」。

### 10.3 Rust 并发与异步约束

**应该做：**

- 可变状态指定唯一所有者：`Arc<RwLock<T>>` 或通过 channel（`mpsc`/`tokio::sync`）传递所有权
- 长生命周期的异步资源（数据库连接池、HTTP client、SSE 流）必须有明确的 struct 所有者
- 使用 `CancellationToken` 实现结构化取消，确保清理逻辑归属清晰
- 并发变更在 Debug 和 Release 模式下分别验证

**不应该做：**

- 不要用 `unsafe` 绕过 `Send` / `Sync` 问题
- 不要在 Tauri State 中存储非 `Send + Sync` 的类型
- 不要随意 `tokio::spawn` 无主任务；每个异步任务必须有明确的取消/清理责任人
- 不要在持有锁的临界区内调用 `.await`（会导致死锁）

### 10.4 测试约束

- 数据库测试默认使用**内存 SQLite**（`:memory:`），仅当测试明确需要磁盘行为时才用临时文件
- 磁盘数据库测试清理时删除整个目录，不单独删 `.sqlite` 文件
- 提供共享测试 fixture，禁止每个测试文件重复造数据
- 测试命名按行为而非实现：`test_feed_parse_atom_valid`
- 禁止 `std::thread::sleep` 类时序断言；用 `tokio::time::timeout` 或轮询等待
- 异步测试使用 `tokio::test`
- AI 协议测试使用 mock HTTP server
- 状态机转换正确性测试
- 段落切分测试

### 10.5 错误处理与日志

- 库级错误用 `thiserror` 派生，应用级错误用 `anyhow`
- 日志用 `tracing` crate：Command 入口打 `info!`，数据库操作打 `debug!`，错误打 `error!`
- SSE 流解析失败打 `warn!` 并重试（网络波动正常），不打 `error!`
- 禁止 `println!` 或 `eprintln!` 用于日志
- 网络超时：30s 连接，120s 请求
- AI 请求失败不崩溃，记录 warn 日志
- 解析失败回退到原始 HTML

### 10.6 React 前端约束

- 所有 Tauri `invoke` 调用封装在 `src-ui/src/api/` 目录下
- 流式输出通过 Tauri Event 推送到前端，不在前端直接建立 HTTP 连接
- 阅读器渲染：清洗后的 HTML 直接注入 WebView，Markdown 转 HTML 在 Rust 侧完成
- 使用 mock 数据独立开发
- 组件渲染测试（mock invoke 返回值）

### 10.7 何时停下询问

以下情况 AI 必须停下并请求人工决策：

1. 引入**新 crate 或 npm 包**（需评估体积、许可证）
2. **修改已有 Tauri Command 签名**（契约变更）
3. **数据库 schema 变更**（迁移编号、字段类型）
4. 涉及**并发模型变更**
5. 性能与安全的**权衡选择**
6. 新 Agent 类型、新 Provider、**新网络协议**
7. 任何需要**修改 AGENT.md 本身**的变更
8. 发现**文档缺失或矛盾**
9. 不确定**原始 Mercury 行为**
10. 遇到**跨平台兼容性问题**

---

## 11. 当前进度

| 维度 | 状态 |
|---|---|
| 项目文档 | ✅ AGENT.md / PLAN.md / 决策记录 |
| Rust 核心 | 🔲 待搭建 Cargo workspace |
| Tauri 壳 | 🔲 待 `cargo tauri init` |
| React 前端 | 🔲 待 `npm create vite` |
| 数据库 Schema | 🔲 待自行设计 |
| Feed 模块 | 🔲 待实现 |
| Reader 管线 | 🔲 待实现 |
| Agent 模块 | 🔲 待实现 |
| 笔记/导出 | 🔲 待实现 |
| CI/CD | 🔲 目标 Day 1 三平台构建 |

---

## 12. 架构决策索引（ADR）

| ADR | 决策 | 状态 |
|---|---|---|
| 001 | Rust over Swift（Tauri 单进程） | 已采纳 |
| 002 | OpenAI 兼容协议 | 已采纳 |
| 003 | Tauri 单进程架构 | 已采纳 |
| 004 | React + TypeScript | 已采纳 |
| 005 | Repository 分层架构（Command → Service → Repository → SQLite） | 已采纳 |
| 006 | Reader 管线固定 7 步流程 | 已采纳 |
| 007 | Agent 统一状态机（5 状态 + 转换规则） | 已采纳 |
| 008 | SQLite + rusqlite（WAL 模式） | 已采纳 |

**规则**：无 ADR 不合并。以下情况必须创建 ADR：
- 数据库 Schema 修改
- 新 Agent 类型
- 新 Provider
- 新协议
- 新平台能力
- 架构调整

---

## 13. 分工

| 功能模块 | 负责人 | Rust 侧 | React 侧 |
|---|---|---|---|
| Feed 与内容管线（①②） | 王康睿 | Feed 解析/同步、OPML、SQLite schema、Readability、Markdown | 订阅源管理、文章列表、阅读器、侧边栏 |
| AI 智能体（③④） | 刘欣慧 | Provider 管理、OpenAI 协议、Summary/Translation Agent | 摘要面板、翻译面板、Agent 配置页 |
| 笔记与基础设施（⑤） | 杜偲妍 | 笔记 CRUD、文摘导出、设置持久化、Tauri Command 契约 | 笔记编辑、导出面板、设置页、通用组件 |

**并行策略**：
- 数据库 Schema 落定后，三人可 mock 数据独立开发
- 刘欣慧可用 mock HTML 开发 AI 协议
- 杜偲妍可用 mock Entry 开发笔记和设置

---

## 14. 路线图（四阶段递增交付）

每个阶段产出可运行的产品增量，后一阶段在前一阶段基础上叠加。

| 阶段 | 里程碑 | 主导 | 产出 |
|---|---|---|---|
| **Stage 1** | 基础阅读器原型 | 王康睿 | 跑通 RSS → 存储 → 列表 → 阅读的最小闭环 |
| **Stage 2** | 阅读体验增强 | 王康睿 | Readability 清洗 + 主题切换 + 搜索 |
| **Stage 3** | AI 功能接入 | 刘欣慧 | OpenAI 协议 + 摘要 + 双语翻译 |
| **Stage 4** | 信息整理与导出 | 杜偲妍 | 笔记 + 文摘导出 + 三平台打包 |

**回滚机制**：每个阶段验收后打 tag（`v0.1-stage1` ~ `v0.4-stage4`），崩盘可 `git reset --hard <tag>` 回到上一阶段。

---

## 15. Definition of Done

功能标记为完成必须满足**全部**条件：

| # | 标准 | 说明 |
|---|---|---|
| 1 | 编译通过 | `cargo build` 零 warning |
| 2 | 测试通过 | `cargo test` 全部通过 |
| 3 | clippy 通过 | `cargo clippy` 零 error |
| 4 | lint 通过 | `cd src-ui && npm run lint` 零 TypeScript error |
| 5 | 无 panic | 所有错误走 Result 传播 |
| 6 | 数据持久化 | 重启不丢失 |
| 7 | 三平台兼容 | 行为一致 |
| 8 | 无回归 | 上一阶段功能仍然正常 |
| 9 | Git tag | 打阶段里程碑 tag |

否则**不得标记完成**。

---

## 16. 已知问题与风险

| 风险 | 严重度 | 缓解措施 |
|---|---|---|
| `feed-rs` 对非标准 RSS 兼容性 | 中 | 异常源记录 warn，不崩溃 |
| `readability` crate 中文提取质量 | 中 | 多种中文 fixture 测试，必要时回退原始 HTML |
| Tauri 在 Linux 上 WebViewGTK 依赖 | 中 | CI 覆盖 Linux 构建 |
| LLM 接口稳定性（SSE 断连恢复） | 中 | 断连恢复、畸形数据容错 |
| AI 流式 SSE 解析健壮性 | 中 | 实现重试机制 |
| API Key 泄露风险 | 低 | 使用平台原生安全存储 |
| SQLite WAL 模式配置 | 低 | 确保连接池正确配置 |
| AI 请求超时阻塞 | 低 | 设置 120s 超时 |

### 注意事项

- Rust 端所有 I/O 操作使用异步 API（`tokio`）
- 前端调用 Tauri API 时需处理异常，提供友好的错误提示
- 文章提取失败时显示原始内容，不崩溃
- 首次启动时自动创建数据库文件和所有表
- Feed 同步启动时自动触发一次，后续按配置间隔定时同步

---

*本文件为 AI 编码助手的完整操作手册。每次任务开始时请先阅读本文件，确认项目状态、约束规则和当前阶段后再开始编码。*
