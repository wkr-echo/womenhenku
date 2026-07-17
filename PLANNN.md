# Mercury 跨平台复刻 — 执行计划

> 状态：Planning ｜ 创建：2026-07-14 ｜ 基于：AGENTS.md v3（三版整合）

本文件为可执行的工程计划。每阶段产出可运行增量，后一阶段在前一阶段基础上叠加。

---

## 第一阶段：基础阅读器原型

> 目标：跑通 RSS → 存储 → 列表 → 阅读的最小闭环
> 产出：可添加订阅源、查看文章列表、打开文章阅读的桌面应用

### 1.1 项目骨架搭建

#### 任务 1.1.1：Cargo workspace + Tauri 初始化

- **核心目标**：搭建 Rust + Tauri + React 的项目骨架，三平台构建通过。
- **任务详情**：
  - `cargo init` 创建单体 `src-tauri/` 项目（目录隔离：`feed/` `db/` `reader/` `agent/` `digest/`）
  - 在 `src-tauri/` 中 `cargo tauri init` 初始化 Tauri 2 项目
  - 配置 `Cargo.toml` 依赖：`tauri`、`rusqlite`（features: bundled）、`reqwest`、`feed-rs`、`tokio`、`tracing`、`serde`、`serde_json`、`dirs`、`thiserror`、`anyhow`
- **涉及文件**：
  - 新建：`src-tauri/Cargo.toml`、`src-tauri/src/main.rs`、`src-tauri/src/lib.rs`、`src-tauri/tauri.conf.json`
- **关键设计**：
  - `main.rs`：启动 Tauri 应用，初始化 tracing 日志
  - `lib.rs`：注册 Tauri Command，声明全局 State（数据库连接池）
- **验证标准**：
  - 自动：`cargo build` 三平台编译通过（GitHub Actions CI 绿灯）
  - 手动：`cargo tauri dev` 弹出空白窗口

#### 任务 1.1.2：React 脚手架

- **核心目标**：初始化 React + TypeScript 前端，与 Tauri 对接成功。
- **任务详情**：
  - 在 `src-ui/` 下 `npm create vite@latest` 初始化 React + TypeScript 项目
  - 安装 Tailwind CSS + shadcn/ui（按需复制组件）
  - 安装 `@tauri-apps/api`，验证 `invoke()` 能调用 Rust 侧
  - 搭建基础布局：侧边栏（280px）+ 内容区（flex），Tailwind 实现
- **涉及文件**：
  - 新建：`src-ui/package.json`、`src-ui/src/main.tsx`、`src-ui/src/App.tsx`、`src-ui/src/components/Sidebar.tsx`、`src-ui/src/components/ContentArea.tsx`
- **关键设计**：
  - 布局使用 CSS Grid 两栏（侧边栏 280px + 内容区 flex）
  - 侧边栏：订阅源列表占位
  - 内容区：文章列表 + 阅读区切换
- **验证标准**：
  - 自动：`cd src-ui && npm run lint` 零 error
  - 手动：`cargo tauri dev` 看到两栏布局，侧边栏和内容区正常渲染

#### 任务 1.1.3：CI 三平台构建

- **核心目标**：GitHub Actions 配置，push 即触发三平台编译 + 测试。
- **任务详情**：
  - 配置 `.github/workflows/ci.yml`：ubuntu-latest、macos-latest、windows-latest
  - 安装 Rust + Node.js + Tauri CLI + 平台依赖（WebViewGTK 等）
  - 步骤：`cargo build` → `cargo test` → `cd src-ui && npm run lint`
- **涉及文件**：
  - 新建：`.github/workflows/ci.yml`
- **验证标准**：
  - 自动：push 后 CI 三平台全部绿色

---

### 1.2 数据库 Schema

#### 任务 1.2.1：Schema 设计与迁移系统

- **核心目标**：自行设计 SQLite schema（功能语义参考原始 Mercury），实现版本化迁移系统。
- **任务详情**：
  - v1 核心表 DDL：

```sql
-- 001_initial_schema.sql

CREATE TABLE feeds (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    url             TEXT NOT NULL UNIQUE,
    title           TEXT NOT NULL,
    description     TEXT DEFAULT '',
    link            TEXT DEFAULT '',
    feed_type       TEXT DEFAULT 'rss',
    last_synced_at  TEXT,
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
    published_at    TEXT,
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

  - 迁移系统：读取 `migrations/` 目录，按编号顺序执行未应用的迁移
  - 初始化时自动创建数据库文件 + 执行所有迁移
- **涉及文件**：
  - 新建：`src-tauri/src/db/mod.rs`、`src-tauri/src/db/migration.rs`、`src-tauri/src/db/migrations/001_initial_schema.sql`
- **关键设计**：
  - `schema_version` 表记录当前迁移版本号
  - WAL 模式：`PRAGMA journal_mode=WAL;`
  - 外键约束：`PRAGMA foreign_keys=ON;`
  - `contents.rendered_html` 缓存 key：`theme_id + entry_id + reader_render_version`
- **验证标准**：
  - 自动：单元测试验证迁移执行、表创建、CRUD 操作
  - 手动：首次启动后检查 SQLite 文件，确认表结构正确

#### 任务 1.2.2：Repository 层实现（Feeds + Entries + Contents）

- **核心目标**：实现 Feeds、Entries、Contents 三个 Repository，封装所有 SQL 操作。
- **任务详情**：
  - `FeedRepository`：insert、find_by_id、find_by_url、find_all、update、delete
  - `EntryRepository`：insert、find_by_id、find_by_feed_id（分页）、mark_read/unread、search（title + summary）
  - `ContentRepository`：insert、find_by_entry_id、update
  - 所有方法返回 `Result<T, RepositoryError>`
- **涉及文件**：
  - 新建：`src-tauri/src/db/repository/feed_repo.rs`、`entry_repo.rs`、`content_repo.rs`、`mod.rs`
- **关键设计**：
  - Repository 持有 `r2d2::Pool<SqliteConnectionManager>` 引用
  - `RepositoryError` 用 `thiserror` 派生
- **验证标准**：
  - 自动：每个 Repository 的单元测试（内存 SQLite），覆盖 CRUD + 边界情况
  - 自动：`cargo test` 全部通过

---

### 1.3 Feed 订阅

#### 任务 1.3.1：Feed 解析

- **核心目标**：实现 Feed 抓取与解析，支持 RSS 0.91/0.92/1.0/2.0、Atom 1.0、JSON Feed。
- **任务详情**：
  - `FeedService::fetch_and_parse(url)` → `reqwest::get(url)` → `feed_rs::parser::parse(bytes)`
  - 将 `feed_rs` 的解析结果映射到 `Feed` 和 `Entry` 模型
  - 错误处理：网络超时（30s）、解析失败（记录 warn 日志，不崩溃）、HTTP 非 200（返回错误）
- **涉及文件**：
  - 新建：`src-tauri/src/feed/service.rs`、`src-tauri/src/feed/parser.rs`、`src-tauri/src/feed/model.rs`
- **关键设计**：
  - `FeedService` 持有 `FeedRepository` + `EntryRepository` + `reqwest::Client`
  - 解析结果去重：按 `entry.link` 或 `entry.id` 判断是否已存在
- **验证标准**：
  - 自动：单元测试用 fixture RSS/Atom/JSON Feed 文件验证解析正确性
  - 手动：添加真实 RSS 源，确认文章列表正确显示

#### 任务 1.3.2：Feed 管理 CRUD + 同步

- **核心目标**：添加/删除/刷新订阅源，手动触发同步。
- **任务详情**：
  - Tauri Command：`add_feed(url)` → 抓取 → 解析 → 存入数据库 → 返回 Feed
  - Tauri Command：`remove_feed(id)` → 级联删除 entries + contents
  - Tauri Command：`refresh_feed(id)` → 重新抓取解析 → 增量更新 entries
  - Tauri Command：`refresh_all_feeds()` → 并发刷新（Semaphore 限制 5 并发）
  - Tauri Command：`list_feeds()` → 返回所有 Feed（含未读计数）
- **涉及文件**：
  - 新建：`src-tauri/src/feed/commands.rs`
  - 修改：`src-tauri/src/lib.rs`（注册 commands）
- **关键设计**：
  - `add_feed` 先验证 URL 可达性，再解析
  - `remove_feed` 使用事务保证一致性
  - 未读计数：`SELECT COUNT(*) FROM entries WHERE feed_id = ? AND is_read = 0`
- **验证标准**：
  - 自动：测试 add → 数据入库、remove → 级联删除、refresh → 增量更新
  - 手动：UI 中添加/删除/刷新订阅源，列表实时更新

#### 任务 1.3.3：文章列表 UI

- **核心目标**：React 文章列表，支持分页、已读/未读标记、点击打开。
- **任务详情**：
  - Tauri Command：`list_entries(feed_id, page, page_size, filter)` → 返回分页 Entry 列表
  - React 组件：`EntryList`（文章列表）、`EntryItem`（单篇文章行）
  - 已读/未读视觉区分（加粗标题 vs 普通）
  - 分页：滚动加载更多或分页按钮
- **涉及文件**：
  - 新建：`src-ui/src/components/EntryList.tsx`、`EntryItem.tsx`
  - 新建：`src-ui/src/api/feed.ts`（invoke 封装层）
- **关键设计**：
  - 列表使用轻量 `EntryListItem`（title, author, published_at, is_read），不含 content
  - 点击文章触发 `select_entry(id)` → 加载 Content → 切换到阅读区
- **验证标准**：
  - 自动：组件渲染测试（mock invoke 返回值）
  - 手动：添加订阅源后文章列表正确显示，点击打开阅读

---

### 1.4 基础阅读 + OPML

#### 任务 1.4.1：基础阅读页

- **核心目标**：展示文章原始 HTML（不含清洗），基础阅读功能。
- **任务详情**：
  - Tauri Command：`get_entry_content(entry_id)` → 返回 Content（raw_html）
  - React 组件：`ReaderView`（阅读区），用 `dangerouslySetInnerHTML` 渲染 HTML
  - 标记已读：打开文章时自动 `mark_read(entry_id)`
- **涉及文件**：
  - 新建：`src-ui/src/components/ReaderView.tsx`
- **关键设计**：
  - 内容区分为文章列表 + 阅读区，默认显示列表，点击文章后切换到阅读区
  - 阅读区顶部：标题 + 作者 + 日期 + 来源
- **验证标准**：
  - 手动：点击文章，阅读区显示原始 HTML，返回列表已读状态更新

#### 任务 1.4.2：OPML 导入导出

- **核心目标**：OPML 文件导入（批量添加订阅源）与导出（导出当前所有订阅源）。
- **任务详情**：
  - Tauri Command：`import_opml(file_path)` → 解析 OPML XML → 逐个 `add_feed`
  - Tauri Command：`export_opml(file_path)` → 将所有 Feed 序列化为 OPML XML → 写入文件
  - 文件选择使用 Tauri `dialog` plugin
- **涉及文件**：
  - 新建：`src-tauri/src/feed/opml.rs`
- **关键设计**：
  - OPML 解析使用 `quick-xml` crate
  - 导入失败不中断：记录失败项，成功项正常添加
  - 导出路径默认 `{home_dir}/mercury_subscriptions.opml`
- **验证标准**：
  - 自动：用 fixture OPML 文件测试导入/导出往返一致性
  - 手动：导入 OPML 文件 → 确认订阅源列表正确 → 导出 → 新文件内容匹配

---

## 第二阶段：阅读体验增强

> 目标：文章内容清洗 + 舒适阅读体验
> 前置：第一阶段完成

### 2.1 Readability 管线

#### 任务 2.1.1：Readability 提取

- **核心目标**：使用 `readability` crate 从原始 HTML 提取正文内容。
- **任务详情**：
  - 实现 `ReaderPipeline::extract(raw_html)` → `readability::extractor::extract(&mut html, &url)` → 返回 extracted HTML
  - 错误处理：提取失败时回退到原始 HTML
- **涉及文件**：
  - 新建：`src-tauri/src/reader/pipeline.rs`
- **关键设计**：
  - 提取结果存入 `contents.cleaned_html`
  - 记录 `contents.readability_version` 用于缓存失效判断
- **验证标准**：
  - 自动：用多种 HTML fixture 测试提取结果（新闻/博客/中文/嵌套表格/畸形标记）
  - 自动：验证提取后不含导航、广告、侧边栏等无关元素

#### 任务 2.1.2：HTML 清洗 + Markdown 转换

- **核心目标**：清洗提取后的 HTML，转换为 GFM Markdown。
- **任务详情**：
  - `scraper` 清洗：移除危险标签（script、style）、保留安全标签
  - `comrak` 转换 HTML → Markdown（GFM 支持：表格、任务列表、删除线）
  - `comrak` 转换 Markdown → 渲染 HTML（注入阅读器主题样式）
- **涉及文件**：
  - 修改：`src-tauri/src/reader/pipeline.rs`
- **关键设计**：
  - 清洗白名单：p, h1-h6, ul, ol, li, a, img, table, pre, code, blockquote, strong, em, del, br
  - 渲染 HTML 缓存 key：`theme_id + entry_id + reader_render_version`
- **验证标准**：
  - 自动：验证清洗后不含 script/style，Markdown 转换正确，GFM 表格/任务列表正常

---

### 2.2 主题与字体

#### 任务 2.2.1：主题切换

- **核心目标**：亮色/暗色主题切换，CSS 变量驱动。
- **任务详情**：
  - 定义 CSS 变量：`--bg-primary`、`--text-primary`、`--link-color` 等
  - React Context：`ThemeProvider` 管理当前主题
  - 持久化：用户选择存入数据库 settings 表
- **涉及文件**：
  - 新建：`src-ui/src/styles/themes.css`、`src-ui/src/contexts/ThemeContext.tsx`
- **验证标准**：
  - 手动：切换亮色/暗色，阅读区和全局 UI 同步变化，重启后保持

#### 任务 2.2.2：自定义字体

- **核心目标**：支持自定义字体（系统字体 + 用户安装字体）。
- **任务详情**：
  - Rust 侧：通过系统 API 枚举可用字体
  - React 侧：字体选择下拉框，实时预览
- **涉及文件**：
  - 新建：`src-tauri/src/platform/font.rs`
- **验证标准**：
  - 手动：选择字体后阅读区字体即时变化

---

### 2.3 搜索与离线缓存

#### 任务 2.3.1：全文搜索

- **核心目标**：搜索文章标题 + 摘要，结果高亮。
- **任务详情**：
  - SQLite FTS5 全文索引：`CREATE VIRTUAL TABLE entries_fts USING fts5(title, summary, content='entries')`
  - Tauri Command：`search_entries(query, page, page_size)`
- **涉及文件**：
  - 修改：`src-tauri/src/db/migrations/002_fts_search.sql`
- **验证标准**：
  - 自动：搜索测试 fixture 数据，验证中文分词和结果排序
  - 手动：搜索关键词，结果正确显示，高亮命中

#### 任务 2.3.2：离线缓存

- **核心目标**：已加载文章缓存 SQLite，断网可阅读。
- **任务详情**：
  - 文章首次加载时，`contents` 表已缓存所有处理结果
  - 阅读时直接读数据库，不重新请求原始 URL
- **验证标准**：
  - 手动：断网后已加载文章仍可正常阅读

---

## 第三阶段：AI 功能接入

> 目标：OpenAI 兼容协议 + 摘要 + 双语翻译
> 前置：第二阶段完成（Readability 输出可用）

### 3.1 Provider 管理与 OpenAI 协议

#### 任务 3.1.1：Provider 配置与管理

- **核心目标**：用户配置 baseURL + API Key + model，支持多 Provider。
- **任务详情**：
  - Provider 模型：id, name, base_url, api_key（加密存储）, default_model, thinking_model, created_at
  - Tauri Command：`add_provider`、`list_providers`、`update_provider`、`delete_provider`、`validate_provider(base_url, api_key)`
  - API Key 加密存储使用 Tauri secure-store plugin
- **涉及文件**：
  - 新建：`src-tauri/src/agent/provider/model.rs`、`commands.rs`
  - 新建：`src-ui/src/components/ProviderConfig.tsx`
- **关键设计**：
  - Provider 验证：发送最小请求 `{model, messages: [{role:"user", content:"hi"}], max_tokens:1}` → 检查 HTTP 200
- **验证标准**：
  - 自动：验证 API 返回 200/401 时 UI 正确提示
  - 手动：配置 Ollama 本地服务 → 验证通过 → 列表显示正常

#### 任务 3.1.2：OpenAI 兼容协议封装

- **核心目标**：封装 `/v1/chat/completions` + SSE 流式，统一 AI 请求入口。
- **任务详情**：
  - `OpenAIClient`：持有 `reqwest::Client` + provider 配置
  - `chat_completion(messages, model, stream)` → POST `/v1/chat/completions`
  - SSE 流式解析：`data: {"choices":[{"delta":{"content":"..."}}]}` → `Stream<Item=String>`
  - 流式通过 Tauri Event 推送到前端：`app_handle.emit("ai-stream", {task_id, content, is_done, agent_type})`
- **涉及文件**：
  - 新建：`src-tauri/src/agent/provider/client.rs`、`sse.rs`
- **关键设计**：
  - `overrideBaseURL + proxyPath` 保留 provider 路径段，兼容代理转发
  - 超时：连接 30s、请求 120s
  - SSE 解析失败 → `warn!` 并重试，不打 `error!`（网络波动正常）
- **验证标准**：
  - 自动：mock HTTP server 测试 SSE 流式解析、断连恢复
  - 自动：测试非 200 响应、超时、畸形 SSE 数据的错误处理
  - 手动：配置真实 API，前端收到流式事件 `ai-stream`

---

### 3.2 Summary Agent

#### 任务 3.2.1：Summary Agent 实现

- **核心目标**：文章摘要生成，可配置语言/详细程度，流式输出。
- **任务详情**：
  - `SummaryAgent`：实现 Agent 统一状态机（Idle → Running → Succeeded/Failed/Cancelled）
  - 输入：`cleaned_markdown` + 配置（target_language, detail_level）
  - Prompt 模板：从 `resources/prompts/summary.default.yaml` 加载，用户可覆盖
  - 流式结果累积 → 存入 `summaries` 表
  - 1 秒防抖、串行执行、不自动重试、latest-only 队列
- **涉及文件**：
  - 新建：`src-tauri/src/agent/summary/agent.rs`、`commands.rs`、`state.rs`
  - 新建：`src-tauri/resources/prompts/summary.default.yaml`
  - 新建：`src-ui/src/components/SummaryPanel.tsx`
- **关键设计**：
  - 状态机转换：Idle → Running（用户触发）→ Succeeded（流完成）/ Failed（API 错误）/ Cancelled（用户取消）
  - Failed 不可直接 → Running，需创建新 Run
  - 配置 key：`Agent.Summary.DefaultTargetLanguage`、`Agent.Summary.DefaultDetailLevel`、`Agent.Summary.PrimaryModelId`
- **验证标准**：
  - 自动：状态机转换正确性测试、防抖逻辑、队列 latest-only 替换
  - 手动：打开文章 → 点击摘要 → 流式输出 → 结果保存 → 重启后恢复

---

### 3.3 Translation Agent

#### 任务 3.3.1：Translation Agent 实现

- **核心目标**：段落级双语翻译，并发控制，重试/清除。
- **任务详情**：
  - `TranslationAgent`：统一状态机
  - 段落切分：按 `<p>`、`<ul>`、`<ol>` 切分 `cleaned_html`
  - 合成前置段落 `seg_meta_title_author` 保持标题对齐
  - 每段落有界并发翻译：`Semaphore` 限制 1~5（默认 3）
  - 结果持久化到 `translations` 表：`{entry_id, segment_index, source_text, translated_text, status}`
- **涉及文件**：
  - 新建：`src-tauri/src/agent/translation/agent.rs`、`segmentation.rs`、`commands.rs`
  - 新建：`src-tauri/resources/prompts/translation.default.yaml`
  - 新建：`src-ui/src/components/TranslationPanel.tsx`（CSS Grid 双栏布局）
- **关键设计**：
  - 配置文件 key：`Agent.Translation.DefaultTargetLanguage`、`Agent.Translation.concurrencyDegree`（1~5，default 3）
  - 支持：恢复（从已有 translations 继续）、重试失败段落、清除所有翻译、回到原文
- **验证标准**：
  - 自动：段落切分测试（p/ul/ol）、并发控制（Semaphore 上限）、重试逻辑
  - 手动：打开文章 → 翻译 → 双语对照显示 → 重试失败段落 → 清除 → 回到原文

---

## 第四阶段：信息整理与导出

> 目标：笔记 + 文摘导出 + 打磨发布
> 前置：第三阶段完成

### 4.1 笔记系统

#### 任务 4.1.1：笔记 CRUD

- **核心目标**：单篇 Markdown 笔记编辑，关联到 Entry。
- **任务详情**：
  - `Note` 模型：id, entry_id, content（Markdown）, created_at, updated_at
  - Tauri Command：`save_note(entry_id, content)`、`get_note(entry_id)`、`delete_note(id)`
  - React：Markdown 编辑器先用 `textarea`，Stage 4 评估是否需语法高亮（Monaco/CodeMirror 为重型依赖，推迟决策）
- **涉及文件**：
  - 新建：`src-tauri/src/db/repository/note_repo.rs`
  - 新建：`src-ui/src/components/NoteEditor.tsx`
- **验证标准**：
  - 自动：CRUD 测试
  - 手动：打开文章 → 编辑笔记 → 保存 → 重启后恢复

---

### 4.2 文摘导出

#### 任务 4.2.1：单篇/多篇文摘导出

- **核心目标**：支持单篇/多篇文摘导出，自定义模板。
- **任务详情**：
  - 导出格式：Markdown、HTML、纯文本
  - 模板引擎：推迟到 Stage 4 决策（`handlebars` 和 `tera` 均可，选 AI 生成模板一次通过率更高者）
  - 单篇导出：当前文章 + 笔记 + 元数据
  - 多篇导出：按日期/订阅源/标签筛选 → 批量渲染
  - 文件保存路径通过 Tauri `dialog` plugin 选择
- **涉及文件**：
  - 新建：`src-tauri/src/digest/exporter.rs`、`template.rs`
  - 新建：`src-tauri/resources/templates/`
- **验证标准**：
  - 自动：模板渲染正确性测试、多篇导出内容完整性
  - 手动：单篇导出 Markdown → 文件内容正确 → 多篇导出 → 格式正确

---

### 4.3 打磨与发布

#### 任务 4.3.1：键盘快捷键

- **核心目标**：全局快捷键体系。
- **任务详情**：
  - `j/k`：上下切换文章、`s`：触发摘要、`t`：触发翻译、`n`：打开笔记、`r`：刷新订阅源
  - 使用 Tauri global shortcut plugin 或 React `useKeyboard`
- **验证标准**：
  - 手动：各快捷键功能正确

#### 任务 4.3.2：桌面通知

- **核心目标**：新文章到达时系统通知。
- **任务详情**：
  - Tauri notification plugin
  - 同步完成后检查新增文章数 → `app_handle.notification().send("新文章", "{count} 篇新文章")`
- **验证标准**：
  - 手动：添加新订阅源 → 同步后有新文章 → 系统通知弹出

#### 任务 4.3.3：三平台打包

- **核心目标**：各平台原生安装包。
- **任务详情**：
  - Windows：`.msi`（WiX Toolset）
  - macOS：`.dmg`
  - Linux：`.AppImage` + `.deb`
  - CI 自动构建发布
- **验证标准**：
  - 手动：三平台安装 → 运行 → 功能正常

#### 任务 4.3.4：设置页

- **核心目标**：集中设置入口。
- **任务详情**：
  - Provider 配置、Model 选择、Agent 参数（语言/详细程度/并发度）
  - Prompt 模板编辑（内置只读，用户可覆盖）
  - 通用：主题、字体、同步间隔
- **涉及文件**：
  - 新建：`src-ui/src/components/SettingsPage.tsx`
- **验证标准**：
  - 手动：修改设置 → 刷新页面 → 设置保持

---

## 依赖关系与实际运行方式

### 数据库 Schema（硬阻塞，全员等待）

数据库 Schema 是整个项目唯一不可并行的环节。由王康睿主导设计（功能语义参考原始 Mercury，内部结构自行设计），三人共同评审确认后，各自基于 Schema 独立推进。

### 实际运行模型（并行优先，mock 先行）

阶段是里程碑标签，不意味串行等待。各模块只要拿到数据库 Schema 就可以用 mock 数据独立开发：

```
数据库 Schema 落定（三人一起）
        │
        ├─→ 王康睿：Feed 解析 + CRUD + 基础阅读 UI（Stage 1）
        │       │
        │       └─→ Readability + Markdown + 主题（Stage 2）
        │
        ├─→ 刘欣慧：用 mock HTML 开发 OpenAI 协议 + SSE 流式 + 状态机
        │       │
        │       └─→ 等 Readability 输出后再对接真实文章（Stage 3）
        │
        └─→ 杜偲妍：用 mock Entry 开发笔记 CRUD + 设置页 + 通用组件
                │
                └─→ 等 Entry 数据就绪后对接真实数据（Stage 4）
```

| 阶段 | 里程碑产物 | 主导负责人 | 并行情况 |
|---|---|---|---|
| Stage 1 | 可添加订阅源、查看文章列表与阅读 | 王康睿 | 刘欣慧 mock 开发 AI 协议；杜偲妍 mock 开发笔记 + 通用组件 |
| Stage 2 | Readability 清洗 + 主题 + 搜索 | 王康睿 | 刘欣慧对接真实文章；杜偲妍继续笔记/设置开发 |
| Stage 3 | 摘要 + 双语翻译 | 刘欣慧 | 王康睿支援 review；杜偲妍继续导出/打包 |
| Stage 4 | 笔记 + 文摘导出 + 三平台打包 | 杜偲妍 | 王康睿 + 刘欣慧支援测试与修复 |

唯一串行依赖：Readability 输出 → AI Agent 的真实文章输入（刘欣慧等王康睿），可用 mock 绕过。

---

## 阶段独立性与回滚机制

每个阶段崩盘可回到上一阶段重新开始，不回退到项目起点。

### 三个硬约束保证独立性

**1. 数据库迁移只增不改**

每个阶段新增 migration 只创建新表/新列（带 DEFAULT），绝不修改已有表结构：

| Migration | 阶段 | 表 |
|---|---|---|
| `001_initial_schema.sql` | Stage 1 | feeds, entries, contents, schema_version |
| `002_fts_search.sql` | Stage 2 | entries_fts（FTS5 虚拟表） |
| `003_agent_tables.sql` | Stage 3 | summaries, translations, providers |
| `004_notes_digest.sql` | Stage 4 | notes, digest_templates |

回滚时后续阶段的表留在数据库中无害（空表），不需要回滚 migration，数据不丢失。

**2. 功能模块文件隔离**

每个阶段的新代码在独立目录下，禁止修改上一阶段的已有文件：

```
Stage 1: src-tauri/src/feed/     src-tauri/src/db/
Stage 2: src-tauri/src/reader/                            ← 不改 feed/, db/
Stage 3: src-tauri/src/agent/                             ← 不改 reader/, feed/
Stage 4: src-tauri/src/digest/   src-tauri/src/notes/     ← 不改 agent/, reader/
```

前端同理：

```
Stage 1: EntryList.tsx, ReaderView.tsx, Sidebar.tsx
Stage 2: CleanedReaderView.tsx  ← 新建，不改 ReaderView.tsx
Stage 3: SummaryPanel.tsx, TranslationPanel.tsx
Stage 4: NoteEditor.tsx, SettingsPage.tsx
```

关键规则：如需增强上一阶段功能，**新建文件而非原地修改**。Stage 2 用 `CleanedReaderView.tsx`（通过配置开关切换），不直接改 `ReaderView.tsx`。Stage 1 的基础阅读路径永远可用，作为降级兜底。

**3. Git 里程碑标签**

每个阶段验收通过后立即打 tag：

| Tag | 阶段 | 产物 |
|---|---|---|
| `v0.1-stage1` | 基础阅读器原型 | 可添加订阅源、列表、阅读 |
| `v0.2-stage2` | 阅读体验增强 | Readability + 主题 + 搜索 |
| `v0.3-stage3` | AI 功能接入 | 摘要 + 双语翻译 |
| `v0.4-stage4` | 信息整理与导出 | 笔记 + 文摘 + 打包 |

### 各阶段崩盘回滚方案

| 崩盘阶段 | 回滚操作 | 影响范围 |
|---|---|---|
| Stage 2 | `git reset --hard v0.1-stage1`，删除 `src-tauri/src/reader/`、`CleanedReaderView.tsx` | 仅丢失 Stage 2 代码 |
| Stage 3 | `git reset --hard v0.2-stage1`，删除 `src-tauri/src/agent/`、AI Panel 组件 | Reader 管线、Feed 解析完好 |
| Stage 4 | `git reset --hard v0.3-stage1`，删除 `src-tauri/src/digest/`、`src-tauri/src/notes/` | Agent 功能完好 |

**Stage 1 崩盘**：项目刚起步，直接 `git reset --hard` 回到 INIT 状态重建，无历史负担。

---

## 核心模块职责清单

| 模块 | 职责 |
|---|---|
| Feed Module | Feed 管理、Feed 同步、OPML 导入导出 |
| Reader Module | 内容提取（Readability）、内容清洗（scraper）、Markdown 转换（comrak） |
| Agent Runtime | Summary Agent、Translation Agent（v2: Tagging Agent） |
| Notes Module | Markdown 笔记 CRUD、文摘管理 |
| Export Module | Markdown 导出、文摘导出 |
| Settings Module | Provider 配置、Prompt 配置、同步配置、主题/字体 |

---

## 风险评估

| 风险 | 严重度 | 缓解措施 |
|---|---|---|
| 非标准 RSS 兼容性 | 中 | `feed-rs` 兜底 + warn 日志 |
| Readability 中文提取质量 | 中 | 中文 HTML fixture 专项测试 |
| Linux WebViewGTK 依赖 | 中 | CI 预装 + 文档说明 |
| 偏离 Mercury 功能行为 | 中 | 复刻优先原则，对照原版行为测试 |
| AI 生成代码偏离项目约束 | 高 | AGENTS.md 硬约束 + ADR 卡控 + Code Review |

---

## 后续阶段（v2 — 上交版本用，杜偲妍负责）

> 以下功能在 v1 四阶段完成后追加。最终上交版本含全部八项功能。

### 标签系统（⑥）

- 手动标签添加与按标签筛选
- AI 标签建议（调用 LLM 分析文章内容推荐标签）
- 标签库维护：合并、别名、清理
- 新增 migration `005_tags.sql`：tags、entry_tags 表

### Token 用量统计（⑦）

- 新增 `usage_stats` 表：provider_id, model, agent_type, prompt_tokens, completion_tokens, created_at
- 每次 AI 调用完成时记录用量
- UI：统计图表（趋势图、排行、成本估算）
- 新增 migration `006_usage_stats.sql`

### 多语言 UI 切换（⑧）

- i18n 方案：`react-i18next` + JSON 资源文件
- 语言资源：`src-ui/src/locales/zh.json`、`en.json`
- 语言选择：设置页下拉框，持久化到 settings 表
