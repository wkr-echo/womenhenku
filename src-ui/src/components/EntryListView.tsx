import { useApp } from "@/contexts/AppContext";
import { cn, formatDate, truncate, t } from "@/lib/utils";
import type { EntryListItem } from "@/lib/types";

export function EntryListView() {
  const { entries, selectedEntry, selectEntry, searchQuery } = useApp();

  if (entries.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-center">
          <div className="text-4xl mb-4 opacity-30">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" className="mx-auto">
              <path strokeLinecap="round" strokeLinejoin="round" d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
            </svg>
          </div>
          <p className="text-[var(--text-tertiary)] text-sm">暂无文章</p>
          <p className="text-[var(--text-tertiary)] text-xs mt-1">{t("选择一个订阅源或添加新的订阅源")}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="divide-y divide-[var(--border-color)]">
        {entries.map((entry) => (
          <EntryItem
            key={entry.id}
            entry={entry}
            isSelected={selectedEntry?.id === entry.id}
            searchQuery={searchQuery}
            onClick={() => selectEntry(entry)}
          />
        ))}
      </div>
    </div>
  );
}

function highlightText(text: string, query: string): React.ReactNode {
  if (!query) return text;
  const idx = text.toLowerCase().indexOf(query.toLowerCase());
  if (idx === -1) return text;
  return (
    <>
      {text.slice(0, idx)}
      <mark className="bg-yellow-300 dark:bg-yellow-600 text-inherit rounded px-0.5">
        {text.slice(idx, idx + query.length)}
      </mark>
      {text.slice(idx + query.length)}
    </>
  );
}

function EntryItem({
  entry,
  isSelected,
  searchQuery,
  onClick,
}: {
  entry: EntryListItem;
  isSelected: boolean;
  searchQuery: string;
  onClick: () => void;
}) {
  return (
    <div
      onClick={onClick}
      className={cn(
        "px-5 py-4 cursor-pointer transition-colors border-l-2",
        isSelected
          ? "border-l-[var(--accent-color)] bg-[var(--bg-tertiary)]"
          : "border-l-transparent hover:bg-[var(--bg-secondary)]"
      )}
    >
      <div className="flex items-start gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            {!entry.isRead && (
              <span className="w-2 h-2 rounded-full bg-[var(--accent-color)] flex-shrink-0" />
            )}
            <h3
              className={cn(
                "text-sm leading-snug",
                !entry.isRead
                  ? "font-semibold text-[var(--text-primary)]"
                  : "font-normal text-[var(--text-secondary)]"
              )}
            >
              {highlightText(truncate(entry.title, 80), searchQuery)}
            </h3>
          </div>
          <div className="flex items-center gap-3 mt-2 text-xs text-[var(--text-tertiary)]">
            {entry.author && <span>{entry.author}</span>}
            <span>{formatDate(entry.publishedAt)}</span>
          </div>
        </div>
      </div>
    </div>
  );
}