import { useApp } from "@/contexts/AppContext";
import { EntryList } from "./EntryList";
import { ReaderView } from "./ReaderView";
import { SettingsPage } from "./SettingsPage";

export function ContentArea() {
  const { viewMode, entries, selectedEntry } = useApp();

  if (viewMode === "settings") {
    return <SettingsPage />;
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
            {entries.length > 0 ? `文章 (${entries.length})` : "文章"}
          </h2>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-[var(--text-tertiary)]">
            未读 {entries.filter((e) => e.is_read === 0).length} / 共 {entries.length}
          </span>
        </div>
      </div>
      <EntryList />
    </div>
  );
}