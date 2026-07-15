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

/// Refresh all feeds (not implemented yet — placeholder).
pub fn refresh_all_feeds(_pool: &DbPool) -> Result<(), String> {
    // TODO: concurrent refresh with Semaphore(5)
    Err("refresh_all_feeds not yet implemented".into())
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

// ============================================================
// OPML
// ============================================================

/// Import feeds from an OPML file.
pub fn import_opml(pool: &DbPool, file_path: &str) -> Result<Vec<crate::db::model::Feed>, String> {
    let outlines =
        crate::feed::opml::parse_opml_file(std::path::Path::new(file_path)).map_err(|e| e.to_string())?;

    let mut feeds = Vec::new();
    let service = crate::feed::service::FeedService::new(pool.clone());
    for outline in &outlines {
        match service.add_feed(&outline.xml_url) {
            Ok(feed) => feeds.push(feed),
            Err(e) => tracing::warn!("OPML import skipped {}: {}", outline.xml_url, e),
        }
    }
    Ok(feeds)
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
