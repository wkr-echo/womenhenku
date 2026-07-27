# 收藏功能 & 全部文章聚合视图

> 可直接复制给 AI。附带测试方法。

---

## 一、收藏（Star）功能

### 1.1 功能概述

每篇文章可以标记为"收藏"。侧边栏有独立的「收藏」入口，点击后显示所有收藏文章。

### 1.2 数据库

`entries` 表已有 `is_starred` 字段（INTEGER NOT NULL DEFAULT 0），无需新增 migration。

### 1.3 Rust 侧新增 Command

```rust
// 切换收藏状态。返回切换后的状态
fn toggle_star(entry_id: i32) -> Result<bool, String>

// 查询收藏文章列表（分页 + 未读筛选）
fn list_starred_entries(page: i32, page_size: i32, unread_only: bool) -> Result<EntryPage, String>

// 侧边栏计数
fn get_sidebar_counts() -> Result<SidebarCounts, String>
// SidebarCounts { total_unread: i32, total_starred: i32, starred_unread: i32 }
```

`EntryRepository` 新增方法：

```rust
fn mark_starred(&self, entry_id: i32, is_starred: bool) -> Result<(), RepositoryError>;
fn count_starred(&self, unread_only: bool) -> Result<i32, RepositoryError>;
fn find_starred(&self, page: i32, page_size: i32, unread_only: bool) -> Result<EntryPage, RepositoryError>;
```

### 1.4 React 侧

**文章列表每行增加收藏按钮：**

```
┌──────────────────────────────────────────────────┐
│  ☆  文章标题                      作者 · 日期    │  ← 未收藏：空心星
│  ★  文章标题                      作者 · 日期    │  ← 已收藏：实心星（黄色）
└──────────────────────────────────────────────────┘
```

- 点击星号 → `invoke('toggle_star', { entryId })` → 即时切换 UI 状态
- 在「收藏」视图中取消收藏 → 该行从列表消失（不刷新整个列表，局部移除）

### 1.5 侧边栏虚拟行

侧边栏 Feed 列表顶部，所有订阅源上方，两个虚拟入口：

```
┌──────────────────────────────┐
│  ⊞  全部文章             123 │  ← 全部 Feed 聚合 + 总未读数
│  ★  收藏 (15)             3  │  ← 收藏总数 + 未读收藏数
├──────────────────────────────┤
│  <antirez>               97  │  ← 下面是各个订阅源
│  Daring Fireball         48  │
│  ...                         │
└──────────────────────────────┘
```

虚拟行特性：
- 不支持右键菜单（不是真实 Feed，不能编辑/删除）
- 「收藏」行括号内数字 = 总收藏数
- 右侧徽标 = 收藏中未读的数量
- 数据来自 `get_sidebar_counts()` Command

### 1.6 测试方法

- [ ] 文章列表中点击空心星 → 变为实心星（黄色），数据库 is_starred=1
- [ ] 点击实心星 → 变为空心星，数据库 is_starred=0
- [ ] 侧边栏点击「收藏」→ 文章列表仅显示已收藏文章
- [ ] 收藏视图中取消收藏 → 该文章从列表消失
- [ ] 侧边栏「收藏 (15) 3」→ 15 是总数，3 是其中未读数
- [ ] 收藏一篇文章 → 侧边栏数字 +1
- [ ] 取消收藏 → 侧边栏数字 -1
- [ ] 重启应用 → 收藏状态保持

---

## 二、全部文章聚合视图

### 2.1 功能概述

侧边栏「全部文章」入口，显示所有订阅源的文章聚合（跨 Feed）。
和单个 Feed 共用同一套文章列表组件，只切换数据源。

### 2.2 侧边栏选择状态

```typescript
type FeedSelection = 
  | { type: "all" }
  | { type: "starred" }
  | { type: "feed"; feedId: number };
```

### 2.3 Rust 侧 Command

```rust
// 跨 Feed 分页查询
fn list_all_entries(page: i32, page_size: i32, unread_only: bool) -> Result<EntryPage, String>;
```

`EntryRepository` 新增方法：

```rust
fn find_all(&self, page: i32, page_size: i32, unread_only: bool) -> Result<EntryPage, RepositoryError>;
```

SQL：

```sql
-- unread_only=false
SELECT e.* FROM entries e 
WHERE e.is_deleted = 0 
ORDER BY e.published_at DESC 
LIMIT ? OFFSET ?;

-- unread_only=true
SELECT e.* FROM entries e 
WHERE e.is_deleted = 0 AND e.is_read = 0 
ORDER BY e.published_at DESC 
LIMIT ? OFFSET ?;

-- 总数
SELECT COUNT(*) FROM entries WHERE is_deleted = 0;
SELECT COUNT(*) FROM entries WHERE is_deleted = 0 AND is_read = 0;
```

### 2.4 React 侧

`Sidebar.tsx`：

```
选中逻辑：
- 点击「全部文章」→ selectedFeed = { type: "all" }
- 点击某个 Feed  → selectedFeed = { type: "feed", feedId }
- 点击「收藏」    → selectedFeed = { type: "starred" }

根据 selectedFeed.type 调用不同 Command：
- "all"     → invoke('list_all_entries', ...)
- "feed"    → invoke('list_entries', { feedId, ... })
- "starred" → invoke('list_starred_entries', ...)
```

### 2.5 文章列表的未读筛选

文章列表顶部工具栏：

```
┌──────────────────────────────────────────────────┐
│  Entries                          [未读]  [···]  │
├──────────────────────────────────────────────────┤
│  ☆  文章标题1                       作者 · 日期  │
│  ★  文章标题2                       作者 · 日期  │
│  ...                                            │
└──────────────────────────────────────────────────┘
```

- [未读] 是一个 Toggle 按钮，默认关闭
- 打开 → 传给后端 `unread_only=true` → 列表刷新
- 在全部文章、单个 Feed、收藏视图中均生效
- [···] 更多菜单：标记选中已读/未读、全部已读/未读、文摘导出

### 2.6 数据流图

```
用户点击侧边栏
      │
      ├─ 「全部文章」→ invoke('list_all_entries', { page, pageSize, unreadOnly })
      ├─ 「收藏」    → invoke('list_starred_entries', { page, pageSize, unreadOnly })
      └─ 某个 Feed  → invoke('list_entries', { feedId, page, pageSize, unreadOnly })
      │
      ▼
  EntryList 组件（同一套，不区分来源）
      │
      ▼
  点击文章 → invoke('get_entry_content', { entryId }) → ReaderView
```

### 2.7 测试方法

- [ ] 添加多个订阅源后有文章 → 点击「全部文章」→ 显示所有文章
- [ ] 「全部文章」旁未读计数 = 所有 Feed 未读数之和
- [ ] 切换到单个 Feed → 文章列表正确过滤为该 Feed 的文章
- [ ] 切换到「收藏」→ 仅显示收藏文章
- [ ] [未读] 开关打开 → 仅显示未读文章
- [ ] [未读] 开关关闭 → 显示全部文章
- [ ] [未读] 开关在「全部文章」/ 单个 Feed / 收藏 中独立工作
- [ ] 侧边栏未读计数实时更新（阅读文章后数字 -1）
- [ ] 收藏文章后「收藏」行数字 +1
- [ ] 分页加载：滚动到底部 → 加载更多
