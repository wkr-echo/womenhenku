import { useApp } from "@/contexts/AppContext";
import { t } from "@/lib/utils";
import { EntryListView } from "./EntryListView";
import { ReaderView } from "./ReaderView";
import { SettingsPageView } from "./SettingsPageView";

export function ContentAreaView() {
  const { viewMode, entries, selectedEntry, selectedFeedId, markAllRead, selectEntry } = useApp();

  if (viewMode === "settings") {
    return <SettingsPageView />;
  }

  const handleMarkAllRead = () => {
    if (!selectedFeedId) return;
    markAllRead(selectedFeedId);
  };

  return (
    <div className="flex-1 flex">
      {/* Column 2: Entry list */}
      <div className="w-[320px] border-r border-[var(--border-color)] flex flex-col bg-[var(--bg-primary)]">
        {/* Toolbar */}
        <div className="px-4 py-3 border-b border-[var(--border-color)] flex items-center justify-between">
          <h2 className="text-sm font-semibold">
            {entries.length > 0 ? t("文章 ({count})").replace("{count}", String(entries.length)) : t("文章")}
          </h2>
          <div className="flex items-center gap-2">
            <span className="text-xs text-[var(--text-tertiary)]">
              {t("未读")} {entries.filter((e) => !e.isRead).length}/{entries.length}
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