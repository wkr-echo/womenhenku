import { useState } from "react";
import { Button } from "@/components/ui";

interface TranslationPanelProps {
  entryId: number;
}

const mockSegments = [
  { source: "随着 AI 工具的普及，编程的方式正在发生根本性的变化。", target: "With the proliferation of AI tools, the way we program is undergoing a fundamental transformation." },
  { source: "第一阶段：代码补全。AI 根据上下文自动补全代码片段。", target: "Phase 1: Code completion. AI automatically completes code snippets based on context." },
  { source: "第二阶段：对话式编程。通过自然语言描述需求，AI 生成完整代码。", target: "Phase 2: Conversational programming. By describing requirements in natural language, AI generates complete code." },
  { source: "第三阶段：自主编程。AI 理解整个项目架构，独立完成复杂功能。", target: "Phase 3: Autonomous programming. AI understands the entire project architecture and independently completes complex features." },
];

export function TranslationPanel({ entryId: _entryId }: TranslationPanelProps) {
  const [translating, setTranslating] = useState(false);
  const [segments, setSegments] = useState<typeof mockSegments>([]);
  const [hasTranslation, setHasTranslation] = useState(false);

  const handleTranslate = () => {
    setTranslating(true);
    setSegments([]);
    // mock translation with progressive loading
    mockSegments.forEach((seg, i) => {
      setTimeout(() => {
        setSegments((prev) => [...prev, seg]);
        if (i === mockSegments.length - 1) {
          setTranslating(false);
          setHasTranslation(true);
        }
      }, i * 300);
    });
  };

  const handleClear = () => {
    setSegments([]);
    setHasTranslation(false);
  };

  return (
    <div className="max-w-4xl mx-auto px-6 py-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-base font-semibold">双语翻译</h2>
        <div className="flex items-center gap-2">
          {hasTranslation && (
            <Button variant="ghost" size="sm" onClick={handleClear}>
              清除翻译
            </Button>
          )}
          <Button
            variant="secondary"
            size="sm"
            onClick={handleTranslate}
            disabled={translating}
          >
            {translating ? (
              <>
                <svg className="animate-spin w-3.5 h-3.5" viewBox="0 0 24 24" fill="none">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" className="opacity-25" />
                  <path d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" fill="currentColor" className="opacity-75" />
                </svg>
                翻译中...
              </>
            ) : (
              "开始翻译"
            )}
          </Button>
        </div>
      </div>

      {segments.length === 0 && !translating && (
        <div className="text-center py-12 text-[var(--text-tertiary)] text-sm">
          点击上方按钮开始双语翻译，原文和译文将分栏对照显示
        </div>
      )}

      {translating && segments.length === 0 && (
        <div className="text-center py-12 text-[var(--text-tertiary)] text-sm">
          正在翻译中...
        </div>
      )}

      <div className="space-y-6">
        {segments.map((seg, i) => (
          <div key={i} className="translation-panel rounded-xl overflow-hidden border border-[var(--border-color)]">
            <div className="source bg-[var(--bg-secondary)]">
              <p className="text-sm">{seg.source}</p>
            </div>
            <div className="target bg-[var(--bg-tertiary)]">
              <p className="text-sm">{seg.target}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}