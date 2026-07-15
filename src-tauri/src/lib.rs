pub mod commands;
pub mod db;
pub mod feed;
pub mod reader;

// ============================================================
// Tauri Command registration (uncomment when Tauri is connected)
// ============================================================
//
// use tauri::Manager;
//
// pub fn run_tauri() {
//     tauri::Builder::default()
//         .setup(|app| {
//             let db_path = db::default_db_path();
//             let pool = db::initialize_database(&db_path)
//                 .expect("Failed to initialize database");
//             app.manage(pool);
//             Ok(())
//         })
//         .invoke_handler(tauri::generate_handler![
//             // Feed management
//             commands::list_feeds_tauri,
//             commands::add_feed_tauri,
//             commands::remove_feed_tauri,
//             commands::refresh_feed_tauri,
//             // Entry queries
//             commands::list_entries_tauri,
//             commands::list_all_entries_tauri,
//             commands::get_entry_tauri,
//             commands::mark_read_tauri,
//             commands::mark_unread_tauri,
//             commands::mark_all_read_tauri,
//             // Content
//             commands::get_entry_content_tauri,
//             // OPML
//             commands::import_opml_tauri,
//             commands::export_opml_tauri,
//             // Search (Stage 2)
//             commands::search_entries_tauri,
//         ])
//         .run(tauri::generate_context!())
//         .expect("error while running tauri application");
// }
