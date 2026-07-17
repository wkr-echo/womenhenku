pub mod commands;
pub mod db;
pub mod digest;
pub mod feed;
pub mod notes;
pub mod platform;
pub mod reader;

#[cfg(feature = "tauri-runtime")]
use std::str::FromStr;
#[cfg(feature = "tauri-runtime")]
use tauri::Emitter;

// ============================================================
// Non-Tauri entry point (for cargo test / standalone binary)
// ============================================================

#[cfg(not(feature = "tauri-runtime"))]
pub fn run() {
    tracing_subscriber::fmt::init();
    tracing::info!("Womenhenku starting (standalone mode)...");

    let db_path = db::default_db_path();
    match db::initialize_database(&db_path) {
        Ok(_pool) => tracing::info!("Database initialized at: {}", db_path.display()),
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    }

    tracing::info!("Womenhenku shutdown complete.");
}

// ============================================================
// Tauri runtime (--features tauri-runtime)
// ============================================================

#[cfg(feature = "tauri-runtime")]
pub fn run() {
    tracing_subscriber::fmt::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = db::default_db_path();
            let pool = db::initialize_database(&db_path)
                .expect("Failed to initialize database");

            // Spawn background auto-sync task (every 30 minutes)
            let sync_pool = pool.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(
                    std::time::Duration::from_secs(30 * 60)
                );
                interval.tick().await; // skip first immediate tick
                loop {
                    interval.tick().await;
                    tracing::info!("Auto-sync: refreshing all feeds...");
                    let svc = crate::feed::service::FeedService::new(sync_pool.clone());
                    if let Err(e) = svc.refresh_all_feeds() {
                        tracing::warn!("Auto-sync failed: {}", e);
                    }
                }
            });

            app.manage(pool);
            tracing::info!("Tauri + database initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Feed management
            list_feeds,
            get_feed,
            add_feed,
            remove_feed,
            refresh_feed,
            refresh_all_feeds,
            // Entry queries
            list_entries,
            list_all_entries,
            get_entry,
            mark_read,
            mark_unread,
            mark_all_read,
            // Content
            get_entry_content,
            process_entry_content,
            // OPML
            import_opml,
            export_opml,
            // Search (Stage 2)
            search_entries,
            // Notes (Stage 4)
            save_note,
            get_note,
            delete_note,
            // Digest export (Stage 4)
            export_single_digest,
            export_multi_digest,
            // Fonts (Stage 2)
            list_system_fonts,
            // System
            open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ============================================================
// Tauri Command wrappers
// ============================================================

#[cfg(feature = "tauri-runtime")]
use tauri::Manager;
#[cfg(feature = "tauri-runtime")]
use tauri::State;
#[cfg(feature = "tauri-runtime")]
use crate::db::DbPool;

// -- Feed management --

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn list_feeds(state: State<'_, DbPool>) -> Result<Vec<crate::db::model::FeedSummary>, String> {
    commands::list_feeds(&state)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn get_feed(state: State<'_, DbPool>, id: i64) -> Result<crate::db::model::Feed, String> {
    commands::get_feed(&state, id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn add_feed(state: State<'_, DbPool>, url: String) -> Result<crate::db::model::Feed, String> {
    commands::add_feed(&state, &url)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn remove_feed(state: State<'_, DbPool>, id: i64) -> Result<(), String> {
    commands::remove_feed(&state, id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn refresh_feed(state: State<'_, DbPool>, id: i64) -> Result<usize, String> {
    commands::refresh_feed(&state, id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn refresh_all_feeds(state: State<'_, DbPool>) -> Result<(), String> {
    commands::refresh_all_feeds(&state)
}

// -- Entry queries --

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn list_entries(
    state: State<'_, DbPool>,
    feed_id: i64,
    page: i32,
    page_size: i32,
    filter: Option<String>,
) -> Result<crate::db::model::EntryPage, String> {
    commands::list_entries(&state, feed_id, page, page_size, filter.as_deref())
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn list_all_entries(
    state: State<'_, DbPool>,
    page: i32,
    page_size: i32,
    filter: Option<String>,
) -> Result<crate::db::model::EntryPage, String> {
    commands::list_all_entries(&state, page, page_size, filter.as_deref())
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn get_entry(state: State<'_, DbPool>, id: i64) -> Result<crate::db::model::Entry, String> {
    commands::get_entry(&state, id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn mark_read(state: State<'_, DbPool>, id: i64) -> Result<(), String> {
    commands::mark_read(&state, id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn mark_unread(state: State<'_, DbPool>, id: i64) -> Result<(), String> {
    commands::mark_unread(&state, id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn mark_all_read(state: State<'_, DbPool>, feed_id: i64) -> Result<(), String> {
    commands::mark_all_read(&state, feed_id)
}

// -- Content --

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn get_entry_content(state: State<'_, DbPool>, entry_id: i64) -> Result<crate::db::model::Content, String> {
    commands::get_entry_content(&state, entry_id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn process_entry_content(state: State<'_, DbPool>, entry_id: i64, url: String) -> Result<crate::db::model::Content, String> {
    commands::process_entry_content(&state, entry_id, &url)
}

// -- OPML --

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn import_opml(
    app: tauri::AppHandle,
    state: State<'_, DbPool>,
    file_path: String,
) -> Result<Vec<crate::feed::opml::ImportResult>, String> {
    let outlines = crate::feed::opml::parse_opml_file(std::path::Path::new(&file_path))
        .map_err(|e| e.to_string())?;
    let pool = state.inner().clone();
    let result = tokio::task::spawn_blocking(move || {
        crate::feed::opml::import_feeds(&pool, &outlines, &|r| {
            let _ = app.emit("opml-import-progress", r.clone());
        })
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(result)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn export_opml(state: State<'_, DbPool>, file_path: String) -> Result<(), String> {
    let pool = state.inner().clone();
    tokio::task::spawn_blocking(move || commands::export_opml(&pool, &file_path))
        .await
        .map_err(|e| e.to_string())?
}

// -- System fonts (Stage 2) --

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn list_system_fonts() -> Result<Vec<String>, String> {
    Ok(crate::platform::font::list_fonts())
}

// -- Open URL in system browser --

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    webbrowser::open(&url).map_err(|e| e.to_string())
}

// -- Search --

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn search_entries(
    state: State<'_, DbPool>,
    query: String,
    page: i32,
    page_size: i32,
) -> Result<crate::db::model::EntryPage, String> {
    commands::search_entries(&state, &query, page, page_size)
}

// -- Notes (Stage 4) --

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn save_note(state: State<'_, DbPool>, entry_id: i64, content: String) -> Result<crate::db::model::Note, String> {
    commands::save_note(&state, entry_id, &content)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn get_note(state: State<'_, DbPool>, entry_id: i64) -> Result<Option<crate::db::model::Note>, String> {
    commands::get_note(&state, entry_id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn delete_note(state: State<'_, DbPool>, entry_id: i64) -> Result<(), String> {
    commands::delete_note(&state, entry_id)
}

// -- Digest export (Stage 4) --

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn export_single_digest(state: State<'_, DbPool>, entry_id: i64, format: String) -> Result<String, String> {
    let fmt = crate::digest::DigestFormat::from_str(&format)?;
    commands::export_single_digest(&state, entry_id, &fmt)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn export_multi_digest(state: State<'_, DbPool>, entry_ids: Vec<i64>, format: String) -> Result<String, String> {
    let fmt = crate::digest::DigestFormat::from_str(&format)?;
    commands::export_multi_digest(&state, &entry_ids, &fmt)
}
