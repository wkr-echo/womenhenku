import { useState } from "react";
import { Button } from "@/components/ui";
import { mockSummary } from "@/api/mock";

interface SummaryPanelProps {
  entryId: number;
}

export function SummaryPanel({ entryId: _entryId }: SummaryPanelProps) {
  const [isGenerating, setIsGenerating] = useState(false);
  const [summary, setSummary] = useState(mockSummary);
  const [streamText, setStreamText] = useState("");

  const handleGenerate = () => {
    setIsGenerating(true);
    setStreamText("");
    // mock streaming
    const text = mockSummary.content;
    let i = 0;
    const interval = setInterval(() => {
      if (i < text.length) {
        setStreamText(text.slice(0, i + 5));
        i += 5;
      } else {
        clearInterval(interval);
        setIsGenerating(false);
        setSummary({ ...mockSummary, content: text });
      }
    }, 30);
  };

  return (
    <div className="max-w-3xl mx-auto px-6 py-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-base font-semibold">AI 摘要</h2>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            size="sm"
            onClick={handleGenerate}
            disabled={isGenerating}
          >
            {isGenerating ? (
              <>
                <svg className="animate-spin w-3.5 h-3.5" viewBox="0 0 24 24" fill="none">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" className="opacity-25" />
                  <path d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" fill="currentColor" className="opacity-75" />
                </svg>
                生成中...
              </>
            ) : summary ? (
              "重新生成"
            ) : (
              "生成摘要"
            )}
          </Button>
        </div>
      </div>

      <div className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-secondary)] p-5 min-h-[120px]">
        {isGenerating ? (
          <p className="text-sm text-[var(--text-secondary)] leading-relaxed">
            {streamText || "正在生成摘要..."}
          </p>
        ) : summary ? (
          <p className="text-sm text-[var(--text-primary)] leading-relaxed">{summary.content}</p>
        ) : (
          <p className="text-sm text-[var(--text-tertiary)] text-center py-8">
            点击上方按钮生成 AI 摘要
          </p>
        )}
      </div>

      {summary && !isGenerating && (
        <div className="mt-3 flex items-center gap-4 text-xs text-[var(--text-tertiary)]">
          <span>语言：{summary.target_language}</span>
          <span>详细程度：{summary.detail_level}</span>
        </div>
      )}
    </div>
  );
}