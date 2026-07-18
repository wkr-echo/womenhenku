import { useState } from "react";
import { useApp } from "@/contexts/AppContext";
import { cn, formatDate, truncate, t } from "@/lib/utils";
import { exportMultiDigest, writeTextFile } from "@/api/feed";
import { isTauri } from "@/api/feed";
import { toast } from "@/components/ui/Toast";
import type { EntryListItem } from "@/lib/types";

export function EntryListView() {
  const { entries, selectedEntry, selectEntry, searchQuery } = useApp();
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [exporting, setExporting] = useState(false);

  const toggleSelect = (id: number) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const clearSelection = () => setSelectedIds(new Set());

  const handleBatchExport = async (format: "markdown" | "html") => {
    if (selectedIds.size === 0) return;
    setExporting(true);
    try {
      const ids = Array.from(selectedIds);
      const content = await exportMultiDigest(ids, format);

      const ext = format === "markdown" ? ".md" : ".html";
      if (isTauri()) {
        const { save } = await import("@tauri-apps/plugin-dialog");
        const { homeDir } = await import("@tauri-apps/api/path");
        const home = await homeDir();
        const filePath = await save({
          defaultPath: `${home}digest${ext}`,
          filters: [{ name: format === "markdown" ? "Markdown" : "HTML", extensions: [ext.slice(1)] }],
        });
        if (!filePath) { setExporting(false); return; }
        await writeTextFile(filePath, content);
        toast(t("已导出"), "success");
      } else {
        const mime = format === "markdown" ? "text/markdown" : "text/html";
        const blob = new Blob([content], { type: mime });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `digest${ext}`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        toast(t("已导出"), "success");
      }
      clearSelection();
    } catch (e: any) {
      toast(t("导出失败: ") + String(e), "error");
    } finally {
      setExporting(false);
    }
  };

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
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto">
        <div className="divide-y divide-[var(--border-color)]">
          {entries.map((entry) => (
            <EntryItem
              key={entry.id}
              entry={entry}
              isSelected={selectedEntry?.id === entry.id}
              isChecked={selectedIds.has(entry.id)}
              searchQuery={searchQuery}
              onClick={() => selectEntry(entry)}
              onCheck={() => toggleSelect(entry.id)}
            />
          ))}
        </div>
      </div>

      {/* Batch export bar */}
      {selectedIds.size > 0 && (
        <div className="flex-shrink-0 border-t border-[var(--border-color)] bg-[var(--bg-secondary)] px-4 py-3 flex items-center justify-between">
          <span className="text-xs text-[var(--text-secondary)]">
            {t("已选 {count} 篇").replace("{count}", String(selectedIds.size))}
          </span>
          <div className="flex items-center gap-2">
            <button
              onClick={() => handleBatchExport("markdown")}
              disabled={exporting}
              className="px-3 py-1.5 text-xs rounded bg-[var(--accent-color)] text-white hover:opacity-90 disabled:opacity-50 transition-opacity"
            >
              {exporting ? "…" : t("导出 Markdown")}
            </button>
            <button
              onClick={() => handleBatchExport("html")}
              disabled={exporting}
              className="px-3 py-1.5 text-xs rounded bg-[var(--bg-tertiary)] text-[var(--text-primary)] hover:bg-[var(--border-color)] disabled:opacity-50 transition-colors"
            >
              {t("导出 HTML")}
            </button>
            <button
              onClick={clearSelection}
              disabled={exporting}
              className="px-2 py-1.5 text-xs text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-colors"
            >
              {t("取消")}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function highlightText(text: string, query: string): React.ReactNode {
  if (!query) return text;
  const lower = text.toLowerCase();
  const q = query.toLowerCase();
  const idx = lower.indexOf(q);
  if (idx === -1) return text;

  const ctxStart = Math.max(0, idx - 20);
  const ctxEnd = Math.min(text.length, idx + q.length + 20);
  const before = ctxStart > 0 ? "..." : "";
  const after = ctxEnd < text.length ? "..." : "";

  return (
    <>
      {before}
      {text.slice(ctxStart, idx)}
      <mark className="bg-yellow-300 dark:bg-yellow-600 text-inherit rounded px-0.5">
        {text.slice(idx, idx + q.length)}
      </mark>
      {text.slice(idx + q.length, ctxEnd)}
      {after}
    </>
  );
}

function EntryItem({
  entry,
  isSelected,
  isChecked,
  searchQuery,
  onClick,
  onCheck,
}: {
  entry: EntryListItem;
  isSelected: boolean;
  isChecked: boolean;
  searchQuery: string;
  onClick: () => void;
  onCheck: () => void;
}) {
  return (
    <div
      onClick={onClick}
      className={cn(
        "px-5 py-4 cursor-pointer transition-colors border-l-2 group flex items-start gap-3",
        isSelected
          ? "border-l-[var(--accent-color)] bg-[var(--bg-tertiary)]"
          : "border-l-transparent hover:bg-[var(--bg-secondary)]"
      )}
    >
      {/* Checkbox */}
      <input
        type="checkbox"
        checked={isChecked}
        onChange={(e) => { e.stopPropagation(); onCheck(); }}
        onClick={(e) => e.stopPropagation()}
        className="flex-shrink-0 mt-0.5 w-4 h-4 rounded border-[var(--border-color)] text-[var(--accent-color)] focus:ring-[var(--accent-color)] cursor-pointer"
      />

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
        {searchQuery && (
          (() => {
            const inTitle = entry.title.toLowerCase().includes(searchQuery.toLowerCase());
            const inSummary = entry.summary?.toLowerCase().includes(searchQuery.toLowerCase());
            if (inTitle) return null;
            if (inSummary) {
              return (
                <p className="mt-1 text-xs text-[var(--text-tertiary)] truncate">
                  {highlightText(entry.summary, searchQuery)}
                </p>
              );
            }
            return null;
          })()
        )}
        <div className="flex items-center gap-3 mt-1 text-xs text-[var(--text-tertiary)]">
          {entry.author && <span>{entry.author}</span>}
          <span>{formatDate(entry.publishedAt)}</span>
        </div>
      </div>
    </div>
  );
}