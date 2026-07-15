import { createContext, useContext, useState, useCallback, type ReactNode } from "react";
import type { FeedSummary, EntryListItem, Entry, ViewMode } from "@/lib/types";
import { mockFeedSummaries, mockEntries, mockApi } from "@/api/mock";

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
  const [feeds, setFeeds] = useState<FeedSummary[]>(mockFeedSummaries);
  const [selectedFeedId, setSelectedFeedId] = useState<number | null>(1);
  const [selectedEntry, setSelectedEntry] = useState<Entry | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>("list");
  const [searchQuery, setSearchQuery] = useState("");
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  const entries = selectedFeedId
    ? searchQuery
      ? mockApi.searchEntries(searchQuery)
      : mockEntries[selectedFeedId] || []
    : [];

  const selectFeed = useCallback((feedId: number) => {
    setSelectedFeedId(feedId);
    setSelectedEntry(null);
    setViewMode("list");
    setSearchQuery("");
  }, []);

  const selectEntry = useCallback((item: EntryListItem) => {
    const fullEntry = mockApi.getEntry(item.id);
    setSelectedEntry(fullEntry || null);
    setViewMode("reader");
  }, []);

  const toggleSidebar = useCallback(() => {
    setSidebarCollapsed((prev) => !prev);
  }, []);

  const addFeed = useCallback((url: string) => {
    const newFeed: FeedSummary = {
      id: Date.now(),
      title: url.replace(/https?:\/\//, "").split("/")[0],
      unreadCount: 0,
    };
    setFeeds((prev) => [...prev, newFeed]);
  }, []);

  const removeFeed = useCallback((id: number) => {
    setFeeds((prev) => prev.filter((f) => f.id !== id));
    if (selectedFeedId === id) {
      setSelectedFeedId(null);
      setSelectedEntry(null);
      setViewMode("list");
    }
  }, [selectedFeedId]);

  const refreshFeed = useCallback((_id: number) => {
    // mock: no-op
  }, []);

  const refreshAll = useCallback(() => {
    // mock: no-op
  }, []);

  return (
    <AppContext.Provider
      value={{
        feeds,
        selectedFeedId,
        selectedEntry,
        viewMode,
        entries,
        searchQuery,
        sidebarCollapsed,
        selectFeed,
        selectEntry,
        setViewMode,
        setSearchQuery,
        toggleSidebar,
        addFeed,
        removeFeed,
        refreshFeed,
        refreshAll,
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