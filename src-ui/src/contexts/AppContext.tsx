import {
  createContext,
  useContext,
  useReducer,
  useCallback,
  useEffect,
  type ReactNode,
} from "react";
import type { FeedSummary, EntryListItem, Entry, EntryPage, ViewMode, Tag } from "@/lib/types";
import { mockFeedSummaries, mockEntries, mockApi, mockListTags } from "@/api/mock";
import {
  isTauri,
  listFeeds as listFeedsReal,
  listEntries as listEntriesReal,
  getEntry as getEntryReal,
  addFeed as addFeedReal,
  removeFeed as removeFeedReal,
  refreshFeed as refreshFeedReal,
  refreshAllFeeds as refreshAllFeedsReal,
  searchEntries as searchEntriesReal,
  markRead as markReadReal,
  listTags as listTagsReal,
} from "@/api/feed";
import { toast } from "@/components/ui/Toast";
import { t } from "@/lib/utils";

// ---- State & Actions ----

interface State {
  feeds: FeedSummary[];
  selectedFeedId: number | null;
  selectedEntry: Entry | null;
  viewMode: ViewMode;
  entries: EntryListItem[];
  searchQuery: string;
  sidebarCollapsed: boolean;
  tags: Tag[];
  selectedTagId: number | null;
  sidebarMode: "feed" | "tag";
  selectedTagIds: number[];
  tagMatchMode: "or" | "and";
  isBatchTagging: boolean;
}

type Action =
  | { type: "SET_FEEDS"; feeds: FeedSummary[] }
  | { type: "SET_SELECTED_FEED_ID"; feedId: number | null }
  | { type: "SET_SELECTED_ENTRY"; entry: Entry | null }
  | { type: "SET_VIEW_MODE"; mode: ViewMode }
  | { type: "SET_ENTRIES"; entries: EntryListItem[] }
  | { type: "SET_SEARCH_QUERY"; query: string }
  | { type: "TOGGLE_SIDEBAR" }
  | { type: "ADD_FEED"; feed: FeedSummary }
  | { type: "REMOVE_FEED"; id: number }
  | { type: "SELECT_FEED"; feedId: number }
  | { type: "SELECT_ENTRY"; entry: Entry | null }
  | { type: "MARK_READ"; entryId: number; feedId: number }
  | { type: "MARK_ALL_READ"; feedId: number }
  | { type: "SET_TAGS"; tags: Tag[] }
  | { type: "SELECT_TAG"; tagId: number | null }
  | { type: "SET_SIDEBAR_MODE"; mode: "feed" | "tag" }
  | { type: "TOGGLE_TAG_SELECTION"; tagId: number }
  | { type: "SET_TAG_MATCH_MODE"; mode: "or" | "and" }
  | { type: "SET_BATCH_TAGGING"; isRunning: boolean };

const initialState: State = {
  feeds: [],
  selectedFeedId: null,
  selectedEntry: null,
  viewMode: "list",
  entries: [],
  searchQuery: "",
  sidebarCollapsed: false,
  tags: [],
  selectedTagId: null,
  sidebarMode: "feed",
  selectedTagIds: [],
  tagMatchMode: "or",
  isBatchTagging: false,
};

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case "SET_FEEDS":
      return { ...state, feeds: action.feeds };
    case "SET_SELECTED_FEED_ID":
      return { ...state, selectedFeedId: action.feedId };
    case "SET_SELECTED_ENTRY":
      return { ...state, selectedEntry: action.entry };
    case "SET_VIEW_MODE":
      return { ...state, viewMode: action.mode };
    case "SET_ENTRIES":
      return { ...state, entries: action.entries };
    case "SET_SEARCH_QUERY":
      return { ...state, searchQuery: action.query };
    case "TOGGLE_SIDEBAR":
      return { ...state, sidebarCollapsed: !state.sidebarCollapsed };
    case "ADD_FEED":
      return { ...state, feeds: [...state.feeds, action.feed] };
    case "REMOVE_FEED": {
      const feeds = state.feeds.filter((f) => f.id !== action.id);
      const isRemovingSelected = state.selectedFeedId === action.id;
      return {
        ...state,
        feeds,
        selectedFeedId: isRemovingSelected ? null : state.selectedFeedId,
        selectedEntry: isRemovingSelected ? null : state.selectedEntry,
        viewMode: isRemovingSelected ? ("list" as ViewMode) : state.viewMode,
      };
    }
    case "SELECT_FEED":
      return {
        ...state,
        selectedFeedId: action.feedId,
        selectedEntry: null,
        viewMode: "list",
        searchQuery: "",
      };
    case "SELECT_ENTRY":
      return { ...state, selectedEntry: action.entry, viewMode: "reader" };
    case "MARK_READ": {
      const feeds = state.feeds.map((f) =>
        f.id === action.feedId && f.unreadCount > 0
          ? { ...f, unreadCount: f.unreadCount - 1 }
          : f
      );
      const entries = state.entries.map((e) =>
        e.id === action.entryId ? { ...e, isRead: true } : e
      );
      const selectedEntry =
        state.selectedEntry?.id === action.entryId
          ? { ...state.selectedEntry, isRead: true }
          : state.selectedEntry;
      return { ...state, feeds, entries, selectedEntry };
    }
    case "MARK_ALL_READ": {
      const feeds = state.feeds.map((f) =>
        f.id === action.feedId ? { ...f, unreadCount: 0 } : f
      );
      const entries = state.entries.map((e) =>
        e.feedId === action.feedId ? { ...e, isRead: true } : e
      );
      return { ...state, feeds, entries };
    }
    case "SET_TAGS":
      return { ...state, tags: action.tags };
    case "SELECT_TAG":
      return {
        ...state,
        selectedTagId: action.tagId,
        selectedFeedId: action.tagId ? null : state.selectedFeedId,
        viewMode: "list",
      };
    case "SET_SIDEBAR_MODE":
      return { ...state, sidebarMode: action.mode };
    case "TOGGLE_TAG_SELECTION": {
      const current = state.selectedTagIds;
      if (current.includes(action.tagId)) {
        const next = current.filter(id => id !== action.tagId);
        return { 
          ...state, 
          selectedTagIds: next,
          viewMode: next.length > 0 ? "list" : state.viewMode,
          selectedFeedId: null,
        };
      } else {
        return { 
          ...state, 
          selectedTagIds: [...current, action.tagId],
          viewMode: "list",
          selectedFeedId: null,
        };
      }
    }
    case "SET_TAG_MATCH_MODE":
      return { ...state, tagMatchMode: action.mode };
    case "SET_BATCH_TAGGING":
      return { ...state, isBatchTagging: action.isRunning };
    default:
      return state;
  }
}

// ---- Context ----

interface AppContextType {
  feeds: FeedSummary[];
  selectedFeedId: number | null;
  selectedEntry: Entry | null;
  viewMode: ViewMode;
  entries: EntryListItem[];
  searchQuery: string;
  sidebarCollapsed: boolean;
  tags: Tag[];
  selectedTagId: number | null;
  sidebarMode: "feed" | "tag";
  selectedTagIds: number[];
  tagMatchMode: "or" | "and";
  isBatchTagging: boolean;

  selectFeed: (feedId: number) => void;
  selectEntry: (entry: EntryListItem) => void;
  setViewMode: (mode: ViewMode) => void;
  setSearchQuery: (query: string) => void;
  toggleSidebar: () => void;
  addFeed: (url: string) => void;
  removeFeed: (id: number) => void;
  refreshFeed: (id: number) => void;
  refreshAll: () => void;
  reloadFeeds: () => void;
  markEntryRead: (id: number) => void;
  markAllRead: (feedId: number) => void;
  selectTag: (tagId: number | null) => void;
  reloadTags: () => void;
  setSidebarMode: (mode: "feed" | "tag") => void;
  toggleTagSelection: (tagId: number) => void;
  setTagMatchMode: (mode: "or" | "and") => void;
  setBatchTagging: (isRunning: boolean) => void;
}

const AppContext = createContext<AppContextType | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);

  // ---- 初始化：加载 feeds ----
  useEffect(() => {
    let cancelled = false;
    if (isTauri()) {
      listFeedsReal()
        .then((data) => {
          if (!cancelled && data.length > 0) {
            dispatch({ type: "SET_FEEDS", feeds: data });
            dispatch({ type: "SET_SELECTED_FEED_ID", feedId: data[0].id });
          }
        })
        .catch(() => {
          if (!cancelled) {
            dispatch({ type: "SET_FEEDS", feeds: mockFeedSummaries });
            dispatch({ type: "SET_SELECTED_FEED_ID", feedId: 1 });
          }
        });
      listTagsReal()
        .then((tags) => {
          if (!cancelled) dispatch({ type: "SET_TAGS", tags });
        })
        .catch(() => {});
    } else {
      dispatch({ type: "SET_FEEDS", feeds: mockFeedSummaries });
      dispatch({ type: "SET_SELECTED_FEED_ID", feedId: 1 });
      mockListTags().then(tags => dispatch({ type: "SET_TAGS", tags }));
    }
    return () => { cancelled = true; };
  }, []);

  // ---- 选中的 feed 或 tag 变化 → 加载 entries ----
  useEffect(() => {
    let cancelled = false;
    
    if (state.selectedTagIds.length > 0) {
      if (isTauri()) {
        import("@tauri-apps/api/core").then(({ invoke }) => {
          invoke<EntryPage>("list_entries_by_tags", { 
            tagIds: state.selectedTagIds, 
            matchMode: state.tagMatchMode,
            page: 1, 
            pageSize: 50 
          })
            .then((page) => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries });
            })
            .catch(() => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: [] });
            });
        });
      } else {
        dispatch({ type: "SET_ENTRIES", entries: mockApi.filterEntriesByTags(state.selectedTagIds, state.tagMatchMode) });
      }
    } else if (state.selectedFeedId) {
      const feedId = state.selectedFeedId;
      if (isTauri()) {
        listEntriesReal(feedId, 1, 50)
          .then((page: EntryPage) => {
            if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries });
          })
          .catch(() => {
            if (!cancelled)
              dispatch({ type: "SET_ENTRIES", entries: mockEntries[feedId] || [] });
          });
      } else {
        dispatch({ type: "SET_ENTRIES", entries: mockEntries[feedId] || [] });
      }
    } else if (state.selectedTagId) {
      if (isTauri()) {
        import("@tauri-apps/api/core").then(({ invoke }) => {
          invoke<EntryPage>("list_entries_by_tag", { tagId: state.selectedTagId, page: 1, pageSize: 50 })
            .then((page) => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries });
            })
            .catch(() => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: [] });
            });
        });
      } else {
        dispatch({ type: "SET_ENTRIES", entries: [] });
      }
    } else {
      dispatch({ type: "SET_ENTRIES", entries: [] });
    }
    return () => { cancelled = true; };
  }, [state.selectedFeedId, state.selectedTagId, state.selectedTagIds, state.tagMatchMode]);

  // ---- 搜索 ----
  useEffect(() => {
    let cancelled = false;
    if (!state.searchQuery) return;

    if (isTauri()) {
      searchEntriesReal(state.searchQuery, 1, 50)
        .then((page: EntryPage) => {
          if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries });
        })
        .catch(() => {
          if (!cancelled)
            dispatch({ type: "SET_ENTRIES", entries: mockApi.searchEntries(state.searchQuery) });
        });
    } else {
      dispatch({ type: "SET_ENTRIES", entries: mockApi.searchEntries(state.searchQuery) });
    }
    return () => { cancelled = true; };
  }, [state.searchQuery]);

  const selectFeed = useCallback((feedId: number) => {
    dispatch({ type: "SELECT_FEED", feedId });
  }, []);

  const selectEntry = useCallback((item: EntryListItem) => {
    if (isTauri()) {
      getEntryReal(item.id)
        .then((entry) => dispatch({ type: "SELECT_ENTRY", entry }))
        .catch(() => {
          const fallback = mockApi.getEntry(item.id);
          dispatch({ type: "SELECT_ENTRY", entry: fallback || null });
        });
    } else {
      const fallback = mockApi.getEntry(item.id);
      dispatch({ type: "SELECT_ENTRY", entry: fallback || null });
    }
  }, []);

  const toggleSidebar = useCallback(() => {
    dispatch({ type: "TOGGLE_SIDEBAR" });
  }, []);

  const addFeedFn = useCallback((url: string) => {
    if (isTauri()) {
      addFeedReal(url)
        .then((feed) => {
          dispatch({ type: "ADD_FEED", feed: { id: feed.id, title: feed.title, unreadCount: 0 } });
        })
        .catch((e) => {
          toast(t("添加订阅源失败: ") + String(e), "error");
        });
    } else {
      const newFeed: FeedSummary = {
        id: Date.now(),
        title: url.replace(/https?:\/\//, "").split("/")[0],
        unreadCount: 0,
      };
      dispatch({ type: "ADD_FEED", feed: newFeed });
    }
  }, []);

  const removeFeedFn = useCallback((id: number) => {
    if (isTauri()) {
      removeFeedReal(id).catch((e) => {
        toast(t("删除订阅源失败: ") + String(e), "error");
      });
    }
    dispatch({ type: "REMOVE_FEED", id });
  }, []);

  const refreshFeedFn = useCallback((id: number) => {
    if (isTauri()) {
      refreshFeedReal(id)
        .then(() => {
          listEntriesReal(id, 1, 50)
            .then((page: EntryPage) => dispatch({ type: "SET_ENTRIES", entries: page.entries }))
            .catch((e) => { toast(t("加载文章列表失败: ") + String(e), "error"); });
        })
        .catch((e) => { toast(t("刷新订阅源失败: ") + String(e), "error"); });
    }
  }, []);

  const refreshAllFn = useCallback(() => {
    if (isTauri()) {
      refreshAllFeedsReal()
        .then(() => listFeedsReal())
        .then((data) => dispatch({ type: "SET_FEEDS", feeds: data }))
        .catch((e) => { toast(t("刷新全部失败: ") + String(e), "error"); });
    }
  }, []);

  const reloadFeeds = useCallback(() => {
    if (isTauri()) {
      listFeedsReal()
        .then((data) => dispatch({ type: "SET_FEEDS", feeds: data }))
        .catch(() => {});
    }
  }, []);

  const markEntryRead = useCallback((id: number) => {
    // Find the entry's feed ID from current entries or selected entry
    const entry = state.entries.find(e => e.id === id);
    const feedId = entry?.feedId ?? state.selectedEntry?.feedId ?? 0;
    if (isTauri()) {
      markReadReal(id).then(() => {
        dispatch({ type: "MARK_READ", entryId: id, feedId });
      }).catch(() => {});
    } else {
      dispatch({ type: "MARK_READ", entryId: id, feedId });
    }
  }, [state.entries, state.selectedEntry]);

  const markAllRead = useCallback((feedId: number) => {
    if (isTauri()) {
      import("@tauri-apps/api/core").then(({ invoke }) => {
        invoke("mark_all_read", { feedId }).catch(() => {});
      });
    }
    dispatch({ type: "MARK_ALL_READ", feedId });
  }, []);

  const selectTag = useCallback((tagId: number | null) => {
    dispatch({ type: "SELECT_TAG", tagId });
  }, []);

  const reloadTags = useCallback(() => {
    if (isTauri()) {
      listTagsReal()
        .then((tags) => dispatch({ type: "SET_TAGS", tags }))
        .catch(() => {});
    }
  }, []);

  const setSidebarMode = useCallback((mode: "feed" | "tag") => {
    dispatch({ type: "SET_SIDEBAR_MODE", mode });
  }, []);

  const toggleTagSelection = useCallback((tagId: number) => {
    dispatch({ type: "TOGGLE_TAG_SELECTION", tagId });
  }, []);

  const setTagMatchMode = useCallback((mode: "or" | "and") => {
    dispatch({ type: "SET_TAG_MATCH_MODE", mode });
  }, []);

  const setBatchTagging = useCallback((isRunning: boolean) => {
    dispatch({ type: "SET_BATCH_TAGGING", isRunning });
  }, []);

  const setViewMode = useCallback((mode: ViewMode) => {
    dispatch({ type: "SET_VIEW_MODE", mode });
  }, []);

  const setSearchQuery = useCallback((query: string) => {
    dispatch({ type: "SET_SEARCH_QUERY", query });
  }, []);

  return (
    <AppContext.Provider
      value={{
        ...state,
        selectFeed,
        selectEntry,
        setViewMode,
        setSearchQuery,
        toggleSidebar,
        addFeed: addFeedFn,
        removeFeed: removeFeedFn,
        refreshFeed: refreshFeedFn,
        refreshAll: refreshAllFn,
        reloadFeeds,
        markEntryRead,
        markAllRead,
        selectTag,
        reloadTags,
        setSidebarMode,
        toggleTagSelection,
        setTagMatchMode,
        setBatchTagging,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useApp must be used within AppProvider");
  return ctx;
}