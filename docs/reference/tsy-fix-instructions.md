# 杜偲妍 — TS 类型修正指令

> 给她的 AI 直接复制粘贴。

## 问题

你的 `src-ui/src/lib/types.ts` 和 `src-ui/src/api/feed.ts` 类型定义与 `docs/reference/command-contract.md`（三人共识的接口契约）不一致。

Rust 侧已按契约实现完毕，serde 序列化字段使用 camelCase。

## 需要改的文件和具体内容

### 1. `src-ui/src/lib/types.ts`

删除 `PagedResult<T>`，新增以下类型：

```typescript
// 侧边栏用（不是 Feed）
export interface FeedSummary {
  id: number;
  title: string;
  unreadCount: number;
}

// 列表条目（不是 Entry）
export interface EntryListItem {
  id: number;
  feedId: number;
  title: string;
  author: string;
  publishedAt: string | null;
  isRead: boolean;  // boolean 不是 number
}

// 分页结果
export interface EntryPage {
  entries: EntryListItem[];  // 注意: entries 不是 items
  total: number;
  page: number;
  pageSize: number;
}

// Entry 保持不动，isRead/isStarred 类型改为 boolean
// Feed 保持不动，去掉 unread_count 字段
```

### 2. `src-ui/src/api/feed.ts`

修正函数签名：

```typescript
// listFeeds 返回 FeedSummary[] 不是 Feed[]
export async function listFeeds(): Promise<FeedSummary[]> {
  return invoke<FeedSummary[]>("list_feeds");
}

// listEntries 返回 EntryPage 不是 PagedResult<Entry>
export async function listEntries(
  feedId: number,
  page: number = 1,
  pageSize: number = 20,
  filter?: string
): Promise<EntryPage> {
  return invoke<EntryPage>("list_entries", { feedId, page, pageSize, filter });
}

// getEntry 返回 Entry
export async function getEntry(id: number): Promise<Entry> {
  return invoke<Entry>("get_entry", { id });
}

// markRead/markUnread 参数名 id（不是 entryId）
export async function markRead(id: number): Promise<void> {
  return invoke("mark_read", { id });
}

export async function markUnread(id: number): Promise<void> {
  return invoke("mark_unread", { id });
}

// getEntryContent 参数名 entryId
export async function getEntryContent(entryId: number): Promise<Content> {
  return invoke<Content>("get_entry_content", { entryId });
}

// searchEntries 返回 EntryPage
export async function searchEntries(
  query: string, page: number = 1, pageSize: number = 20
): Promise<EntryPage> {
  return invoke<EntryPage>("search_entries", { query, page, pageSize });
}

// addFeed/removeFeed/refreshFeed 参数保持不变，返回类型也保持
```

### 3. `src-ui/src/contexts/AppContext.tsx`

用新类型替换旧引用：

- `feeds: Feed[]` → `feeds: FeedSummary[]`
- `entries: Entry[]` → `entries: EntryListItem[]`
- `mockFeeds` → 改为 `mockFeedSummaries: FeedSummary[]`（只有 id/title/unreadCount）
- 所有 `page.entries` 引用保持不变（之前如果写的是 `page.items` 需要改成 `page.entries`）

## 不需要改的

以下组件和类型不动：
- `ReaderView.tsx`、`Sidebar.tsx`、`EntryList.tsx` 内部逻辑
- `Content`、`Entry` 类型（仅 isRead/isStarred 改 boolean）
- mock.ts（只需修正类型适配）
- 所有 Stage 3/4 的 API（summary、translation、note 等）

## 验证

改完后 `npm run build` 零 error 即可。
