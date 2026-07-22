import { useState, useEffect, useCallback } from "react";
import { t } from "@/lib/utils";
import { getEntryTags, listTags, addTag, tagEntry, untagEntry, getTagRecommendations } from "@/api/feed";
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
}

interface TagRecommendation {
  id: number;
  entryId: number;
  tagName: string;
  sourceType: string;
  confidence: number;
  createdAt: string;
}

export function TagPanelView({ entryId }: TagPanelViewProps) {
  const [entryTags, setEntryTags] = useState<Tag[]>([]);
  const [allTags, setAllTags] = useState<Tag[]>([]);
  const [newTagName, setNewTagName] = useState("");
  const [recommendations, setRecommendations] = useState<TagRecommendation[]>([]);
  const [isLoadingRecommendations, setIsLoadingRecommendations] = useState(false);

  useEffect(() => {
    async function loadTags() {
      try {
        const [entryT, allT] = await Promise.all([getEntryTags(entryId), listTags()]);
        setEntryTags(entryT);
        setAllTags(allT);
      } catch (e) {
        console.error("Failed to load tags", e);
        toast(t("加载标签失败"), "error");
      }
    }
    loadTags();
  }, [entryId]);

  useEffect(() => {
    async function loadRecommendations() {
      setIsLoadingRecommendations(true);
      try {
        const recs = await getTagRecommendations(entryId);
        setRecommendations(recs);
      } catch (e) {
        console.error("Failed to load recommendations", e);
      } finally {
        setIsLoadingRecommendations(false);
      }
    }
    loadRecommendations();
  }, [entryId]);

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
      toast(t("标签已添加"), "success");
    } catch (e: any) {
      console.error("Failed to add tag", e);
      toast(t("添加标签失败: ") + String(e), "error");
    }
  }, [newTagName, allTags, entryId]);

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
      console.error("Failed to toggle tag", e);
      toast(String(e), "error");
    }
  }, [entryTags, allTags, entryId]);

  const handleAddRecommendedTag = useCallback(async (rec: TagRecommendation) => {
    try {
      const existingTag = allTags.find(t => t.name.toLowerCase() === rec.tagName.toLowerCase());
      if (existingTag) {
        await tagEntry(entryId, existingTag.id);
        setEntryTags(prev => [...prev, existingTag]);
      } else {
        const color = TAG_COLORS[Math.floor(Math.random() * TAG_COLORS.length)];
        const tag = await addTag(rec.tagName, color);
        await tagEntry(entryId, tag.id);
        setEntryTags(prev => [...prev, tag]);
        setAllTags(prev => [...prev, tag]);
      }
      setRecommendations(prev => prev.filter(r => r.id !== rec.id));
      toast(t("标签已添加"), "success");
    } catch (e: any) {
      console.error("Failed to add recommended tag", e);
      toast(String(e), "error");
    }
  }, [allTags, entryId]);

  return (
    <div style={{
      position: "absolute", right: 0, top: 8, width: 280,
      backgroundColor: "white", border: "1px solid #e5e7eb",
      borderRadius: 8, boxShadow: "0 10px 25px rgba(0,0,0,0.1)",
      padding: 16, zIndex: 100
    }}>
      <div style={{ marginBottom: 16 }}>
        <input
          type="text"
          value={newTagName}
          onChange={(e) => setNewTagName(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") { e.preventDefault(); handleAddNewTag(); }
          }}
          placeholder="输入标签"
          style={{
            width: "100%", padding: "8px 12px", fontSize: 14,
            backgroundColor: "#f9fafb", border: "1px solid #d1d5db",
            borderRadius: 6, boxSizing: "border-box"
          }}
        />
        <button
          onClick={handleAddNewTag}
          disabled={!newTagName.trim()}
          style={{
            marginTop: 8, width: "100%", padding: "8px", fontSize: 12,
            backgroundColor: "#2563eb", color: "white", border: "none",
            borderRadius: 6, cursor: "pointer"
          }}
        >添加</button>
      </div>

      {isLoadingRecommendations ? (
        <div style={{ marginBottom: 16 }}>
          <p style={{ fontSize: 12, color: "#6b7280", marginBottom: 4 }}>AI 推荐标签</p>
          <p style={{ fontSize: 12, color: "#9ca3af" }}>加载中...</p>
        </div>
      ) : recommendations.length > 0 ? (
        <div style={{ marginBottom: 16 }}>
          <p style={{ fontSize: 12, color: "#6b7280", marginBottom: 4 }}>AI 推荐标签 ({recommendations.length})</p>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
            {recommendations.map((rec, index) => (
              <span
                key={`${rec.id}-${index}`}
                onClick={() => handleAddRecommendedTag(rec)}
                style={{
                  padding: "4px 8px", fontSize: 12, borderRadius: 12,
                  backgroundColor: "#dbeafe", color: "#1d4ed8",
                  border: "1px solid #93c5fd",
                  cursor: "pointer"
                }}
              >
                + {rec.tagName}
              </span>
            ))}
          </div>
        </div>
      ) : null}

      <div style={{ marginBottom: 16 }}>
        <p style={{ fontSize: 12, color: "#6b7280", marginBottom: 4 }}>已有标签 ({allTags.length})</p>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
          {allTags.map(tag => (
            <span
              key={tag.id}
              onClick={() => handleToggleTag(tag.id)}
              style={{
                padding: "4px 8px", fontSize: 12, borderRadius: 12,
                backgroundColor: "#f3f4f6", color: "#4b5563",
                border: "1px solid #d1d5db",
                cursor: "pointer",
                opacity: entryTags.some(et => et.id === tag.id) ? 0.5 : 1
              }}
            >
              {tag.name}
            </span>
          ))}
        </div>
      </div>

      <div style={{ borderTop: "1px solid #e5e7eb", paddingTop: 12 }}>
        <p style={{ fontSize: 12, color: "#6b7280", marginBottom: 4 }}>已应用 ({entryTags.length})</p>
        {entryTags.length > 0 ? (
          <div>
            {entryTags.map(tag => (
              <button
                key={tag.id}
                onClick={() => handleToggleTag(tag.id)}
                style={{
                  width: "100%",
                  marginBottom: 6,
                  padding: "10px 14px", fontSize: 14,
                  backgroundColor: "#2563eb", color: "white",
                  border: "2px solid #1d4ed8",
                  borderRadius: 8,
                  cursor: "pointer",
                  fontWeight: "bold",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between"
                }}
              >
                <span>{tag.name}</span>
                <span style={{ fontSize: 20, fontWeight: "normal" }}>✕</span>
              </button>
            ))}
          </div>
        ) : (
          <p style={{ fontSize: 12, color: "#9ca3af" }}>暂无标签</p>
        )}
      </div>
    </div>
  );
}