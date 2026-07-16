import {
  createContext,
  useContext,
  useReducer,
  useCallback,
  useEffect,
  type ReactNode,
} from "react";
import type { FeedSummary, EntryListItem, Entry, EntryPage, ViewMode } from "@/lib/types";
import { mockFeedSummaries, mockEntries, mockApi } from "@/api/mock";
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
  | { type: "SELECT_ENTRY"; entry: Entry | null };

const initialState: State = {
  feeds: [],
  selectedFeedId: null,
  selectedEntry: null,
  viewMode: "list",
  entries: [],
  searchQuery: "",
  sidebarCollapsed: false,
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

  selectFeed: (feedId: number) => void;
  selectEntry: (entry: EntryListItem) => void;
  setViewMode: (mode: ViewMode) => void;
  setSearchQuery: (query: string) => void;
  toggleSidebar: () => void;
  addFeed: (url: string) => void;
  removeFeed: (id: number) => void;
  refreshFeed: (id: number) => void;
  refreshAll: () => void;
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
    } else {
      dispatch({ type: "SET_FEEDS", feeds: mockFeedSummaries });
      dispatch({ type: "SET_SELECTED_FEED_ID", feedId: 1 });
    }
    return () => { cancelled = true; };
  }, []);

  // ---- 选中的 feed 变化 → 加载 entries ----
  useEffect(() => {
    let cancelled = false;
    if (!state.selectedFeedId) {
      dispatch({ type: "SET_ENTRIES", entries: [] });
      return;
    }

    if (isTauri()) {
      listEntriesReal(state.selectedFeedId, 1, 50)
        .then((page: EntryPage) => {
          if (!cancelled) dispatch({ type: "SET_ENTRIES", entries: page.entries });
        })
        .catch(() => {
          if (!cancelled)
            dispatch({ type: "SET_ENTRIES", entries: mockEntries[state.selectedFeedId!] || [] });
        });
    } else {
      dispatch({ type: "SET_ENTRIES", entries: mockEntries[state.selectedFeedId] || [] });
    }
    return () => { cancelled = true; };
  }, [state.selectedFeedId]);

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