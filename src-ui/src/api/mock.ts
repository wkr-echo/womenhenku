import type { FeedSummary, EntryListItem, Entry, Content, Provider, Note, Summary } from "@/lib/types";

// ============ Mock Data ============

export const mockFeedSummaries: FeedSummary[] = [
  { id: 1, title: "阮一峰的网络日志", unreadCount: 12 },
  { id: 2, title: "少数派", unreadCount: 5 },
  { id: 3, title: "Hacker Talk", unreadCount: 0 },
];

export const mockEntries: Record<number, EntryListItem[]> = {
  1: [
    { id: 101, feedId: 1, title: "科技爱好者周刊（第 300 期）：AI 时代的编程新范式", author: "阮一峰", publishedAt: new Date(Date.now() - 3600000).toISOString(), isRead: false },
    { id: 102, feedId: 1, title: "WebAssembly 入门指南", author: "阮一峰", publishedAt: new Date(Date.now() - 7200000).toISOString(), isRead: false },
    { id: 103, feedId: 1, title: "Rust 异步编程深入理解", author: "阮一峰", publishedAt: new Date(Date.now() - 86400000).toISOString(), isRead: true },
    { id: 104, feedId: 1, title: "CSS Container Queries 实战", author: "阮一峰", publishedAt: new Date(Date.now() - 172800000).toISOString(), isRead: true },
    { id: 105, feedId: 1, title: "每周科技新闻汇总（7月第一周）", author: "阮一峰", publishedAt: new Date(Date.now() - 259200000).toISOString(), isRead: true },
  ],
  2: [
    { id: 201, feedId: 2, title: "如何打造高效的个人工作流", author: "少数派", publishedAt: new Date(Date.now() - 1800000).toISOString(), isRead: false },
    { id: 202, feedId: 2, title: "macOS 15 新功能全面解析", author: "少数派", publishedAt: new Date(Date.now() - 43200000).toISOString(), isRead: false },
  ],
  3: [],
};

export const mockEntryDetail: Record<number, Entry> = {
  101: {
    id: 101, feedId: 1, guid: "g1",
    title: "科技爱好者周刊（第 300 期）：AI 时代的编程新范式",
    author: "阮一峰", link: "https://example.com/1",
    summary: "随着 AI 工具的普及，编程的方式正在发生根本性的变化...",
    publishedAt: new Date(Date.now() - 3600000).toISOString(),
    updatedAt: null, isRead: false, isStarred: false,
    createdAt: "2026-07-14T00:00:00Z",
  },
  102: {
    id: 102, feedId: 1, guid: "g2",
    title: "WebAssembly 入门指南", author: "阮一峰",
    link: "https://example.com/2",
    summary: "WebAssembly 是一种新的二进制格式，可以在浏览器中运行...",
    publishedAt: new Date(Date.now() - 7200000).toISOString(),
    updatedAt: null, isRead: false, isStarred: false,
    createdAt: "2026-07-13T00:00:00Z",
  },
  103: {
    id: 103, feedId: 1, guid: "g3",
    title: "Rust 异步编程深入理解", author: "阮一峰",
    link: "https://example.com/3",
    summary: "异步编程是 Rust 的核心特性之一，本文深入探讨...",
    publishedAt: new Date(Date.now() - 86400000).toISOString(),
    updatedAt: null, isRead: true, isStarred: true,
    createdAt: "2026-07-12T00:00:00Z",
  },
  104: {
    id: 104, feedId: 1, guid: "g4",
    title: "CSS Container Queries 实战", author: "阮一峰",
    link: "https://example.com/4",
    summary: "Container Queries 终于在所有主流浏览器中得到支持...",
    publishedAt: new Date(Date.now() - 172800000).toISOString(),
    updatedAt: null, isRead: true, isStarred: false,
    createdAt: "2026-07-11T00:00:00Z",
  },
  105: {
    id: 105, feedId: 1, guid: "g5",
    title: "每周科技新闻汇总（7月第一周）", author: "阮一峰",
    link: "https://example.com/5",
    summary: "本周科技界重要新闻：AI 芯片新突破...",
    publishedAt: new Date(Date.now() - 259200000).toISOString(),
    updatedAt: null, isRead: true, isStarred: false,
    createdAt: "2026-07-10T00:00:00Z",
  },
  201: {
    id: 201, feedId: 2, guid: "sg1",
    title: "如何打造高效的个人工作流", author: "少数派",
    link: "https://example.com/s1",
    summary: "一个高效的工作流能让你事半功倍...",
    publishedAt: new Date(Date.now() - 1800000).toISOString(),
    updatedAt: null, isRead: false, isStarred: false,
    createdAt: "2026-07-14T00:00:00Z",
  },
  202: {
    id: 202, feedId: 2, guid: "sg2",
    title: "macOS 15 新功能全面解析", author: "少数派",
    link: "https://example.com/s2",
    summary: "macOS 15 带来了许多令人兴奋的新功能...",
    publishedAt: new Date(Date.now() - 43200000).toISOString(),
    updatedAt: null, isRead: false, isStarred: false,
    createdAt: "2026-07-13T00:00:00Z",
  },
};

export const mockContent: Content = {
  id: 1,
  entryId: 101,
  rawHtml: "<html><body><h1>科技爱好者周刊（第 300 期）</h1><p>随着 AI 工具的普及，编程的方式正在发生根本性的变化。从 Copilot 到 Cursor，从 ChatGPT 到 Claude，AI 辅助编程已经成为主流。</p><h2>AI 编程的三个阶段</h2><p>第一阶段：代码补全。AI 根据上下文自动补全代码片段。</p><p>第二阶段：对话式编程。通过自然语言描述需求，AI 生成完整代码。</p><p>第三阶段：自主编程。AI 理解整个项目架构，独立完成复杂功能。</p><h2>对开发者的影响</h2><p>有人担心 AI 会取代程序员，但更准确的看法是：AI 正在改变程序员的工作方式。未来的程序员需要更多关注架构设计、需求理解和代码审查，而不是逐行编写代码。</p><blockquote><p>技术不会取代人类，但使用技术的人会取代不使用技术的人。</p></blockquote><h2>结论</h2><p>拥抱 AI 工具，提升自己的编程效率，是每个开发者现在就应该做的事情。</p></body></html>",
  cleanedHtml: "<h1>科技爱好者周刊（第 300 期）</h1><p>随着 AI 工具的普及，编程的方式正在发生根本性的变化。</p><h2>AI 编程的三个阶段</h2><p>第一阶段：代码补全。AI 根据上下文自动补全代码片段。</p><p>第二阶段：对话式编程。通过自然语言描述需求，AI 生成完整代码。</p><p>第三阶段：自主编程。AI 理解整个项目架构，独立完成复杂功能。</p>",
  cleanedMarkdown: "# 科技爱好者周刊（第 300 期）\n\n随着 AI 工具的普及，编程的方式正在发生根本性的变化。\n\n## AI 编程的三个阶段\n\n1. 第一阶段：代码补全\n2. 第二阶段：对话式编程\n3. 第三阶段：自主编程",
  renderedHtml: "<h1>科技爱好者周刊（第 300 期）</h1><p>随着 AI 工具的普及，编程的方式正在发生根本性的变化。</p><h2>AI 编程的三个阶段</h2><ol><li>第一阶段：代码补全</li><li>第二阶段：对话式编程</li><li>第三阶段：自主编程</li></ol>",
  readabilityVersion: 1,
  createdAt: "2026-07-14T00:00:00Z",
  updatedAt: null,
};

export const mockProviders: Provider[] = [
  { id: 1, name: "Ollama (本地)", baseUrl: "http://localhost:11434/v1", apiKey: "", defaultModel: "qwen2.5:7b", thinkingModel: "qwen2.5:7b", createdAt: "2026-07-01T00:00:00Z" },
  { id: 2, name: "DeepSeek", baseUrl: "https://api.deepseek.com/v1", apiKey: "sk-***", defaultModel: "deepseek-chat", thinkingModel: "deepseek-chat", createdAt: "2026-07-10T00:00:00Z" },
];

export const mockSummary: Summary = {
  id: 1,
  entryId: 101,
  content: "本文讨论了 AI 工具对编程方式的深远影响，将 AI 编程分为三个阶段：代码补全、对话式编程和自主编程。作者认为 AI 不会取代程序员，而是改变工作方式，建议开发者积极拥抱 AI 工具提升效率。",
  targetLanguage: "zh-CN",
  detailLevel: "standard",
  status: "succeeded",
  createdAt: "2026-07-14T00:00:00Z",
};

export const mockNote: Note = {
  id: 1,
  entryId: 101,
  content: "## 我的笔记\n\n这篇文章的观点很有启发性：\n\n- AI 编程三阶段的分析很清晰\n- 对开发者影响的判断很中肯\n- 需要补充的是：AI 编程在安全性和代码质量方面的挑战\n\n下一步可以深入研究 Cursor 和 Copilot 的实际使用对比。",
  createdAt: "2026-07-14T00:00:00Z",
  updatedAt: "2026-07-14T00:00:00Z",
};

// ============ Mock API 实现 ============

export const mockApi = {
  listFeeds: (): FeedSummary[] => mockFeedSummaries,

  listEntries: (feedId: number): EntryListItem[] => {
    return mockEntries[feedId] || [];
  },

  getEntry: (id: number): Entry | undefined => {
    return mockEntryDetail[id];
  },

  getEntryContent: (_entryId: number): Content => mockContent,

  getSummary: (_entryId: number): Summary | null => mockSummary,

  getNote: (_entryId: number): Note | null => mockNote,

  searchEntries: (query: string): EntryListItem[] => {
    const allEntries = Object.values(mockEntries).flat();
    return allEntries.filter(
      (e) =>
        e.title.toLowerCase().includes(query.toLowerCase())
    );
  },

  listProviders: (): Provider[] => mockProviders,
};