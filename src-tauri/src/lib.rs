pub mod agent;
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
        .plugin(tauri_plugin_notification::init())
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

            // 初始化 Prompt 管理器
            let mut prompt_dirs = Vec::new();
            if let Ok(dir) = app.path().resource_dir() {
                prompt_dirs.push(dir.join("resources"));
            }
            prompt_dirs.push(std::path::PathBuf::from("resources"));
            if let Ok(cwd) = std::env::current_dir() {
                prompt_dirs.push(cwd.join("resources"));
            }

            let mut prompt_manager_opt = None;
            for dir in &prompt_dirs {
                if dir.join("prompts").exists() {
                    if let Ok(mgr) = agent::prompt::PromptManager::load(dir) {
                        prompt_manager_opt = Some(mgr);
                        tracing::info!("Loaded prompts from: {:?}", dir.join("prompts"));
                        break;
                    }
                }
            }

            let prompt_manager = std::sync::Arc::new(
                prompt_manager_opt.unwrap_or_else(|| {
                    tracing::warn!("No prompt files found, using built-in defaults");
                    agent::prompt::PromptManager::empty()
                }),
            );

            // 初始化 Agent Service
            let agent_service = std::sync::Arc::new(
                agent::service::AgentService::new(pool.clone(), prompt_manager.clone()),
            );

            app.manage(pool);
            app.manage(agent_service);
            app.manage(prompt_manager);
            tracing::info!("Tauri + database + agent initialized");
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
            // Provider management
            add_provider,
            list_providers,
            update_provider,
            delete_provider,
            add_provider_model,
            list_provider_models,
            delete_provider_model,
            validate_provider,
            // Agent (Stage 3)
            generate_summary,
            get_summary,
            cancel_summary,
            clear_summary,
            translate_entry,
            get_translation,
            cancel_translation,
            clear_translation,
            retry_failed_segments,
            // Settings
            get_setting,
            set_setting,
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
fn refresh_feed(app: tauri::AppHandle, state: State<'_, DbPool>, id: i64) -> Result<usize, String> {
    let new_count = commands::refresh_feed(&state, id)?;
    if new_count > 0 {
        use tauri_plugin_notification::NotificationExt;
        let feed_name = commands::get_feed(&state, id)
            .map(|f| f.title)
            .unwrap_or_else(|_| "?".to_string());
        let _ = app.notification()
            .builder()
            .title(format!("{} 篇新文章", new_count))
            .body(format!("来自 {}", feed_name))
            .show();
    }
    Ok(new_count)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn refresh_all_feeds(app: tauri::AppHandle, state: State<'_, DbPool>) -> Result<usize, String> {
    let total_new = commands::refresh_all_feeds(&state)?;
    if total_new > 0 {
        use tauri_plugin_notification::NotificationExt;
        let _ = app.notification()
            .builder()
            .title(format!("{} 篇新文章", total_new))
            .body("所有订阅源已刷新")
            .show();
    }
    Ok(total_new)
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

// ============================================================
// -- Provider management (Stage 3) —
//    注意：这些命令直接调用 AgentService 的方法，
//    不经过 commands.rs（遵守阶段隔离规则）
// ============================================================

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn add_provider(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    provider: crate::db::model::NewProvider,
) -> Result<crate::db::model::Provider, String> {
    agent_service.add_provider(&provider)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn list_providers(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
) -> Result<Vec<crate::db::model::Provider>, String> {
    agent_service.list_providers()
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn update_provider(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    id: i64,
    update: crate::db::model::UpdateProvider,
) -> Result<crate::db::model::Provider, String> {
    agent_service.update_provider(id, &update)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn delete_provider(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    id: i64,
) -> Result<(), String> {
    agent_service.delete_provider(id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn add_provider_model(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    model: crate::db::model::NewProviderModel,
) -> Result<crate::db::model::ProviderModel, String> {
    agent_service.add_provider_model(&model)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn list_provider_models(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    provider_id: i64,
) -> Result<Vec<crate::db::model::ProviderModel>, String> {
    agent_service.list_provider_models(provider_id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn delete_provider_model(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    id: i64,
) -> Result<(), String> {
    agent_service.delete_provider_model(id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn validate_provider(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    base_url: String,
    api_key: String,
    model: String,
) -> Result<bool, String> {
    agent_service
        .validate_provider(&base_url, &api_key, &model)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================
// -- Agent commands (Stage 3) —
//    注意：同样不经过 commands.rs，直接调用 AgentService
// ============================================================

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn generate_summary(
    app: tauri::AppHandle,
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    entry_id: i64,
    target_language: Option<String>,
    detail_level: Option<String>,
    force: Option<bool>,
) -> Result<(), String> {
    let app_handle = app.clone();

    let on_event = move |event: crate::agent::client::AiStreamEvent| {
        let _ = app_handle.emit("ai-stream", serde_json::to_value(&event).unwrap_or_default());
    };

    let lang = target_language.unwrap_or_else(|| "zh-CN".to_string());
    let detail = detail_level.unwrap_or_else(|| "standard".to_string());

    agent_service.generate_summary(entry_id, &lang, &detail, force.unwrap_or(false), on_event).await.map_err(|e| e.to_string())
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn get_summary(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    entry_id: i64,
) -> Result<Option<String>, String> {
    agent_service.get_latest_summary_text(entry_id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn cancel_summary(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    entry_id: i64,
) -> Result<(), String> {
    agent_service.cancel_summary(entry_id).await.map_err(|e| e.to_string())
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn clear_summary(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    entry_id: i64,
) -> Result<(), String> {
    agent_service.clear_summary(entry_id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn translate_entry(
    app: tauri::AppHandle,
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    entry_id: i64,
    target_language: Option<String>,
    concurrency: Option<usize>,
    force: Option<bool>,
) -> Result<(), String> {
    let app_handle = app.clone();

    let on_event = move |event: crate::agent::client::AiStreamEvent| {
        let _ = app_handle.emit("ai-stream", serde_json::to_value(&event).unwrap_or_default());
    };

    let lang = target_language.unwrap_or_else(|| "zh-CN".to_string());
    let conc = concurrency.unwrap_or(3);

    agent_service.translate_entry(entry_id, &lang, conc, force.unwrap_or(false), on_event).await.map_err(|e| e.to_string())
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn get_translation(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    entry_id: i64,
) -> Result<Option<String>, String> {
    agent_service.get_latest_translation_text(entry_id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn cancel_translation(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    entry_id: i64,
) -> Result<(), String> {
    agent_service.cancel_translation(entry_id).await.map_err(|e| e.to_string())
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn clear_translation(
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    entry_id: i64,
) -> Result<(), String> {
    agent_service.clear_translation(entry_id)
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn retry_failed_segments(
    app: tauri::AppHandle,
    agent_service: State<'_, std::sync::Arc<crate::agent::service::AgentService>>,
    entry_id: i64,
) -> Result<(), String> {
    let app_handle = app.clone();

    let on_event = move |event: crate::agent::client::AiStreamEvent| {
        let _ = app_handle.emit("ai-stream", serde_json::to_value(&event).unwrap_or_default());
    };

    agent_service
        .retry_failed_segments(entry_id, on_event)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================
// -- Settings (Stage 4) --
// ============================================================

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn get_setting(state: State<'_, DbPool>, key: String) -> Result<Option<String>, String> {
    let pool = state.inner();
    let conn = pool.get().map_err(|e| e.to_string())?;
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(val) => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(feature = "tauri-runtime")]
#[tauri::command]
fn set_setting(state: State<'_, DbPool>, key: String, value: String) -> Result<(), String> {
    let pool = state.inner();
    let conn = pool.get().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
