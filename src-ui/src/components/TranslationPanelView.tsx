import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui";
import { t } from "@/lib/utils";
import {
  translateEntry,
  getTranslationText,
  cancelTranslation,
  clearTranslation as clearTranslationApi,
  listenAiStream,
  type AiStreamEvent,
} from "@/api/provider";

interface TranslationPanelProps {
  entryId: number;
}

interface SegmentPair {
  index: number;
  source: string;
  target: string;
}

export function TranslationPanelView({ entryId }: TranslationPanelProps) {
  const [translating, setTranslating] = useState(false);
  const [segments, setSegments] = useState<SegmentPair[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [streamSegments, setStreamSegments] = useState<string[]>([]);
  const unlistenRef = useRef<(() => void) | null>(null);

  // Reset and reload when entry changes
  useEffect(() => {
    setSegments([]);
    setError(null);

    unlistenRef.current?.();
    unlistenRef.current = null;

    const loadTranslation = async () => {
      try {
        const text = await getTranslationText(entryId);
        if (text) {
          const parsed = parseTranslationOutput(text);
          setSegments(parsed);
        }
      } catch {}
    };
    loadTranslation();

    listenAiStream((event: AiStreamEvent) => {
      if (event.agentType !== "translation") return;
      if (event.entryId !== entryId) return;

      if (event.isDone) {
        setTranslating(false);
        if (event.error) {
          setError(event.error);
        } else {
          getTranslationText(entryId).then((text) => {
            if (text) {
              const parsed = parseTranslationOutput(text);
              if (parsed.length > 0) setSegments(parsed);
            }
          });
        }
        return;
      }

      setStreamSegments((prev) => [...prev, event.content]);
      setTranslating(true);
    }).then((unlisten) => {
      unlistenRef.current = unlisten;
    });

    return () => {
      unlistenRef.current?.();
    };
  }, [entryId]);

  const handleTranslate = async () => {
    setTranslating(true);
    setError(null);
    setStreamSegments([]);

    try {
      let targetLanguage = "zh-CN";
      let concurrency = 3;
      try {
        const saved = localStorage.getItem("agentConfig");
        if (saved) {
          const cfg = JSON.parse(saved);
          targetLanguage = cfg.translationLanguage || targetLanguage;
          concurrency = cfg.concurrencyDegree || concurrency;
        }
      } catch {}
      await translateEntry(entryId, targetLanguage, concurrency, segments.length > 0);
    } catch (err: any) {
      setTranslating(false);
      setError(String(err));
    }
  };

  const handleCancel = async () => {
    try {
      await cancelTranslation(entryId);
    } catch {
      // 忽略
    }
    setTranslating(false);
  };

  const handleClear = async () => {
    try {
      await clearTranslationApi(entryId);
    } catch {
      // 忽略
    }
    setSegments([]);
    setStreamSegments([]);
    setError(null);
  };

  return (
    <div className="max-w-4xl mx-auto px-6 py-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-base font-semibold">{t("双语翻译")}</h2>
        <div className="flex items-center gap-2">
          {segments.length > 0 && (
            <Button variant="ghost" size="sm" onClick={handleClear}>
              {t("清除翻译")}
            </Button>
          )}
          {translating ? (
            <Button variant="ghost" size="sm" onClick={handleCancel}>
              {t("取消")}
            </Button>
          ) : (
            <Button
              variant="secondary"
              size="sm"
              onClick={handleTranslate}
            >
              {segments.length > 0 ? t("重新翻译") : t("开始翻译")}
            </Button>
          )}
        </div>
      </div>

      {segments.length === 0 && !translating && !error && (
        <div className="text-center py-12 text-[var(--text-tertiary)] text-sm">
          {t("点击上方按钮开始双语翻译，原文和译文将分栏对照显示")}
        </div>
      )}

      {error && (
        <div className="text-center py-8 text-sm text-red-500">
          <p>{t("翻译出错")}</p>
          <p className="text-xs text-[var(--text-tertiary)] mt-1">{error}</p>
          <Button variant="ghost" size="sm" onClick={() => setError(null)} className="mt-2">
            {t("关闭")}
          </Button>
        </div>
      )}

      {translating && (
        <div className="flex items-center gap-2 justify-center py-4 text-sm text-[var(--text-tertiary)]">
          <svg className="animate-spin w-4 h-4 text-[var(--accent-color)]" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" className="opacity-25" />
            <path d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" fill="currentColor" className="opacity-75" />
          </svg>
          <span>{t("正在翻译中...")}</span>
        </div>
      )}

      {streamSegments.length > 0 && translating && (
        <div className="mb-4 p-3 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)]">
          <p className="text-xs text-[var(--text-tertiary)] mb-1">{t("实时翻译进度：")}</p>
          {streamSegments.slice(-5).map((s, i) => (
            <p key={i} className="text-sm text-[var(--text-secondary)]">{s}</p>
          ))}
        </div>
      )}

      <div className="space-y-6">
        {segments.map((seg, i) => (
          <div key={i} className="translation-panel rounded-xl overflow-hidden border border-[var(--border-color)]">
            <div className="source bg-[var(--bg-secondary)] p-4">
              <p className="text-sm text-[var(--text-primary)]">{seg.source}</p>
            </div>
            <div className="target bg-[var(--bg-tertiary)] p-4 border-t border-[var(--border-color)]">
              <div className="flex items-center gap-2 mb-1">
                <span className="text-xs px-1.5 py-0.5 rounded bg-[var(--accent-color)]/10 text-[var(--accent-color)]">
                  {seg.index + 1}
                </span>
              </div>
              <p className="text-sm text-[var(--text-primary)]">{seg.target}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

/**
 * 从翻译结果文本中解析段落对
 * 格式: [1]\n原文: xxx\n译文: xxx\n\n[2]\n...
 */
function parseTranslationOutput(text: string): SegmentPair[] {
  const segments: SegmentPair[] = [];
  const blocks = text.split("\n\n");

  for (const block of blocks) {
    const lines = block.trim().split("\n");
    if (lines.length < 2) continue;

    // 第一行: [index]
    const indexMatch = lines[0].match(/\[(\d+)\]/);
    const index = indexMatch ? parseInt(indexMatch[1]) - 1 : segments.length;

    // 查找原文和译文行
    let source = "";
    let target = "";
    for (const line of lines) {
      if (line.startsWith("原文: ")) {
        source = line.slice("原文: ".length);
      } else if (line.startsWith("译文: ")) {
        target = line.slice("译文: ".length);
      }
    }

    if (source || target) {
      segments.push({ index, source, target: target || "(翻译失败)" });
    }
  }

  return segments;
}
