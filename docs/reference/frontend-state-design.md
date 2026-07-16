# 前端状态设计

> 本文件定义 React 侧的状态管理结构、Context 组织、路由逻辑。
> Stage 3 流式状态标注「开发中验证」，其余全部锁定。

---

## 全局状态树

```typescript
// src-ui/src/contexts/AppContext.tsx

interface AppState {
  // === 导航 ===
  selectedFeedId: number | null;          // 当前选中的 Feed，null = 全部文章
  selectedEntryId: number | null;         // 当前打开的文章
  viewMode: "list" | "reader";           // 列表模式 / 阅读模式

  // === 数据 ===
  feeds: FeedSummary[];                   // 侧边栏 Feed 列表（含未读计数）
  entries: EntryListItem[];               // 当前文章列表
  entriesTotal: number;                   // 分页总数
  entriesPage: number;                    // 当前页码
  entriesFilter: string | null;           // "unread" / "starred" / null

  // === 阅读 ===
  currentContent: Content | null;         // 当前文章的 Content
  contentLoading: boolean;                // Content 加载中

  // === 主题 ===
  theme: "light" | "dark";               // ★ Stage 2
  fontFamily: string;                     // ★ Stage 2
  searchQuery: string;                    // ★ Stage 2
  searchResults: EntryListItem[];         // ★ Stage 2

  // === AI Agent（★ Stage 3，开发中验证） ===
  summaryTaskId: number | null;
  summaryStatus: "idle" | "running" | "done" | "failed";
  summaryResult: string | null;

  translationTaskId: number | null;
  translationSegments: TranslationSegment[];

  // === Agent UI ===
  showSummaryPanel: boolean;              // 摘要面板可见
  showTranslationPanel: boolean;          // 翻译面板可见

  // === 笔记（★ Stage 4） ===
  noteContent: string;
  noteDirty: boolean;                     // 笔记未保存

  // === 设置（★ Stage 4） ===
  settings: AppSettings | null;
}

type AppAction =
  // 导航
  | { type: "SELECT_FEED"; feedId: number | null }
  | { type: "SELECT_ENTRY"; entryId: number }
  | { type: "BACK_TO_LIST" }
  // 数据
  | { type: "SET_FEEDS"; feeds: FeedSummary[] }
  | { type: "SET_ENTRIES"; entries: EntryListItem[]; total: number; page: number }
  | { type: "MARK_READ"; entryId: number }
  // 阅读
  | { type: "SET_CONTENT"; content: Content }
  | { type: "SET_CONTENT_LOADING"; loading: boolean }
  // 主题
  | { type: "SET_THEME"; theme: "light" | "dark" }
  | { type: "SET_FONT"; fontFamily: string }
  // 搜索
  | { type: "SET_SEARCH_QUERY"; query: string }
  | { type: "SET_SEARCH_RESULTS"; results: EntryListItem[] }
  // AI
  | { type: "START_SUMMARY"; taskId: number }
  | { type: "SUMMARY_STREAM"; content: string }
  | { type: "SUMMARY_DONE" }
  | { type: "SUMMARY_FAILED" }
  | { type: "START_TRANSLATION"; taskId: number }
  | { type: "TRANSLATION_SEGMENT_UPDATE"; segment: TranslationSegment }
  | { type: "TRANSLATION_DONE" }
  | { type: "TOGGLE_SUMMARY_PANEL" }
  | { type: "TOGGLE_TRANSLATION_PANEL" }
  // 笔记
  | { type: "SET_NOTE"; content: string }
  | { type: "SET_NOTE_DIRTY"; dirty: boolean }
  // 设置
  | { type: "SET_SETTINGS"; settings: AppSettings };
```

---

## Context 组织

```typescript
// AppContext — 全局状态
const AppContext = createContext<{
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
}>(null!);

// ThemeContext — 主题（Stage 2）
//   从 AppState.theme 派生，提供 CSS 变量注入
const ThemeContext = createContext<{
  theme: "light" | "dark";
  toggleTheme: () => void;
}>(null!);
```

**不使用 Redux/Zustand。** 仅 Context + useReducer，保持 AI 生成代码的一致性。

---

## 路由/页面切换逻辑

无 URL 路由（桌面应用非 Web），通过 `viewMode` 控制：

```
[应用启动]
    │
    ▼
┌─────────────┐    点击文章    ┌─────────────┐
│  list 模式   │ ──────────→  │ reader 模式  │
│ 侧边栏+列表  │ ←──────────  │ 侧边栏+阅读  │
└─────────────┘   点击返回    └─────────────┘
    │                              │
    │ 点击摘要/翻译按钮             │ 点击摘要/翻译按钮
    ▼                              ▼
┌─────────────┐              ┌─────────────┐
│ SummaryPanel│              │Translation   │
│ (弹出/侧拉) │              │Panel (覆盖)  │
└─────────────┘              └─────────────┘
```

**侧边栏行为**：
- 默认展开，宽度 280px，可拖拽调整
- 点击 Feed → `SELECT_FEED(feedId)` → 加载该 Feed 的文章列表
- 点击"全部文章" → `SELECT_FEED(null)` → 跨 Feed 聚合

**阅读区行为**：
- 点击文章 → `SELECT_ENTRY(entryId)` → `viewMode="reader"` → 加载 Content
- 返回按钮 → `BACK_TO_LIST` → `viewMode="list"`
- 打开文章时自动 `mark_read`（延迟 1 秒，用户快速划过不触发）

---

## 数据流

```
React Component
    ↓ dispatch(action)
AppContext (useReducer)
    ↓ 副作用触发
useEffect / Event Listener
    ↓ invoke() / listen()
Tauri Command / Event
    ↓
Rust Core
```

**数据加载模式**（所有数据获取走同一模式）：
```typescript
// src-ui/src/hooks/useDataFetch.ts
function useDataFetch<T>(
  fetchFn: () => Promise<T>,
  onSuccess: (data: T) => AppAction
): { data: T | null; loading: boolean; error: string | null }
```

---

## Stage 3 流式状态（开发中验证）

AI 流式输出通过 Tauri Event `ai-stream` 传递，不走 Command。前端监听模式：

```typescript
// 监听流式事件
const unlisten = await listen<StreamPayload>("ai-stream", (event) => {
  if (event.payload.agent_type === "summary") {
    if (event.payload.is_done) {
      dispatch({ type: "SUMMARY_DONE" });
    } else {
      dispatch({ type: "SUMMARY_STREAM", content: event.payload.content });
    }
  }
  if (event.payload.agent_type === "translation") {
    dispatch({
      type: "TRANSLATION_SEGMENT_UPDATE",
      segment: {
        index: event.payload.segment_index!,
        source_text: "",
        translated_text: event.payload.content,
        status: event.payload.is_done ? "done" : "running",
      },
    });
  }
});

// 组件卸载时取消订阅
return () => { unlisten(); };
```

---

## 文件结构（`src-ui/src/`）

```
src/
├── main.tsx                    # 入口
├── App.tsx                     # 根组件，Context Provider
├── api/                        # Tauri invoke 封装
│   ├── feed.ts                 # Feed/Entry/Content 相关
│   ├── agent.ts                # Summary/Translation 相关
│   ├── notes.ts                # Notes/Digest 相关
│   └── settings.ts             # Settings 相关
├── contexts/
│   ├── AppContext.tsx           # 全局状态
│   └── ThemeContext.tsx         # 主题（Stage 2）
├── hooks/
│   ├── useDataFetch.ts         # 通用数据加载
│   ├── useKeyboard.ts          # 键盘快捷键（Stage 4）
│   └── useAiStream.ts          # AI 流式监听（Stage 3）
├── components/
│   ├── ui/                     # shadcn/ui 组件
│   ├── Sidebar.tsx             # 侧边栏
│   ├── EntryList.tsx           # 文章列表
│   ├── EntryItem.tsx           # 文章行
│   ├── ReaderView.tsx          # 基础阅读（Stage 1）
│   ├── CleanedReaderView.tsx   # 清洗后阅读（Stage 2）
│   ├── SummaryPanel.tsx        # 摘要面板（Stage 3）
│   ├── TranslationPanel.tsx    # 翻译面板（Stage 3）
│   ├── ProviderConfig.tsx      # Provider 配置（Stage 3）
│   ├── NoteEditor.tsx          # 笔记编辑器（Stage 4）
│   ├── SettingsPage.tsx        # 设置页（Stage 4）
│   └── SearchBar.tsx           # 搜索框（Stage 2）
├── styles/
│   └── themes.css              # 主题 CSS 变量
└── types/
    └── index.ts                # TS 类型定义
```

---

## 注意事项

- 所有状态变化通过 `dispatch`，不在组件内部直接 `setState` 管理业务数据
- `AppState` 是唯一状态源。LocalStorage/React State 仅用于 UI 临时态（如输入框内容、下拉框展开/收起）
- 分页数据不累积（每次翻页替换 `entries`），搜索同类替换
- Stage 3 的流式状态结构留弹性——实际开发中 `ai-stream` 事件格式可能需要微调
