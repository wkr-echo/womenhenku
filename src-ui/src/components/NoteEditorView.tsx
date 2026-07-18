import { useState, useEffect } from "react";
import { Button } from "@/components/ui";
import { isTauri, saveNote as saveNoteApi, getNote as getNoteApi } from "@/api/feed";
import { toast } from "@/components/ui/Toast";
import { t } from "@/lib/utils";

interface NoteEditorViewProps {
  entryId: number;
}

const NOTE_PLACEHOLDER = `## 我的笔记

在这里写下你的想法…

> 提示：支持 Markdown 语法，使用 Ctrl+S 保存`;

export function NoteEditorView({ entryId }: NoteEditorViewProps) {
  const [content, setContent] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [lastSaved, setLastSaved] = useState<string | null>(null);
  const [hasChanges, setHasChanges] = useState(false);

  // Load note on mount: Tauri real API — empty if no existing note
  useEffect(() => {
    if (isTauri()) {
      getNoteApi(entryId)
        .then((note) => {
          setContent(note?.content ?? "");
        })
        .catch(() => setContent(""))
        .finally(() => setIsLoading(false));
    } else {
      setContent("");
      setIsLoading(false);
    }
  }, [entryId]);

  const handleSave = () => {
    setIsSaving(true);
    if (isTauri()) {
      saveNoteApi(entryId, content)
        .then(() => {
          setHasChanges(false);
          setLastSaved(new Date().toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" }));
          toast(t("笔记已保存"), "success");
        })
        .catch(() => {
          toast(t("保存失败"), "error");
        })
        .finally(() => setIsSaving(false));
    } else {
      // Mock save
      setTimeout(() => {
        setHasChanges(false);
        setLastSaved(new Date().toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" }));
        toast(t("笔记已保存"), "success");
        setIsSaving(false);
      }, 500);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setContent(e.target.value);
    if (!hasChanges) setHasChanges(true);
  };

  // Keyboard shortcut: Ctrl+S / Cmd+S
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "s") {
        e.preventDefault();
        if (hasChanges) handleSave();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [content, hasChanges]);

  if (isLoading) {
    return <div className="max-w-3xl mx-auto px-6 py-12 text-center text-[var(--text-tertiary)]">{t("加载中...")}</div>;
  }

  return (
    <div className="max-w-3xl mx-auto px-6 py-6">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <h2 className="text-base font-semibold">{t("笔记")}</h2>
          {lastSaved && !hasChanges && (
            <span className="text-xs text-[var(--text-tertiary)]">
              {t("已保存于")} {lastSaved}</span>
          )}
          {hasChanges && (
            <span className="text-xs text-[var(--warning-color)]">{t("未保存")}</span>
          )}
        </div>
        <Button
          size="sm"
          onClick={handleSave}
          disabled={!hasChanges || isSaving}
        >
          {isSaving ? t("保存中...") : t("保存")}
        </Button>
      </div>

      <div className="note-editor">
        <textarea
          value={content}
          onChange={handleChange}
          placeholder={NOTE_PLACEHOLDER}
          className="w-full min-h-[400px] rounded-xl border border-[var(--border-color)] bg-[var(--bg-secondary)] p-5 text-sm text-[var(--text-primary)] resize-y focus:outline-none focus:ring-2 focus:ring-[var(--accent-color)] focus:border-transparent transition-colors placeholder:text-[var(--text-tertiary)]"
        />
      </div>

      <div className="mt-4 p-4 rounded-xl border border-[var(--border-color)] bg-[var(--bg-secondary)]">
        <h3 className="text-xs font-semibold text-[var(--text-tertiary)] uppercase mb-2">{t("预览")}</h3>
        <div className="reader-content text-sm">
          {content.split("\n").map((line, i) => {
            if (line.startsWith("## ")) {
              return <h2 key={i} className="!mt-2 !mb-1">{line.slice(3)}</h2>;
            }
            if (line.startsWith("# ")) {
              return <h1 key={i} className="!mt-2 !mb-1">{line.slice(2)}</h1>;
            }
            if (line.startsWith("- ")) {
              return <li key={i} className="ml-4">{line.slice(2)}</li>;
            }
            if (line.trim() === "") {
              return <div key={i} className="h-3" />;
            }
            return <p key={i}>{line}</p>;
          })}
        </div>
      </div>

      <div className="mt-4 text-xs text-[var(--text-tertiary)]">
        <kbd className="px-1.5 py-0.5 rounded bg-[var(--bg-tertiary)] border border-[var(--border-color)]">Ctrl+S</kbd> {t("快速保存")}
      </div>
    </div>
  );
}