# Mercury 跨平台复刻 — AI Agent 项目记忆体

本文件为 AI 编码助手的项目入口与最高优先级约束。AI 接到任务时必须优先阅读本文件。

**文档优先级**（冲突时以高优先级为准）：
1. AGENTS.md（本文件）
2. ADR（`docs/adr/`）
3. PLAN.md（`docs/PLAN.md`）
4. INIT.md（`docs/INIT.md`）
5. 代码实现
6. AI 自身推断

---

## 文件结构速览

```
WOMENHENKU/
├── AGENTS.md                  # 本文件：AI 入口与项目约束
├── docs/
│   ├── INIT.md                # 项目定义、功能清单、技术选型
│   ├── PLAN.md                # 执行计划
│   ├── cross-platform-analysis.md  # 原始 Mercury 平台耦合分析
│   ├── key-decisions-summary.md    # 全流程关键决策总览
│   ├── decisions/             # 构建过程决策记录
│   │   ├── init-decision-log.md
│   │   ├── agent-construction-log.md
│   │   └── plan-construction-log.md
│   ├── adr/                   # 架构决策记录（9 份）
│   └── reference/             # 实现参考文档
│       ├── command-contract.md           # Tauri Command 契约清单
│       ├── frontend-state-design.md      # 前端状态设计
│       └── mercury-behavior-manual.md    # Mercury 行为手册
├── src-tauri/                 # Rust 核心 + Tauri 后端（待创建）
│   ├── src/main.rs            # 应用入口
│   ├── src/lib.rs             # Tauri Command 注册
│   └── resources/prompts/     # Prompt 模板（内置只读）
├── src-ui/                    # React 前端（待创建）
│   ├── src/main.tsx           # 前端入口
│   ├── src/api/               # Tauri invoke 封装层
│   └── src/                   # 组件与页面
└── scripts/                   # 构建/测试脚本（待创建）
```

---

## 1. 项目目标与复刻原则

### 项目目标

将 macOS 原生 RSS 阅读器 Mercury 复刻为跨平台桌面应用，目标平台 Windows 10+ / macOS 12+ / Linux（Wayland + X11）。

核心原则：本地优先（无需账号/登录）、大模型中立（OpenAI 兼容协议）、AI 辅助编码可追溯。

### 复刻优先于重设计

本项目不是设计新的 RSS 阅读器，而是对 Mercury 的跨平台复刻。

**复刻的是功能体验，不是代码**。原始 Mercury 使用 Swift + GRDB + SwiftUI，复刻版使用 Rust + rusqlite + React。内部实现完全自己设计，但用户看到什么、能做什么、交互流程是什么——这些功能行为要与原版一致。

**允许**：技术实现不同、编程语言不同、UI 框架不同、数据库结构不同、内部架构不同。

**不允许**：功能行为改变、用户交互流程改变、Agent 行为改变、数据语义改变。

Mercury 原版的功能行为是唯一参考标准，内部实现自行设计。

---

## 2. 架构与管线

### 三层架构

```
React UI
    ↓  invoke()
Tauri Command（参数校验、错误转换，不含业务逻辑）
    ↓
Rust Core（所有业务逻辑、数据库、网络、AI 协议）
```

**Rust 核心**：Feed、Reader、Agent Runtime、Database、Search、Settings、Export。

**Tauri Command 层**：粗粒度命令接口，前端与核心的契约边界。禁止包含业务逻辑。

**React 前端**：纯展示与交互。禁止直接访问文件系统、数据库、网络、系统 API。

- 单进程架构：Rust 核心直接嵌入 Tauri 2，零 IPC。
- 业务逻辑不得依赖 UI，数据库层不得依赖平台 API。
- 数据库单进程写入。

### 阶段隔离（崩盘可回滚）

每个阶段的代码在独立目录下，**禁止修改上一阶段的已有文件**：

```
Stage 1: src-tauri/src/feed/     src-tauri/src/db/
Stage 2: src-tauri/src/reader/                            ← 不改 feed/, db/
Stage 3: src-tauri/src/agent/                             ← 不改 reader/, feed/
Stage 4: src-tauri/src/digest/   src-tauri/src/notes/     ← 不改 agent/, reader/
```

如需增强上一阶段功能，**新建文件而非原地修改**。例如：Stage 2 不修改 `ReaderView.tsx`，而是新建 `CleanedReaderView.tsx` 通过配置开关切换。这样上一阶段的代码路径永远可用，作为降级兜底。

每个阶段完成后打 Git tag：`v0.1-stage1`、`v0.2-stage2`、`v0.3-stage3`、`v0.4-stage4`。崩盘时 `git reset --hard <上一阶段tag>` 即可回到已知良好状态。

### Reader 管线（固定流程）

文章处理必须遵循以下管线，所有步骤在 Rust Core 完成：

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
Rendered HTML
    ↓
Reader View（React 纯展示）
```

React 禁止执行 Readability、Markdown 转换、HTML 清洗。所有内容处理在 Rust Core 完成。

---

## 3. 技术选型

| 层级 | 选型 | 说明 |
|---|---|---|
| 语言 | Rust 1.75+ | 跨平台一等公民，Tauri 同语言单进程 |
| UI 壳 | Tauri 2 | WebView2 / WKWebView / WebViewGTK |
| 前端 | React + TypeScript + Vite | AI 训练数据最丰富，生成质量最高 |
| CSS | Tailwind CSS | AI 见过最多，与 shadcn/ui 共生 |
| 组件库 | shadcn/ui | 代码复制到项目内，报错在自己文件里 |
| 状态管理 | React Context + useReducer | 官方方案，够用，报错简单 |
| 数据库 | SQLite（`rusqlite`，WAL 模式） | schema 参考原始 Mercury 设计 |
| HTTP | `reqwest` | Rust 生态标准 |
| Feed 解析 | `feed-rs` | RSS 0.91/0.92/1.0/2.0、Atom 1.0、JSON Feed |
| 文章提取 | `readability` crate | Mozilla Readability 的 Rust 移植 |
| HTML 清洗 | `scraper` | CSS 选择器友好 |
| Markdown | `comrak` | GFM 完整支持 |
| AI 协议 | OpenAI 兼容（`/v1/chat/completions` + SSE） | 覆盖 DeepSeek/Ollama/vLLM 等 |
| 日志 | `tracing` | 结构化日志，禁止 `println!` |
| 错误处理 | `thiserror`（库层）+ `anyhow`（应用层） | 分层错误策略 |
| 打包 | `.msi` / `.dmg` / `.AppImage` + `.deb` | 各平台原生安装格式 |

---

## 4. 构建与运行

### 环境要求

| 工具 | 版本 | 说明 |
|---|---|---|
| Rust | 1.80+（stable） | 2024 年 7 月稳定版，AI 训练数据覆盖 |
| Node.js | 20 LTS | LTS 线，npm 包兼容性最好 |
| Tauri CLI | 2.x（最新 stable） | `cargo install tauri-cli --version "^2"` |

项目根目录放置 `.rustfmt.toml`（空文件，默认风格）和 `.prettierrc`（`{ "singleQuote": true, "semi": true }`），保证三人格式化一致。

### 常用命令

| 命令 | 用途 |
|---|---|
| `cargo tauri dev` | 开发运行（同时启动 Rust 后端 + React 前端） |
| `cargo tauri build` | 生产构建 |
| `cargo test` | 运行 Rust 侧测试 |
| `cd src-ui && npm run lint` | 前端代码检查 |

入口文件：`src-tauri/src/main.rs`（应用入口）、`src-tauri/src/lib.rs`（Command 注册）、`src-ui/src/main.tsx`（前端入口）。

---

## 5. 功能清单

### v1（必须实现）

| # | 功能 | 要点 |
|---|---|---|
| ① | Feed 订阅与内容呈现 | RSS/Atom/JSON Feed 解析、OPML 导入导出、自动同步、文章列表（已读/未读/搜索/分页） |
| ② | 文章清洗与阅读体验 | Readability 提取、HTML 清洗、Markdown 转换（GFM）、主题切换、字体定制 |
| ③ | AI 摘要 | 可配置语言/详细程度、流式输出、自定义 Prompt 模板 |
| ④ | AI 双语翻译 | 段落级双语对照布局、多段落并发、重试/清除 |
| ⑤ | 笔记与文摘导出 | 单篇 Markdown 笔记、单篇/多篇文摘导出、自定义模板 |

### v2（推迟）

| # | 功能 |
|---|---|
| ⑥ | 标签系统（手动/AI 建议/批量/标签库维护） |
| ⑦ | Token 用量统计（Provider/Model/Agent 维度图表） |
| ⑧ | 多语言 UI 切换与运行日志 |

---

## 6. 编码约定

- 与用户用中文沟通；代码注释与文档用英文。
- 禁用 emoji 于代码注释与文档。
- Markdown 中用 backticks 包裹代码引用。

**Rust 侧：**

- 模块名用 `snake_case`，类型/trait 用 `CamelCase`。
- 数据库迁移不可回退（只增不改），每次迁移需对应测试。
- Tauri Command 函数签名即契约，变更需同步更新前端调用。
- Tauri Command 错误传递：`Result<T, String>`，用 `.map_err(|e| e.to_string())` 在边界转换。前端 `catch` 直接获取字符串。
- 默认数值类型 `i32` / `f64`；仅在平台 API 要求时使用 `usize` / `c_float`。

**React 侧：**

- 组件文件 `*View.tsx`，hooks 文件 `use*.ts`，类型定义 `types.ts`。
- CSS：Tailwind CSS，禁止额外 CSS 框架。
- 组件库：shadcn/ui，按需复制组件到 `src-ui/src/components/ui/`。
- 状态管理：React Context + `useReducer`，不引入 Redux/Zustand。
- v1 仅中文，所有用户可见字符串通过伪 i18n 函数 `t()` 包裹，v2 升级 react-i18next。

**通用：**

- 跨平台路径用 `dirs` crate，禁止硬编码 `/` 或 `\`。
- 异步操作用 `async/await`，禁止阻塞主线程。

---

## 7. 设计决策与实现要点

以下为关键技术决策，每项含方案、理由、实现要点与验证方式。

### 7.1 Readability 文章提取

**方案**：使用 `readability` crate。

**理由**：Mozilla Readability 的 Rust 移植，效果可靠，无需手动实现复杂算法。

**实现要点**：使用 `readability::extractor` 提取文章内容；配合 `scraper` 进行 HTML 清洗和预处理；提取后转换为 Markdown 使用 `comrak`。

**验证方式**：编写单元测试，使用已知 HTML fixture 验证提取结果（标准博客、中文/UTF-8 编码、嵌套表格、畸形标记）；检查提取内容是否包含导航、广告等无关元素。

### 7.2 AI 流式输出 IPC 传递

**方案**：Tauri Event API（非前端直连 HTTP）。

**理由**：原生支持，与 Tauri 框架深度集成；AI 请求统一走 Rust Core，不暴露 API Key 到前端。

**实现要点**：Rust 端 `app_handle.emit("ai-stream", payload)` 发送事件；前端 `listen("ai-stream", callback)` 监听；使用 `UnlistenFn` 管理订阅生命周期（组件挂载时订阅、卸载时取消）。事件数据格式：`{ task_id, content, is_done, agent_type: "summary" | "translation" }`。

**验证方式**：前端能实时接收流式事件，输出完整无数据丢失，任务完成后正确触发 `is_done`。

### 7.3 SQLite Schema 设计

**方案**：自行设计 SQLite schema，功能语义参考原始 Mercury。

**理由**：原始 Mercury 使用 Swift 的 GRDB（macOS 专属），无法直接复用。自行设计可充分利用 SQLite 特性，字段语义保持一致即可。

**实现要点**：WAL 模式 `PRAGMA journal_mode=WAL`；v1 核心表：`feeds`、`entries`、`contents`、`summaries`、`translations`、`notes`；版本化迁移系统（`migrations/` 目录，编号递增）。

**验证方式**：数据库初始化成功，所有表创建正确；迁移系统能正确执行版本升级；数据插入和查询正常。

### 7.4 后台 Feed 同步与并发

**方案**：Rust Tokio 异步任务。

**理由**：高性能、内存安全，充分利用 Rust 并发优势。

**实现要点**：`tokio::spawn` 创建同步任务；`tokio::sync::Semaphore` 控制并发数（默认 5 个）；`tokio::time::interval` 实现定时同步（默认每 30 分钟）；SQLite WAL 模式支持并发读。

**验证方式**：同步任务在后台运行不阻塞 UI；并发数正确限制在 5 以内；定时任务按配置间隔执行。

### 7.5 双语翻译段落对照布局

**方案**：前端 CSS Grid 双栏布局。

**理由**：灵活、响应式，符合现代前端开发实践。

**实现要点**：CSS Grid 双栏布局（原文 + 译文）；Flexbox 处理段落对齐；响应式设计（大屏并排，小屏堆叠）；段落展开/折叠交互；`scroll-snap` 同步滚动。

**验证方式**：双语内容正确并排显示；响应式布局在不同屏幕尺寸下正常；段落对齐和换行正确。

### 7.6 跨平台路径与数据存储

**方案**：Rust `dirs` crate。

**理由**：跨平台标准路径，简单可靠。

**实现要点**：数据库路径 `{data_local_dir}/mercury/mercury.db`；配置路径 `{config_local_dir}/mercury/config.toml`；凭证存储使用 Tauri secure-store 插件。

**验证方式**：在 Windows、macOS、Linux 上正确定位数据目录；数据库文件正确创建和读写；配置文件正确加载和保存。

### 7.7 Feed 格式兼容性

**方案**：使用 `feed-rs` 库。

**理由**：支持 RSS 0.91/0.92/1.0/2.0、Atom 1.0、JSON Feed 全格式。

**实现要点**：`feed_rs::parser::parse` 解析 Feed；解析失败的 Feed 记录错误日志而非崩溃。

**验证方式**：测试各种 Feed 格式；解析失败的 Feed 有明确错误提示；解析结果正确映射到数据模型。

### 7.8 AI 客户端协议

**方案**：OpenAI 兼容协议（`/v1/chat/completions`）。

**理由**：覆盖 DeepSeek/Ollama/vLLM 等所有兼容服务，用户自行配置 baseURL + API Key + model。

**实现要点**：`reqwest` 实现 HTTP 客户端；支持 `stream=true` 流式输出；支持自定义 headers；设置合理超时，避免长时间阻塞。

**验证方式**：能正确调用 OpenAI API；能正确调用 Ollama 本地服务；流式输出正常工作。

---

## 8. 数据库优先规则

SQLite 是唯一事实来源。所有业务状态必须持久化。

**禁止**：React State 保存业务数据、LocalStorage 保存业务数据、内存对象作为唯一状态。

**允许**：UI 状态（如当前选中的标签页）、临时缓存、查询缓存。

所有核心数据最终必须写入数据库。首次启动时自动创建数据库文件和所有表。Feed 同步启动时自动触发一次，后续按配置间隔定时同步。

### 迁移阶段映射

每个阶段的数据库迁移只新增表/列（带 DEFAULT），**绝不修改已有表结构**：

| Migration | 阶段 | 表 |
|---|---|---|
| `001_initial_schema.sql` | Stage 1 | feeds, entries, contents, schema_version |
| `002_fts_search.sql` | Stage 2 | entries_fts（FTS5 虚拟表） |
| `003_agent_tables.sql` | Stage 3 | summaries, translations, providers |
| `004_notes_digest.sql` | Stage 4 | notes, digest_templates |
| `005_tags.sql` | v2 | tags, entry_tags |
| `006_usage_stats.sql` | v2 | usage_stats |

回滚到 Stage 2 时，Stage 3/4 的表留在数据库中无害（空表），不需要回滚 migration。这保证了崩盘恢复时数据库完整无损。

---

## 9. Agent 运行时合约

以下合约与 Mercury 原版行为严格一致，复刻中不得偏离。

### 9.1 统一状态机

所有 Agent（Summary、Translation、Tagging）必须实现统一状态机：

```
Idle  →  Running  →  Succeeded
                  →  Failed
                  →  Cancelled
```

**允许的转换**：`Idle → Running → Succeeded`、`Idle → Running → Failed`、`Idle → Running → Cancelled`。

**禁止的转换**：`Idle → Succeeded`（跳过执行）、`Failed → Running`（直接恢复失败任务）。

重试必须创建新的 Run。

### 9.2 全局执行策略

- 不自动取消进行中的后台任务。
- 取消只能来自用户显式操作。
- 等待队列为 latest-only 替换策略（新请求覆盖旧等待项）。
- 当前每类 Agent 限制：active slot 1 个 + waiting slot 1 个。

### 9.3 Translation 合约

- v1 仅在 Reader 内可用。
- 段落切分粒度固定为 `<p>`、`<ul>`、`<ol>`。
- 可为文章标题/作者合成一个前置段落，保持双语对照中标题对齐。
- 执行模式：每段落有界并发（`tokio::sync::Semaphore`），可配置范围 `1...5`，默认 `3`。
- Translation 结果需持久化到数据库。
- 支持恢复、重试、清除、回到原文的工具栏语义。

### 9.4 Summary 合约

- 用户确认后启用自动摘要。
- 1 秒防抖，串行执行。
- 不自动重试（失败后需用户手动触发）。
- 等待队列 latest-only 替换：新请求覆盖旧等待项，已在执行的摘要不中断。

### 9.5 Prompt 管理

- Prompt 属于资源文件，内置在 `src-tauri/resources/prompts/` 下。
- 内置 Prompt 永远只读。
- 用户可通过文件覆盖或新增 Prompt。
- 禁止在代码中硬编码 Prompt 文本。

---

## 10. AI 使用指南

### 工作流程

1. **先读文档再动手**：优先查阅 `docs/` 下的 INIT.md、决策记录、ADR，理解项目全貌。
2. **Tauri Command 契约先行**：新增或修改前后端交互接口时，先协定 Command 签名，再分别实现 Rust 和 React 侧。
3. **变更后更新本文件**：任何影响项目级约束的变更需同步更新 AGENTS.md。

### 技术选型硬原则（面向 AI 辅助新手）

团队成员为大一新生，全部依赖 AI 辅助编码，无法处理复杂编译器报错。所有技术决策必须遵循：

1. **优先使用 AI 训练数据最丰富的方案**：官方模板、社区主流用法、Tauri 默认配置。偏离主流意味着 AI 生成的代码正确率断崖下降。
2. **编译器报错必须能直指问题位置**：新手能复制错误信息发给 AI，AI 一次修正成功。禁止多 crate 循环引用、多层 trait bound 连锁错误等新手无法定位的架构。
3. **优先单体结构，推迟拆分**：一个 `Cargo.toml`、一个 `package.json`、目录隔离满足需求。crate 拆分是优化手段，不是必需品。
4. **AI 一次生成对的概率 = 一切**：选型不看「技术先进性」，看「AI 见过多少正确样本」。

### Repository 分层（数据库操作必须经过的路径）

```
Tauri Command
    ↓
Service（业务逻辑）
    ↓
Repository（数据访问）
    ↓
SQLite
```

禁止在 Tauri Command 中直接拼接 SQL。所有数据库操作通过 Repository 层封装。

### Rust 并发与异步约束

**应该做：**

- 可变状态指定唯一所有者：`Arc<RwLock<T>>` 或通过 channel（`mpsc`/`tokio::sync`）传递所有权。
- 长生命周期的异步资源（数据库连接池、HTTP client、SSE 流）必须有明确的 struct 所有者。
- 使用 `CancellationToken` 实现结构化取消，确保清理逻辑归属清晰。
- 并发变更在 Debug 和 Release 模式下分别验证。

**不应该做：**

- 不要用 `unsafe` 绕过 `Send` / `Sync` 问题。
- 不要在 Tauri State 中存储非 `Send + Sync` 的类型。
- 不要随意 `tokio::spawn` 无主任务；每个异步任务必须有明确的取消/清理责任人。
- 不要在持有锁的临界区内调用 `.await`（会导致死锁）。

### 测试约束

- 数据库测试默认使用**内存 SQLite**（`:memory:`），仅当测试明确需要磁盘行为时才用临时文件（`tempdir` crate）。
- 磁盘数据库测试清理时删除整个目录，不单独删 `.sqlite` 文件。
- 提供共享测试 fixture（测试用 Feed 数据、HTML 样本、数据库种子），禁止每个测试文件重复造数据。
- 测试命名按行为而非实现：`test_feed_parse_atom_valid`。
- 禁止 `std::thread::sleep` 类时序断言；用 `tokio::time::timeout` 或轮询等待。
- 异步测试使用 `tokio::test`。

### 错误处理与日志

- 库级错误用 `thiserror` 派生，应用级错误用 `anyhow`。
- 日志用 `tracing` crate：Command 入口打 `info!`，数据库操作打 `debug!`，错误打 `error!`。
- SSE 流解析失败打 `warn!` 并重试（网络波动正常），不打 `error!`。
- 禁止 `println!` 或 `eprintln!` 用于日志。

### React 前端约束

- 所有 Tauri `invoke` 调用封装在 `src-ui/src/api/` 目录下，一个模块对应一类 Command 域（`feed.ts`、`agent.ts`、`notes.ts`）。
- Tauri Command 返回的 Rust 类型与前端 TypeScript 类型必须同步定义；考虑用 `ts-rs` crate 自动生成类型。
- 流式输出通过 Tauri Event 推送到前端，不在前端直接建立 HTTP 连接。
- 阅读器渲染：清洗后的 HTML 直接注入 WebView，Markdown 转 HTML 在 Rust 侧完成，前端只负责展示。

### 何时停下询问

以下情况 AI 必须停下并请求人工决策：

1. 引入新 crate 或 npm 包（需评估体积、许可证）
2. 修改已有 Tauri Command 签名（契约变更）
3. 数据库 schema 变更（迁移编号、字段类型）
4. 涉及并发模型变更
5. 性能与安全的权衡选择
6. 新 Agent 类型、新 Provider、新网络协议
7. 任何需要修改 AGENTS.md 本身的变更

---

## 11. 当前进度

| 维度 | 状态 |
|---|---|
| 项目文档 | ✅ INIT.md / 决策记录 / ADR×4 / AGENTS.md |
| Rust 核心 | 🔲 待搭建 Cargo workspace |
| Tauri 壳 | 🔲 待 `cargo tauri init` |
| React 前端 | 🔲 待 `npm create vite` |
| 数据库 Schema | 🔲 待自行设计 |
| CI/CD | 🔲 目标 Day 1 三平台构建 |

---

## 12. 架构决策索引

已归档的架构决策记录见 `docs/adr/`，讨论过程见 `docs/decisions/`：

| ADR | 决策 | 理由摘要 |
|---|---|---|
| 001 | Rust over Swift | Swift on Windows 不成熟；Rust 单进程零 IPC |
| 002 | OpenAI 兼容协议 | 行业事实标准，v1 无需自建多协议抽象 |
| 003 | Tauri 单进程 | 同语言无 IPC 开销，三平台原生 WebView |
| 004 | React + TypeScript | AI 训练数据最丰富，生态最全 |

---

## 13. 分工（按功能模块）

| 功能模块 | 负责人 | Rust 侧 | React 侧 |
|---|---|---|---|
| Feed 与内容管线（①②） | 王康睿 | Feed 解析/同步、OPML、SQLite schema、Readability、Markdown | 订阅源管理、文章列表、阅读器、侧边栏 |
| AI 智能体（③④） | 刘欣慧 | Provider 管理、OpenAI 协议、Summary/Translation Agent | 摘要面板、翻译面板、Agent 配置页 |
| 笔记与基础设施（⑤ + v2: ⑥⑦⑧） | 杜偲妍 | 笔记 CRUD、文摘导出、设置持久化、Tauri Command 契约 | 笔记编辑、导出面板、设置页、通用组件 |

---

## 14. 路线图（四阶段递增交付）

每个阶段产出可运行的产品增量，后一阶段在前一阶段基础上叠加。

### 第一阶段：基础阅读器原型

**目标**：跑通 RSS → 存储 → 列表 → 阅读的最小闭环。

- 搭建项目骨架：Cargo workspace + Tauri 初始化 + React 脚手架 + CI 三平台构建
- SQLite schema 设计与迁移（Feed、Entry、Content）
- Feed 解析（RSS/Atom/JSON Feed）+ 添加/删除/刷新订阅源
- 文章列表（分页、已读/未读标记）
- 基础阅读页（HTML 直出，不含清洗）
- OPML 导入导出

### 第二阶段：阅读体验增强

**目标**：文章内容清洗 + 舒适阅读体验。

- Readability 内容提取管线
- HTML 清洗（`scraper`）+ Markdown 转换（`comrak`，GFM 支持）
- 主题切换（亮色/暗色）+ 自定义字体
- 搜索（标题 + 摘要）
- 离线缓存（已加载文章缓存 SQLite）

### 第三阶段：AI 功能接入

**目标**：OpenAI 兼容协议 + 摘要 + 双语翻译。

- Provider 管理与验证（baseURL + API Key + model 配置）
- OpenAI 兼容协议封装（`/v1/chat/completions` + SSE 流式）
- 文章摘要（可配置语言/详细程度、流式输出）
- 段落级双语翻译（并发控制、重试/清除）
- Agent 配置 UI（Provider/Model/Prompt 定制）

### 第四阶段：信息整理与导出

**目标**：笔记 + 文摘导出 + 打磨发布。

- 单篇 Markdown 笔记编辑
- 单篇/多篇文摘导出（自定义模板）
- 设置页（通用配置/Prompt 模板编辑）
- 键盘快捷键体系（`j/k` 切换、`s` 摘要、`t` 翻译、`n` 笔记）
- 桌面通知（新文章到达）
- 三平台打包（`.msi` / `.dmg` / `.AppImage` + `.deb`）

### 后续（v2）

- ⑥ 标签系统、⑦ Token 用量统计、⑧ 多语言 UI 切换与运行日志

---

## 15. Definition of Done

功能标记为完成必须满足全部条件：

- `cargo build` 编译通过，零 warning
- `cargo test` 全部通过
- `cargo clippy` 零 error
- `cd src-ui && npm run lint` 零 TypeScript error
- 无 panic（所有错误走 Result 传播）
- Tauri Command 契约稳定（签名不随意变更）
- 数据可持久化（重启不丢失）
- 跨平台兼容（三平台行为一致）

否则不得标记完成。

---

## 16. 已知问题与风险

### 当前风险

- 项目 Planning 阶段，尚未开始编码。
- `feed-rs` 对非标准 RSS 的兼容性待验证。
- `readability` crate 对中文页面的提取质量待评估。
- Tauri 在 Linux 上的 WebViewGTK 依赖需额外处理。
- LLM 接口稳定性需实现重试机制（SSE 断连恢复）。

### 近期注意事项

- Rust 端所有 I/O 操作使用异步 API（`tokio`）。
- SQLite WAL 模式需确保正确配置连接池。
- 前端调用 Tauri API 时需处理异常，提供友好的错误提示。
- AI 请求需设置合理超时（建议 120s），避免长时间阻塞。
- 跨平台路径使用 `dirs` crate，避免硬编码路径。
- 文章提取失败时显示原始内容，不崩溃。

### 参考资料

- 原始 Mercury 功能行为参考：`docs/cross-platform-analysis.md`（仅参考功能描述，不参考代码实现）
- 决策讨论全过程：`docs/decisions/`
- Mercury 原版 AGENTS.md：原始仓库根目录
