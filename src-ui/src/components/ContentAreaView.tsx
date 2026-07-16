import { useApp } from "@/contexts/AppContext";
import { t } from "@/lib/utils";
import { EntryListView } from "./EntryListView";
import { ReaderView } from "./ReaderView";
import { SettingsPageView } from "./SettingsPageView";

export function ContentAreaView() {
  const { viewMode, entries, selectedEntry } = useApp();

  if (viewMode === "settings") {
    return <SettingsPageView />;
  }

  if (viewMode === "reader" && selectedEntry) {
    return <ReaderView />;
  }

  return (
    <div className="flex-1 flex flex-col">
      {/* Toolbar */}
      <div className="px-5 py-3 border-b border-[var(--border-color)] flex items-center justify-between bg-[var(--bg-primary)]">
        <div className="flex items-center gap-2">
          <h2 className="text-sm font-semibold">
            {entries.length > 0 ? t("文章 ({count})").replace("{count}", String(entries.length)) : t("文章")}
          </h2>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-[var(--text-tertiary)]">
            {t("未读")} {entries.filter((e) => !e.isRead).length} / {t("共")} {entries.length}
          </span>
        </div>
      </div>
      <EntryListView />
    </div>
  );
}