import { useState, useEffect, useRef } from "react";
import { useApp } from "@/contexts/AppContext";
import { mockContent, mockSummary } from "@/api/mock";
import { formatDate, t } from "@/lib/utils";
import { Button } from "@/components/ui";
import { SummaryPanelView } from "./SummaryPanelView";
import { NoteEditorView } from "./NoteEditorView";
import { isTauri, getEntryContent as getEntryContentReal, processEntryContent, exportSingleDigest } from "@/api/feed";
import {
  translateEntry,
  getTranslationText,
  cancelTranslation,
  clearTranslation as clearTranslationApi,
  listenAiStream,
  type AiStreamEvent,
} from "@/api/provider";
import { toast } from "@/components/ui/Toast";
import type { Content } from "@/lib/types";

type ReaderTab = "read" | "summary" | "notes";
type ExportFormat = "markdown" | "html" | "plaintext";
type TranslationMode = "original" | "bilingual";

interface SegPair { source: string; translated: string; status: "pending" | "streaming" | "success" | "failed"; }
interface TranslationState { mode: TranslationMode; entryId: number | null; }

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
  const [translation, setTranslation] = useState<TranslationState>({ mode: "original", entryId: null });
  const [translating, setTranslating] = useState(false);
  const [segments, setSegments] = useState<SegPair[]>([]);
  const unlistenRef = useRef<(() => void) | null>(null);
  const translationEntryRef = useRef<number | null>(null);

  const translationMode = translation.mode;
  // Bilingual view only active when mode is bilingual AND entry matches
  const showBilingual = translation.mode === "bilingual" && translation.entryId === selectedEntry?.id;

  const handleExport = async (format: ExportFormat) => {
    setShowExportMenu(false);
    setExporting(true);
    try {
      const content = isTauri()
        ? await exportSingleDigest(selectedEntry!.id, format)
        : mockDigestExport(selectedEntry!, format);

      const extensions: Record<ExportFormat, string> = {
        markdown: ".md",
        html: ".html",
        plaintext: ".txt",
      };

      if (isTauri()) {
        const { save } = await import("@tauri-apps/plugin-dialog");
        const { homeDir } = await import("@tauri-apps/api/path");
        const home = await homeDir();
        const filePath = await save({
          defaultPath: `${home}${selectedEntry!.title}${extensions[format]}`,
          filters: [{ name: EXPORT_LABELS[format], extensions: [extensions[format].slice(1)] }],
        });
        if (!filePath) { setExporting(false); return; }
        // Write via backend
        await exportSingleDigest(selectedEntry!.id, format);
        toast(t(`已导出到 ${filePath}`), "success");
      } else {
        const mimeTypes: Record<ExportFormat, string> = {
          markdown: "text/markdown",
          html: "text/html",
          plaintext: "text/plain",
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
      }
    } catch (e: any) {
      toast(t("导出失败: ") + String(e), "error");
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

      // Helper: check if Content has usable rendered output with current pipeline version
      const PIPELINE_VERSION = 3;
      const hasContent = (c: Content) =>
        (!!c.renderedHtml || !!c.cleanedHtml || !!c.rawHtml)
        && c.readabilityVersion >= PIPELINE_VERSION;

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
    if (!selectedEntry) return;
    const id = selectedEntry.id;
    const timer = setTimeout(() => markEntryRead(id), 1000);
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedEntry?.id]);

  // Translation: load existing + listen for stream events
  useEffect(() => {
    if (!selectedEntry) return;
    const entryId = selectedEntry.id;
    translationEntryRef.current = entryId;
    setTranslation({ mode: "original", entryId: null });
    setSegments([]);
    setTranslating(false);

    unlistenRef.current?.();
    unlistenRef.current = null;

    // Load cached translation from DB (only if still on same entry)
    getTranslationText(entryId).then((text) => {
      if (text && translationEntryRef.current === entryId) {
        setSegments(parseTranslation(text));
      }
    }).catch(() => {});

    listenAiStream((event: AiStreamEvent) => {
      if (event.agentType !== "translation") return;
      // Ignore events for other entries (is_done has entryId=0, so also check ref)
      if (translationEntryRef.current !== entryId) return;
      if (event.entryId && event.entryId !== entryId) return;

      if (event.isDone) {
        // Final: load from DB for correctness (only if still on same entry)
        getTranslationText(entryId).then((text) => {
          if (translationEntryRef.current === entryId) {
            if (text) setSegments(parseTranslation(text));
            setTranslating(false);
          }
        }).catch(() => {
          if (translationEntryRef.current === entryId) setTranslating(false);
        });
        return;
      }

      // Stream delta: [{segIdx}/{total}] chunk
      const match = event.content.match(/^\[(\d+)\/(\d+)\]\s*(.*)/s);
      if (match) {
        const segIdx = parseInt(match[1], 10) - 1;
        const delta = match[3];
        setSegments((prev) => {
          const next = [...prev];
          if (segIdx >= 0 && segIdx < next.length) {
            next[segIdx] = {
              ...next[segIdx],
              translated: next[segIdx].translated + delta,
              status: "streaming",
            };
          }
          return next;
        });
      }
    }).then((unlisten) => { unlistenRef.current = unlisten; });

    return () => {
      unlistenRef.current?.();
      translationEntryRef.current = null;
    };
  }, [selectedEntry?.id]);

  const handleTranslate = async () => {
    // Guard: content must be loaded for current entry
    if (contentLoading || !content?.cleanedHtml && !content?.rawHtml) return;

    // Pre-split source segments from content HTML
    const html = content.cleanedHtml || content.rawHtml;
    const sources = splitContentIntoSegments(html);
    if (sources.length === 0) return;

    const entryId = selectedEntry.id;
    translationEntryRef.current = entryId;

    // Initialize segments with source text
    const initial: SegPair[] = sources.map((src) => ({
      source: src,
      translated: "",
      status: "pending" as const,
    }));
    setSegments(initial);
    setTranslating(true);
    setTranslation({ mode: "bilingual", entryId });

    try {
      let lang = "zh-CN"; let conc = 3;
      try { const cfg = JSON.parse(localStorage.getItem("agentConfig") || "{}"); lang = cfg.translationLanguage || lang; conc = cfg.concurrencyDegree || conc; } catch {}
      await translateEntry(entryId, lang, conc);
    } catch {
      if (translationEntryRef.current === entryId) {
        setTranslating(false);
        setTranslation({ mode: "original", entryId: null });
      }
    }
  };

  const handleClearTranslation = async () => {
    await clearTranslationApi(selectedEntry.id).catch(() => {});
    setSegments([]);
    setTranslation({ mode: "original", entryId: null });
    translationEntryRef.current = null;
  };

  const tabs: { key: ReaderTab; label: string; shortcut: string }[] = [
    { key: "read", label: t("阅读"), shortcut: "" },
    { key: "summary", label: t("摘要"), shortcut: "S" },
    { key: "notes", label: t("笔记"), shortcut: "N" },
  ];

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="px-6 py-4 border-b border-[var(--border-color)]">
        <div className="flex items-start gap-3 mb-3">
          <div className="flex-1 min-w-0">
            <h1 className="text-lg font-semibold text-[var(--text-primary)] leading-snug">
              {selectedEntry.title}
            </h1>
            <div className="flex items-center gap-3 mt-1 text-xs text-[var(--text-tertiary)]">
              {selectedEntry.author && <span>{selectedEntry.author}</span>}
              <span>{formatDate(selectedEntry.publishedAt)}</span>
              {selectedEntry.link && (
                <button
                  onClick={() => {
                    if (isTauri()) {
                      import("@tauri-apps/api/core").then(({ invoke }) => {
                        invoke("open_url", { url: selectedEntry!.link }).catch(() => {
                          window.open(selectedEntry!.link, "_blank");
                        });
                      });
                    } else {
                      window.open(selectedEntry!.link, "_blank");
                    }
                  }}
                  className="text-[var(--link-color)] hover:underline cursor-pointer"
                >
                  {t("原文")}
                </button>
              )}
            </div>
          </div>
          {/* Translation buttons + Export */}
          <div className="flex items-center gap-1">
            {!showBilingual ? (
              <button
                onClick={handleTranslate} disabled={translating}
                className="px-2 py-1 text-xs rounded bg-[var(--bg-tertiary)] hover:bg-[var(--bg-secondary)] disabled:opacity-50"
              >{translating ? "…" : t("翻译")}</button>
            ) : (
              <>
                <button onClick={() => setTranslation({ mode: "original", entryId: null })} className="px-2 py-1 text-xs rounded bg-[var(--bg-tertiary)] hover:bg-[var(--bg-secondary)]">{t("回到原文")}</button>
                {segments.length > 0 && (
                  <button onClick={handleClearTranslation} className="px-2 py-1 text-xs rounded bg-[var(--bg-tertiary)] text-red-500 hover:bg-[var(--bg-secondary)]">{t("清除翻译")}</button>
                )}
              </>
            )}
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
              showBilingual ? (
                <div className="translation-bilingual space-y-6">
                  {segments.map((seg, i) => (
                    <div key={i} className="grid grid-cols-2 gap-4 items-start">
                      <div className="reader-content" dangerouslySetInnerHTML={{ __html: seg.source }} />
                      <div className={`reader-content ${
                        seg.status === "pending" ? "text-[var(--text-tertiary)]" :
                        seg.status === "streaming" ? "text-[var(--text-secondary)]" :
                        "text-[var(--text-primary)]"
                      }`}>
                        {seg.status === "pending" ? (
                          <span className="italic">{t("排队中...")}</span>
                        ) : seg.status === "streaming" ? (
                          <span>{seg.translated || t("翻译中...")}</span>
                        ) : seg.status === "failed" ? (
                          <span className="text-red-500">{t("翻译失败")}</span>
                        ) : (
                          <span>{seg.translated}</span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div
                  className="reader-content"
                  dangerouslySetInnerHTML={{ __html: (content.renderedHtml || content.cleanedHtml || content.rawHtml)! }}
                />
              )
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

/** Split HTML content into text segments matching backend split_html_into_segments. */
function splitContentIntoSegments(html: string | null | undefined): string[] {
  if (!html) return [];
  try {
    const doc = new DOMParser().parseFromString(html, "text/html");
    const elements = doc.querySelectorAll("p, ul, ol, h1, h2, h3, h4, h5, h6, blockquote, pre");
    const segments: string[] = [];
    elements.forEach((el) => {
      const text = el.textContent?.trim();
      if (text) segments.push(text);
    });
    if (segments.length === 0) {
      const all = doc.body.textContent?.trim();
      if (all) segments.push(all);
    }
    return segments;
  } catch {
    const text = html.replace(/<[^>]+>/g, "").trim();
    return text ? [text] : [];
  }
}

/** Parse translation output text (backend format) into segment array. */
function parseTranslation(text: string): SegPair[] {
  if (!text || !text.trim()) return [];
  const segments: SegPair[] = [];
  const parts = text.split(/\n(?=\[\d+\]\s*\n)/);
  for (const part of parts) {
    const srcM = part.match(/^原文[：:]\s*(.+?)(?:\n译文[：:]|\n\(翻译失败\)|\n\n|\n\[|\n?$)/ms);
    const trM = part.match(/^译文[：:]\s*([\s\S]+?)(?:\n\n|\n\[|\n?$)/m);
    const failM = /\(翻译失败\)/.test(part);
    if (srcM) {
      const source = srcM[1].trim();
      if (source) {
        segments.push({
          source,
          translated: failM ? "" : (trM ? trM[1].trim() : ""),
          status: failM ? "failed" : (trM ? "success" : "pending"),
        });
      }
    }
  }
  // Fallback: line-by-line
  if (segments.length === 0) {
    const lines = text.split("\n");
    let cur: { src?: string; trans?: string } = {};
    for (const line of lines) {
      if (/^\[\d+\]/.test(line.trim())) {
        if (cur.src) segments.push({ source: cur.src, translated: cur.trans || "", status: cur.trans ? "success" : "pending" });
        cur = {};
      } else if (line.startsWith("原文:") || line.startsWith("原文：")) {
        cur.src = line.replace(/^原文[：:]\s*/, "").trim();
      } else if (line.startsWith("译文:") || line.startsWith("译文：")) {
        cur.trans = line.replace(/^译文[：:]\s*/, "").trim();
      }
    }
    if (cur.src) segments.push({ source: cur.src, translated: cur.trans || "", status: cur.trans ? "success" : "pending" });
  }
  return segments;
}