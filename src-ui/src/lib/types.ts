export interface Feed {
  id: number;
  url: string;
  title: string;
  description: string;
  link: string;
  feedType: string;
  lastSyncedAt: string | null;
  createdAt: string;
}

export interface FeedSummary {
  id: number;
  title: string;
  unreadCount: number;
}

export interface Entry {
  id: number;
  feedId: number;
  guid: string;
  title: string;
  author: string;
  link: string;
  summary: string;
  publishedAt: string | null;
  updatedAt: string | null;
  isRead: boolean;
  isStarred: boolean;
  createdAt: string;
}

export interface EntryListItem {
  id: number;
  feedId: number;
  title: string;
  author: string;
  summary: string;
  publishedAt: string | null;
  isRead: boolean;
}

export interface EntryPage {
  entries: EntryListItem[];
  total: number;
  page: number;
  pageSize: number;
}

export interface Content {
  id: number;
  entryId: number;
  rawHtml: string;
  cleanedHtml: string | null;
  cleanedMarkdown: string | null;
  renderedHtml: string | null;
  readabilityVersion: number;
  createdAt: string;
  updatedAt: string | null;
}

export interface ImportResult {
  xmlUrl: string;
  title: string;
  success: boolean;
  message: string;
}

export interface Provider {
  id: number;
  name: string;
  baseUrl: string;
  apiKey: string;
  defaultModel: string;
  thinkingModel: string;
  createdAt: string;
}

export interface Summary {
  id: number;
  entryId: number;
  content: string;
  targetLanguage: string;
  detailLevel: string;
  status: string;
  createdAt: string;
}

export interface Translation {
  id: number;
  entryId: number;
  segmentIndex: number;
  sourceText: string;
  translatedText: string;
  status: string;
}

export interface Note {
  id: number;
  entryId: number;
  content: string;
  createdAt: string;
  updatedAt: string;
}

export type ViewMode = "list" | "reader" | "settings";
export type Theme = "light" | "dark";

export interface AgentConfig {
  targetLanguage: string;
  detailLevel: "brief" | "standard" | "detailed";
  concurrencyDegree: number;
  primaryModelId: string;
}

export interface DigestTemplate {
  id: number;
  name: string;
  description: string;
  body: string;
  format: string;
  isDefault: boolean;
  created_at: string;
  updated_at: string;
}
