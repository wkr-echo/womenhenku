export interface Feed {
  id: number;
  url: string;
  title: string;
  description: string;
  link: string;
  feed_type: string;
  last_synced_at: string | null;
  created_at: string;
  unread_count?: number;
}

export interface Entry {
  id: number;
  feed_id: number;
  guid: string;
  title: string;
  author: string;
  link: string;
  summary: string;
  published_at: string | null;
  updated_at: string | null;
  is_read: number;
  is_starred: number;
  created_at: string;
}

export interface Content {
  id: number;
  entry_id: number;
  raw_html: string;
  cleaned_html: string;
  cleaned_markdown: string;
  rendered_html: string;
  readability_version: number;
  created_at: string;
  updated_at: string | null;
}

export interface Provider {
  id: number;
  name: string;
  base_url: string;
  api_key: string;
  default_model: string;
  thinking_model: string;
  created_at: string;
}

export interface Summary {
  id: number;
  entry_id: number;
  content: string;
  target_language: string;
  detail_level: string;
  status: string;
  created_at: string;
}

export interface Translation {
  id: number;
  entry_id: number;
  segment_index: number;
  source_text: string;
  translated_text: string;
  status: string;
}

export interface Note {
  id: number;
  entry_id: number;
  content: string;
  created_at: string;
  updated_at: string;
}

export type ViewMode = "list" | "reader" | "settings";
export type Theme = "light" | "dark";

export interface PagedResult<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface AgentConfig {
  targetLanguage: string;
  detailLevel: "brief" | "standard" | "detailed";
  concurrencyDegree: number;
  primaryModelId: string;
}