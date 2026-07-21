// Tauri Command implementations — Stage 1
//
// Each function below implements a command-contract.md Tauri Command.
// When Tauri is connected, wrap with #[tauri::command] and register in lib.rs.
//
// Registration code (uncomment when Tauri available):
//
//   #[tauri::command]
//   fn list_feeds(state: tauri::State<'_, DbPool>) -> Result<Vec<FeedSummary>, String> {
//       commands::list_feeds(&state).map_err(|e| e.to_string())
//   }
//
//   // In main.rs or lib.rs setup:
//   app.manage(pool);

use crate::db::model::{Content, EntryPage, FeedSummary};
use crate::db::repository::{ContentRepository, EntryRepository, FeedRepository};
use crate::db::DbPool;

// ============================================================
// Feed management
// ============================================================

/// List all feeds with unread counts. Used by Sidebar.
pub fn list_feeds(pool: &DbPool) -> Result<Vec<FeedSummary>, String> {
    let repo = FeedRepository::new(pool.clone());
    repo.find_all_with_unread_count().map_err(|e| e.to_string())
}

/// Get a single feed by id.
pub fn get_feed(pool: &DbPool, id: i64) -> Result<crate::db::model::Feed, String> {
    let repo = FeedRepository::new(pool.clone());
    let feed = repo.find_by_id(id).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Feed id={} not found", id))?;
    Ok(feed)
}

/// Add a new feed subscription. Fetches URL, parses, stores entries.
/// Note: requires FeedService (async HTTP), registered separately.
pub fn add_feed(pool: &DbPool, url: &str) -> Result<crate::db::model::Feed, String> {
    let service = crate::feed::service::FeedService::new(pool.clone());
    service.add_feed(url).map_err(|e| e.to_string())
}

/// Remove a feed and all its entries (cascade).
pub fn remove_feed(pool: &DbPool, id: i64) -> Result<(), String> {
    let service = crate::feed::service::FeedService::new(pool.clone());
    service.remove_feed(id).map_err(|e| e.to_string())
}

/// Refresh a single feed: re-fetch, insert new entries.
pub fn refresh_feed(pool: &DbPool, id: i64) -> Result<usize, String> {
    let service = crate::feed::service::FeedService::new(pool.clone());
    service.refresh_feed(id).map_err(|e| e.to_string())
}

/// Refresh all feeds concurrently with max 5 concurrent fetches.
/// Returns total number of new entries found.
pub fn refresh_all_feeds(pool: &DbPool) -> Result<usize, String> {
    let service = crate::feed::service::FeedService::new(pool.clone());
    service.refresh_all_feeds().map_err(|e| e.to_string())
}

// ============================================================
// Entry queries
// ============================================================

/// List entries for a specific feed with pagination. Used by EntryList.
pub fn list_entries(
    pool: &DbPool,
    feed_id: i64,
    page: i32,
    page_size: i32,
    filter: Option<&str>,
) -> Result<EntryPage, String> {
    let repo = EntryRepository::new(pool.clone());
    repo.list_by_feed(feed_id, page, page_size, filter)
        .map_err(|e| e.to_string())
}

/// List entries across all feeds.
pub fn list_all_entries(
    pool: &DbPool,
    page: i32,
    page_size: i32,
    filter: Option<&str>,
) -> Result<EntryPage, String> {
    let repo = EntryRepository::new(pool.clone());
    repo.list_all(page, page_size, filter)
        .map_err(|e| e.to_string())
}

/// Get a single entry by id.
pub fn get_entry(pool: &DbPool, id: i64) -> Result<crate::db::model::Entry, String> {
    let repo = EntryRepository::new(pool.clone());
    let entry = repo.find_by_id(id).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Entry id={} not found", id))?;
    Ok(entry)
}

/// Mark an entry as read.
pub fn mark_read(pool: &DbPool, id: i64) -> Result<(), String> {
    let repo = EntryRepository::new(pool.clone());
    repo.mark_read(id).map_err(|e| e.to_string())
}

/// Mark an entry as unread.
pub fn mark_unread(pool: &DbPool, id: i64) -> Result<(), String> {
    let repo = EntryRepository::new(pool.clone());
    repo.mark_unread(id).map_err(|e| e.to_string())
}

/// Mark all entries in a feed as read.
pub fn mark_all_read(pool: &DbPool, feed_id: i64) -> Result<(), String> {
    let repo = EntryRepository::new(pool.clone());
    repo.mark_all_read_in_feed(feed_id)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

// ============================================================
// Content reading
// ============================================================

/// Get the content for an entry. Used by ReaderView.
pub fn get_entry_content(pool: &DbPool, entry_id: i64) -> Result<Content, String> {
    let repo = ContentRepository::new(pool.clone());
    let content = repo.find_by_entry_id(entry_id).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Content for entry_id={} not found", entry_id))?;
    Ok(content)
}

/// Run the reader pipeline on an entry and store the results. Stage 2.
pub fn process_entry_content(pool: &DbPool, entry_id: i64, url: &str) -> Result<Content, String> {
    let service = crate::reader::service::ReaderService::new(pool.clone());
    service.process_entry(entry_id, url).map_err(|e| e.to_string())
}

// ============================================================
// OPML
// ============================================================

/// Import feeds from an OPML file with batching, HTTPS filter, and auto-sync.
pub fn import_opml(pool: &DbPool, file_path: &str) -> Result<Vec<crate::feed::opml::ImportResult>, String> {
    let outlines =
        crate::feed::opml::parse_opml_file(std::path::Path::new(file_path)).map_err(|e| e.to_string())?;
    Ok(crate::feed::opml::import_feeds(pool, &outlines, &|_| {}))
}

/// Export all feeds to an OPML file.
pub fn export_opml(pool: &DbPool, file_path: &str) -> Result<(), String> {
    crate::feed::opml::export_opml_file(pool, std::path::Path::new(file_path))
        .map_err(|e| e.to_string())
}

// ============================================================
// Search (Stage 2)
// ============================================================

/// Full-text search on entries.
pub fn search_entries(
    pool: &DbPool,
    query: &str,
    page: i32,
    page_size: i32,
) -> Result<EntryPage, String> {
    let repo = EntryRepository::new(pool.clone());
    repo.search(query, page, page_size).map_err(|e| e.to_string())
}

// ============================================================
// Notes (Stage 4)
// ============================================================

/// Save or update a note for an entry.
pub fn save_note(pool: &DbPool, entry_id: i64, content: &str) -> Result<crate::db::model::Note, String> {
    crate::notes::save_note(pool, entry_id, content)
}

/// Get a note by entry_id.
pub fn get_note(pool: &DbPool, entry_id: i64) -> Result<Option<crate::db::model::Note>, String> {
    crate::notes::get_note(pool, entry_id)
}

/// Delete a note by entry_id.
pub fn delete_note(pool: &DbPool, entry_id: i64) -> Result<(), String> {
    crate::notes::delete_note(pool, entry_id)
}

// ============================================================
// Tags (Stage 5)
// ============================================================

pub fn add_tag(pool: &DbPool, name: &str, color: &str) -> Result<crate::db::model::Tag, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.insert(name, color).map_err(|e| e.to_string())
}

pub fn list_tags(pool: &DbPool) -> Result<Vec<crate::db::model::Tag>, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.find_all().map_err(|e| e.to_string())
}

pub fn get_tag(pool: &DbPool, id: i64) -> Result<crate::db::model::Tag, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.find_by_id(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Tag id={} not found", id))
}

pub fn update_tag(pool: &DbPool, id: i64, name: &str, color: &str) -> Result<crate::db::model::Tag, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.update(id, name, color).map_err(|e| e.to_string())
}

pub fn delete_tag(pool: &DbPool, id: i64) -> Result<(), String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.delete(id).map_err(|e| e.to_string())
}

pub fn tag_entry(pool: &DbPool, entry_id: i64, tag_id: i64) -> Result<(), String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.add_tag_to_entry(entry_id, tag_id, "manual", 0.0).map_err(|e| e.to_string())
}

pub fn untag_entry(pool: &DbPool, entry_id: i64, tag_id: i64) -> Result<(), String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.remove_tag_from_entry(entry_id, tag_id).map_err(|e| e.to_string())
}

pub fn get_entry_tags(pool: &DbPool, entry_id: i64) -> Result<Vec<crate::db::model::Tag>, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.find_tags_by_entry_id(entry_id).map_err(|e| e.to_string())
}

pub fn get_tags_with_count(pool: &DbPool) -> Result<Vec<(crate::db::model::Tag, i64)>, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.find_tags_with_entry_count().map_err(|e| e.to_string())
}

pub fn get_tag_stats(pool: &DbPool, tag_id: i64) -> Result<serde_json::Value, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    let count = repo.get_tag_entry_count(tag_id).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "entryCount": count }))
}

pub fn list_entries_by_tag(pool: &DbPool, tag_id: i64, page: i32, page_size: i32) -> Result<EntryPage, String> {
    let repo = crate::db::repository::EntryRepository::new(pool.clone());
    repo.list_by_tag(tag_id, page, page_size).map_err(|e| e.to_string())
}

// ============================================================
// Tags Enhancements (Stage 5)
// ============================================================

pub fn update_tag_status(pool: &DbPool, id: i64, is_provisional: bool) -> Result<crate::db::model::Tag, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.update_status(id, is_provisional).map_err(|e| e.to_string())
}

pub fn merge_tags(pool: &DbPool, source_id: i64, target_id: i64) -> Result<(), String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.merge_tags(source_id, target_id).map_err(|e| e.to_string())
}

pub fn add_tag_alias(pool: &DbPool, tag_id: i64, alias: &str) -> Result<crate::db::model::TagAlias, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.add_alias(tag_id, alias).map_err(|e| e.to_string())
}

pub fn remove_tag_alias(pool: &DbPool, tag_id: i64, alias: &str) -> Result<(), String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.remove_alias(tag_id, alias).map_err(|e| e.to_string())
}

pub fn get_tag_aliases(pool: &DbPool, tag_id: i64) -> Result<Vec<crate::db::model::TagAlias>, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.find_aliases_by_tag_id(tag_id).map_err(|e| e.to_string())
}

pub fn save_tag_recommendations(pool: &DbPool, entry_id: i64, recommendations: Vec<(String, String, f64)>) -> Result<(), String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.save_recommendations(entry_id, &recommendations).map_err(|e| e.to_string())
}

pub fn get_tag_recommendations(pool: &DbPool, entry_id: i64) -> Result<Vec<crate::db::model::TagRecommendation>, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.find_recommendations_by_entry_id(entry_id).map_err(|e| e.to_string())
}

pub fn tag_entries_batch(pool: &DbPool, entry_ids: Vec<i64>, tag_id: i64) -> Result<(), String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    for entry_id in entry_ids {
        repo.add_tag_to_entry(entry_id, tag_id, "batch", 0.0).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn find_potential_duplicates(pool: &DbPool) -> Result<Vec<(crate::db::model::Tag, crate::db::model::Tag, String)>, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.find_potential_duplicates().map_err(|e| e.to_string())
}

pub fn find_unused_tags(pool: &DbPool) -> Result<Vec<crate::db::model::Tag>, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.find_unused().map_err(|e| e.to_string())
}

pub fn delete_unused_tags(pool: &DbPool) -> Result<usize, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.delete_unused_tags().map_err(|e| e.to_string())
}

pub fn get_tag_by_name(pool: &DbPool, name: &str) -> Result<Option<crate::db::model::Tag>, String> {
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    repo.find_by_name(name).map_err(|e| e.to_string())
}

// ============================================================
// LLM Usage Stats (Stage 5)
// ============================================================

pub fn insert_llm_usage_event(pool: &DbPool, event: crate::db::model::LlmUsageEvent) -> Result<(), String> {
    let repo = crate::db::repository::LlmUsageRepository::new(pool.clone());
    repo.insert_event(&event).map_err(|e| e.to_string())
}

pub fn get_llm_usage_stats(pool: &DbPool, days: i64, agent_type: Option<String>) -> Result<crate::db::model::LlmUsageStats, String> {
    let repo = crate::db::repository::LlmUsageRepository::new(pool.clone());
    let agent_type_opt = agent_type.as_deref();
    repo.get_stats(days, agent_type_opt).map_err(|e| e.to_string())
}

pub fn get_llm_daily_usage(pool: &DbPool, days: i64, agent_type: Option<String>) -> Result<Vec<crate::db::model::DailyUsage>, String> {
    let repo = crate::db::repository::LlmUsageRepository::new(pool.clone());
    let agent_type_opt = agent_type.as_deref();
    repo.get_daily_usage(days, agent_type_opt).map_err(|e| e.to_string())
}

pub fn get_llm_provider_usage(pool: &DbPool, days: i64) -> Result<Vec<crate::db::model::ProviderUsage>, String> {
    let repo = crate::db::repository::LlmUsageRepository::new(pool.clone());
    repo.get_provider_usage(days).map_err(|e| e.to_string())
}

pub fn get_llm_model_usage(pool: &DbPool, days: i64) -> Result<Vec<crate::db::model::ModelUsage>, String> {
    let repo = crate::db::repository::LlmUsageRepository::new(pool.clone());
    repo.get_model_usage(days).map_err(|e| e.to_string())
}

pub fn get_llm_agent_usage(pool: &DbPool, days: i64) -> Result<Vec<crate::db::model::AgentUsage>, String> {
    let repo = crate::db::repository::LlmUsageRepository::new(pool.clone());
    repo.get_agent_usage(days).map_err(|e| e.to_string())
}

pub fn cleanup_old_llm_events(pool: &DbPool, retention_days: i64) -> Result<usize, String> {
    let repo = crate::db::repository::LlmUsageRepository::new(pool.clone());
    repo.cleanup_old_events(retention_days).map_err(|e| e.to_string())
}

// ============================================================
// Settings (Stage 5)
// ============================================================

pub fn get_setting(pool: &DbPool, key: &str) -> Result<Option<String>, String> {
    let repo = crate::db::repository::SettingsRepository::new(pool.clone());
    repo.get(key).map_err(|e| e.to_string())
}

pub fn set_setting(pool: &DbPool, key: &str, value: &str) -> Result<(), String> {
    let repo = crate::db::repository::SettingsRepository::new(pool.clone());
    repo.set(key, value).map_err(|e| e.to_string())
}

pub fn delete_setting(pool: &DbPool, key: &str) -> Result<(), String> {
    let repo = crate::db::repository::SettingsRepository::new(pool.clone());
    repo.delete(key).map_err(|e| e.to_string())
}

// ============================================================
// Digest export (Stage 4)
// ============================================================

/// Export a single entry as digest.
pub fn export_single_digest(
    pool: &DbPool,
    entry_id: i64,
    format: &crate::digest::DigestFormat,
) -> Result<String, String> {
    crate::digest::export_single(pool, entry_id, format)
}

/// Export multiple entries as digest.
pub fn export_multi_digest(
    pool: &DbPool,
    entry_ids: &[i64],
    format: &crate::digest::DigestFormat,
) -> Result<String, String> {
    crate::digest::export_multi(pool, entry_ids, format)
}

// ============================================================
// File utilities
// ============================================================

/// Write text content to a file path. Used by export functions.
pub fn write_text_file(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))
}
