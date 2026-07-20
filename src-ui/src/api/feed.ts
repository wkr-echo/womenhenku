import type { Feed, Entry, Content, EntryPage, Provider, Summary, Note, FeedSummary, ImportResult, Tag } from "@/lib/types";

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

export async function refreshAllFeeds(): Promise<number> {
  return invoke<number>("refresh_all_feeds");
}

export async function listFeeds(): Promise<FeedSummary[]> {
  return invoke<FeedSummary[]>("list_feeds");
}

export async function importOpml(filePath: string): Promise<ImportResult[]> {
  return invoke<ImportResult[]>("import_opml", { filePath });
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

export async function processEntryContent(entryId: number, url: string): Promise<Content> {
  return invoke<Content>("process_entry_content", { entryId, url });
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

// ============ Summary API (deprecated — use api/provider.ts) ============

export async function getSummary(entryId: number): Promise<string | null> {
  return invoke<string | null>("get_summary", { entryId });
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

export async function listSystemFonts(): Promise<string[]> {
  return invoke<string[]>("list_system_fonts");
}

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

/// Write text content to a file path on disk.
export async function writeTextFile(path: string, content: string): Promise<void> {
  return invoke("write_text_file", { path, content });
}

// ============ Tags API (Stage 5) ============

export async function addTag(name: string, color: string = "#3b82f6"): Promise<Tag> {
  return invoke<Tag>("add_tag", { name, color });
}

export async function listTags(): Promise<Tag[]> {
  return invoke<Tag[]>("list_tags");
}

export async function getTag(id: number): Promise<Tag> {
  return invoke<Tag>("get_tag", { id });
}

export async function updateTag(id: number, name: string, color: string): Promise<Tag> {
  return invoke<Tag>("update_tag", { id, name, color });
}

export async function deleteTag(id: number): Promise<void> {
  return invoke("delete_tag", { id });
}

export async function tagEntry(entryId: number, tagId: number): Promise<void> {
  return invoke("tag_entry", { entryId, tagId });
}

export async function untagEntry(entryId: number, tagId: number): Promise<void> {
  return invoke("untag_entry", { entryId, tagId });
}

export async function getEntryTags(entryId: number): Promise<Tag[]> {
  return invoke<Tag[]>("get_entry_tags", { entryId });
}

export async function getTagsWithCount(): Promise<[Tag, number][]> {
  return invoke<[Tag, number][]>("get_tags_with_count");
}

export async function getTagStats(tagId: number): Promise<{ entryCount: number }> {
  return invoke<{ entryCount: number }>("get_tag_stats", { tagId });
}
