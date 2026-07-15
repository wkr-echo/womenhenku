import { createContext, useContext, useState, useCallback, type ReactNode } from "react";
import type { Feed, Entry, ViewMode } from "@/lib/types";
import { mockFeeds, mockEntries, mockApi } from "@/api/mock";

interface AppContextType {
  feeds: Feed[];
  selectedFeedId: number | null;
  selectedEntry: Entry | null;
  viewMode: ViewMode;
  entries: Entry[];
  searchQuery: string;
  sidebarCollapsed: boolean;

  selectFeed: (feedId: number) => void;
  selectEntry: (entry: Entry) => void;
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
  const [feeds, setFeeds] = useState<Feed[]>(mockFeeds);
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

  const selectEntry = useCallback((entry: Entry) => {
    setSelectedEntry(entry);
    setViewMode("reader");
  }, []);

  const toggleSidebar = useCallback(() => {
    setSidebarCollapsed((prev) => !prev);
  }, []);

  const addFeed = useCallback((url: string) => {
    const newFeed: Feed = {
      id: Date.now(),
      url,
      title: url.replace(/https?:\/\//, "").split("/")[0],
      description: "",
      link: url,
      feed_type: "rss",
      last_synced_at: null,
      created_at: new Date().toISOString(),
      unread_count: 0,
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
    // mock: just update last_synced_at
    setFeeds((prev) =>
      prev.map((f) =>
        f.id === _id ? { ...f, last_synced_at: new Date().toISOString() } : f
      )
    );
  }, []);

  const refreshAll = useCallback(() => {
    setFeeds((prev) =>
      prev.map((f) => ({ ...f, last_synced_at: new Date().toISOString() }))
    );
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