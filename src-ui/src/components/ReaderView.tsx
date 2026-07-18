import { useState, useEffect, useRef } from "react";
import { useApp } from "@/contexts/AppContext";
import { mockContent } from "@/api/mock";
import { formatDate, t } from "@/lib/utils";
import { SummaryPanelView } from "./SummaryPanelView";
import { NoteEditorView } from "./NoteEditorView";
import { isTauri, getEntryContent as getEntryContentReal, processEntryContent, exportSingleDigest, writeTextFile, getNote } from "@/api/feed";
import {
  translateEntry,
  getTranslationText,
  cancelTranslation,
  clearTranslation as clearTranslationApi,
  listenAiStream,
  getSummaryText,
  type AiStreamEvent,
} from "@/api/provider";
import { toast } from "@/components/ui/Toast";
import type { Content } from "@/lib/types";

type SidePanel = "summary" | "notes" | null;
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
  const { selectedEntry, entries, selectEntry, setViewMode, markEntryRead } = useApp();

  // ============================================================
  // All hooks — must be called unconditionally before any return
  // ============================================================

  const [sidePanel, setSidePanel] = useState<SidePanel>(null);
  const [exporting, setExporting] = useState(false);
  const [translation, setTranslation] = useState<TranslationState>({ mode: "original", entryId: null });
  const [translating, setTranslating] = useState(false);
  const [segments, setSegments] = useState<SegPair[]>([]);
  const unlistenRef = useRef<(() => void) | null>(null);
  const translationEntryRef = useRef<number | null>(null);

  const [content, setContent] = useState<Content | null>(null);
  const [contentLoading, setContentLoading] = useState(false);
  const [contentError, setContentError] = useState<string | null>(null);

  const showBilingual = translation.mode === "bilingual" && translation.entryId === selectedEntry?.id;

  // Load real content from backend when entry changes.
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

      const PIPELINE_VERSION = 3;
      const hasContent = (c: Content) =>
        (!!c.renderedHtml || !!c.cleanedHtml || !!c.rawHtml)
        && c.readabilityVersion >= PIPELINE_VERSION;

      try {
        const c = await getEntryContentReal(selectedEntry!.id);
        if (hasContent(c)) {
          if (!cancelled) setContent(c);
          return;
        }
        if (!cancelled) setContentLoading(true);
        throw new Error("Empty cache");
      } catch {
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

    let cancelled = false;

    getTranslationText(entryId).then((text) => {
      if (!cancelled && text) {
        setSegments(parseTranslation(text));
      }
    }).catch(() => {});

    listenAiStream((event: AiStreamEvent) => {
      if (cancelled) return;
      if (event.agentType !== "translation") return;
      if (translationEntryRef.current !== entryId) return;
      if (event.entryId && event.entryId !== entryId) return;

      if (event.isDone) {
        getTranslationText(entryId).then((text) => {
          if (!cancelled && translationEntryRef.current === entryId) {
            if (text) setSegments(parseTranslation(text));
            setTranslating(false);
          }
        }).catch(() => {
          if (!cancelled && translationEntryRef.current === entryId) setTranslating(false);
        });
        return;
      }

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
    }).then((unlisten) => {
      if (cancelled) {
        unlisten();
      } else {
        unlistenRef.current = unlisten;
      }
    });

    return () => {
      cancelled = true;
      unlistenRef.current?.();
      unlistenRef.current = null;
      translationEntryRef.current = null;
    };
  }, [selectedEntry?.id]);

  const handleTranslate = async () => {
    if (contentLoading || !content?.cleanedHtml && !content?.rawHtml) return;

    const html = content.cleanedHtml || content.rawHtml;
    const sources = splitContentIntoSegments(html);
    if (sources.length === 0) return;

    const entryId = selectedEntry!.id;
    translationEntryRef.current = entryId;

    const initial: SegPair[] = sources.map((src) => ({
      source: src,
      translated: "",
      status: "pending" as const,
    }));
    setSegments(initial);
    setTranslating(true);
    setTranslation({ mode: "bilingual", entryId });

    try {
      let lang = "中文"; let conc = 3;
      try { const cfg = JSON.parse(localStorage.getItem("agentConfig") || "{}"); lang = cfg.translationLanguage || lang; conc = cfg.concurrencyDegree || conc; } catch {}

      if ((lang === "中文" || lang === "zh-CN") && sources.every(isPrimarilyChinese)) {
        setSegments(sources.map((src) => ({
          source: src,
          translated: src,
          status: "success" as const,
        })));
        setTranslating(false);
        return;
      }

      await translateEntry(entryId, lang, conc);
    } catch {
      if (translationEntryRef.current === entryId) {
        setTranslating(false);
        setTranslation({ mode: "original", entryId: null });
      }
    }
  };

  const handleClearTranslation = async () => {
    await clearTranslationApi(selectedEntry!.id).catch(() => {});
    setSegments([]);
    setTranslation({ mode: "original", entryId: null });
    translationEntryRef.current = null;
  };

  const isPanelOpen = sidePanel !== null;

  // Keyboard shortcuts: s=summary, n=notes, t=translate, j/k=navigate, Escape=close panel
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT") return;

      // j/k: navigate to previous/next article
      if (e.key === "j" || e.key === "k") {
        e.preventDefault();
        const currentIdx = entries.findIndex((en) => en.id === selectedEntry?.id);
        if (currentIdx === -1) return;
        const nextIdx = e.key === "j" ? currentIdx + 1 : currentIdx - 1;
        if (nextIdx < 0 || nextIdx >= entries.length) return;
        selectEntry(entries[nextIdx]);
        return;
      }

      switch (e.key.toLowerCase()) {
        case "s": e.preventDefault(); togglePanel("summary"); break;
        case "n": e.preventDefault(); togglePanel("notes"); break;
        case "t": e.preventDefault(); handleTranslate(); break;
        case "escape": e.preventDefault(); setSidePanel(null); break;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [sidePanel, contentLoading, content, translating, showBilingual, segments.length, entries, selectedEntry, selectEntry]);

  // ============================================================
  // Derived values and callbacks
  // ============================================================

  const togglePanel = (panel: SidePanel) => {
    setSidePanel((prev) => (prev === panel ? null : panel));
  };

  const handleExportArticle = async (format: ExportFormat) => {
    setExporting(true);
    try {
      const digest = await exportSingleDigest(selectedEntry!.id, format);
      const ext = { markdown: ".md", html: ".html", plaintext: ".txt" }[format];
      await saveFile(`${selectedEntry!.title}${ext}`, digest, format);
    } catch (e: any) {
      toast(t("导出失败: ") + String(e), "error");
    } finally {
      setExporting(false);
    }
  };

  const handleExportSummary = async () => {
    setExporting(true);
    try {
      const text = await getSummaryText(selectedEntry!.id);
      if (!text) { toast(t("暂无摘要"), "info"); setExporting(false); return; }
      await saveFile(`${selectedEntry!.title}-摘要.md`, text, "markdown");
    } catch (e: any) {
      toast(t("导出失败: ") + String(e), "error");
    } finally {
      setExporting(false);
    }
  };

  const handleExportNote = async () => {
    setExporting(true);
    try {
      const note = await getNote(selectedEntry!.id);
      if (!note || !note.content) { toast(t("暂无笔记"), "info"); setExporting(false); return; }
      await saveFile(`${selectedEntry!.title}-笔记.md`, note.content, "markdown");
    } catch (e: any) {
      toast(t("导出失败: ") + String(e), "error");
    } finally {
      setExporting(false);
    }
  };

  const saveFile = async (defaultName: string, content: string, format: ExportFormat) => {
    if (isTauri()) {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const { homeDir } = await import("@tauri-apps/api/path");
      const home = await homeDir();
      const ext = defaultName.split(".").pop() || "md";
      const filePath = await save({
        defaultPath: `${home}${defaultName}`,
        filters: [{ name: EXPORT_LABELS[format], extensions: [ext] }],
      });
      if (!filePath) return;
      await writeTextFile(filePath, content);
      toast(t("已导出"), "success");
    } else {
      // Browser fallback
      const mimeMap: Record<string, string> = { markdown: "text/markdown", html: "text/html", plaintext: "text/plain" };
      const blob = new Blob([content], { type: mimeMap[format] || "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url; a.download = defaultName;
      document.body.appendChild(a); a.click();
      document.body.removeChild(a); URL.revokeObjectURL(url);
      toast(t("已导出"), "success");
    }
  };

  // ============================================================
  // Render
  // ============================================================

  if (!selectedEntry) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-[var(--text-tertiary)] text-sm">{t("选择一篇文章开始阅读")}</p>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="px-6 py-4 border-b border-[var(--border-color)]">
        <div className="flex items-start gap-3">
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

          {/* Toolbar buttons */}
          <div className="flex items-center gap-1 flex-shrink-0">
            {/* Translation */}
            {!showBilingual ? (
              <button
                onClick={handleTranslate} disabled={translating}
                className="px-2 py-1 text-xs rounded bg-[var(--bg-tertiary)] hover:bg-[var(--bg-secondary)] disabled:opacity-50"
                title={t("翻译") + " (T)"}
              >{translating ? "…" : t("翻译")}</button>
            ) : (
              <>
                <button onClick={() => setTranslation({ mode: "original", entryId: null })} className="px-2 py-1 text-xs rounded bg-[var(--bg-tertiary)] hover:bg-[var(--bg-secondary)]">{t("回到原文")}</button>
                {segments.length > 0 && (
                  <button onClick={handleClearTranslation} className="px-2 py-1 text-xs rounded bg-[var(--bg-tertiary)] text-red-500 hover:bg-[var(--bg-secondary)]">{t("清除翻译")}</button>
                )}
              </>
            )}

            {/* Summary toggle */}
            <button
              onClick={() => togglePanel("summary")}
              className={`px-2 py-1 text-xs rounded transition-colors ${sidePanel === "summary" ? "bg-[var(--accent-color)] text-white" : "bg-[var(--bg-tertiary)] hover:bg-[var(--bg-secondary)]"}`}
              title={t("摘要") + " (S)"}
            >{t("摘要")}</button>

            {/* Notes toggle */}
            <button
              onClick={() => togglePanel("notes")}
              className={`px-2 py-1 text-xs rounded transition-colors ${sidePanel === "notes" ? "bg-[var(--accent-color)] text-white" : "bg-[var(--bg-tertiary)] hover:bg-[var(--bg-secondary)]"}`}
              title={t("笔记") + " (N)"}
            >{t("笔记")}</button>

            {/* Export buttons */}
            <span className="text-[10px] text-[var(--text-tertiary)] mx-1 select-none">{t("导出")}</span>
            <button
              onClick={() => handleExportArticle("markdown")}
              disabled={exporting}
              className="p-1.5 rounded-lg hover:bg-[var(--bg-tertiary)] text-[var(--text-secondary)] transition-colors disabled:opacity-50"
              title={t("导出原文 (Markdown)")}
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </button>
            <button
              onClick={handleExportSummary}
              disabled={exporting}
              className="p-1.5 rounded-lg hover:bg-[var(--bg-tertiary)] text-[var(--text-secondary)] transition-colors disabled:opacity-50"
              title={t("导出 AI 摘要 (Markdown)")}
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </button>
            <button
              onClick={handleExportNote}
              disabled={exporting}
              className="p-1.5 rounded-lg hover:bg-[var(--bg-tertiary)] text-[var(--text-secondary)] transition-colors disabled:opacity-50"
              title={t("导出笔记 (Markdown)")}
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path strokeLinecap="round" strokeLinejoin="round" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      {/* Main content + side panel */}
      <div className="flex-1 flex overflow-hidden">
        {/* Content area */}
        <div className="flex-1 overflow-y-auto">
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
        </div>

        {/* Slide-out side panel */}
        <div
          className={`border-l border-[var(--border-color)] bg-[var(--bg-primary)] overflow-y-auto transition-all duration-200 ease-in-out flex-shrink-0 ${
            isPanelOpen ? "w-[400px]" : "w-0 border-l-0"
          }`}
        >
          {isPanelOpen && (
            <div className="w-[400px]">
              <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--border-color)] sticky top-0 bg-[var(--bg-primary)] z-10">
                <h2 className="text-sm font-semibold">
                  {sidePanel === "summary" ? t("AI 摘要") : t("笔记")}
                </h2>
                <button
                  onClick={() => setSidePanel(null)}
                  className="p-1 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-colors"
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M18 6L6 18M6 6l12 12" />
                  </svg>
                </button>
              </div>
              <div className="p-4">
                {sidePanel === "summary" ? (
                  <SummaryPanelView entryId={selectedEntry.id} />
                ) : (
                  <NoteEditorView entryId={selectedEntry.id} />
                )}
              </div>
            </div>
          )}
        </div>
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

function isPrimarilyChinese(text: string): boolean {
  const chars = text.replace(/\s/g, "");
  if (chars.length === 0) return false;
  let cjk = 0;
  for (const ch of chars) {
    const code = ch.charCodeAt(0);
    if ((code >= 0x4E00 && code <= 0x9FFF) ||
        (code >= 0x3400 && code <= 0x4DBF) ||
        (code >= 0x20000 && code <= 0x2A6DF) ||
        (code >= 0xF900 && code <= 0xFAFF) ||
        (code >= 0x3000 && code <= 0x303F) ||
        (code >= 0xFF00 && code <= 0xFFEF)) {
      cjk++;
    }
  }
  return cjk / chars.length > 0.3;
}

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