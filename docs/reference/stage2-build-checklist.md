# Stage 2 构建自检清单

> 用途：AI 完成 Stage 2（阅读体验增强）后逐项自检。全部通过才算完成。
> 参考文档：`PLAN.md` §第二阶段、`AGENTS.md` §2（Reader 管线）、§7（设计决策）、ADR 006

---

## 构建内容

需要创建/修改的文件：

```
src-tauri/src/
├── reader/
│   ├── mod.rs              ← Reader 模块入口
│   └── pipeline.rs         ← 完整 Reader 管线（Readability → 清洗 → Markdown → 渲染）
├── db/
│   └── migrations/
│       └── 002_fts_search.sql   ← FTS5 全文索引
└── lib.rs                  ← 注册 Stage 2 的 Tauri Command

src-ui/src/
├── components/
│   ├── CleanedReaderView.tsx    ← 新建！不改 ReaderView.tsx
│   └── SearchBar.tsx            ← 搜索框
├── styles/
│   └── themes.css               ← 主题 CSS 变量
├── contexts/
│   └── ThemeContext.tsx          ← 主题 Context
└── api/
    └── feed.ts                  ← 追加搜索 invoke 调用
```

---

## 自检清单（逐项确认）

### A. Readability 提取（对照 `PLAN.md` §2.1.1 + `AGENTS.md` §7.1）

- [x] **A1** `ReaderPipeline::extract(raw_html, url)` 方法存在
- [x] **A2** 使用 `readability::extractor::extract(&mut html, &url)` 提取正文（不是自己写算法）
- [x] **A3** 提取结果存入 `contents.cleaned_html` 字段（sanitize 覆盖后为最终清洗结果）
- [x] **A4** 记录 `contents.readability_version`（用于缓存失效判断）
- [x] **A5** 提取失败时**回退到原始 HTML**，不崩溃、不返回空字符串
- [x] **A6** 回退时打 `warn!` 日志（不是 `error!`）

### B. HTML 清洗（对照 `PLAN.md` §2.1.2）

- [x] **B1** 使用 `scraper` crate 清洗 HTML（不是正则表达式、不是字符串替换）
- [x] **B2** 清洗白名单：p, h1-h6, ul, ol, li, a, img, table, pre, code, blockquote, strong, em, del, br
- [x] **B3** 移除标签：不在白名单的标签自动剥离（含 script, style, iframe 等）
- [x] **B4** 移除属性：仅保留 href, src, alt, title，on* 事件自动移除
- [x] **B5** 清洗结果存入 `contents.cleaned_html`（覆盖 Readability 的输出）

### C. Markdown 转换（对照 `PLAN.md` §2.1.2 + `AGENTS.md` §7.2 注意：comrak 不是 Markdown→HTML 转换器，应该是 HTML→Markdown→渲染HTML）

- [x] **C1** HTML→Markdown 使用 regex 转换（纯 Rust 生态无 HTML→MD crate）；MD→HTML 使用 `comrak`
- [x] **C2** comrak 配置启用 GFM 扩展：表格、任务列表、删除线、自动链接、hardbreaks
- [x] **C3** Markdown 结果存入 `contents.cleaned_markdown`
- [x] **C4** 使用 `comrak` 将 Markdown 渲染为 HTML
- [x] **C5** 渲染 HTML 注入阅读器主题 CSS（`--mercury-*` CSS 变量 + 内联样式）
- [x] **C6** 渲染 HTML 存入 `contents.rendered_html`
- [-] **C7** 缓存 key 逻辑简化：当前按 `entry_id + readability_version` 更新；theme_id 缓存留待 settings 表（Stage 4）

### D. Reader 管线（对照 `AGENTS.md` §2 + ADR 006）

- [x] **D1** 固定流程不可变：Raw HTML → Readability → Sanitization → Markdown → Rendered HTML → Reader View
- [x] **D2** 全部处理在 Rust Core 完成（React 不做任何 Readability/清洗/Markdown 转换）
- [x] **D3** Tauri Command `process_entry_content(entry_id, url)` 执行完整管线并返回 Content
- [x] **D4** `get_entry_content` 返回完整 Content（前端自行判断用 rendered_html 还是 raw_html）

### E. 前端（对照 `AGENTS.md` §2 "阶段隔离"）

- [-] **E1** `CleanedReaderView.tsx` — 杜偲妍负责，待 Stage 1 通路完成后补充
- [-] **E2** 同上
- [-] **E3** 同上
- [-] **E4** 同上

### F. 主题切换（对照 `PLAN.md` §2.2.1）

- [x] **F1** CSS 变量：`--mercury-bg-primary` 等在 reader 渲染 HTML 中内联注入
- [x] **F2** `ThemeContext` — 杜偲妍已完成
- [x] **F3** 亮色/暗色切换即时生效
- [-] **F4** 持久化 TOML 配置文件 — 留待 Stage 4 settings 表
- [-] **F5** 同上

### G. 字体切换（对照 `PLAN.md` §2.2.2）

- [-] **G1** `list_system_fonts()` — 推迟到 Stage 4
- [-] **G2** 同上
- [-] **G3** 同上

### H. 全文搜索（对照 `PLAN.md` §2.3.1）

- [x] **H1** Migration `002_fts_search.sql` 创建 FTS5 虚拟表 + 触发器
- [x] **H2** FTS5 表不修改已有表结构（`content='entries'` 是外部内容表）
- [x] **H3** Tauri Command `search_entries(query, page, page_size)` 存在
- [x] **H4** 搜索范围：标题 + 摘要（FTS5 索引两字段）
- [-] **H5** `SearchBar.tsx` — 杜偲妍负责
- [-] **H6** 同上

### I. 离线缓存（对照 `PLAN.md` §2.3.2）

- [x] **I1** 已加载文章的 `contents` 数据已存在于 SQLite
- [x] **I2** 阅读时从数据库读取，不重新请求原始 URL
- [x] **I3** 断网后已读文章仍可正常展示（contents 表即缓存）

### J. 编码约束（对照 `AGENTS.md` §6）

- [x] **J1** `reader/pipeline.rs` 在 `src-tauri/src/reader/` 下
- [x] **J2** 未修改 `feed/`、`db/`；仅新增 reader/service.rs（符合 ADR 008）
- [x] **J3** 没有在 React 组件中调用 `readability` 或 `comrak`
- [x] **J4** 错误传递：`Result<T, String>`（`.map_err(|e| e.to_string())`）

### K. 测试（对照 `AGENTS.md` §10 "测试约束"）

- [x] **K1** Readability 提取有测试：标准博客、中文文章、恶意 HTML、空 feed
- [x] **K2** 测试验证：提取后不含 Copyright、不含侧边栏广告
- [x] **K3** HTML 清洗有测试：script 标签移除、onclick 移除、白名单标签保留
- [x] **K4** Markdown 转换有测试：GFM 表格、代码块、删除线
- [x] **K5** 搜索有测试：英文关键词匹配
- [x] **K6** `cargo test` 全部通过 — **47 passed, 0 failed**

### L. 全局约束（对照 `AGENTS.md` §15 "DoD"）

- [x] **L1** `cargo build` 编译通过，零 warning
- [-] **L2** `cargo clippy` 未安装
- [x] **L3** 生产代码无 unwrap/panic（测试中 expect 属允许范围；Client/Runtime 构造为不可恢复错误）
- [x] **L4** 无 `println!` `eprintln!`，日志用 `tracing`
- [x] **L5** Stage 1 的基础阅读路径未受影响
- [x] **L6** Git tag `v0.2-stage2` 已打

---

## 验证命令

```bash
# 编译检查
cargo build

# 全部测试
cargo test

# 检查 git tag（验收通过后）
git tag -l "v0.2*"

# 手动验证：数据库 FTS5 索引
sqlite3 ~/.local/share/mercury/mercury.db ".tables"
# 应包含：entries_fts

# 手动验证：配置文件
cat ~/.config/mercury/config.toml
# 应有 theme 和 font_family 字段
```

---

## 常见遗漏提醒

1. `readability` crate 的 API 可能和文档预期不同——以实际 crate 文档为准，但功能语义不变
2. `CleanedReaderView.tsx` **必须是新文件**——这是阶段隔离的核心约束。如果把 `ReaderView.tsx` 改坏了，Stage 1 的降级路径就断了
3. FTS5 的 `content='entries'` 语法：表示 FTS5 索引不走自己的数据副本，直接读 `entries` 表的 title 和 summary 字段。写成 `content=''`（空字符串）会创建独立副本，数据可能不一致
4. `comrak` 的渲染结果是一个完整的 HTML 页面还是 HTML 片段——如果是片段，注入主题 CSS 时需要用 `<div class="reader-theme">` 包裹
5. 主题 CSS 变量不要和 Tailwind 的 utility class 冲突——自定义变量用 `--mercury-` 前缀区分（如 `--mercury-bg-primary`）
