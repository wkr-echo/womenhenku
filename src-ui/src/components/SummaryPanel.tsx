import { useState, useEffect, useRef, useCallback } from "react";
import { Button, Dropdown } from "@/components/ui";
import { t } from "@/lib/utils";
import {
  generateSummary,
  getSummaryText,
  cancelSummary,
  clearSummary as clearSummaryApi,
  listenAiStream,
  type AiStreamEvent,
} from "@/api/provider";

interface SummaryPanelProps {
  entryId: number;
}

export function SummaryPanel({ entryId }: SummaryPanelProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [summaryText, setSummaryText] = useState("");
  const [hasExistingSummary, setHasExistingSummary] = useState(false);
  const [targetLanguage, setTargetLanguage] = useState("zh");
  const [detailLevel, setDetailLevel] = useState<"brief" | "standard" | "detailed">("standard");
  const [autoEnabled, setAutoEnabled] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [modelName, setModelName] = useState("");
  const [tokenCount, setTokenCount] = useState(0);

  const textRef = useRef<HTMLDivElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  // Load config from localStorage
  useEffect(() => {
    try {
      const saved = localStorage.getItem("agentConfig");
      if (saved) {
        const cfg = JSON.parse(saved);
        setTargetLanguage(cfg.summaryLanguage || "zh");
        setDetailLevel(cfg.summaryDetail || "standard");
      }
    } catch {}
  }, []);

  // Load existing summary and setup stream listener
  useEffect(() => {
    setIsExpanded(false);
    setIsRunning(false);
    setSummaryText("");
    setErrorMessage(null);
    setHasExistingSummary(false);

    unlistenRef.current?.();
    unlistenRef.current = null;

    getSummaryText(entryId)
      .then((text) => {
        if (text) {
          setSummaryText(text);
          setHasExistingSummary(true);
        }
      })
      .catch(() => {});

    listenAiStream((event: AiStreamEvent) => {
      if (event.agentType !== "summary") return;
      if (event.entryId && event.entryId !== entryId) return;

      if (event.isDone) {
        setIsRunning(false);
        if (event.error) {
          setErrorMessage(event.error);
        } else {
          getSummaryText(entryId).then((text) => {
            if (text) {
              setSummaryText(text);
              setHasExistingSummary(true);
            }
          });
        }
        return;
      }

      setSummaryText((prev) => prev + event.content);
      setIsRunning(true);
    }).then((unlisten) => {
      unlistenRef.current = unlisten;
    });

    return () => {
      unlistenRef.current?.();
    };
  }, [entryId]);

  // Auto-scroll
  useEffect(() => {
    if (textRef.current) {
      textRef.current.scrollTop = textRef.current.scrollHeight;
    }
  }, [summaryText]);

  // Auto-summary with debounce
  useEffect(() => {
    if (!autoEnabled || !isExpanded) return;
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      handleGenerate();
    }, 1000);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [entryId, autoEnabled, isExpanded]);

  const handleGenerate = useCallback(async () => {
    if (isRunning) return;
    setIsRunning(true);
    setSummaryText("");
    setErrorMessage(null);
    try {
      let lang = "zh-CN";
      let detail = "standard";
      try {
        const saved = localStorage.getItem("agentConfig");
        if (saved) {
          const cfg = JSON.parse(saved);
          lang = cfg.summaryLanguage || lang;
          detail = cfg.summaryDetail || detail;
        }
      } catch {}
      await generateSummary(entryId, targetLanguage || lang, detailLevel || detail, hasExistingSummary);
    } catch (e: any) {
      setIsRunning(false);
      setErrorMessage(String(e));
    }
  }, [entryId, isRunning, targetLanguage, detailLevel, hasExistingSummary]);

  const handleCancel = async () => {
    try { await cancelSummary(entryId); } catch {}
    setIsRunning(false);
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(summaryText).catch(() => {});
  };

  const handleClear = async () => {
    try { await clearSummaryApi(entryId); } catch {}
    setSummaryText("");
    setHasExistingSummary(false);
    setErrorMessage(null);
  };

  const languages = [
    { label: "中文", value: "zh" },
    { label: "English", value: "en" },
    { label: "日本語", value: "ja" },
    { label: "한국어", value: "ko" },
  ];

  const detailLevels = [
    { label: t("简短"), value: "brief" },
    { label: t("标准"), value: "standard" },
    { label: t("详细"), value: "detailed" },
  ];

  return (
    <div className="border-t border-[var(--border-color)] bg-[var(--bg-primary)]">
      {/* Title bar — always visible */}
      <div
        className="flex items-center gap-2 px-6 py-3 cursor-pointer hover:bg-[var(--bg-secondary)] transition-colors select-none"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <svg
          className={`w-4 h-4 text-[var(--text-tertiary)] transition-transform ${isExpanded ? "rotate-90" : ""}`}
          fill="none" stroke="currentColor" viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
        </svg>
        <span className="text-sm font-medium">{t("Summary")}</span>
        {hasExistingSummary && (
          <span className="w-2 h-2 rounded-full bg-[var(--accent-color)]" />
        )}
        {isRunning && (
          <svg className="animate-spin w-3.5 h-3.5 text-[var(--accent-color)]" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" className="opacity-25" />
            <path d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" fill="currentColor" className="opacity-75" />
          </svg>
        )}
      </div>

      {/* Expanded panel */}
      {isExpanded && (
        <div className="px-6 pb-4 space-y-3">
          {/* Config row */}
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1 text-xs text-[var(--text-tertiary)]">
              <span>{t("语言")}:</span>
              <Dropdown
                items={languages}
                value={targetLanguage}
                onChange={(v) => setTargetLanguage(v)}
              />
            </div>
            <div className="flex items-center gap-1 text-xs text-[var(--text-tertiary)]">
              <span>{t("详细程度")}:</span>
              <div className="flex rounded-md border border-[var(--border-color)] overflow-hidden">
                {detailLevels.map((d) => (
                  <button
                    key={d.value}
                    onClick={() => setDetailLevel(d.value as typeof detailLevel)}
                    className={`px-2 py-1 text-xs transition-colors ${
                      detailLevel === d.value
                        ? "bg-[var(--accent-color)] text-white"
                        : "hover:bg-[var(--bg-tertiary)] text-[var(--text-secondary)]"
                    }`}
                  >
                    {d.label}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* Toolbar */}
          <div className="flex items-center gap-2 flex-wrap">
            <label className="flex items-center gap-1 text-xs text-[var(--text-tertiary)] cursor-pointer">
              <input
                type="checkbox"
                checked={autoEnabled}
                onChange={(e) => setAutoEnabled(e.target.checked)}
                className="accent-[var(--accent-color)]"
              />
              {t("自动摘要")}
            </label>
            <Button size="sm" onClick={handleGenerate} disabled={isRunning}>
              {hasExistingSummary ? t("重新生成") : t("生成摘要")}
            </Button>
            {isRunning && (
              <Button size="sm" variant="ghost" onClick={handleCancel}>
                {t("中止")}
              </Button>
            )}
            {summaryText && !isRunning && (
              <>
                <Button size="sm" variant="ghost" onClick={handleCopy}>
                  {t("复制")}
                </Button>
                <Button size="sm" variant="ghost" onClick={handleClear}>
                  {t("清除")}
                </Button>
              </>
            )}
          </div>

          {/* Text area */}
          <div
            ref={textRef}
            className="max-h-60 overflow-y-auto rounded-lg border border-[var(--border-color)] bg-[var(--bg-secondary)] p-4 text-sm leading-relaxed"
          >
            {errorMessage ? (
              <p className="text-red-500">{t("摘要生成失败，请重试")}</p>
            ) : isRunning && !summaryText ? (
              <p className="text-[var(--text-tertiary)]">{t("正在生成摘要...")}</p>
            ) : summaryText ? (
              <div
                className="prose prose-sm dark:prose-invert max-w-none"
                dangerouslySetInnerHTML={{ __html: summaryText }}
              />
            ) : (
              <p className="text-[var(--text-tertiary)]">{t("点击「生成摘要」开始")}</p>
            )}
          </div>

          {/* Footer */}
          {!isRunning && summaryText && modelName && (
            <p className="text-xs text-[var(--text-tertiary)]">
              {t("模型")}: {modelName} · {tokenCount} tokens
            </p>
          )}
        </div>
      )}
    </div>
  );
}
