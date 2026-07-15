import { useState } from "react";
import { useApp } from "@/contexts/AppContext";
import { mockContent, mockSummary } from "@/api/mock";
import { formatDate } from "@/lib/utils";
import { Button } from "@/components/ui";
import { SummaryPanel } from "./SummaryPanel";
import { TranslationPanel } from "./TranslationPanel";
import { NoteEditor } from "./NoteEditor";

type ReaderTab = "read" | "summary" | "translate" | "notes";

export function ReaderView() {
  const { selectedEntry, setViewMode } = useApp();
  const [activeTab, setActiveTab] = useState<ReaderTab>("read");

  if (!selectedEntry) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-[var(--text-tertiary)] text-sm">选择一篇文章开始阅读</p>
      </div>
    );
  }

  const content = mockContent;

  const tabs: { key: ReaderTab; label: string; shortcut: string }[] = [
    { key: "read", label: "阅读", shortcut: "" },
    { key: "summary", label: "摘要", shortcut: "S" },
    { key: "translate", label: "翻译", shortcut: "T" },
    { key: "notes", label: "笔记", shortcut: "N" },
  ];

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="px-6 py-4 border-b border-[var(--border-color)]">
        <div className="flex items-center gap-3 mb-3">
          <button
            onClick={() => setViewMode("list")}
            className="p-1.5 rounded-lg hover:bg-[var(--bg-tertiary)] text-[var(--text-secondary)] transition-colors"
            title="返回列表"
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
              <span>{formatDate(selectedEntry.published_at)}</span>
              {selectedEntry.link && (
                <a
                  href={selectedEntry.link}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-[var(--link-color)] hover:underline"
                >
                  原文
                </a>
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
            <div
              className="reader-content"
              dangerouslySetInnerHTML={{ __html: content.rendered_html || content.cleaned_html || content.raw_html }}
            />
          </div>
        )}
        {activeTab === "summary" && <SummaryPanel entryId={selectedEntry.id} />}
        {activeTab === "translate" && <TranslationPanel entryId={selectedEntry.id} />}
        {activeTab === "notes" && <NoteEditor entryId={selectedEntry.id} />}
      </div>
    </div>
  );
}