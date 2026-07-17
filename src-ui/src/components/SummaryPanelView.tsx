import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui";
import { t } from "@/lib/utils";
import {
  generateSummary,
  getSummaryText,
  cancelSummary,
  listenAiStream,
  type AiStreamEvent,
} from "@/api/provider";

interface SummaryPanelProps {
  entryId: number;
}

export function SummaryPanelView({ entryId }: SummaryPanelProps) {
  const [isGenerating, setIsGenerating] = useState(false);
  const [summary, setSummary] = useState<string | null>(null);
  const [streamText, setStreamText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const didInitRef = useRef(false);

  // 初始化：加载已有的摘要
  useEffect(() => {
    if (didInitRef.current) return;
    didInitRef.current = true;

    getSummaryText(entryId)
      .then((text) => {
        if (text) setSummary(text);
      })
      .catch(() => {
        // 没有摘要也正常
      });

    // 监听 AI 流式事件
    listenAiStream((event: AiStreamEvent) => {
      if (event.agentType !== "summary") return;

      if (event.isDone) {
        setIsGenerating(false);
        if (event.error) {
          setError(event.error);
        } else {
          // 完成后重新获取完整文本
          getSummaryText(entryId).then((text) => {
            if (text) setSummary(text);
          });
        }
        return;
      }

      // 累积流式内容
      setStreamText((prev) => prev + event.content);
    }).then((unlisten) => {
      unlistenRef.current = unlisten;
    });

    return () => {
      unlistenRef.current?.();
    };
  }, [entryId]);

  const handleGenerate = async () => {
    setIsGenerating(true);
    setStreamText("");
    setError(null);

    try {
      await generateSummary(entryId);
    } catch (err: any) {
      setIsGenerating(false);
      setError(String(err));
    }
  };

  const handleCancel = async () => {
    try {
      await cancelSummary(entryId);
    } catch {
      // 忽略取消错误
    }
    setIsGenerating(false);
  };

  return (
    <div className="max-w-3xl mx-auto px-6 py-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-base font-semibold">{t("AI 摘要")}</h2>
        <div className="flex items-center gap-2">
          {isGenerating ? (
            <Button variant="ghost" size="sm" onClick={handleCancel}>
              {t("取消")}
            </Button>
          ) : (
            <Button
              variant="secondary"
              size="sm"
              onClick={handleGenerate}
            >
              {summary ? t("重新生成") : t("生成摘要")}
            </Button>
          )}
        </div>
      </div>

      <div className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-secondary)] p-5 min-h-[120px]">
        {isGenerating ? (
          <div>
            <div className="flex items-center gap-2 mb-2">
              <svg className="animate-spin w-4 h-4 text-[var(--accent-color)]" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" className="opacity-25" />
                <path d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" fill="currentColor" className="opacity-75" />
              </svg>
              <span className="text-xs text-[var(--text-tertiary)]">{t("正在生成...")}</span>
            </div>
            {streamText && (
              <p className="text-sm text-[var(--text-primary)] leading-relaxed">{streamText}</p>
            )}
          </div>
        ) : error ? (
          <div>
            <p className="text-sm text-red-500 mb-2">{t("生成失败")}</p>
            <p className="text-xs text-[var(--text-tertiary)]">{error}</p>
            <Button variant="ghost" size="sm" onClick={() => setError(null)} className="mt-2">
              {t("关闭")}
            </Button>
          </div>
        ) : summary ? (
          <p className="text-sm text-[var(--text-primary)] leading-relaxed whitespace-pre-wrap">{summary}</p>
        ) : (
          <p className="text-sm text-[var(--text-tertiary)] text-center py-8">
            {t("点击上方按钮生成 AI 摘要")}
          </p>
        )}
      </div>
    </div>
  );
}