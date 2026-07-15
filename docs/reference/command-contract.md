# Tauri Command 契约清单

> 本文件定义所有 Rust ↔ React 之间的 Tauri Command 接口契约。
> 三人各自实现时以此为准。签名变更需三人共识 + 同步更新本文件。

---

## 约定

- Rust 侧：所有 Command 返回 `Result<T, String>`（`T` 需实现 `Serialize`）
- React 侧：`import { invoke } from '@tauri-apps/api/core'`
- `Option<T>` 表示可选参数/字段
- 日期时间字段统一使用 ISO 8601 字符串（`String`）

---

## 类型定义

```typescript
// === 基础类型（TS 侧） ===

interface Feed {
  id: number;
  url: string;
  title: string;
  description: string;
  link: string;
  feed_type: string;        // "rss" | "atom" | "json"
  last_synced_at: string | null;
  created_at: string;
}

interface FeedSummary {
  id: number;
  title: string;
  unread_count: number;
}

interface EntryListItem {
  id: number;
  feed_id: number;
  title: string;
  author: string;
  published_at: string | null;
  is_read: boolean;
}

interface Entry {
  id: number;
  feed_id: number;
  guid: string;
  title: string;
  author: string;
  link: string;
  summary: string;
  published_at: string | null;
  updated_at: string | null;
  is_read: boolean;
  is_starred: boolean;
  created_at: string;
}

interface Content {
  id: number;
  entry_id: number;
  raw_html: string;
  cleaned_html: string | null;
  cleaned_markdown: string | null;
  rendered_html: string | null;
}

interface EntryPage {
  entries: EntryListItem[];
  total: number;
  page: number;
  page_size: number;
}

interface Provider {
  id: number;
  name: string;
  base_url: string;
  default_model: string;
  thinking_model: string | null;
  created_at: string;
}

interface NoteSummary {
  id: number;
  entry_id: number;
  updated_at: string;
}

interface Note {
  id: number;
  entry_id: number;
  content: string;
  created_at: string;
  updated_at: string;
}

interface TranslationSegment {
  index: number;
  source_text: string;
  translated_text: string | null;
  status: "pending" | "running" | "done" | "failed";
}

interface AppSettings {
  theme: string;              // "light" | "dark"
  font_family: string;
  sync_interval: number;      // 分钟
  summary_language: string;
  summary_detail_level: string;
  translation_language: string;
  translation_concurrency: number;
}
```

---

## Stage 1: Feed 订阅与基础阅读

### Feed 管理

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `add_feed` | `fn add_feed(url: String) -> Result<Feed, String>` | `invoke<Feed>('add_feed', { url })` | 抓取 URL → 解析 → 存入数据库 → 返回 Feed。URL 不可达返回错误 |
| `remove_feed` | `fn remove_feed(id: i32) -> Result<(), String>` | `invoke<void>('remove_feed', { id })` | 级联删除 entries + contents |
| `list_feeds` | `fn list_feeds() -> Result<Vec<FeedSummary>, String>` | `invoke<FeedSummary[]>('list_feeds')` | 返回所有 Feed（含未读计数） |
| `refresh_feed` | `fn refresh_feed(id: i32) -> Result<(), String>` | `invoke<void>('refresh_feed', { id })` | 重新抓取解析 → 增量更新 entries |
| `refresh_all_feeds` | `fn refresh_all_feeds() -> Result<(), String>` | `invoke<void>('refresh_all_feeds')` | 并发刷新所有 Feed（Semaphore 限制 5） |
| `get_feed` | `fn get_feed(id: i32) -> Result<Feed, String>` | `invoke<Feed>('get_feed', { id })` | 获取单个 Feed 详情 |

### Entry 查询

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `list_entries` | `fn list_entries(feed_id: i32, page: i32, page_size: i32, filter: Option<String>) -> Result<EntryPage, String>` | `invoke<EntryPage>('list_entries', { feedId, page, pageSize, filter })` | filter: "unread" / "starred" / null（全部）。page 从 1 开始 |
| `list_all_entries` | `fn list_all_entries(page: i32, page_size: i32, filter: Option<String>) -> Result<EntryPage, String>` | `invoke<EntryPage>('list_all_entries', { page, pageSize, filter })` | 所有 Feed 的文章，跨 Feed 聚合 |
| `get_entry` | `fn get_entry(id: i32) -> Result<Entry, String>` | `invoke<Entry>('get_entry', { id })` | 获取 Entry 详情（不含 content） |
| `mark_read` | `fn mark_read(id: i32) -> Result<(), String>` | `invoke<void>('mark_read', { id })` | 标记为已读 |
| `mark_unread` | `fn mark_unread(id: i32) -> Result<(), String>` | `invoke<void>('mark_unread', { id })` | 标记为未读 |
| `mark_all_read` | `fn mark_all_read(feed_id: i32) -> Result<(), String>` | `invoke<void>('mark_all_read', { feedId })` | 标记某 Feed 下所有文章已读 |

### Content 读取

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `get_entry_content` | `fn get_entry_content(entry_id: i32) -> Result<Content, String>` | `invoke<Content>('get_entry_content', { entryId })` | Stage 1 返回 raw_html；Stage 2 后返回 cleaned/rendered |

### OPML

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `import_opml` | `fn import_opml(file_path: String) -> Result<Vec<Feed>, String>` | `invoke<Feed[]>('import_opml', { filePath })` | 解析 OPML → 逐个 add_feed。失败项记录日志，成功项正常返回 |
| `export_opml` | `fn export_opml(file_path: String) -> Result<(), String>` | `invoke<void>('export_opml', { filePath })` | 序列化所有 Feed 为 OPML XML |

---

## Stage 2: 阅读体验增强

### 内容处理

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `process_entry_content` | `fn process_entry_content(entry_id: i32) -> Result<Content, String>` | `invoke<Content>('process_entry_content', { entryId })` | 执行完整 Reader Pipeline → 更新并返回 Content |

### 搜索

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `search_entries` | `fn search_entries(query: String, page: i32, page_size: i32) -> Result<EntryPage, String>` | `invoke<EntryPage>('search_entries', { query, page, pageSize })` | 搜索标题 + 摘要（FTS5） |

### 主题与字体

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `list_system_fonts` | `fn list_system_fonts() -> Result<Vec<String>, String>` | `invoke<string[]>('list_system_fonts')` | 枚举系统可用字体 |

---

## Stage 3: AI 功能

### Provider 管理

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `add_provider` | `fn add_provider(name: String, base_url: String, api_key: String, default_model: String) -> Result<Provider, String>` | `invoke<Provider>('add_provider', { name, baseUrl, apiKey, defaultModel })` | api_key 存 Tauri secure-store |
| `list_providers` | `fn list_providers() -> Result<Vec<Provider>, String>` | `invoke<Provider[]>('list_providers')` | 不返回 api_key |
| `update_provider` | `fn update_provider(id: i32, name: String, base_url: String, api_key: Option<String>, default_model: String) -> Result<Provider, String>` | `invoke<Provider>('update_provider', { id, name, baseUrl, apiKey, defaultModel })` | api_key 为 None 时不更新 |
| `delete_provider` | `fn delete_provider(id: i32) -> Result<(), String>` | `invoke<void>('delete_provider', { id })` | |
| `validate_provider` | `fn validate_provider(base_url: String, api_key: String) -> Result<bool, String>` | `invoke<boolean>('validate_provider', { baseUrl, apiKey })` | 发送最小请求验证连通性 |

### Summary Agent

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `start_summary` | `fn start_summary(entry_id: i32, provider_id: i32, target_language: String, detail_level: String) -> Result<i32, String>` | `invoke<number>('start_summary', { entryId, providerId, targetLanguage, detailLevel })` | 返回 task_id。1 秒防抖、latest-only 队列 |
| `cancel_summary` | `fn cancel_summary(task_id: i32) -> Result<(), String>` | `invoke<void>('cancel_summary', { taskId })` | |
| `get_summary_status` | `fn get_summary_status(entry_id: i32) -> Result<Option<String>, String>` | `invoke<string | null>('get_summary_status', { entryId })` | 返回状态：null（无）/ "running" / "succeeded" / "failed" |
| `get_summary_result` | `fn get_summary_result(entry_id: i32) -> Result<Option<String>, String>` | `invoke<string | null>('get_summary_result', { entryId })` | 返回摘要文本或 null |

### Translation Agent

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `start_translation` | `fn start_translation(entry_id: i32, provider_id: i32, target_language: String) -> Result<i32, String>` | `invoke<number>('start_translation', { entryId, providerId, targetLanguage })` | 返回 task_id |
| `retry_translation_segment` | `fn retry_translation_segment(entry_id: i32, segment_index: i32) -> Result<(), String>` | `invoke<void>('retry_translation_segment', { entryId, segmentIndex })` | 重试单个失败段落 |
| `cancel_translation` | `fn cancel_translation(task_id: i32) -> Result<(), String>` | `invoke<void>('cancel_translation', { taskId })` | |
| `clear_translation` | `fn clear_translation(entry_id: i32) -> Result<(), String>` | `invoke<void>('clear_translation', { entryId })` | 删除该 Entry 的所有翻译结果 |
| `get_translation_segments` | `fn get_translation_segments(entry_id: i32) -> Result<Vec<TranslationSegment>, String>` | `invoke<TranslationSegment[]>('get_translation_segments', { entryId })` | 返回所有段落及其翻译状态 |

### AI 流式事件（Tauri Event，非 Command）

| Event | Payload | 说明 |
|---|---|---|
| `ai-stream` | `{ task_id: number, agent_type: "summary" \| "translation", segment_index?: number, content: string, is_done: boolean }` | Rust 通过 `app_handle.emit("ai-stream", payload)` 推送 |

---

## Stage 4: 笔记与导出

### 笔记

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `save_note` | `fn save_note(entry_id: i32, content: String) -> Result<Note, String>` | `invoke<Note>('save_note', { entryId, content })` | 创建或更新 |
| `get_note` | `fn get_note(entry_id: i32) -> Result<Option<Note>, String>` | `invoke<Note \| null>('get_note', { entryId })` | |
| `delete_note` | `fn delete_note(id: i32) -> Result<(), String>` | `invoke<void>('delete_note', { id })` | |

### 导出

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `export_single_digest` | `fn export_single_digest(entry_id: i32, format: String, file_path: String) -> Result<(), String>` | `invoke<void>('export_single_digest', { entryId, format, filePath })` | format: "markdown" / "html" |
| `export_multi_digest` | `fn export_multi_digest(entry_ids: Vec<i32>, format: String, file_path: String) -> Result<(), String>` | `invoke<void>('export_multi_digest', { entryIds, format, filePath })` | |

### 设置

| Command | Rust 签名 | TS 调用 | 说明 |
|---|---|---|---|
| `get_settings` | `fn get_settings() -> Result<AppSettings, String>` | `invoke<AppSettings>('get_settings')` | |
| `update_settings` | `fn update_settings(settings: AppSettings) -> Result<(), String>` | `invoke<void>('update_settings', { settings })` | |

---

## v2 后续（上交版本用）

| Command | Rust 签名 | 说明 |
|---|---|---|
| `add_tag` | `fn add_tag(entry_id: i32, tag_name: String) -> Result<(), String>` | |
| `remove_tag` | `fn remove_tag(entry_id: i32, tag_name: String) -> Result<(), String>` | |
| `get_entry_tags` | `fn get_entry_tags(entry_id: i32) -> Result<Vec<String>, String>` | |
| `filter_by_tag` | `fn filter_by_tag(tag_name: String, page: i32, page_size: i32) -> Result<EntryPage, String>` | |
| `suggest_tags` | `fn suggest_tags(entry_id: i32, provider_id: i32) -> Result<Vec<String>, String>` | AI 标签建议 |
| `get_usage_stats` | `fn get_usage_stats() -> Result<Vec<UsageStat>, String>` | Token 用量统计 |
