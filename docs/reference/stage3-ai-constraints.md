# Stage 3 AI 开发约束文档

> 给刘欣慧的 AI 助手 — 违反任一条，整个任务作废。

---

## 允许修改的文件（白名单）

| 允许 | 说明 |
|---|---|
| `src-tauri/src/agent/` | **新建整个目录**，所有 AI 代码放这里 |
| `src-tauri/src/db/migrations/003_agent_tables.sql` | **新建**，providers + summaries + translations 表 |
| `src-tauri/src/lib.rs` | **仅追加**：`pub mod agent;` 和 `invoke_handler` 追加新命令 |
| `src-tauri/Cargo.toml` | **仅追加**新依赖，不删不改已有项 |
| `src-tauri/capabilities/default.json` | **仅追加**新权限 |
| `src-tauri/resources/prompts/` | **新建目录**，放 Prompt 模板 |
| `src-ui/src/api/agent.ts` | **新建**，Agent 相关的 invoke 封装 |
| `src-ui/src/contexts/AppContext.tsx` | **仅追加**新的 Action 类型和 case，不改已有 |
| `src-ui/src/components/SummaryPanelView.tsx` | 可修改，接真实后端 |
| `src-ui/src/components/TranslationPanelView.tsx` | 可修改，接真实后端 |
| `src-ui/src/components/SettingsPageView.tsx` | 可修改，Provider 配置接真实后端 |

## 绝对禁止修改的文件

| 禁止 | 后果 |
|---|---|
| `src-tauri/src/feed/` 目录下任何文件 | 合并拒绝 |
| `src-tauri/src/db/` 下除 migrations 外任何文件 | 合并拒绝 |
| `src-tauri/src/db/migrations/001_*.sql` `002_*.sql` | 合并拒绝 |
| `src-tauri/src/reader/` 目录下任何文件 | 合并拒绝 |
| `src-tauri/src/commands.rs` | 合并拒绝（所有业务逻辑在 agent/ 里写） |
| `src-ui/src/api/feed.ts` | 合并拒绝 |
| `src-ui/src/components/EntryListView.tsx` | 合并拒绝 |
| `src-ui/src/components/ReaderView.tsx` | 合并拒绝 |
| `src-ui/src/components/SidebarView.tsx` | 合并拒绝 |
| 已有 Tauri Command 函数签名 | 合并拒绝 |

## 必须遵守的规则

1. **Stage 隔离**：所有新代码在 `agent/` 下，不修改已有目录
2. **迁移只增不改**：只新建 003，不碰 001/002
3. **Command 只增不改**：在 `invoke_handler!` 末尾追加，不改已有条目
4. **Reducer 只增不改**：在 `AppContext.tsx` 的 Action 和 reducer 末尾追加，不改已有 case
5. **自包含**：Agent 代码不依赖 `feed::service` 或 `reader::pipeline` 的内部实现
6. **通过 `get_entry_content` 获取文章**：调用已有命令拿 `cleanedMarkdown`

## 验证方法

```bash
# 王康睿在合并前会跑这个命令：
git diff origin/dev --name-only

# 输出必须只包含白名单文件，出现任何其他文件 = 不合并
```

## 对接口

| 需要的 | 从哪里拿 |
|---|---|
| 数据库连接池 | `State<'_, DbPool>` — Tauri 已注入 |
| 文章 Markdown 内容 | `commands::get_entry_content(&state, entry_id)` → `Content.cleaned_markdown` |
| 流式推前端 | `app_handle.emit("ai-stream", payload)` |
| 前端 invoke 模板 | 参考 `src-ui/src/api/feed.ts` |
