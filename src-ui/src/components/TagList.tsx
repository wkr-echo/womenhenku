import { useState, useMemo, useCallback } from "react";
import { useApp } from "@/contexts/AppContext";
import { Input, Button, Modal } from "@/components/ui";
import { cn, t } from "@/lib/utils";
import type { Tag } from "@/lib/types";
import { toast } from "@/components/ui/Toast";
import { deleteTag, updateTag } from "@/api/feed";

const TAG_COLORS = [
  "#ef4444", "#f97316", "#f59e0b", "#84cc16", "#22c55e",
  "#10b981", "#14b8a6", "#06b6d4", "#0ea5e9", "#3b82f6",
  "#6366f1", "#8b5cf6", "#a855f7", "#d946ef", "#ec4899",
];

export function TagList() {
  const { 
    tags, 
    selectedTagIds, 
    tagMatchMode,
    isBatchTagging,
    toggleTagSelection, 
    setTagMatchMode,
    reloadTags 
  } = useApp();

  const [searchQuery, setSearchQuery] = useState("");
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; tag: Tag } | null>(null);
  const [renameModal, setRenameModal] = useState<Tag | null>(null);
  const [deleteModal, setDeleteModal] = useState<Tag | null>(null);
  const [newTagName, setNewTagName] = useState("");

  const filteredTags = useMemo(() => {
    if (!searchQuery) return tags;
    const lower = searchQuery.toLowerCase();
    return tags.filter(tag => 
      tag.name.toLowerCase().includes(lower)
    );
  }, [tags, searchQuery]);

  const handleTagClick = useCallback((tag: Tag) => {
    toggleTagSelection(tag.id);
    setContextMenu(null);
  }, [toggleTagSelection]);

  const handleContextMenu = useCallback((e: React.MouseEvent, tag: Tag) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, tag });
  }, []);

  const handleCloseContextMenu = useCallback(() => {
    setContextMenu(null);
  }, []);

  const handleRename = useCallback(async (tag: Tag) => {
    if (!newTagName.trim()) {
      toast(t("标签名称不能为空"), "error");
      return;
    }
    if (tags.some(t => t.name.toLowerCase() === newTagName.toLowerCase() && t.id !== tag.id)) {
      toast(t("标签名称已存在"), "error");
      return;
    }
    try {
      await updateTag(tag.id, newTagName.trim(), tag.color);
      reloadTags();
      toast(t("标签已更新"), "success");
    } catch {
      toast(t("更新失败"), "error");
    }
    setRenameModal(null);
    setNewTagName("");
    setContextMenu(null);
  }, [tags, reloadTags]);

  const handleDelete = useCallback(async (tag: Tag) => {
    try {
      await deleteTag(tag.id);
      reloadTags();
      toast(t("标签已删除"), "success");
    } catch {
      toast(t("删除失败"), "error");
    }
    setDeleteModal(null);
    setContextMenu(null);
  }, [reloadTags]);

  const getTagColor = useCallback((tag: Tag) => {
    return tag.color || TAG_COLORS[tag.id % TAG_COLORS.length];
  }, []);

  return (
    <div className="flex flex-col h-full">
      <div className="px-3 py-2 border-b border-[var(--border-color)]">
        <div className="flex items-center gap-2">
          <Input
            placeholder={t("搜索标签...")}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="text-xs flex-1"
          />
          <select
            value={tagMatchMode}
            onChange={(e) => setTagMatchMode(e.target.value as "or" | "and")}
            className="text-xs px-2 py-1.5 rounded-lg bg-[var(--sidebar-hover)] text-[var(--text-secondary)] border border-[var(--border-color)] cursor-pointer"
          >
            <option value="or">{t("匹配任一")}</option>
            <option value="and">{t("匹配所有")}</option>
          </select>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto py-2">
        <div style={{ padding: "0 12px" }}>
        {filteredTags.length > 0 ? (
          filteredTags.map((tag) => (
            <button
              key={tag.id}
              onClick={() => handleTagClick(tag)}
              onContextMenu={(e) => handleContextMenu(e, tag)}
              role="button"
              aria-label={`toggle tag ${tag.name}`}
              className={cn(
                "w-full flex items-center gap-2 px-3 py-2 mb-1 rounded-lg text-left transition-colors",
                selectedTagIds.includes(tag.id)
                  ? "bg-blue-100 border border-blue-300"
                  : "bg-white border border-transparent hover:bg-gray-50 hover:border-gray-200"
              )}
            >
              <div
                className="w-3 h-3 rounded-full flex-shrink-0"
                style={{ backgroundColor: getTagColor(tag) }}
              />
              <span className="text-sm font-medium text-gray-800 flex-1 truncate">
                {tag.name}
              </span>
              <span className="text-xs text-gray-500">
                {tag.usageCount}
              </span>
              {selectedTagIds.includes(tag.id) && (
                <span className="text-sm font-bold text-blue-600">✓</span>
              )}
            </button>
          ))
        ) : (
          <p className="text-sm text-gray-500 text-center py-8">
            {searchQuery ? t("没有找到标签") : t("暂无标签")}
          </p>
        )}
        </div>
      </div>

      <div className="px-3 py-2 border-t border-[var(--border-color)]">
        <p className="text-xs text-gray-500">
          {t("已选择")} {selectedTagIds.length} {t("个标签")}
        </p>
      </div>

      {contextMenu && (
        <>
          <div className="fixed inset-0 z-50" onClick={handleCloseContextMenu} />
          <div
            className="fixed z-50 bg-[var(--sidebar-bg)] border border-[var(--border-color)] rounded-lg shadow-xl py-1 min-w-[160px]"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            <button
              onClick={() => {
                setRenameModal(contextMenu.tag);
                setNewTagName(contextMenu.tag.name);
              }}
              disabled={isBatchTagging}
              className={cn(
                "w-full px-4 py-2 text-left text-sm text-[var(--text-secondary)] hover:bg-[var(--sidebar-hover)] transition-colors",
                isBatchTagging && "opacity-50 cursor-not-allowed"
              )}
            >
              {t("重命名")}
            </button>
            <button
              onClick={() => setDeleteModal(contextMenu.tag)}
              disabled={isBatchTagging}
              className={cn(
                "w-full px-4 py-2 text-left text-sm text-[var(--danger-color)] hover:bg-[var(--sidebar-hover)] transition-colors",
                isBatchTagging && "opacity-50 cursor-not-allowed"
              )}
            >
              {t("删除")}
            </button>
          </div>
        </>
      )}

      <Modal
        open={!!renameModal}
        onClose={() => { setRenameModal(null); setNewTagName(""); }}
        title={t("重命名标签")}
      >
        <div className="space-y-4">
          <Input
            value={newTagName}
            onChange={(e) => setNewTagName(e.target.value)}
            placeholder={t("输入新标签名称")}
            autoFocus
          />
          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => { setRenameModal(null); setNewTagName(""); }}>
              {t("取消")}
            </Button>
            <Button size="sm" onClick={() => renameModal && handleRename(renameModal)}>
              {t("确认")}
            </Button>
          </div>
        </div>
      </Modal>

      <Modal
        open={!!deleteModal}
        onClose={() => setDeleteModal(null)}
        title={t("删除标签")}
      >
        <div className="space-y-4">
          <p className="text-sm text-[var(--text-secondary)]">
            {t("此操作将移除此标签在所有文章上的关联")}
          </p>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setDeleteModal(null)}>
              {t("取消")}
            </Button>
            <Button size="sm" variant="danger" onClick={() => deleteModal && handleDelete(deleteModal)}>
              {t("删除")}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}