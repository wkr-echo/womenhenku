import { useState } from "react";
import { useApp } from "@/contexts/AppContext";
import { Button, Input, Modal } from "@/components/ui";
import { cn, t } from "@/lib/utils";
import type { FeedSummary, Tag } from "@/lib/types";

export function SidebarView() {
  const {
    feeds,
    selectedFeedId,
    selectFeed,
    searchQuery,
    setSearchQuery,
    sidebarCollapsed,
    toggleSidebar,
    addFeed,
    removeFeed,
    refreshAll,
    setViewMode,
    tags,
    selectedTagId,
    selectTag,
  } = useApp();

  const [showAddModal, setShowAddModal] = useState(false);
  const [feedUrl, setFeedUrl] = useState("");
  const [refreshAnimating, setRefreshAnimating] = useState(false);

  const handleAddFeed = () => {
    if (feedUrl.trim()) {
      addFeed(feedUrl.trim());
      setFeedUrl("");
      setShowAddModal(false);
    }
  };

  const handleRefreshAll = () => {
    setRefreshAnimating(true);
    refreshAll();
    setTimeout(() => setRefreshAnimating(false), 1000);
  };

  if (sidebarCollapsed) {
    return (
      <div className="w-12 h-full flex flex-col items-center py-4 gap-3 border-r border-[var(--border-color)] bg-[var(--sidebar-bg)]">
        <button
          onClick={toggleSidebar}
          className="p-2 rounded-lg hover:bg-[var(--sidebar-hover)] text-[var(--text-secondary)] transition-colors"
          title={t("展开侧边栏")}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
          </svg>
        </button>
        <button
          onClick={() => setViewMode("settings")}
          className="p-2 rounded-lg hover:bg-[var(--sidebar-hover)] text-[var(--text-secondary)] transition-colors mt-auto"
          title={t("设置")}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path strokeLinecap="round" strokeLinejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </button>
      </div>
    );
  }

  return (
    <div className="w-[280px] h-full flex flex-col border-r border-[var(--border-color)] bg-[var(--sidebar-bg)] animate-slide-in">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--border-color)]">
        <h1 className="font-semibold text-sm text-[var(--text-primary)]">Platinum</h1>
        <div className="flex items-center gap-1">
          <button
            onClick={handleRefreshAll}
            className="p-1.5 rounded-lg hover:bg-[var(--sidebar-hover)] text-[var(--text-secondary)] transition-colors"
            title={t("刷新全部")}
          >
            <svg
              className={cn("w-4 h-4", refreshAnimating && "animate-spin")}
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
          <button
            onClick={toggleSidebar}
            className="p-1.5 rounded-lg hover:bg-[var(--sidebar-hover)] text-[var(--text-secondary)] transition-colors"
            title={t("收起侧边栏")}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 19l-7-7 7-7" />
            </svg>
          </button>
        </div>
      </div>

      {/* Search */}
      <div className="px-3 py-2">
        <Input
          placeholder={t("搜索文章...")}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="text-xs"
        />
      </div>

      {/* Feed list */}
      <div className="flex-1 overflow-y-auto py-2">
        <div className="px-3 mb-2">
          <p className="text-xs font-medium text-[var(--text-tertiary)] uppercase tracking-wider px-2 py-1">
            {t("订阅源")}
          </p>
        </div>
        {feeds.map((feed) => (
          <FeedItem
            key={feed.id}
            feed={feed}
            isSelected={selectedFeedId === feed.id}
            onSelect={() => selectFeed(feed.id)}
            onRemove={() => removeFeed(feed.id)}
          />
        ))}
        {feeds.length === 0 && (
          <p className="text-sm text-[var(--text-tertiary)] text-center py-4">
            {t("暂无订阅源")}
          </p>
        )}

        {/* Tags list */}
        {tags.length > 0 && (
          <>
            <div className="px-3 mb-2 mt-4">
              <p className="text-xs font-medium text-[var(--text-tertiary)] uppercase tracking-wider px-2 py-1">
                {t("标签")}
              </p>
            </div>
            {tags.map((tag) => (
              <TagItem
                key={tag.id}
                tag={tag}
                isSelected={selectedTagId === tag.id}
                onSelect={() => selectTag(tag.id)}
              />
            ))}
          </>
        )}
      </div>

      {/* Bottom actions */}
      <div className="px-3 py-3 border-t border-[var(--border-color)] flex gap-2">
        <Button
          variant="secondary"
          size="sm"
          className="flex-1"
          onClick={() => setShowAddModal(true)}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
          </svg>
          {t("添加订阅")}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setViewMode("settings")}
          title={t("设置")}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path strokeLinecap="round" strokeLinejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </Button>
      </div>

      {/* Add Feed Modal */}
      <Modal
        open={showAddModal}
        onClose={() => setShowAddModal(false)}
        title={t("添加订阅源")}
      >
        <div className="space-y-4">
          <Input
            placeholder={t("输入 RSS/Atom/JSON Feed 地址...")}
            value={feedUrl}
            onChange={(e) => setFeedUrl(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddFeed()}
          />
          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setShowAddModal(false)}>
              {t("取消")}
            </Button>
            <Button size="sm" onClick={handleAddFeed}>
              {t("添加")}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

function FeedItem({
  feed,
  isSelected,
  onSelect,
  onRemove,
}: {
  feed: FeedSummary;
  isSelected: boolean;
  onSelect: () => void;
  onRemove: () => void;
}) {
  return (
    <div
      onClick={onSelect}
      className={cn(
        "group flex items-center gap-2 px-3 py-2 mx-2 rounded-lg cursor-pointer transition-colors",
        isSelected
          ? "bg-[var(--sidebar-active)] text-[var(--text-primary)]"
          : "hover:bg-[var(--sidebar-hover)] text-[var(--text-secondary)]"
      )}
    >
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium truncate">{feed.title}</span>
        </div>
      </div>
      {feed.unreadCount > 0 && (
        <span className="flex-shrink-0 min-w-[20px] h-5 rounded-full bg-[var(--accent-color)] text-white text-xs flex items-center justify-center px-1.5">
          {feed.unreadCount}
        </span>
      )}
      <button
        onClick={(e) => {
          e.stopPropagation();
          onRemove();
        }}
        className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-[var(--sidebar-active)] text-[var(--text-tertiary)] hover:text-[var(--danger-color)] transition-all flex-shrink-0"
        title={t("删除订阅源")}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </button>
    </div>
  );
}

function TagItem({
  tag,
  isSelected,
  onSelect,
}: {
  tag: Tag;
  isSelected: boolean;
  onSelect: () => void;
}) {
  return (
    <div
      onClick={onSelect}
      className={cn(
        "flex items-center gap-2 px-3 py-1.5 mx-2 rounded-lg cursor-pointer transition-colors",
        isSelected
          ? "bg-[var(--sidebar-active)]"
          : "hover:bg-[var(--sidebar-hover)]"
      )}
    >
      <div
        className="w-2 h-2 rounded-full flex-shrink-0"
        style={{ backgroundColor: tag.color }}
      />
      <span className="text-sm truncate text-[var(--text-secondary)]">{tag.name}</span>
    </div>
  );
}