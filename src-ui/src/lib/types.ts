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
  apiKeyRef: string;
  isDefault: boolean;
  createdAt: string;
  updatedAt: string | null;
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
  summaryLanguage: string;
  summaryDetail: "brief" | "standard" | "detailed";
  translationLanguage: string;
  concurrencyDegree: number;
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

export interface Tag {
  id: number;
  name: string;
  normalizedName: string;
  color: string;
  isProvisional: boolean;
  usageCount: number;
  createdAt: string;
}

export interface TagWithCount {
  tag: Tag;
  count: number;
}

export interface TagAlias {
  id: number;
  tagId: number;
  alias: string;
  normalizedAlias: string;
  createdAt: string;
}

export interface DuplicateTagPair {
  tagA: Tag;
  tagB: Tag;
  reason: string;
}

export interface TagRecommendation {
  id: number;
  entryId: number;
  tagName: string;
  sourceType: string;
  confidence: number;
  createdAt: string;
}

export interface LlmUsageEvent {
  id: number;
  providerId: number;
  providerName: string;
  providerBaseUrl: string;
  providerHost: string;
  modelId: number;
  modelName: string;
  agentType: string;
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
  requestStatus: string;
  timestamp: string;
  createdAt: string;
}

export interface LlmUsageStats {
  totalTokens: number;
  promptTokens: number;
  completionTokens: number;
  requestCount: number;
  successRate: number;
  avgTokensPerRequest: number;
}

export interface DailyUsage {
  date: string;
  totalTokens: number;
  promptTokens: number;
  completionTokens: number;
  requestCount: number;
}

export interface ProviderUsage {
  providerId: number;
  providerName: string;
  totalTokens: number;
  requestCount: number;
}

export interface ModelUsage {
  modelId: number;
  modelName: string;
  totalTokens: number;
  requestCount: number;
}

export interface AgentUsage {
  agentType: string;
  totalTokens: number;
  requestCount: number;
}

export interface Setting {
  id: number;
  key: string;
  value: string;
  createdAt: string;
  updatedAt: string;
}
