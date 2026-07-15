import { useApp } from "@/contexts/AppContext";
import { cn, formatDate, truncate } from "@/lib/utils";
import type { Entry } from "@/lib/types";

export function EntryList() {
  const { entries, selectedEntry, selectEntry } = useApp();

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
          <p className="text-[var(--text-tertiary)] text-xs mt-1">选择一个订阅源或添加新的订阅源</p>
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
            onClick={() => selectEntry(entry)}
          />
        ))}
      </div>
    </div>
  );
}

function EntryItem({
  entry,
  isSelected,
  onClick,
}: {
  entry: Entry;
  isSelected: boolean;
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
            {entry.is_read === 0 && (
              <span className="w-2 h-2 rounded-full bg-[var(--accent-color)] flex-shrink-0" />
            )}
            <h3
              className={cn(
                "text-sm leading-snug",
                entry.is_read === 0
                  ? "font-semibold text-[var(--text-primary)]"
                  : "font-normal text-[var(--text-secondary)]"
              )}
            >
              {truncate(entry.title, 80)}
            </h3>
          </div>
          <p className="text-xs text-[var(--text-tertiary)] mt-1.5 line-clamp-2 leading-relaxed">
            {truncate(entry.summary, 120)}
          </p>
          <div className="flex items-center gap-3 mt-2 text-xs text-[var(--text-tertiary)]">
            {entry.author && <span>{entry.author}</span>}
            <span>{formatDate(entry.published_at)}</span>
            {entry.is_starred === 1 && (
              <svg width="12" height="12" viewBox="0 0 24 24" fill="var(--warning-color)" stroke="var(--warning-color)" strokeWidth="1.5">
                <path strokeLinecap="round" strokeLinejoin="round" d="M11.48 3.499a.562.562 0 011.04 0l2.125 5.111a.563.563 0 00.475.345l5.518.442c.499.04.701.663.321.988l-4.204 3.602a.563.563 0 00-.182.557l1.285 5.385a.562.562 0 01-.84.61l-4.725-2.885a.563.563 0 00-.586 0L6.982 20.54a.562.562 0 01-.84-.61l1.285-5.386a.562.562 0 00-.182-.557l-4.204-3.602a.563.563 0 01.321-.988l5.518-.442a.563.563 0 00.475-.345L11.48 3.5z" />
              </svg>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}