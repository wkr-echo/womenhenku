import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { t } from "@/lib/utils";
import { getEntryTags, listTags, addTag, tagEntry, untagEntry, generateTagRecommendations } from "@/api/feed";
import { useApp } from "@/contexts/AppContext";
import type { Tag } from "@/lib/types";
import { toast } from "@/components/ui/Toast";

const TAG_COLORS = [
  "#ef4444", "#f97316", "#f59e0b", "#84cc16", "#22c55e",
  "#10b981", "#14b8a6", "#06b6d4", "#0ea5e9", "#3b82f6",
];

interface TagPanelViewProps {
  entryId: number;
  selectedEntryTitle: string;
  contentMarkdown?: string;
  onClose: () => void;
}

interface TagSuggestion {
  tagName: string;
  sourceType: "ai" | "nlp";
  confidence: number;
}

/** NLP: extract capitalized words (>=3 chars, <=4 words, <=25 chars total) from title+summary */
function extractNlpEntities(title: string, summary?: string): string[] {
  const results = new Set<string>();
  const re = /\b[A-Z][a-zA-Z]{2,}(?:\s+[A-Z][a-zA-Z]{2,}){0,3}\b/g;
  const titleMatches = title.match(re) || [];
  for (const m of titleMatches) {
    if (m.length <= 25 && m.split(/\s+/).length <= 4) results.add(m);
  }
  if (summary) {
    const sm = summary.match(re) || [];
    for (const m of sm) {
      if (m.length <= 25 && m.split(/\s+/).length <= 4) results.add(m);
    }
  }
  return Array.from(results).slice(0, 10);
}

export function TagPanelView({ entryId, selectedEntryTitle, contentMarkdown, onClose }: TagPanelViewProps) {
  const { reloadTags, tags: appTags } = useApp();
  const panelRef = useRef<HTMLDivElement>(null);
  const [entryTags, setEntryTags] = useState<Tag[]>([]);

  // Click outside to close
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [onClose]);
  const [allTags, setAllTags] = useState<Tag[]>([]);
  const [newTagName, setNewTagName] = useState("");
  const [aiSuggestions, setAiSuggestions] = useState<TagSuggestion[]>([]);
  const [isAiLoading, setIsAiLoading] = useState(false);
  const aiAbortRef = useRef(false);

  // NLP extraction (sync, runs immediately on render)
  const nlpSuggestions = useMemo(() => {
    return extractNlpEntities(selectedEntryTitle, contentMarkdown).map(name => ({
      tagName: name,
      sourceType: "nlp" as const,
      confidence: 0.5,
    }));
  }, [selectedEntryTitle, contentMarkdown]);

  // Load tags + auto-generate AI recommendations on entry change
  useEffect(() => {
    let cancelled = false;
    aiAbortRef.current = false;

    (async () => {
      try {
        const [entryT, allT] = await Promise.all([getEntryTags(entryId), listTags()]);
        if (!cancelled) { setEntryTags(entryT); setAllTags(allT); }
      } catch (e: any) {
        if (!cancelled) toast(t("加载标签失败") + ": " + String(e?.message || String(e)), "error");
      }
    })();

    // Auto-generate AI recommendations (silent fail)
    (async () => {
      setIsAiLoading(true);
      try {
        const existingNames = appTags.map(t => t.name);
        const recs = await generateTagRecommendations(entryId, existingNames);
        if (!cancelled && !aiAbortRef.current) {
          setAiSuggestions(recs.map(r => ({
            tagName: r.tagName,
            sourceType: "ai" as const,
            confidence: r.confidence,
          })));
        }
      } catch {
        // Silent fail — NLP results still show
      } finally {
        if (!cancelled) setIsAiLoading(false);
      }
    })();

    return () => {
      cancelled = true;
      aiAbortRef.current = true;
    };
  }, [entryId, appTags]);

  // Merge AI + NLP, filter applied tags, AI first
  const allSuggestions = useMemo(() => {
    const appliedNames = new Set(entryTags.map(t => t.name.toLowerCase()));
    const seen = new Set<string>();
    const merged: TagSuggestion[] = [];
    for (const s of [...aiSuggestions, ...nlpSuggestions]) {
      const lower = s.tagName.toLowerCase();
      if (appliedNames.has(lower) || seen.has(lower)) continue;
      seen.add(lower);
      merged.push(s);
    }
    return merged;
  }, [aiSuggestions, nlpSuggestions, entryTags]);

  const handleAddNewTag = useCallback(async () => {
    if (!newTagName.trim()) return;
    const tagName = newTagName.trim();
    try {
      const existingTag = allTags.find(t => t.name.toLowerCase() === tagName.toLowerCase());
      if (existingTag) {
        await tagEntry(entryId, existingTag.id);
        setEntryTags(prev => [...prev, existingTag]);
      } else {
        const color = TAG_COLORS[Math.floor(Math.random() * TAG_COLORS.length)];
        const tag = await addTag(tagName, color);
        await tagEntry(entryId, tag.id);
        setEntryTags(prev => [...prev, tag]);
        setAllTags(prev => [...prev, tag]);
      }
      setNewTagName("");
      reloadTags();
    } catch (e: any) {
      toast(t("添加标签失败: ") + String(e), "error");
    }
  }, [newTagName, allTags, entryId, reloadTags]);

  const handleToggleTag = useCallback(async (tagId: number) => {
    const isTagged = entryTags.some(t => t.id === tagId);
    try {
      if (isTagged) {
        await untagEntry(entryId, tagId);
        setEntryTags(prev => prev.filter(t => t.id !== tagId));
      } else {
        await tagEntry(entryId, tagId);
        const tag = allTags.find(t => t.id === tagId);
        if (tag) setEntryTags(prev => [...prev, tag]);
      }
    } catch (e: any) {
      toast(String(e), "error");
    }
  }, [entryTags, allTags, entryId]);

  const handleAddSuggested = useCallback(async (suggestion: TagSuggestion) => {
    try {
      const existingTag = allTags.find(t => t.name.toLowerCase() === suggestion.tagName.toLowerCase());
      if (existingTag) {
        await tagEntry(entryId, existingTag.id);
        setEntryTags(prev => [...prev, existingTag]);
      } else {
        const color = TAG_COLORS[Math.floor(Math.random() * TAG_COLORS.length)];
        const tag = await addTag(suggestion.tagName, color);
        await tagEntry(entryId, tag.id);
        setEntryTags(prev => [...prev, tag]);
        setAllTags(prev => [...prev, tag]);
      }
      if (suggestion.sourceType === "ai") {
        setAiSuggestions(prev => prev.filter(s => s.tagName !== suggestion.tagName));
      }
      reloadTags();
    } catch (e: any) {
      toast(String(e), "error");
    }
  }, [allTags, entryId, reloadTags]);

  return (
    <div ref={panelRef} style={{
      position: "absolute", right: 0, top: 8, width: 280,
      backgroundColor: "white", border: "1px solid #e5e7eb",
      borderRadius: 8, boxShadow: "0 10px 25px rgba(0,0,0,0.1)",
      padding: 16, zIndex: 100, maxHeight: "80vh", overflowY: "auto",
    }}>
      <div style={{ marginBottom: 16 }}>
        <input type="text" value={newTagName} onChange={(e) => setNewTagName(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") { e.preventDefault(); handleAddNewTag(); } }}
          placeholder={t("输入标签")}
          style={{ width: "100%", padding: "8px 12px", fontSize: 14,
            backgroundColor: "#f9fafb", border: "1px solid #d1d5db", borderRadius: 6, boxSizing: "border-box" }} />
        <button onClick={handleAddNewTag} disabled={!newTagName.trim()}
          style={{ marginTop: 8, width: "100%", padding: "8px", fontSize: 12,
            backgroundColor: "#2563eb", color: "white", border: "none", borderRadius: 6, cursor: "pointer" }}>
          {t("添加")}
        </button>
      </div>

      {(allSuggestions.length > 0 || isAiLoading) && (
        <div style={{ marginBottom: 16 }}>
          <p style={{ fontSize: 12, color: "#6b7280", marginBottom: 4 }}>
            {t("建议标签")}
            {isAiLoading && <span style={{ color: "#9ca3af", marginLeft: 4 }}>{t("AI 生成中...")}</span>}
          </p>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
            {aiSuggestions.map((s, i) => (
              <span key={`ai-${i}`} onClick={() => handleAddSuggested(s)}
                style={{ padding: "4px 8px", fontSize: 12, borderRadius: 12,
                  backgroundColor: "#dbeafe", color: "#1d4ed8", border: "1px solid #93c5fd", cursor: "pointer" }}>
                + {s.tagName}
              </span>
            ))}
            {nlpSuggestions.map((s, i) => (
              <span key={`nlp-${i}`} onClick={() => handleAddSuggested(s)}
                style={{ padding: "4px 8px", fontSize: 12, borderRadius: 12,
                  backgroundColor: "#f3f4f6", color: "#4b5563", border: "1px solid #d1d5db", cursor: "pointer" }}>
                + {s.tagName}
              </span>
            ))}
          </div>
        </div>
      )}

      <div style={{ marginBottom: 16 }}>
        <p style={{ fontSize: 12, color: "#6b7280", marginBottom: 4 }}>{t("已有标签")} ({allTags.length})</p>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
          {allTags.map(tag => (
            <span key={tag.id} onClick={() => handleToggleTag(tag.id)}
              style={{ padding: "4px 8px", fontSize: 12, borderRadius: 12,
                backgroundColor: "#f3f4f6", color: "#4b5563", border: "1px solid #d1d5db",
                cursor: "pointer", opacity: entryTags.some(et => et.id === tag.id) ? 0.5 : 1 }}>
              {tag.name}
            </span>
          ))}
        </div>
      </div>

      <div style={{ borderTop: "1px solid #e5e7eb", paddingTop: 12 }}>
        <p style={{ fontSize: 12, color: "#6b7280", marginBottom: 4 }}>{t("已应用")} ({entryTags.length})</p>
        {entryTags.length > 0 ? (
          <div>
            {entryTags.map(tag => (
              <button key={tag.id} onClick={() => handleToggleTag(tag.id)}
                style={{ width: "100%", marginBottom: 6, padding: "10px 14px", fontSize: 14,
                  backgroundColor: "#2563eb", color: "white", border: "2px solid #1d4ed8",
                  borderRadius: 8, cursor: "pointer", fontWeight: "bold",
                  display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                {tag.name} <span style={{ fontSize: 12, opacity: 0.7 }}>×</span>
              </button>
            ))}
          </div>
        ) : (
          <p style={{ fontSize: 12, color: "#9ca3af" }}>{t("暂无")}</p>
        )}
      </div>
    </div>
  );
}
