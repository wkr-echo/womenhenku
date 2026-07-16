import { useState, useEffect } from "react";
import { useApp } from "@/contexts/AppContext";
import { mockContent, mockSummary } from "@/api/mock";
import { formatDate, t } from "@/lib/utils";
import { Button } from "@/components/ui";
import { SummaryPanelView } from "./SummaryPanelView";
import { TranslationPanelView } from "./TranslationPanelView";
import { NoteEditorView } from "./NoteEditorView";
import { isTauri, getEntryContent as getEntryContentReal, processEntryContent, exportSingleDigest } from "@/api/feed";
import { toast } from "@/components/ui/Toast";
import type { Content } from "@/lib/types";

type ReaderTab = "read" | "summary" | "translate" | "notes";
type ExportFormat = "markdown" | "html" | "plaintext";

const EXPORT_LABELS: Record<ExportFormat, string> = {
  markdown: "Markdown",
  html: "HTML",
  plaintext: "纯文本",
};

export function ReaderView() {
  const { selectedEntry, setViewMode, markEntryRead } = useApp();
  const [activeTab, setActiveTab] = useState<ReaderTab>("read");
  const [exporting, setExporting] = useState(false);
  const [showExportMenu, setShowExportMenu] = useState(false);

  const handleExport = async (format: ExportFormat) => {
    setShowExportMenu(false);
    setExporting(true);
    try {
      const content = isTauri()
        ? await exportSingleDigest(selectedEntry!.id, format)
        : mockDigestExport(selectedEntry!, format);

      const mimeTypes: Record<ExportFormat, string> = {
        markdown: "text/markdown",
        html: "text/html",
        plaintext: "text/plain",
      };
      const extensions: Record<ExportFormat, string> = {
        markdown: ".md",
        html: ".html",
        plaintext: ".txt",
      };

      const blob = new Blob([content], { type: mimeTypes[format] });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${selectedEntry!.title}${extensions[format]}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      toast(t(`已导出 ${EXPORT_LABELS[format]}`), "success");
    } catch {
      toast(t("导出失败"), "error");
    } finally {
      setExporting(false);
    }
  };

  if (!selectedEntry) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-[var(--text-tertiary)] text-sm">{t("选择一篇文章开始阅读")}</p>
      </div>
    );
  }

  const [content, setContent] = useState<Content | null>(null);
  const [contentLoading, setContentLoading] = useState(false);
  const [contentError, setContentError] = useState<string | null>(null);

  // Load real content from backend when entry changes.
  // If content hasn't been processed yet, run the reader pipeline first.
  useEffect(() => {
    if (!selectedEntry) return;
    let cancelled = false;
    setContentLoading(true);
    setContentError(null);

    async function load() {
      if (!isTauri()) {
        if (!cancelled) { setContent(mockContent); setContentLoading(false); }
        return;
      }

      // Helper: check if Content has usable rendered output
      const hasContent = (c: Content) =>
        !!c.renderedHtml || !!c.cleanedHtml || !!c.rawHtml;

      try {
        // Try to get existing cached content
        const c = await getEntryContentReal(selectedEntry!.id);
        if (hasContent(c)) {
          // Content is already processed and cached
          if (!cancelled) setContent(c);
          return;
        }
        // Content row exists but is empty — run pipeline
        if (!cancelled) setContentLoading(true);
        throw new Error("Empty cache");
      } catch {
        // Content not yet processed — run the reader pipeline
        try {
          const url = selectedEntry!.link || "";
          if (!url) throw new Error("No article URL");
          await processEntryContent(selectedEntry!.id, url);
          const c = await getEntryContentReal(selectedEntry!.id);
          if (!cancelled) setContent(c);
        } catch (e: any) {
          if (!cancelled) {
            setContent(null);
            setContentError(String(e));
          }
        }
      } finally {
        if (!cancelled) setContentLoading(false);
      }
    }

    load();
    return () => { cancelled = true; };
  }, [selectedEntry?.id]);

  // Mark as read after 1 second of viewing
  useEffect(() => {
    if (!selectedEntry || !isTauri()) return;
    const timer = setTimeout(() => {
      markEntryRead(selectedEntry.id);
    }, 1000);
    return () => clearTimeout(timer);
  }, [selectedEntry?.id]);

  const tabs: { key: ReaderTab; label: string; shortcut: string }[] = [
    { key: "read", label: t("阅读"), shortcut: "" },
    { key: "summary", label: t("摘要"), shortcut: "S" },
    { key: "translate", label: t("翻译"), shortcut: "T" },
    { key: "notes", label: t("笔记"), shortcut: "N" },
  ];

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="px-6 py-4 border-b border-[var(--border-color)]">
        <div className="flex items-center gap-3 mb-3">
          <button
            onClick={() => setViewMode("list")}
            className="p-1.5 rounded-lg hover:bg-[var(--bg-tertiary)] text-[var(--text-secondary)] transition-colors"
            title={t("返回列表")}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path strokeLinecap="round" strokeLinejoin="round" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
            </svg>
          </button>
          <div className="flex-1 min-w-0">
            <h1 className="text-lg font-semibold text-[var(--text-primary)] leading-snug">
              {selectedEntry.title}
            </h1>
            <div className="flex items-center gap-3 mt-1 text-xs text-[var(--text-tertiary)]">
              {selectedEntry.author && <span>{selectedEntry.author}</span>}
              <span>{formatDate(selectedEntry.publishedAt)}</span>
              {selectedEntry.link && (
                <a
                  href={selectedEntry.link}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-[var(--link-color)] hover:underline"
                >
                  {t("原文")}
                </a>
              )}
            </div>
          </div>
          {/* Export dropdown */}
          <div className="relative">
            <button
              onClick={() => setShowExportMenu(!showExportMenu)}
              disabled={exporting}
              className="p-1.5 rounded-lg hover:bg-[var(--bg-tertiary)] text-[var(--text-secondary)] transition-colors disabled:opacity-50"
              title={t("导出文摘")}
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </button>
            {showExportMenu && (
              <div className="absolute right-0 top-full mt-1 w-32 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] shadow-lg z-50 py-1">
                {(Object.keys(EXPORT_LABELS) as ExportFormat[]).map((fmt) => (
                  <button
                    key={fmt}
                    onClick={() => handleExport(fmt)}
                    className="w-full text-left px-3 py-2 text-xs text-[var(--text-primary)] hover:bg-[var(--bg-secondary)] transition-colors"
                  >
                    {t(EXPORT_LABELS[fmt])}
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Tabs */}
        <div className="flex items-center gap-1">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
                activeTab === tab.key
                  ? "bg-[var(--accent-color)] text-white"
                  : "text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]"
              }`}
            >
              {tab.label}
              {tab.shortcut && (
                <span className="ml-1.5 text-[10px] opacity-60">{tab.shortcut}</span>
              )}
            </button>
          ))}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {activeTab === "read" && (
          <div className="max-w-3xl mx-auto px-6 py-6">
            {contentLoading ? (
              <p className="text-sm text-[var(--text-tertiary)]">{t("加载中...")}</p>
            ) : content?.renderedHtml || content?.cleanedHtml || content?.rawHtml ? (
              <div
                className="reader-content"
                dangerouslySetInnerHTML={{ __html: (content.renderedHtml || content.cleanedHtml || content.rawHtml)! }}
              />
            ) : (
              <div className="text-sm text-[var(--text-secondary)] leading-relaxed whitespace-pre-wrap">
                {contentError ? (
                  <div className="text-[var(--text-tertiary)]">
                    <p className="text-red-500 mb-2">加载失败: {contentError}</p>
                    <p>{selectedEntry.summary || t("暂无内容")}</p>
                  </div>
                ) : (
                  selectedEntry.summary || t("暂无内容")
                )}
              </div>
            )}
          </div>
        )}
        {activeTab === "summary" && <SummaryPanelView entryId={selectedEntry.id} />}
        {activeTab === "translate" && <TranslationPanelView entryId={selectedEntry.id} />}
        {activeTab === "notes" && <NoteEditorView entryId={selectedEntry.id} />}
      </div>
    </div>
  );
}

/** Mock digest export for non-Tauri (browser) mode. */
function mockDigestExport(entry: { title: string; author: string; link: string }, format: ExportFormat): string {
  const title = entry.title;
  const author = entry.author;
  const link = entry.link;

  switch (format) {
    case "markdown":
      return `# ${title}\n\n**作者**: ${author}\n\n**原文链接**: ${link}\n\n---\n\n> 这是 mock 内容。在 Tauri 环境中将显示真实文章内容。\n`;
    case "html":
      return `<!DOCTYPE html>\n<html>\n<head><meta charset="utf-8"><title>${title}</title></head>\n<body>\n<h1>${title}</h1>\n<p><strong>作者:</strong> ${author}</p>\n<p><a href="${link}">原文链接</a></p>\n<hr>\n<blockquote>这是 mock 内容。在 Tauri 环境中将显示真实文章内容。</blockquote>\n</body>\n</html>\n`;
    case "plaintext":
      return `${title}\n${"=".repeat(title.length)}\n\n作者: ${author}\n原文链接: ${link}\n\n这是 mock 内容。在 Tauri 环境中将显示真实文章内容。\n`;
  }
}