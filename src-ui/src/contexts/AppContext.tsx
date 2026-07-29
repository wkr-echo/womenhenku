import {
  createContext,
  useContext,
  useReducer,
  useCallback,
  useEffect,
  useRef,
  type ReactNode,
} from "react";
import type { FeedSummary, EntryListItem, Entry, EntryPage, ViewMode, Tag, SidebarCounts, FeedSelection } from "@/lib/types";
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
  toggleStar as toggleStarReal,
  getSidebarCounts as getSidebarCountsReal,
  listAllEntries as listAllEntriesReal,
} from "@/api/feed";
import { toast } from "@/components/ui/Toast";
import { t } from "@/lib/utils";

// ---- State & Actions ----

interface State {
  feeds: FeedSummary[];
  feedSelection: FeedSelection;
  selectedEntry: Entry | null;
  viewMode: ViewMode;
  entries: EntryListItem[];
  entriesTotal: number;
  currentPage: number;
  isLoadingMore: boolean;
  searchQuery: string;
  sidebarCollapsed: boolean;
  tags: Tag[];
  sidebarMode: "feed" | "tag";
  selectedTagIds: number[];
  tagMatchMode: "or" | "and";
  isBatchTagging: boolean;
  sidebarCounts: SidebarCounts;
  syncStatus: "idle" | "syncing" | "failed";
  syncError: string;
}

type Action =
  | { type: "SET_FEEDS"; feeds: FeedSummary[] }
  | { type: "SET_FEED_SELECTION"; selection: FeedSelection }
  | { type: "SET_SELECTED_ENTRY"; entry: Entry | null }
  | { type: "SET_VIEW_MODE"; mode: ViewMode }
  | { type: "SET_ENTRIES"; entries: EntryListItem[]; total: number }
  | { type: "SET_SEARCH_QUERY"; query: string }
  | { type: "TOGGLE_SIDEBAR" }
  | { type: "ADD_FEED"; feed: FeedSummary }
  | { type: "REMOVE_FEED"; id: number }
  | { type: "MARK_READ"; entryId: number; feedId: number }
  | { type: "MARK_ALL_READ"; feedId: number }
  | { type: "TOGGLE_STAR"; entryId: number }
  | { type: "SET_TAGS"; tags: Tag[] }
  | { type: "SET_SIDEBAR_MODE"; mode: "feed" | "tag" }
  | { type: "TOGGLE_TAG_SELECTION"; tagId: number }
  | { type: "SET_TAG_MATCH_MODE"; mode: "or" | "and" }
  | { type: "SET_BATCH_TAGGING"; isRunning: boolean }
  | { type: "SET_SIDEBAR_COUNTS"; counts: SidebarCounts }
  | { type: "SET_SYNC_STATUS"; status: "idle" | "syncing" | "failed"; error?: string }
  | { type: "APPEND_ENTRIES"; entries: EntryListItem[] }
  | { type: "SET_PAGE"; page: number }
  | { type: "SET_LOADING_MORE"; loading: boolean };

const initialState: State = {
  feeds: [],
  feedSelection: { type: "all" },
  selectedEntry: null,
  viewMode: "list",
  entries: [],
  entriesTotal: 0,
  currentPage: 1,
  isLoadingMore: false,
  searchQuery: "",
  sidebarCollapsed: false,
  tags: [],
  sidebarMode: "feed",
  selectedTagIds: [],
  tagMatchMode: "or",
  isBatchTagging: false,
  sidebarCounts: { totalUnread: 0, totalStarred: 0, starredUnread: 0 },
  syncStatus: "idle",
  syncError: "",
};

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case "SET_FEEDS":
      return { ...state, feeds: action.feeds };
    case "SET_FEED_SELECTION":
      return { ...state, feedSelection: action.selection, selectedEntry: null, viewMode: "list", searchQuery: "", selectedTagIds: [] };
    case "SET_VIEW_MODE":
      return { ...state, viewMode: action.mode };
    case "SET_ENTRIES":
      return { ...state, entries: action.entries, entriesTotal: action.total, currentPage: 1 };
    case "APPEND_ENTRIES":
      return { ...state, entries: [...state.entries, ...action.entries], isLoadingMore: false };
    case "SET_LOADING_MORE":
      return { ...state, isLoadingMore: action.loading };
    case "SET_SEARCH_QUERY":
      return { ...state, searchQuery: action.query };
    case "TOGGLE_SIDEBAR":
      return { ...state, sidebarCollapsed: !state.sidebarCollapsed };
    case "ADD_FEED":
      return { ...state, feeds: [...state.feeds, action.feed] };
    case "REMOVE_FEED": {
      const feeds = state.feeds.filter((f) => f.id !== action.id);
      const isRemovingSelected = state.feedSelection.type === "feed" && state.feedSelection.feedId === action.id;
      return {
        ...state,
        feeds,
        feedSelection: isRemovingSelected ? { type: "all" } : state.feedSelection,
        selectedEntry: isRemovingSelected ? null : state.selectedEntry,
        viewMode: isRemovingSelected ? "list" : state.viewMode,
      };
    }
    case "SET_SELECTED_ENTRY":
      return { ...state, selectedEntry: action.entry, viewMode: "reader" };
    case "MARK_READ": {
      const feeds = state.feeds.map((f) =>
        (state.feedSelection.type === "feed" && f.id === state.feedSelection.feedId) && f.unreadCount > 0
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
      const counts = { ...state.sidebarCounts, totalUnread: Math.max(0, state.sidebarCounts.totalUnread - 1) };
      return { ...state, feeds, entries, selectedEntry, sidebarCounts: counts };
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
    case "TOGGLE_STAR": {
      const entries = state.entries.map((e) =>
        e.id === action.entryId ? { ...e, isStarred: !e.isStarred } : e
      );
      return { ...state, entries };
    }
    case "SET_TAGS":
      return { ...state, tags: action.tags };
    case "SET_SIDEBAR_MODE":
      if (action.mode === "feed") {
        return { ...state, sidebarMode: "feed", selectedTagIds: [], selectedEntry: null, viewMode: "list", feedSelection: { type: "all" }, entries: [], entriesTotal: 0 };
      }
      return { ...state, sidebarMode: action.mode, selectedEntry: null, viewMode: "list", entries: [], entriesTotal: 0 };
    case "TOGGLE_TAG_SELECTION": {
      const current = state.selectedTagIds;
      if (current.includes(action.tagId)) {
        const next = current.filter(id => id !== action.tagId);
        return {
          ...state,
          selectedTagIds: next,
          viewMode: next.length > 0 ? "list" : state.viewMode,
        };
      } else {
        return {
          ...state,
          selectedTagIds: [...current, action.tagId],
          viewMode: "list",
        };
      }
    }
    case "SET_TAG_MATCH_MODE":
      return { ...state, tagMatchMode: action.mode };
    case "SET_BATCH_TAGGING":
      return { ...state, isBatchTagging: action.isRunning };
    case "SET_SIDEBAR_COUNTS":
      return { ...state, sidebarCounts: action.counts };
    case "SET_SYNC_STATUS":
      return { ...state, syncStatus: action.status, syncError: action.error ?? "" };
    default:
      return state;
  }
}

// ---- Context ----

interface AppContextType {
  feeds: FeedSummary[];
  feedSelection: FeedSelection;
  selectedEntry: Entry | null;
  viewMode: ViewMode;
  entries: EntryListItem[];
  entriesTotal: number;
  currentPage: number;
  isLoadingMore: boolean;
  searchQuery: string;
  sidebarCollapsed: boolean;
  tags: Tag[];
  sidebarMode: "feed" | "tag";
  selectedTagIds: number[];
  tagMatchMode: "or" | "and";
  isBatchTagging: boolean;
  sidebarCounts: SidebarCounts;
  syncStatus: "idle" | "syncing" | "failed";
  syncError: string;

  selectAll: () => void;
  selectStarred: () => void;
  selectFeed: (feedId: number) => void;
  selectTag: (tagId: number | null) => void;
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
  toggleStar: (entryId: number) => Promise<void>;
  reloadTags: () => void;
  setSidebarMode: (mode: "feed" | "tag") => void;
  toggleTagSelection: (tagId: number) => void;
  setTagMatchMode: (mode: "or" | "and") => void;
  setBatchTagging: (isRunning: boolean) => void;
  loadMore: () => void;
  hasMore: boolean;
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
            dispatch({ type: "SET_FEED_SELECTION", selection: { type: "all" } });
          }
        })
        .catch(() => {
          if (!cancelled) {
            dispatch({ type: "SET_FEEDS", feeds: mockFeedSummaries });
            dispatch({ type: "SET_FEED_SELECTION", selection: { type: "all" } });
          }
        });
      listTagsReal()
        .then((tags) => {
          if (!cancelled) dispatch({ type: "SET_TAGS", tags });
        })
        .catch(() => {});
    } else {
      dispatch({ type: "SET_FEEDS", feeds: mockFeedSummaries });
      dispatch({ type: "SET_FEED_SELECTION", selection: { type: "all" } });
      mockListTags().then(tags => dispatch({ type: "SET_TAGS", tags }));
    }
    return () => { cancelled = true; };
  }, []);

  // ---- 加载 sidebar counts ----
  useEffect(() => {
    if (isTauri()) {
      getSidebarCountsReal().then(counts => {
        dispatch({ type: "SET_SIDEBAR_COUNTS", counts });
      }).catch(() => {});
    }
  }, [state.feeds]); // reload when feeds change

  // ---- 选中的 feed/tag/starred/all 变化 → 加载 entries ----
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
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries, total: page.total });
            })
            .catch(() => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: [], total: 0 });
            });
        });
      } else {
        const mockEntries2 = mockApi.filterEntriesByTags(state.selectedTagIds, state.tagMatchMode);
        dispatch({ type: "SET_ENTRIES", entries: mockEntries2, total: mockEntries2.length });
      }
    } else {
      const sel = state.feedSelection;
      if (sel.type === "all") {
        if (isTauri()) {
          listAllEntriesReal(1, 50)
            .then((page) => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries, total: page.total });
            })
            .catch(() => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: [], total: 0 });
            });
        } else {
          const allMock = Object.values(mockEntries).flat();
          dispatch({ type: "SET_ENTRIES", entries: allMock, total: allMock.length });
        }
      } else if (sel.type === "starred") {
        if (isTauri()) {
          listAllEntriesReal(1, 50, "starred")
            .then((page) => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries, total: page.total });
            })
            .catch(() => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: [], total: 0 });
            });
        } else {
          dispatch({ type: "SET_ENTRIES", entries: [], total: 0 });
        }
      } else if (sel.type === "feed") {
        if (isTauri()) {
          listEntriesReal(sel.feedId, 1, 50)
            .then((page: EntryPage) => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries, total: page.total });
            })
            .catch(() => {
              if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: mockEntries[sel.feedId] || [], total: (mockEntries[sel.feedId] || []).length });
            });
        } else {
          dispatch({ type: "SET_ENTRIES", entries: mockEntries[sel.feedId] || [], total: (mockEntries[sel.feedId] || []).length });
        }
      } else if (sel.type === "tag") {
        if (isTauri()) {
          import("@tauri-apps/api/core").then(({ invoke }) => {
            invoke<EntryPage>("list_entries_by_tag", { tagId: sel.tagId, page: 1, pageSize: 50 })
              .then((page) => {
                if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries, total: page.total });
              })
              .catch(() => {
                if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: [], total: 0 });
              });
          });
        } else {
          dispatch({ type: "SET_ENTRIES", entries: [], total: 0 });
        }
      } else {
        dispatch({ type: "SET_ENTRIES", entries: [], total: 0 });
      }
    }
    return () => { cancelled = true; };
  }, [state.feedSelection, state.selectedTagIds, state.tagMatchMode]);

  // ---- 搜索 ----
  useEffect(() => {
    let cancelled = false;
    if (!state.searchQuery) return;

    if (isTauri()) {
      searchEntriesReal(state.searchQuery, 1, 50)
        .then((page: EntryPage) => {
          if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries, total: page.total });
        })
        .catch(() => {
          if (!cancelled)
            dispatch({ type: "SET_ENTRIES", entries: mockApi.searchEntries(state.searchQuery), total: mockApi.searchEntries(state.searchQuery).length });
        });
    } else {
      dispatch({ type: "SET_ENTRIES", entries: mockApi.searchEntries(state.searchQuery), total: mockApi.searchEntries(state.searchQuery).length });
    }
    return () => { cancelled = true; };
  }, [state.searchQuery]);

  const selectAll = useCallback(() => {
    dispatch({ type: "SET_FEED_SELECTION", selection: { type: "all" } });
  }, []);

  const selectStarred = useCallback(() => {
    dispatch({ type: "SET_FEED_SELECTION", selection: { type: "starred" } });
  }, []);

  const selectFeed = useCallback((feedId: number) => {
    dispatch({ type: "SET_FEED_SELECTION", selection: { type: "feed", feedId } });
  }, []);

  const selectTagFn = useCallback((tagId: number | null) => {
    if (tagId) {
      dispatch({ type: "SET_FEED_SELECTION", selection: { type: "tag", tagId } });
    }
  }, []);

  const selectEntry = useCallback((item: EntryListItem) => {
    if (isTauri()) {
      getEntryReal(item.id)
        .then((entry) => {
          dispatch({ type: "SET_SELECTED_ENTRY", entry });
          markEntryRead(item.id);
        })
        .catch(() => {
          const fallback = mockApi.getEntry(item.id);
          dispatch({ type: "SET_SELECTED_ENTRY", entry: fallback || null });
        });
    } else {
      const fallback = mockApi.getEntry(item.id);
      dispatch({ type: "SET_SELECTED_ENTRY", entry: fallback || null });
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
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
            .then((page: EntryPage) => dispatch({ type: "SET_ENTRIES", entries: page.entries, total: page.total }))
            .catch((e) => { toast(t("加载文章列表失败: ") + String(e), "error"); });
        })
        .catch((e) => { toast(t("刷新订阅源失败: ") + String(e), "error"); });
    }
  }, []);

  const refreshAllFn = useCallback(() => {
    if (isTauri()) {
      dispatch({ type: "SET_SYNC_STATUS", status: "syncing" });
      refreshAllFeedsReal()
        .then(() => listFeedsReal())
        .then((data) => {
          dispatch({ type: "SET_FEEDS", feeds: data });
          dispatch({ type: "SET_SYNC_STATUS", status: "idle" });
        })
        .catch((e) => {
          dispatch({ type: "SET_SYNC_STATUS", status: "failed", error: String(e) });
        });
    } else {
      dispatch({ type: "SET_SYNC_STATUS", status: "idle" });
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

  const toggleStarFn = useCallback(async (entryId: number) => {
    try {
      const starred = await toggleStarReal(entryId);
      dispatch({ type: "TOGGLE_STAR", entryId });
      // Reload sidebar counts
      if (isTauri()) {
        const counts = await getSidebarCountsReal();
        dispatch({ type: "SET_SIDEBAR_COUNTS", counts });
      }
      // If viewing starred and unstarred, remove from list
      if (state.feedSelection.type === "starred" && !starred) {
        dispatch({ type: "SET_ENTRIES", entries: state.entries.filter(e => e.id !== entryId), total: state.entries.filter(e => e.id !== entryId).length });
      }
    } catch (e) {
      toast(t("收藏操作失败: ") + String(e), "error");
    }
  }, [state.feedSelection.type, state.entries]);

  const markAllRead = useCallback((feedId: number) => {
    if (isTauri()) {
      import("@tauri-apps/api/core").then(({ invoke }) => {
        invoke("mark_all_read", { feedId }).catch(() => {});
      });
    }
    dispatch({ type: "MARK_ALL_READ", feedId });
  }, []);

  const selectTag = useCallback((tagId: number | null) => {
    selectTagFn(tagId);
  }, [selectTagFn]);

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

  // Use a ref for isLoadingMore to avoid recreating loadMore callback on every load state change
  const isLoadingMoreRef = useRef(false);

  const loadMore = useCallback(() => {
    if (isLoadingMoreRef.current) return;
    const nextPage = state.currentPage + 1;
    isLoadingMoreRef.current = true;
    dispatch({ type: "SET_LOADING_MORE", loading: true });

    const sel = state.feedSelection;
    const fetchPage = (): Promise<EntryPage> => {
      if (state.selectedTagIds.length > 0) {
        return import("@tauri-apps/api/core").then(({ invoke }) =>
          invoke<EntryPage>("list_entries_by_tags", {
            tagIds: state.selectedTagIds,
            matchMode: state.tagMatchMode,
            page: nextPage,
            pageSize: 50,
          })
        );
      }
      if (sel.type === "all") return listAllEntriesReal(nextPage, 50);
      if (sel.type === "starred") return listAllEntriesReal(nextPage, 50, "starred");
      if (sel.type === "feed") return listEntriesReal(sel.feedId, nextPage, 50);
      if (sel.type === "tag") {
        return import("@tauri-apps/api/core").then(({ invoke }) =>
          invoke<EntryPage>("list_entries_by_tag", { tagId: sel.tagId, page: nextPage, pageSize: 50 })
        );
      }
      return Promise.resolve({ entries: [], total: state.entriesTotal, page: nextPage, pageSize: 50 });
    };

    fetchPage()
      .then((page) => {
        isLoadingMoreRef.current = false;
        if (page.entries.length > 0) {
          dispatch({ type: "APPEND_ENTRIES", entries: page.entries });
          dispatch({ type: "SET_PAGE", page: nextPage });
        } else {
          dispatch({ type: "SET_LOADING_MORE", loading: false });
        }
      })
      .catch(() => {
        isLoadingMoreRef.current = false;
        dispatch({ type: "SET_LOADING_MORE", loading: false });
      });
  }, [state.feedSelection, state.selectedTagIds, state.tagMatchMode, state.currentPage, state.entriesTotal]);

  const hasMore = state.entries.length < state.entriesTotal;

  const setSearchQuery = useCallback((query: string) => {
    dispatch({ type: "SET_SEARCH_QUERY", query });
  }, []);

  return (
    <AppContext.Provider
      value={{
        ...state,
        selectAll,
        selectStarred,
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
        toggleStar: toggleStarFn,
        selectTag,
        reloadTags,
        setSidebarMode,
        toggleTagSelection,
        setTagMatchMode,
        setBatchTagging,
        loadMore,
        hasMore,
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