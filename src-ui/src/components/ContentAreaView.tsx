import { useApp } from "@/contexts/AppContext";
import { t } from "@/lib/utils";
import { EntryListView } from "./EntryListView";
import { ReaderView } from "./ReaderView";
import { SettingsPageView } from "./SettingsPageView";

export function ContentAreaView() {
  const { viewMode, entries, entriesTotal, selectedEntry, feedSelection, feeds, sidebarCounts, markAllRead, selectEntry } = useApp();

  if (viewMode === "settings") {
    return <SettingsPageView />;
  }

  // Get accurate unread count — sidebar data comes from DB COUNT, not limited by pagination
  const currentUnread = (() => {
    switch (feedSelection.type) {
      case "feed":
        return feeds.find(f => f.id === feedSelection.feedId)?.unreadCount ?? 0;
      case "all":
        return sidebarCounts.totalUnread;
      case "starred":
        return sidebarCounts.starredUnread;
      default:
        return entries.filter(e => !e.isRead).length;
    }
  })();

  const handleMarkAllRead = () => {
    if (feedSelection.type === "feed") {
      markAllRead(feedSelection.feedId);
    }
  };

  return (
    <div className="flex-1 flex">
      {/* Column 2: Entry list */}
      <div
        className="flex-shrink border-r border-[var(--border-color)] flex flex-col bg-[var(--bg-primary)]"
        style={{ width: "min(320px, 24vw)" }}
      >
        {/* Toolbar */}
        <div className="px-4 py-3 border-b border-[var(--border-color)] flex items-center justify-between">
          <h2 className="text-sm font-semibold">
            {entriesTotal > 0 ? t("文章 ({count})").replace("{count}", String(entriesTotal)) : t("文章")}
          </h2>
          <div className="flex items-center gap-2">
            <span className="text-xs text-[var(--text-tertiary)]">
              {t("未读")} {currentUnread}/{entriesTotal}
            </span>
            {entries.some((e) => !e.isRead) && (
              <button
                onClick={handleMarkAllRead}
                className="text-xs text-[var(--link-color)] hover:underline"
              >
                {t("全部已读")}
              </button>
            )}
          </div>
        </div>
        <EntryListView />
      </div>

      {/* Column 3: Reader */}
      <div className="flex-1 flex flex-col min-w-0">
        {selectedEntry ? (
          <ReaderView key={selectedEntry?.id ?? "empty"} />
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <p className="text-[var(--text-tertiary)] text-sm">{t("选择一篇文章开始阅读")}</p>
          </div>
        )}
      </div>
    </div>
  );
}