import type { Tag, TagAlias, DuplicateTagPair, FeedSummary, EntryListItem, Entry, Content } from "@/lib/types";

export const mockFeedSummaries: FeedSummary[] = [
  { id: 1, title: "技术博客", unreadCount: 5 },
  { id: 2, title: "科技新闻", unreadCount: 12 },
];

export const mockEntries: Record<number, EntryListItem[]> = {
  1: [
    { id: 1, feedId: 1, title: "React 18 新特性详解", author: "张三", summary: "React 18 引入了许多新特性...", publishedAt: "2026-07-20T08:00:00Z", isRead: false },
    { id: 2, feedId: 1, title: "TypeScript 高级类型", author: "李四", summary: "深入理解 TypeScript 高级类型...", publishedAt: "2026-07-19T10:00:00Z", isRead: true },
    { id: 3, feedId: 1, title: "Rust 入门指南", author: "王五", summary: "从零开始学习 Rust 编程语言...", publishedAt: "2026-07-18T09:00:00Z", isRead: false },
  ],
  2: [
    { id: 4, feedId: 2, title: "AI 技术突破", author: "赵六", summary: "最新 AI 技术进展...", publishedAt: "2026-07-20T07:00:00Z", isRead: false },
    { id: 5, feedId: 2, title: "量子计算进展", author: "钱七", summary: "量子计算最新研究成果...", publishedAt: "2026-07-19T11:00:00Z", isRead: true },
  ],
};

export const mockContent: Content = {
  id: 1,
  entryId: 1,
  rawHtml: "<html><body><h1>React 18 新特性详解</h1><p>React 18 引入了许多新特性...</p></body></html>",
  cleanedHtml: "<h1>React 18 新特性详解</h1><p>React 18 引入了许多新特性...</p>",
  cleanedMarkdown: "# React 18 新特性详解\n\nReact 18 引入了许多新特性...",
  renderedHtml: "<div class='article'><h1>React 18 新特性详解</h1><p>React 18 引入了许多新特性...</p></div>",
  readabilityVersion: 1,
  createdAt: "2026-07-20T08:00:00Z",
  updatedAt: "2026-07-20T08:00:00Z",
};

export const mockApi = {
  searchEntries: (query: string): EntryListItem[] => {
    const lowerQuery = query.toLowerCase();
    return Object.values(mockEntries).flat().filter(e => 
      e.title.toLowerCase().includes(lowerQuery) || 
      e.author?.toLowerCase().includes(lowerQuery)
    );
  },
  getEntry: (id: number): Entry | null => {
    const entryList = Object.values(mockEntries).flat().find(e => e.id === id);
    if (!entryList) return null;
    return {
      id: entryList.id,
      feedId: entryList.feedId,
      guid: `guid-${entryList.id}`,
      title: entryList.title,
      author: entryList.author,
      link: `https://example.com/${entryList.id}`,
      summary: entryList.summary,
      publishedAt: entryList.publishedAt,
      updatedAt: entryList.publishedAt,
      isRead: entryList.isRead,
      isStarred: false,
      createdAt: "2026-07-20T08:00:00Z",
    };
  },
};

let tags: Tag[] = [
  { id: 1, name: "machine learning", normalizedName: "machine learning", color: "#3b82f6", isProvisional: false, usageCount: 5, createdAt: "2026-07-01T00:00:00Z" },
  { id: 2, name: "python", normalizedName: "python", color: "#10b981", isProvisional: false, usageCount: 3, createdAt: "2026-07-02T00:00:00Z" },
  { id: 3, name: "rust", normalizedName: "rust", color: "#f59e0b", isProvisional: false, usageCount: 8, createdAt: "2026-07-03T00:00:00Z" },
  { id: 4, name: "articles", normalizedName: "articles", color: "#8b5cf6", isProvisional: false, usageCount: 2, createdAt: "2026-07-04T00:00:00Z" },
  { id: 5, name: "article", normalizedName: "article", color: "#ec4899", isProvisional: false, usageCount: 1, createdAt: "2026-07-05T00:00:00Z" },
  { id: 6, name: "programming", normalizedName: "programming", color: "#06b6d4", isProvisional: false, usageCount: 4, createdAt: "2026-07-06T00:00:00Z" },
  { id: 7, name: "programing", normalizedName: "programing", color: "#84cc16", isProvisional: true, usageCount: 0, createdAt: "2026-07-07T00:00:00Z" },
];

let aliases: TagAlias[] = [
  { id: 1, tagId: 1, alias: "ML", normalizedAlias: "ml", createdAt: "2026-07-01T00:00:00Z" },
  { id: 2, tagId: 1, alias: "ai", normalizedAlias: "ai", createdAt: "2026-07-01T00:00:00Z" },
];

let nextTagId = 8;
let nextAliasId = 3;

function normalizeName(name: string): string {
  return name.trim().toLowerCase().replace(/[-_.\s]+/g, " ").trim();
}

export async function mockAddTag(name: string, color: string = "#3b82f6"): Promise<Tag> {
  const normalizedName = normalizeName(name);
  const existing = tags.find(t => t.normalizedName === normalizedName);
  if (existing) {
    return existing;
  }
  const newTag: Tag = {
    id: nextTagId++,
    name,
    normalizedName,
    color,
    isProvisional: false,
    usageCount: 0,
    createdAt: new Date().toISOString(),
  };
  tags.push(newTag);
  return newTag;
}

export async function mockListTags(): Promise<Tag[]> {
  return [...tags];
}

export async function mockGetTag(id: number): Promise<Tag> {
  const tag = tags.find(t => t.id === id);
  if (!tag) throw new Error(`Tag id=${id} not found`);
  return tag;
}

export async function mockUpdateTag(id: number, name: string, color: string): Promise<Tag> {
  const index = tags.findIndex(t => t.id === id);
  if (index === -1) throw new Error(`Tag id=${id} not found`);
  tags[index] = {
    ...tags[index],
    name,
    normalizedName: normalizeName(name),
    color,
  };
  return tags[index];
}

export async function mockDeleteTag(id: number): Promise<void> {
  const index = tags.findIndex(t => t.id === id);
  if (index === -1) throw new Error(`Tag id=${id} not found`);
  tags.splice(index, 1);
  aliases = aliases.filter(a => a.tagId !== id);
}

export async function mockRenameTag(id: number, newName: string): Promise<Tag> {
  return mockUpdateTag(id, newName, (tags.find(t => t.id === id)?.color) || "#3b82f6");
}

export async function mockAddTagAlias(tagId: number, alias: string): Promise<TagAlias> {
  const tag = tags.find(t => t.id === tagId);
  if (!tag) throw new Error(`Tag id=${tagId} not found`);
  const existing = aliases.find(a => a.tagId === tagId && a.normalizedAlias === normalizeName(alias));
  if (existing) return existing;
  const newAlias: TagAlias = {
    id: nextAliasId++,
    tagId,
    alias,
    normalizedAlias: normalizeName(alias),
    createdAt: new Date().toISOString(),
  };
  aliases.push(newAlias);
  return newAlias;
}

export async function mockRemoveTagAlias(tagId: number, alias: string): Promise<void> {
  const normalized = normalizeName(alias);
  const index = aliases.findIndex(a => a.tagId === tagId && a.normalizedAlias === normalized);
  if (index === -1) throw new Error("Alias not found");
  aliases.splice(index, 1);
}

export async function mockGetTagAliases(tagId: number): Promise<TagAlias[]> {
  return aliases.filter(a => a.tagId === tagId);
}

export async function mockMergeTags(targetId: number, sourceIds: number[]): Promise<void> {
  const target = tags.find(t => t.id === targetId);
  if (!target) throw new Error(`Target tag id=${targetId} not found`);
  const sources = tags.filter(t => sourceIds.includes(t.id));
  sources.forEach(source => {
    target.usageCount += source.usageCount;
    aliases.forEach(a => {
      if (a.tagId === source.id) {
        a.tagId = targetId;
      }
    });
  });
  tags = tags.filter(t => !sourceIds.includes(t.id));
}

export async function mockDetectDuplicateTags(): Promise<DuplicateTagPair[]> {
  const duplicates: DuplicateTagPair[] = [];
  for (let i = 0; i < tags.length; i++) {
    for (let j = i + 1; j < tags.length; j++) {
      const t1 = tags[i];
      const t2 = tags[j];
      if (t1.normalizedName.endsWith("s") && t1.normalizedName.slice(0, -1) === t2.normalizedName) {
        duplicates.push({ tagA: t1, tagB: t2, reason: "复数变体" });
      } else if (t1.normalizedName.replace(/\s/g, "") === t2.normalizedName.replace(/\s/g, "")) {
        duplicates.push({ tagA: t1, tagB: t2, reason: "命名变体" });
      } else {
        const dist = levenshteinDistance(t1.normalizedName, t2.normalizedName);
        if (dist > 0 && dist <= 2 && Math.min(t1.normalizedName.length, t2.normalizedName.length) >= 6) {
          duplicates.push({ tagA: t1, tagB: t2, reason: "拼写变体" });
        }
      }
    }
  }
  return duplicates;
}

export async function mockFindUnusedTags(): Promise<Tag[]> {
  return tags.filter(t => t.usageCount === 0);
}

export async function mockDeleteUnusedTags(): Promise<number> {
  const count = tags.filter(t => t.usageCount === 0).length;
  tags = tags.filter(t => t.usageCount > 0);
  return count;
}

function levenshteinDistance(a: string, b: string): number {
  const dp: number[][] = Array(a.length + 1).fill(null).map(() => Array(b.length + 1).fill(0));
  for (let i = 0; i <= a.length; i++) dp[i][0] = i;
  for (let j = 0; j <= b.length; j++) dp[0][j] = j;
  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      dp[i][j] = Math.min(dp[i - 1][j] + 1, dp[i][j - 1] + 1, dp[i - 1][j - 1] + cost);
    }
  }
  return dp[a.length][b.length];
}