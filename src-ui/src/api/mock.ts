import type { Feed, Entry, Content, Provider, Note, Summary } from "@/lib/types";

// ============ Mock Data ============

export const mockFeeds: Feed[] = [
  {
    id: 1,
    url: "https://ruanyifeng.com/blog/atom.xml",
    title: "阮一峰的网络日志",
    description: "科技、人文、思考",
    link: "https://ruanyifeng.com",
    feed_type: "atom",
    last_synced_at: new Date().toISOString(),
    created_at: "2026-07-01T00:00:00Z",
    unread_count: 12,
  },
  {
    id: 2,
    url: "https://sspai.com/feed",
    title: "少数派",
    description: "数字生活指南",
    link: "https://sspai.com",
    feed_type: "rss",
    last_synced_at: new Date().toISOString(),
    created_at: "2026-07-02T00:00:00Z",
    unread_count: 5,
  },
  {
    id: 3,
    url: "https://hackertalk.net/feed",
    title: "Hacker Talk",
    description: "科技前沿讨论",
    link: "https://hackertalk.net",
    feed_type: "rss",
    last_synced_at: new Date().toISOString(),
    created_at: "2026-07-03T00:00:00Z",
    unread_count: 0,
  },
];

export const mockEntries: Record<number, Entry[]> = {
  1: [
    { id: 101, feed_id: 1, guid: "g1", title: "科技爱好者周刊（第 300 期）：AI 时代的编程新范式", author: "阮一峰", link: "https://example.com/1", summary: "随着 AI 工具的普及，编程的方式正在发生根本性的变化...", published_at: new Date(Date.now() - 3600000).toISOString(), updated_at: null, is_read: 0, is_starred: 0, created_at: "2026-07-14T00:00:00Z" },
    { id: 102, feed_id: 1, guid: "g2", title: "WebAssembly 入门指南", author: "阮一峰", link: "https://example.com/2", summary: "WebAssembly 是一种新的二进制格式，可以在浏览器中运行...", published_at: new Date(Date.now() - 7200000).toISOString(), updated_at: null, is_read: 0, is_starred: 0, created_at: "2026-07-13T00:00:00Z" },
    { id: 103, feed_id: 1, guid: "g3", title: "Rust 异步编程深入理解", author: "阮一峰", link: "https://example.com/3", summary: "异步编程是 Rust 的核心特性之一，本文深入探讨...", published_at: new Date(Date.now() - 86400000).toISOString(), updated_at: null, is_read: 1, is_starred: 1, created_at: "2026-07-12T00:00:00Z" },
    { id: 104, feed_id: 1, guid: "g4", title: "CSS Container Queries 实战", author: "阮一峰", link: "https://example.com/4", summary: "Container Queries 终于在所有主流浏览器中得到支持...", published_at: new Date(Date.now() - 172800000).toISOString(), updated_at: null, is_read: 1, is_starred: 0, created_at: "2026-07-11T00:00:00Z" },
    { id: 105, feed_id: 1, guid: "g5", title: "每周科技新闻汇总（7月第一周）", author: "阮一峰", link: "https://example.com/5", summary: "本周科技界重要新闻：AI 芯片新突破...", published_at: new Date(Date.now() - 259200000).toISOString(), updated_at: null, is_read: 1, is_starred: 0, created_at: "2026-07-10T00:00:00Z" },
  ],
  2: [
    { id: 201, feed_id: 2, guid: "sg1", title: "如何打造高效的个人工作流", author: "少数派", link: "https://example.com/s1", summary: "一个高效的工作流能让你事半功倍...", published_at: new Date(Date.now() - 1800000).toISOString(), updated_at: null, is_read: 0, is_starred: 0, created_at: "2026-07-14T00:00:00Z" },
    { id: 202, feed_id: 2, guid: "sg2", title: "macOS 15 新功能全面解析", author: "少数派", link: "https://example.com/s2", summary: "macOS 15 带来了许多令人兴奋的新功能...", published_at: new Date(Date.now() - 43200000).toISOString(), updated_at: null, is_read: 0, is_starred: 0, created_at: "2026-07-13T00:00:00Z" },
  ],
  3: [],
};

export const mockContent: Content = {
  id: 1,
  entry_id: 101,
  raw_html: "<html><body><h1>科技爱好者周刊（第 300 期）</h1><p>随着 AI 工具的普及，编程的方式正在发生根本性的变化。从 Copilot 到 Cursor，从 ChatGPT 到 Claude，AI 辅助编程已经成为主流。</p><h2>AI 编程的三个阶段</h2><p>第一阶段：代码补全。AI 根据上下文自动补全代码片段。</p><p>第二阶段：对话式编程。通过自然语言描述需求，AI 生成完整代码。</p><p>第三阶段：自主编程。AI 理解整个项目架构，独立完成复杂功能。</p><h2>对开发者的影响</h2><p>有人担心 AI 会取代程序员，但更准确的看法是：AI 正在改变程序员的工作方式。未来的程序员需要更多关注架构设计、需求理解和代码审查，而不是逐行编写代码。</p><blockquote><p>技术不会取代人类，但使用技术的人会取代不使用技术的人。</p></blockquote><h2>结论</h2><p>拥抱 AI 工具，提升自己的编程效率，是每个开发者现在就应该做的事情。</p></body></html>",
  cleaned_html: "<h1>科技爱好者周刊（第 300 期）</h1><p>随着 AI 工具的普及，编程的方式正在发生根本性的变化。</p><h2>AI 编程的三个阶段</h2><p>第一阶段：代码补全。AI 根据上下文自动补全代码片段。</p><p>第二阶段：对话式编程。通过自然语言描述需求，AI 生成完整代码。</p><p>第三阶段：自主编程。AI 理解整个项目架构，独立完成复杂功能。</p>",
  cleaned_markdown: "# 科技爱好者周刊（第 300 期）\n\n随着 AI 工具的普及，编程的方式正在发生根本性的变化。\n\n## AI 编程的三个阶段\n\n1. 第一阶段：代码补全\n2. 第二阶段：对话式编程\n3. 第三阶段：自主编程",
  rendered_html: "<h1>科技爱好者周刊（第 300 期）</h1><p>随着 AI 工具的普及，编程的方式正在发生根本性的变化。</p><h2>AI 编程的三个阶段</h2><ol><li>第一阶段：代码补全</li><li>第二阶段：对话式编程</li><li>第三阶段：自主编程</li></ol>",
  readability_version: 1,
  created_at: "2026-07-14T00:00:00Z",
  updated_at: null,
};

export const mockProviders: Provider[] = [
  { id: 1, name: "Ollama (本地)", base_url: "http://localhost:11434/v1", api_key: "", default_model: "qwen2.5:7b", thinking_model: "qwen2.5:7b", created_at: "2026-07-01T00:00:00Z" },
  { id: 2, name: "DeepSeek", base_url: "https://api.deepseek.com/v1", api_key: "sk-***", default_model: "deepseek-chat", thinking_model: "deepseek-chat", created_at: "2026-07-10T00:00:00Z" },
];

export const mockSummary: Summary = {
  id: 1,
  entry_id: 101,
  content: "本文讨论了 AI 工具对编程方式的深远影响，将 AI 编程分为三个阶段：代码补全、对话式编程和自主编程。作者认为 AI 不会取代程序员，而是改变工作方式，建议开发者积极拥抱 AI 工具提升效率。",
  target_language: "zh-CN",
  detail_level: "standard",
  status: "succeeded",
  created_at: "2026-07-14T00:00:00Z",
};

export const mockNote: Note = {
  id: 1,
  entry_id: 101,
  content: "## 我的笔记\n\n这篇文章的观点很有启发性：\n\n- AI 编程三阶段的分析很清晰\n- 对开发者影响的判断很中肯\n- 需要补充的是：AI 编程在安全性和代码质量方面的挑战\n\n下一步可以深入研究 Cursor 和 Copilot 的实际使用对比。",
  created_at: "2026-07-14T00:00:00Z",
  updated_at: "2026-07-14T00:00:00Z",
};

// ============ Mock API 实现 ============

export const mockApi = {
  listFeeds: (): Feed[] => mockFeeds,

  listEntries: (feedId: number): Entry[] => {
    return mockEntries[feedId] || [];
  },

  getEntryContent: (_entryId: number): Content => mockContent,

  getSummary: (_entryId: number): Summary | null => mockSummary,

  getNote: (_entryId: number): Note | null => mockNote,

  searchEntries: (query: string): Entry[] => {
    const allEntries = Object.values(mockEntries).flat();
    return allEntries.filter(
      (e) =>
        e.title.toLowerCase().includes(query.toLowerCase()) ||
        e.summary.toLowerCase().includes(query.toLowerCase())
    );
  },

  listProviders: (): Provider[] => mockProviders,
};