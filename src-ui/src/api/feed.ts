import type { Feed, Entry, Content, EntryPage, Provider, Summary, Note, FeedSummary } from "@/lib/types";

// 检查是否在 Tauri 环境中
export function isTauri(): boolean {
  return typeof window !== "undefined" && ("__TAURI__" in window || "__TAURI_INTERNALS__" in window);
}

// 动态导入 Tauri invoke，避免在非 Tauri 环境报错
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke<T>(cmd, args);
  }
  throw new Error("Not in Tauri environment");
}

// ============ Feed API ============

export async function addFeed(url: string): Promise<Feed> {
  return invoke<Feed>("add_feed", { url });
}

export async function removeFeed(id: number): Promise<void> {
  return invoke("remove_feed", { id });
}

export async function refreshFeed(id: number): Promise<void> {
  return invoke("refresh_feed", { id });
}

export async function refreshAllFeeds(): Promise<void> {
  return invoke("refresh_all_feeds");
}

export async function listFeeds(): Promise<FeedSummary[]> {
  return invoke<FeedSummary[]>("list_feeds");
}

export async function importOpml(filePath: string): Promise<void> {
  return invoke("import_opml", { filePath });
}

export async function exportOpml(filePath: string): Promise<void> {
  return invoke("export_opml", { filePath });
}

// ============ Entry API ============

export async function listEntries(
  feedId: number,
  page: number = 1,
  pageSize: number = 20,
  filter?: string
): Promise<EntryPage> {
  return invoke<EntryPage>("list_entries", {
    feedId,
    page,
    pageSize,
    filter,
  });
}

export async function getEntry(id: number): Promise<Entry> {
  return invoke<Entry>("get_entry", { id });
}

export async function getEntryContent(entryId: number): Promise<Content> {
  return invoke<Content>("get_entry_content", { entryId });
}

export async function markRead(id: number): Promise<void> {
  return invoke("mark_read", { id });
}

export async function markUnread(id: number): Promise<void> {
  return invoke("mark_unread", { id });
}

export async function searchEntries(
  query: string,
  page: number = 1,
  pageSize: number = 20
): Promise<EntryPage> {
  return invoke<EntryPage>("search_entries", { query, page, pageSize });
}

// ============ Provider API ============

export async function addProvider(provider: Omit<Provider, "id" | "created_at">): Promise<Provider> {
  return invoke<Provider>("add_provider", { provider });
}

export async function listProviders(): Promise<Provider[]> {
  return invoke<Provider[]>("list_providers");
}

export async function updateProvider(provider: Provider): Promise<void> {
  return invoke("update_provider", { provider });
}

export async function deleteProvider(id: number): Promise<void> {
  return invoke("delete_provider", { id });
}

export async function validateProvider(baseUrl: string, apiKey: string): Promise<boolean> {
  return invoke<boolean>("validate_provider", { baseUrl, apiKey });
}

// ============ Summary API ============

export async function generateSummary(entryId: number): Promise<void> {
  return invoke("generate_summary", { entryId });
}

export async function getSummary(entryId: number): Promise<Summary | null> {
  return invoke<Summary | null>("get_summary", { entryId });
}

export async function cancelSummary(entryId: number): Promise<void> {
  return invoke("cancel_summary", { entryId });
}

// ============ Translation API ============

export async function translateEntry(entryId: number): Promise<void> {
  return invoke("translate_entry", { entryId });
}

export async function retryFailedSegments(entryId: number): Promise<void> {
  return invoke("retry_failed_segments", { entryId });
}

export async function clearTranslation(entryId: number): Promise<void> {
  return invoke("clear_translation", { entryId });
}

// ============ Note API ============

export async function saveNote(entryId: number, content: string): Promise<void> {
  return invoke("save_note", { entryId, content });
}

export async function getNote(entryId: number): Promise<Note | null> {
  return invoke<Note | null>("get_note", { entryId });
}

export async function deleteNote(id: number): Promise<void> {
  return invoke("delete_note", { id });
}

// ============ Settings API ============

export async function getTheme(): Promise<string> {
  return invoke<string>("get_theme");
}

export async function setTheme(theme: string): Promise<void> {
  return invoke("set_theme", { theme });
}

export async function getSetting(key: string): Promise<string | null> {
  return invoke<string | null>("get_setting", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}

// ============ Digest API (Stage 4) ============

export async function exportSingleDigest(entryId: number, format: string = "markdown"): Promise<string> {
  return invoke<string>("export_single_digest", { entryId, format });
}

export async function exportMultiDigest(entryIds: number[], format: string = "markdown"): Promise<string> {
  return invoke<string>("export_multi_digest", { entryIds, format });
}
