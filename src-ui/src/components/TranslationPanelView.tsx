import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui";
import { t } from "@/lib/utils";
import {
  translateEntry,
  getTranslationText,
  cancelTranslation,
  clearTranslation as clearTranslationApi,
  listenAiStream,
  getEntrySegments,
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
  const didInitRef = useRef(false);

  // 鍒濆鍖栵細鍔犺浇宸叉湁缈昏瘧
  useEffect(() => {
    if (didInitRef.current) return;
    didInitRef.current = true;

    // 鍔犺浇宸叉湁鐨勭炕璇戠粨鏋?    const loadTranslation = async () => {
      try {
        const text = await getTranslationText(entryId);
        if (text) {
          // 浠庢牸寮忓寲鐨勬枃鏈腑瑙ｆ瀽娈佃惤瀵?          const parsed = parseTranslationOutput(text);
          setSegments(parsed);
        }
      } catch {
        // 娌℃湁缈昏瘧涔熸甯?      }
    };
    loadTranslation();

    // 鐩戝惉 AI 娴佸紡浜嬩欢
    listenAiStream((event: AiStreamEvent) => {
      if (event.agentType !== "translation") return;

      if (event.isDone) {
        setTranslating(false);
        if (event.error) {
          setError(event.error);
        } else {
          // 缈昏瘧瀹屾垚鍚庨噸鏂板姞杞?          getTranslationText(entryId).then((text) => {
            if (text) {
              const parsed = parseTranslationOutput(text);
              if (parsed.length > 0) setSegments(parsed);
            }
          });
        }
        return;
      }

      // 娴佸紡鍐呭鍙兘甯︽湁娈佃惤鏍囪瘑濡?"[1/4] 璇戞枃"
      setStreamSegments((prev) => [...prev, event.content]);
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
      await translateEntry(entryId);
    } catch (err: any) {
      setTranslating(false);
      setError(String(err));
    }
  };

  const handleCancel = async () => {
    try {
      await cancelTranslation(entryId);
    } catch {
      // 蹇界暐
    }
    setTranslating(false);
  };

  const handleClear = async () => {
    try {
      await clearTranslationApi(entryId);
    } catch {
      // 蹇界暐
    }
    setSegments([]);
    setStreamSegments([]);
    setError(null);
  };

  return (
    <div className="max-w-4xl mx-auto px-6 py-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-base font-semibold">{t("鍙岃缈昏瘧")}</h2>
        <div className="flex items-center gap-2">
          {segments.length > 0 && (
            <Button variant="ghost" size="sm" onClick={handleClear}>
              {t("娓呴櫎缈昏瘧")}
            </Button>
          )}
          {translating ? (
            <Button variant="ghost" size="sm" onClick={handleCancel}>
              {t("鍙栨秷")}
            </Button>
          ) : (
            <Button
              variant="secondary"
              size="sm"
              onClick={handleTranslate}
            >
              {segments.length > 0 ? t("閲嶆柊缈昏瘧") : t("寮€濮嬬炕璇?)}
            </Button>
          )}
        </div>
      </div>

      {segments.length === 0 && !translating && !error && (
        <div className="text-center py-12 text-[var(--text-tertiary)] text-sm">
          {t("鐐瑰嚮涓婃柟鎸夐挳寮€濮嬪弻璇炕璇戯紝鍘熸枃鍜岃瘧鏂囧皢鍒嗘爮瀵圭収鏄剧ず")}
        </div>
      )}

      {error && (
        <div className="text-center py-8 text-sm text-red-500">
          <p>{t("缈昏瘧鍑洪敊")}</p>
          <p className="text-xs text-[var(--text-tertiary)] mt-1">{error}</p>
          <Button variant="ghost" size="sm" onClick={() => setError(null)} className="mt-2">
            {t("鍏抽棴")}
          </Button>
        </div>
      )}

      {translating && (
        <div className="flex items-center gap-2 justify-center py-4 text-sm text-[var(--text-tertiary)]">
          <svg className="animate-spin w-4 h-4 text-[var(--accent-color)]" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" className="opacity-25" />
            <path d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" fill="currentColor" className="opacity-75" />
          </svg>
          <span>{t("姝ｅ湪缈昏瘧涓?..")}</span>
        </div>
      )}

      {streamSegments.length > 0 && translating && (
        <div className="mb-4 p-3 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)]">
          <p className="text-xs text-[var(--text-tertiary)] mb-1">{t("瀹炴椂缈昏瘧杩涘害锛?)}</p>
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
 * 浠庣炕璇戠粨鏋滄枃鏈腑瑙ｆ瀽娈佃惤瀵? * 鏍煎紡: [1]\n鍘熸枃: xxx\n璇戞枃: xxx\n\n[2]\n...
 */
function parseTranslationOutput(text: string): SegmentPair[] {
  const segments: SegmentPair[] = [];
  const blocks = text.split("\n\n");

  for (const block of blocks) {
    const lines = block.trim().split("\n");
    if (lines.length < 2) continue;

    // 绗竴琛? [index]
    const indexMatch = lines[0].match(/\[(\d+)\]/);
    const index = indexMatch ? parseInt(indexMatch[1]) - 1 : segments.length;

    // 鏌ユ壘鍘熸枃鍜岃瘧鏂囪
    let source = "";
    let target = "";
    for (const line of lines) {
      if (line.startsWith("鍘熸枃: ")) {
        source = line.slice("鍘熸枃: ".length);
      } else if (line.startsWith("璇戞枃: ")) {
        target = line.slice("璇戞枃: ".length);
      }
    }

    if (source || target) {
      segments.push({ index, source, target: target || "(缈昏瘧澶辫触)" });
    }
  }

  return segments;
}

