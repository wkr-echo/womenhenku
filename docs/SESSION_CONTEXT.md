# Session Context — 快速上手文档

> 新 AI 窗口打开后，**先读这个文件**，再读 `AGENTS.md`。

---

## 1. 项目身份

- **项目名**：Platinum（Mercury RSS 阅读器的跨平台复刻）
- **仓库**：`https://github.com/wkr-echo/platinum.git`（分支 `dev`）
- **本地路径**：`/home/wkr/womenhenku`
- **GitHub 推送**：需走代理 `127.0.0.1:7897`

## 2. 技术栈速览

| 层 | 技术 |
|---|---|
| 语言 | Rust 1.80+（stable） |
| 框架 | Tauri 2 |
| 前端 | React + TypeScript + Vite + Tailwind CSS + shadcn/ui |
| 数据库 | SQLite（rusqlite, bundled, WAL 模式） |
| HTTP | reqwest（async） |
| Feed 解析 | feed-rs |
| 文章提取 | readability crate（Mozilla Readability 的 Rust 移植） |
| HTML 清洗 | scraper |
| Markdown | comrak（GFM） |
| AI 协议 | OpenAI 兼容（SSE 流式） |

## 3. 项目目录结构

```
womenhenku/
├── AGENTS.md                  # 项目规范（先读这个）
├── docs/
│   ├── PLAN.md                # 执行计划
│   ├── reference/
│   │   ├── command-contract.md    # Tauri Command 契约
│   │   ├── frontend-state-design.md
│   │   └── mercury-behavior-manual.md  # Mercury 原版行为参考
│   └── adr/                   # 架构决策记录（9 个）
├── src-tauri/                 # Rust 核心
│   ├── src/
│   │   ├── lib.rs             # Tauri Command 注册
│   │   ├── commands.rs        # Command 实现（参数校验 + 错误转换）
│   │   ├── db/                # 数据库层
│   │   │   ├── mod.rs         # 初始化 + 迁移
│   │   │   ├── model.rs       # 数据模型
│   │   │   ├── repository/    # Repository 层
│   │   │   └── migrations/    # SQL 迁移文件
│   │   ├── feed/              # Feed 解析/同步
│   │   │   ├── parser.rs      # feed-rs 封装
│   │   │   └── service.rs     # 订阅管理（async）
│   │   ├── reader/            # 阅读器管线 ⚠️ 最近频繁修改
│   │   │   ├── pipeline.rs    # 7 步管线
│   │   │   └── service.rs     # 缓存 + 编排（PIPELINE_VERSION 在这里）
│   │   ├── agent/             # AI Agent
│   │   │   ├── client.rs      # OpenAI 协议客户端
│   │   │   ├── summary.rs     # 摘要 Agent
│   │   │   ├── translation.rs # 翻译 Agent
│   │   │   └── crypto.rs      # API Key 加密
│   │   └── platform/          # 平台抽象层
│   └── resources/prompts/     # AI Prompt 模板（内置只读）
└── src-ui/                    # React 前端
    ├── src/
    │   ├── api/               # Tauri invoke 封装
    │   ├── components/        # UI 组件（*View.tsx）
    │   ├── contexts/          # React Context
    │   ├── locales/           # i18n（zh.ts, en.ts, 320 keys）
    │   └── lib/               # 工具函数
    └── package.json
```

## 4. 已完成功能（v1 + v2）

- [x] Feed 订阅管理（RSS/Atom/JSON Feed、添加/删除/刷新、OPML 导入导出）
- [x] 文章阅读器（Readability 提取 + 清洗 + Markdown 渲染）
- [x] AI 摘要（流式输出、可配置）
- [x] AI 双语翻译（段落级双语对照、并发 3）
- [x] 笔记与文摘导出（Markdown/HTML）
- [x] Provider 管理（多 Provider、API Key 加密存储）
- [x] 标签系统（AI 推荐 + NLP 回退 + 批量标签）
- [x] Token 用量统计
- [x] i18n（中/英，320 keys，伪 i18n t() → 准备升级 react-i18next）
- [x] Feed 异步化（add_feed/refresh_feed 不再阻塞 UI）
- [x] 相对 URL 解析（/posts/xxx → 完整 URL）
- [x] 收藏文章 + 全部文章视图
- [x] 侧边栏同步状态栏
- [x] 响应式布局（CSS min() 弹性宽度）

## 5. 当前版本与状态

- **PIPELINE_VERSION = 6**（`src-tauri/src/reader/service.rs:31`）
- **最新 tag**：`v2.1.0`（dev 分支）
- **测试**：96 个 Rust 测试全过 | `cargo clippy` 零 warning | 前端 build 通过
- **缓存机制**：`contents` 表存 `readability_version`，只有 `>= PIPELINE_VERSION` 才命中缓存。低于则用缓存的 `raw_html` 重新跑管线。

---

## 6. 最近修复的关键 Bug（重要教训）

### Bug #1：代码段消失 —— 不要给 Readability 加"保护" ⚠️

**症状**：文章中 `<pre><code>` 代码块不显示。

**错误尝试**（4 次无效修复）：
1. 正则 `.*?` → `[\s\S]*?` 修复多行匹配
2. 用 `<p data-preblock="N">` 占位符在 Readability 前"保护" pre 块
3. 修复正则转义、修复双重 `<code>` 嵌套
4. 多次升降 PIPELINE_VERSION

**根因**：Readability **原生就完美保留** `<pre>` 块（含换行和缩进）！"保护"逻辑把 `<pre>` 替换成 `<p>` 占位符，Readability 看到无意义占位文本直接把整个 `<p>` 删了，代码段消失。

**最终正确修复**：**完全移除 pre 块保护代码**，让 `extract()` 直接把原始 HTML 传给 Readability。

**教训**：
- **先测试原生行为再决定是否需要 workaround**
- Readability crate 对 `<pre>`、`<code>` 等语义标签处理得很好
- 管线缓存版本号升级后旧缓存不会自动失效，需手动升版号

### Bug #2：Feed 添加卡死 UI

**根因**：`add_feed`/`refresh_feed` 是同步函数，内部用 `block_on` 阻塞了主线程。

**修复**：全链路异步化（`async fn` + `.await`），移除 `block_on` 和嵌套 tokio runtime。

### Bug #3：翻译跨文章串扰

**根因**：`TranslationState` 只存 `mode`，没绑定 `entryId`。

**修复**：`TranslationState { mode, entryId }` + `showBilingual` 双重检查 + `<ReaderView key={selectedEntry.id}>`。

### Bug #4：Provider 验证失败

**根因**：`max_tokens: 1` 太短某些 API 返回非 2xx。非 2xx 被当 `Err` 抛出。

**修复**：`max_tokens: 1→5`，非 2xx 返回 `Ok(false)` 而非 `Err`。

---

## 7. 阅读器管线（核心数据流）

```
Feed Entry → 抓取 raw_html → 存入 DB
    ↓ 用户打开文章
process_entry():
  Tier 1: cleaned_html 存在 && readability_version >= PIPELINE_VERSION → 直接返回缓存
  Tier 2: raw_html 在 DB → 用缓存 raw_html 重新跑管线
  Tier 3: 抓取 URL → 存 raw_html → 跑管线
    ↓
管线（pipeline.rs）:
  extract(raw_html) → Readability 内容提取
  sanitize(extracted) → HTML 白名单清洗（scraper）
  to_markdown(cleaned) → HTML → Markdown（comrak GFM）
  render(markdown) → Markdown → 注入主题 CSS 的 HTML
    ↓
存入 contents 表（cleaned_html, markdown, rendered_html, readability_version）
    ↓
React ReaderView 纯展示
```

**React 禁止执行**：Readability 提取、Markdown 转换、Feed 解析、HTML 清洗。

---

## 8. 待办事项

| # | 任务 | 说明 |
|---|---|---|
| 1 | Feed 错误信息持久化 | 当前 Feed 添加/刷新失败的错误信息只在 Toast 一闪而过，需持久化 |
| 2 | 相对 URL fallback | `parser.rs` 和 `commands.rs` 已有 `Url::join()` 处理 |

---

## 9. 常用命令

```bash
# 开发运行
cd /home/wkr/womenhenku && cargo tauri dev

# Rust 测试
cd src-tauri && cargo test --lib

# Rust 检查
cd src-tauri && cargo clippy

# 前端构建
cd src-ui && npm run build

# 前端 lint
cd src-ui && npm run lint

# Git（走代理）
git push origin dev
# 代理：127.0.0.1:7897
```

## 10. 数据库位置

- Linux: `~/.local/share/mercury/mercury.db`
- 表结构见 `AGENTS.md` 第 8.3 节

## 11. 避免踩坑清单

1. **不要给 Readability 加"保护"或预处理** —— 原生行为就是正确的
2. **修改管线后必须升 PIPELINE_VERSION** —— 否则旧缓存不会重新处理
3. **Rust 侧所有 I/O 用 async** —— 禁止 `block_on` 阻塞主线程
4. **代码块格式丢失** —— 检查 `to_markdown()` 的正则（`[\s\S]*?` 非贪婪），不是 Readability 的问题
5. **前端流式输出** —— 通过 Tauri Event 推送，不在前端直接建 HTTP 连接
6. **翻译状态** —— 必须绑定 `entryId`，否则跨文章串扰
7. **命令命名** —— 动词开头（`add_feed`、`list_entries`）
8. **日志** —— 用 `tracing`，禁止 `println!`
9. **错误处理** —— 库用 `thiserror`，应用用 `anyhow`，Command 边界 `.map_err(|e| e.to_string())`
