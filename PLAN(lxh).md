# Mercury 跨平台复刻 — 执行计划

> 状态：Planning  
> 版本：v1.2  
> 创建：2026-07-14  
> 基于：INIT.md / 11111.md (v1 scope) / cross-platform-analysis.md / 汇报稿件.pdf

本文件是可执行的工程计划。每阶段产出可运行增量，后一阶段在前一阶段基础上叠加。

---

## 项目背景

Mercury 是一款 macOS 原生、本地优先的 RSS 阅读器，配备高度可定制的 AI 功能。

本项目目标：在保留 Mercury 产品体验的前提下，实现 **Windows / macOS / Linux** 三平台运行。

**技术选型：**
- 语言：**Rust**（跨平台一等公民）
- UI 壳：**Tauri 2**（同语言单进程，零 IPC 开销）
- 前端：**React + TypeScript + TailwindCSS + shadcn/ui**
- 数据库：**SQLite**（`rusqlite`，WAL 模式）
- Feed 解析：`feed-rs`
- 文章提取：`readability` crate（Mozilla Readability Rust 移植）
- HTML 清洗：`scraper` | Markdown：`comrak`（GFM）
- AI 协议：**OpenAI 兼容**（支持 OpenAI / DeepSeek / Ollama / vLLM / OpenRouter）

---

## v1 范围（八项核心功能全部实现）

| 编号 | 功能域 | 说明 |
|---|---|---|
| ① | **Feed 系统** | RSS/Atom/JSON Feed 订阅、OPML 导入导出、自动同步、未读计数 |
| ② | **内容清洗** | Readability 提取、HTML 清洗、Markdown 转换、主题切换、字体定制 |
| ③ | **AI 摘要** | 文章摘要（流式输出、可配语言/详细程度、自定义 Prompt） |
| ④ | **AI 翻译** | 段落级双语翻译（并发、重试、清除、HY-MT2 优化） |
| ⑤ | **多语言与日志调试** | UI 中英文切换、运行时日志、调试面板 |
| ⑥ | **Token 用量统计** | Provider/Model/Agent 级别统计、趋势图表 |
| ⑦ | **笔记与文摘导出** | 单篇 Markdown 笔记、单篇/多篇文摘导出（Markdown/HTML） |
| ⑧ | **标签系统** | 手动标签、按标签筛选、AI 标签建议、标签库管理 |

---

## 第一阶段：基础阅读器原型

> **目标**：跑通 RSS → 存储 → 列表 → 阅读的最小闭环  
> **产出**：可添加订阅源、查看文章列表、打开文章阅读的桌面应用  
> **里程碑 tag**：`v0.1-stage1`

### 1.1 项目骨架搭建

#### 任务 1.1.1：Cargo Workspace + Tauri 初始化

- **核心目标**：搭建 Rust + Tauri + React 的项目骨架，三平台构建通过。
- **操作步骤**：
  1. `cargo init` 创建 workspace，结构：`src-tauri/`（Tauri 壳）、`mercury-core/`（业务逻辑 crate）、`mercury-store/`（数据库 crate）
  2. `cargo tauri init` 在 `src-tauri/` 初始化 Tauri 2 项目
  3. 配置 `Cargo.toml` 依赖：`tauri`、`rusqlite`（features: bundled）、`reqwest`、`feed-rs`、`tokio`、`tracing`、`serde`、`serde_json`、`dirs`、`thiserror`、`anyhow`
- **涉及文件**：`src-tauri/Cargo.toml`、`src-tauri/src/main.rs`、`src-tauri/src/lib.rs`、`src-tauri/tauri.conf.json`
- **关键设计**：
  - `main.rs`：启动 Tauri 应用，初始化 tracing 日志，WAL 模式开启 SQLite
  - `lib.rs`：注册 Tauri Command，声明全局 State（数据库连接）
- **验证标准**：
  - 自动：`cargo build` 三平台编译通过
  - 手动：`cargo tauri dev` 弹出空白窗口

#### 任务 1.1.2：React 脚手架

- **核心目标**：初始化 React + TypeScript 前端，与 Tauri 对接成功。
- **操作步骤**：
  1. 在 `src-ui/` 下 `npm create vite@latest` 初始化 React + TypeScript
  2. 安装 Tailwind CSS + `@tauri-apps/api`
  3. 搭建基础布局：侧边栏（280px）+ 内容区（flex）
- **涉及文件**：`src-ui/package.json`、`src-ui/src/main.tsx`、`src-ui/src/App.tsx`、`src-ui/src/components/Sidebar.tsx`、`src-ui/src/components/ContentArea.tsx`
- **验证标准**：
  - 手动：`cargo tauri dev` 看到两栏布局

#### 任务 1.1.3：CI 三平台构建

- **核心目标**：GitHub Actions 配置，push 触发三平台编译 + 测试。
- **操作步骤**：
  1. 配置 `.github/workflows/ci.yml`：ubuntu-latest、macos-latest、windows-latest
  2. 安装 Rust + Node.js + Tauri CLI + 平台依赖
  3. 步骤：`cargo build` → `cargo test` → `cd src-ui && npm run lint`
- **验证标准**：push 后 CI 三平台全部绿色

---

### 1.2 数据库 Schema

#### 任务 1.2.1：Schema 设计与迁移系统

- **核心目标**：设计 SQLite schema（功能语义参考原始 Mercury，内部结构自行设计），实现版本化迁移系统。
- **操作步骤**：
  1. 设计 v1 核心表：

```sql
-- 001_initial_schema.sql

CREATE TABLE feeds (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    url             TEXT NOT NULL UNIQUE,
    title           TEXT NOT NULL,
    description     TEXT DEFAULT '',
    link            TEXT DEFAULT '',
    feed_type       TEXT DEFAULT 'rss',  -- rss / atom / json
    last_synced_at  TEXT,                -- ISO 8601
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE entries (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id         INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    guid            TEXT NOT NULL,
    title           TEXT NOT NULL DEFAULT '',
    author          TEXT DEFAULT '',
    link            TEXT DEFAULT '',
    summary         TEXT DEFAULT '',
    published_at    TEXT,                -- ISO 8601
    updated_at      TEXT,
    is_read         INTEGER NOT NULL DEFAULT 0,
    is_starred      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(feed_id, guid)
);

CREATE TABLE contents (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id            INTEGER NOT NULL UNIQUE REFERENCES entries(id) ON DELETE CASCADE,
    raw_html            TEXT DEFAULT '',
    cleaned_html        TEXT DEFAULT '',
    cleaned_markdown    TEXT DEFAULT '',
    rendered_html       TEXT DEFAULT '',
    readability_version INTEGER DEFAULT 1,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT
);

CREATE TABLE schema_version (
    version     INTEGER PRIMARY KEY,
    applied_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

  2. 实现 `migrations/` 目录，按编号顺序执行未应用的迁移
  3. 初始化时自动创建数据库文件 + 执行所有迁移
- **涉及文件**：`src-tauri/src/db/mod.rs`（数据库初始化）、`src-tauri/src/db/migration.rs`（迁移引擎）、`src-tauri/src/db/migrations/001_initial_schema.sql`
- **关键设计**：
  - WAL 模式：`PRAGMA journal_mode=WAL;`
  - 外键约束：`PRAGMA foreign_keys=ON;`
  - `contents.rendered_html` 为缓存字段，失效时重建
- **验证标准**：
  - 自动：单元测试验证迁移执行、表创建、CRUD
  - 手动：首次启动后检查 SQLite 文件，确认表结构正确

#### 任务 1.2.2：Repository 层实现

- **核心目标**：实现 Feeds、Entries、Contents 三个 Repository，封装所有 SQL 操作。
- **操作步骤**：
  - `FeedRepository`：insert、find_by_id、find_by_url、find_all、update、delete
  - `EntryRepository`：insert、find_by_id、find_by_feed_id（分页）、mark_read/unread、search
  - `ContentRepository`：insert、find_by_entry_id、update
  - 所有方法返回 `Result<T, RepositoryError>`
- **涉及文件**：`src-tauri/src/db/repository/feed_repo.rs`、`entry_repo.rs`、`content_repo.rs`、`mod.rs`
- **关键设计**：
  - Repository 持有数据库连接引用
  - `RepositoryError` 用 `thiserror` 派生
- **验证标准**：自动：每个 Repository 单元测试（内存 SQLite），覆盖 CRUD + 边界情况

---

### 1.3 Feed 订阅

#### 任务 1.3.1：Feed 解析与抓取

- **核心目标**：实现 Feed 抓取与解析，支持 RSS 0.91/0.92/1.0/2.0、Atom 1.0、JSON Feed。
- **操作步骤**：
  1. `FeedService::fetch_and_parse(url)` → `reqwest::get(url)` → `feed_rs::parser::parse(bytes)`
  2. 将 `feed_rs` 的解析结果映射到 `Feed` 和 `Entry` 模型
  3. 错误处理：网络超时 30s、解析失败记录 warn 日志、HTTP 非 200 返回错误
- **涉及文件**：`src-tauri/src/feed/service.rs`、`src-tauri/src/feed/parser.rs`、`src-tauri/src/feed/model.rs`
- **关键设计**：解析结果去重：按 `entry.guid` 或 `entry.link` 判断是否已存在
- **验证标准**：
  - 自动：单元测试用 fixture RSS/Atom/JSON Feed 文件验证解析正确性
  - 手动：添加真实 RSS 源，确认文章列表正确显示

#### 任务 1.3.2：Feed 管理 CRUD + 同步

- **核心目标**：添加/删除/刷新订阅源，手动触发同步。
- **操作步骤**：
  - Tauri Command：`add_feed(url)` → 抓取 → 解析 → 存入数据库
  - Tauri Command：`remove_feed(id)` → 级联删除 entries + contents
  - Tauri Command：`refresh_feed(id)` → 重新抓取解析 → 增量更新 entries
  - Tauri Command：`refresh_all_feeds()` → 并发刷新（Semaphore 限制 5 并发）
  - Tauri Command：`list_feeds()` → 返回所有 Feed（含未读计数）
- **涉及文件**：`src-tauri/src/feed/commands.rs`，修改 `src-tauri/src/lib.rs`（注册 commands）
- **关键设计**：
  - `add_feed` 先验证 URL 可达性再解析
  - `remove_feed` 使用事务保证一致性
  - 未读计数：`SELECT COUNT(*) FROM entries WHERE feed_id = ? AND is_read = 0`
- **验证标准**：
  - 自动：test_add → 数据入库、test_remove → 级联删除、test_refresh → 增量更新
  - 手动：UI 中添加/删除/刷新订阅源，列表实时更新

#### 任务 1.3.3：文章列表 UI

- **核心目标**：React 文章列表，支持分页、已读/未读标记、点击打开。
- **操作步骤**：
  - Tauri Command：`list_entries(feed_id, page, page_size, filter)` → 返回分页 Entry 列表
  - React 组件：`EntryList` + `EntryItem`
  - 已读/未读视觉区分（加粗标题 vs 普通）
  - 分页：滚动加载更多
- **涉及文件**：`src-ui/src/components/EntryList.tsx`、`EntryItem.tsx`、`src-ui/src/api/feed.ts`（invoke 封装层）
- **关键设计**：列表使用轻量 `EntryListItem`（title, author, published_at, is_read），不含 content
- **验证标准**：手动：添加订阅源后文章列表正确显示，点击打开阅读

---

### 1.4 基础阅读 + OPML

#### 任务 1.4.1：基础阅读页

- **核心目标**：展示文章原始 HTML（不含清洗），基础阅读功能。
- **操作步骤**：
  - Tauri Command：`get_entry_content(entry_id)` → 返回 Content（raw_html）
  - React 组件：`ReaderView`，用 `dangerouslySetInnerHTML` 渲染 HTML
  - 标记已读：打开文章时自动 `mark_read(entry_id)`
- **涉及文件**：`src-ui/src/components/ReaderView.tsx`
- **验证标准**：手动：点击文章，阅读区显示原始 HTML，返回列表已读状态更新

#### 任务 1.4.2：OPML 导入导出

- **核心目标**：OPML 文件导入（批量添加）与导出（备份订阅源）。
- **操作步骤**：
  - Tauri Command：`import_opml(file_path)` → 解析 OPML XML → 逐个 `add_feed`
  - Tauri Command：`export_opml(file_path)` → 序列化为 OPML XML → 写入文件
  - 文件选择使用 Tauri `dialog` plugin
- **涉及文件**：`src-tauri/src/feed/opml.rs`
- **关键设计**：
  - OPML 解析使用 `quick-xml` crate
  - 导入失败不中断：记录失败项，成功项正常添加
- **验证标准**：
  - 自动：用 fixture OPML 文件测试导入/导出往返一致性
  - 手动：导入 OPML → 确认订阅源列表正确 → 导出 → 内容匹配

---

## 第二阶段：阅读体验增强

> **目标**：文章内容清洗 + 舒适阅读体验  
> **里程碑 tag**：`v0.2-stage2`

### 2.1 Readability 管线

#### 任务 2.1.1：Readability 提取

- **核心目标**：使用 `readability` crate 从原始 HTML 提取正文内容。
- **操作步骤**：
  1. 实现 `ReaderPipeline::extract(raw_html)` → `readability::extractor::extract(&mut html, &url)`
  2. 提取结果存入 `contents.cleaned_html`
  3. 错误处理：提取失败时回退到原始 HTML
- **涉及文件**：`src-tauri/src/reader/pipeline.rs`
- **验证标准**：
  - 自动：用多种 HTML fixture（新闻/博客/中文/嵌套表格/畸形标记）验证提取正确性
  - 自动：验证提取后不含导航、广告、侧边栏等无关元素

#### 任务 2.1.2：HTML 清洗 + Markdown 转换

- **核心目标**：清洗提取后的 HTML，转换为 GFM Markdown。
- **操作步骤**：
  - `scraper` 清洗：移除危险标签（script、style），保留安全标签（p, h1-h6, ul, ol, li, a, img, table, pre, code, blockquote, strong, em, del, br）
  - `comrak` 转换 HTML → Markdown（GFM 支持：表格、任务列表、删除线、代码块）
  - `comrak` 转换 Markdown → 渲染 HTML（注入阅读器主题样式）
- **涉及文件**：修改 `src-tauri/src/reader/pipeline.rs`
- **关键设计**：渲染 HTML 缓存 key：`theme_id + entry_id + reader_render_version`
- **验证标准**：自动：验证清洗后不含 script/style，Markdown 转换正确

---

### 2.2 界面优化

#### 任务 2.2.1：主题切换

- **核心目标**：亮色/暗色主题切换，CSS 变量驱动。
- **操作步骤**：
  1. 定义 CSS 变量：`--bg-primary`、`--text-primary`、`--link-color` 等
  2. React Context：`ThemeProvider` 管理当前主题
  3. 持久化：用户选择存入数据库 settings 表
- **涉及文件**：`src-ui/src/styles/themes.css`、`src-ui/src/contexts/ThemeContext.tsx`
- **验证标准**：手动：切换亮色/暗色，阅读区和全局 UI 同步变化，重启后保持

#### 任务 2.2.2：自定义字体

- **核心目标**：支持自定义字体（系统字体）。
- **操作步骤**：
  - Rust 侧：通过系统 API 枚举可用字体（Tauri 插件或系统调用）
  - React 侧：字体选择下拉框，实时预览
- **涉及文件**：`src-tauri/src/platform/font.rs`
- **验证标准**：手动：选择字体后阅读区字体即时变化

---

### 2.3 搜索

#### 任务 2.3.1：全文搜索

- **核心目标**：搜索文章标题 + 摘要，结果高亮。
- **操作步骤**：
  - SQLite FTS5 全文索引：`CREATE VIRTUAL TABLE entries_fts USING fts5(title, summary, content='entries')`
  - Tauri Command：`search_entries(query, page, page_size)`
- **涉及文件**：`src-tauri/src/db/migrations/002_fts_search.sql`
- **验证标准**：
  - 自动：搜索测试 fixture 数据，验证结果排序
  - 手动：搜索关键词，结果正确显示，高亮命中

---

## 第三阶段：AI 功能接入

> **目标**：OpenAI 兼容协议 + 摘要 + 双语翻译 + 标签系统  
> **里程碑 tag**：`v0.3-stage3`

### 3.1 Provider 管理与 OpenAI 协议

#### 任务 3.1.1：Provider 配置与管理

- **核心目标**：用户配置 baseURL + API Key + model，支持多 Provider。
- **操作步骤**：
  - Provider 模型：id, name, base_url, api_key（加密）, default_model, thinking_model
  - Tauri Command：`add_provider`、`list_providers`、`update_provider`、`delete_provider`、`validate_provider`
  - API Key 使用 Tauri secure-store plugin 加密存储
- **涉及文件**：`src-tauri/src/agent/provider/model.rs`、`commands.rs`、`src-ui/src/components/ProviderConfig.tsx`
- **验证标准**：
  - 自动：验证 API 返回 200/401 时 UI 正确提示
  - 手动：配置 Ollama 本地服务 → 验证通过 → 列表显示正常

#### 任务 3.1.2：OpenAI 兼容协议封装

- **核心目标**：封装 `/v1/chat/completions` + SSE 流式，统一 AI 请求入口。
- **操作步骤**：
  - `OpenAIClient`：持有 `reqwest::Client` + provider 配置
  - `chat_completion(messages, model, stream)` → POST `/v1/chat/completions`
  - SSE 流式解析 → 通过 Tauri Event 推送到前端：`app_handle.emit("ai-stream", payload)`
- **涉及文件**：`src-tauri/src/agent/provider/client.rs`、`sse.rs`
- **关键设计**：
  - 超时：连接 30s、请求 120s
  - SSE 解析失败 → `warn!` 并重试
- **验证标准**：
  - 自动：mock HTTP server 测试 SSE 流式解析、断连恢复
  - 手动：配置真实 API，前端收到流式事件

---

### 3.2 Summary Agent

#### 任务 3.2.1：Summary Agent 实现

- **核心目标**：文章摘要生成，可配置语言/详细程度，流式输出。
- **操作步骤**：
  1. `SummaryAgent`：实现统一状态机（Idle → Running → Succeeded/Failed/Cancelled）
  2. 输入：`cleaned_markdown` + 配置（target_language, detail_level）
  3. Prompt 模板：从 `resources/prompts/summary.default.yaml` 加载
  4. 流式结果累积 → 存入 `summaries` 表
- **涉及文件**：`src-tauri/src/agent/summary/agent.rs`、`commands.rs`、`state.rs`、`resources/prompts/summary.default.yaml`、`src-ui/src/components/SummaryPanel.tsx`
- **关键设计**：
  - 1 秒防抖、串行执行、latest-only 队列
  - Failed 不可直接 → Running，需创建新 Run
- **验证标准**：
  - 自动：状态机转换正确性测试、防抖逻辑
  - 手动：打开文章 → 点击摘要 → 流式输出 → 结果保存 → 重启后恢复

---

### 3.3 Translation Agent

#### 任务 3.3.1：Translation Agent 实现

- **核心目标**：段落级双语翻译，并发控制，重试/清除。
- **操作步骤**：
  1. `TranslationAgent`：统一状态机
  2. 段落切分：按 `<p>`、`<ul>`、`<ol>` 切分 `cleaned_html`
  3. 每段落有界并发翻译：Semaphore 限制 1~5（默认 3）
  4. 结果持久化到 `translations` 表
- **涉及文件**：`src-tauri/src/agent/translation/agent.rs`、`segmentation.rs`、`commands.rs`、`resources/prompts/translation.default.yaml`、`src-ui/src/components/TranslationPanel.tsx`
- **关键设计**：
  - 支持：恢复（从已有 translations 继续）、重试失败段落、清除所有翻译
  - HY-MT2 优化模式
- **验证标准**：
  - 自动：段落切分测试、并发控制（Semaphore 上限）、重试逻辑
  - 手动：打开文章 → 翻译 → 双语对照 → 重试失败段落 → 清除

---

### 3.4 标签系统

#### 任务 3.4.1：标签 Schema 与 Repository

- **核心目标**：支持手动标签添加与按标签筛选。
- **操作步骤**：
  1. 新增 `tags` 表 + `entry_tags` 关联表
  2. `TagRepository`：CRUD + 按 entry 查询
  3. Tauri Command：`add_tag`、`remove_tag`、`get_entry_tags`、`filter_by_tag`
- **涉及文件**：`src-tauri/src/db/migrations/003_tags.sql`、`src-tauri/src/db/repository/tag_repo.rs`

#### 任务 3.4.2：AI 标签建议

- **核心目标**：利用 AI 对文章内容分析并推荐标签。
- **操作步骤**：
  - 调用 Summary Agent 的底层 LLM 客户端，传入文章内容 + Prompt
  - 返回标签列表，用户确认后添加
- **验证标准**：手动：打开文章 → 点击"建议标签" → AI 返回标签列表 → 确认添加

#### 任务 3.4.3：标签库管理

- **核心目标**：标签库维护、合并、别名、清理。
- **操作步骤**：
  - Tauri Command：`merge_tags`、`alias_tag`、`cleanup_tags`
  - UI：标签管理页面（增删改查、统计使用频率）
- **涉及文件**：`src-ui/src/components/TagManager.tsx`

---

## 第四阶段：笔记与导出 + 多语言 + Token 统计 + 打磨发布

> **目标**：笔记 + 文摘导出 + 多语言界面 + Token 用量统计 + 打磨发布  
> **里程碑 tag**：`v0.4-stage4`

### 4.1 笔记系统

- **核心目标**：单篇 Markdown 笔记，关联到 Entry。
- **操作步骤**：
  - `Note` 模型：id, entry_id, content（Markdown）, created_at, updated_at
  - Tauri Command：`save_note`、`get_note`、`delete_note`
  - React：Markdown 编辑器（textarea 起步，后续评估 CodeMirror）
- **涉及文件**：`src-tauri/src/db/repository/note_repo.rs`、`src-ui/src/components/NoteEditor.tsx`

### 4.2 文摘导出

- **核心目标**：支持单篇/多篇文摘导出，自定义模板。
- **操作步骤**：
  - 导出格式：Markdown、HTML
  - 单篇导出：当前文章 + 笔记 + 元数据
  - 多篇导出：按日期/订阅源/标签筛选 → 批量渲染
  - 文件保存路径通过 Tauri `dialog` plugin 选择
- **涉及文件**：`src-tauri/src/digest/exporter.rs`、`src-tauri/resources/templates/`

### 4.3 多语言支持

- **核心目标**：UI 中英文切换。
- **操作步骤**：
  - i18n 方案：`react-i18next` + JSON 资源文件
  - 语言资源：`src-ui/src/locales/zh.json`、`en.json`
  - 语言选择：设置页下拉框，持久化到 settings 表
- **涉及文件**：`src-ui/src/locales/`、`src-ui/src/i18n.ts`
- **验证标准**：手动：切换中英文 → 界面文本全部切换

### 4.4 Token 用量统计

- **核心目标**：Provider/Model/Agent 级别 Token 用量统计。
- **操作步骤**：
  - `usage_stats` 表：provider_id, model, agent_type, prompt_tokens, completion_tokens, created_at
  - 每次 AI 调用完成时记录用量
  - UI：统计图表（趋势图、排行、成本估算）
- **涉及文件**：`src-tauri/src/db/migrations/004_usage_stats.sql`、`src-ui/src/components/UsageStats.tsx`

### 4.5 打磨与发布

- **键盘快捷键**：`j/k` 切换文章、`s` 摘要、`t` 翻译、`r` 刷新
- **桌面通知**：同步完成后通知新文章数量
- **三平台打包**：Windows `.msi`、macOS `.dmg`、Linux `.AppImage` + `.deb`
- **CI 自动构建发布**

---

## 架构设计

### 整体架构

```
React UI (src-ui/)
    ↓ invoke() / Events (Tauri IPC)
Tauri Command (src-tauri/src/commands/)
    ↓
Rust Service (src-tauri/src/*/service.rs)
    ↓
Repository (src-tauri/src/db/repository/)
    ↓
SQLite (rusqlite, WAL mode)
```

- 所有业务逻辑位于 Rust 侧
- UI 仅负责展示
- 数据库单进程写入

### Reader 管线（固定不可变）

```
Feed Entry
    ↓
Raw HTML (contents.raw_html)
    ↓
Readability 提取 (contents.cleaned_html)
    ↓
HTML 清洗（scraper）
    ↓
Markdown 转换（comrak）(contents.cleaned_markdown)
    ↓
渲染 HTML（comrak + 主题样式）(contents.rendered_html)
    ↓
Reader View（前端展示）
```

**所有步骤在 Rust Core 完成，React 不执行任何内容处理。**

### Agent 统一状态机

```
                    ┌─────┐
                    │Idle │
                    └──┬──┘
                       │ 用户触发
                       ▼
                   ┌───────┐
          ┌────────│Running│────────┐
          │        └───┬───┘        │
          ▼            ▼            ▼
     ┌─────────┐ ┌──────────┐ ┌───────────┐
     │Succeeded│ │  Failed  │ │ Cancelled │
     └─────────┘ └──────────┘ └───────────┘
```

- Idle → Running：用户触发
- Running → Succeeded：流完成
- Running → Failed：API 错误
- Running → Cancelled：用户取消
- Failed/Cancelled 不可直接 → Running，需创建新 Run

---

## 数据库 Schema 演进

| Migration | 阶段 | 表 |
|---|---|---|
| `001_initial_schema.sql` | Stage 1 | feeds, entries, contents, schema_version |
| `002_fts_search.sql` | Stage 2 | entries_fts（FTS5 虚拟表） |
| `003_tags.sql` | Stage 3 | tags, entry_tags |
| `004_agent_tables.sql` | Stage 3 | summaries, translations, providers |
| `005_usage_stats.sql` | Stage 4 | usage_stats |
| `006_notes.sql` | Stage 4 | notes |
| `007_digest_templates.sql` | Stage 4 | digest_templates |

每个阶段只新增 migration，不修改已有表结构。

---

## 依赖关系与并行开发

```
数据库 Schema 落定（团队评审通过）
        │
        ├─→ 成员 A：Feed 解析 + CRUD + 基础阅读 UI（Stage 1）
        │       │
        │       └─→ Readability + Markdown + 主题切换（Stage 2）
        │
        ├─→ 成员 B：用 mock HTML 开发 OpenAI 协议 + SSE 流式 + 状态机
        │       │
        │       ├─→ 等 Readability 输出后再对接真实文章（Stage 3 — Summary）
        │       └─→ 等真实文章对接后再开发翻译（Stage 3 — Translation）
        │
        └─→ 成员 C：用 mock Entry 开发笔记 CRUD + 标签系统 + 设置页
                │
                ├─→ 对接真实数据后完成笔记/导出（Stage 4 — 笔记）
                ├─→ 开发标签 UI + AI 标签建议（Stage 3 — 标签）
                └─→ i18n 资源 + Token 统计看板（Stage 4）

```

| 阶段 | 里程碑 | 并行策略 |
|---|---|---|
| Stage 1 | Feed + 基础阅读 | 成员 A 主导；成员 B mock AI；成员 C mock 笔记/标签/设置 |
| Stage 2 | Readability + 主题 + 搜索 | 成员 A 主导；成员 B 对接真实文章；成员 C 继续笔记/标签 |
| Stage 3 | AI 摘要 + 翻译 + 标签 | 成员 B 主导；成员 A review；成员 C 开发标签 UI |
| Stage 4 | 笔记 + 导出 + i18n + Token 统计 + 打包 | 成员 C 主导；全员测试与修复 |

---

## 阶段独立性与回滚机制

**三个硬约束：**

1. **数据库迁移只增不改** — 每个阶段新增 migration，不修改已有表
2. **功能模块文件隔离** — 每个阶段在独立目录下开发，禁止修改上一阶段已有文件
3. **Git 里程碑标签** — 每个阶段验收通过后打 tag

| 崩盘阶段 | 回滚操作 |
|---|---|
| Stage 2 | `git reset --hard v0.1-stage1`，删除 `reader/` |
| Stage 3 | `git reset --hard v0.2-stage1`，删除 `agent/` 和 AI Panel 组件 |
| Stage 4 | `git reset --hard v0.3-stage1`，删除 `digest/`、`notes/`、`locales/` |

---

## 风险评估

| 风险 | 严重度 | 缓解措施 |
|---|---|---|
| 非标准 RSS 兼容性 | 中 | `feed-rs` 兜底 + warn 日志 |
| Readability 中文质量 | 中 | 中文 HTML fixture 专项测试 |
| Linux WebViewGTK | 中 | CI 预装 + 文档说明 |
| AI 成本控制 | 低 | 用户自配 Provider，无隐藏费用 |
| 偏离 Mercury 行为 | 中 | 复刻优先原则，对照原版行为测试 |

---

## Definition of Done

1. 所有单元测试通过
2. 手动验收测试通过（按 checklist 逐项确认）
3. Git 里程碑 tag 已打
4. 该阶段 CI 绿色
5. 无未解决的 P0/P1 bug
6. 数据库 migration 可回滚（只增不改）
7. 前端 build 无 error/warning
8. 文档（README 或文件注释）已更新
