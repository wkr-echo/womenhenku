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
use crate::db::repository::{ContentRepository, EntryRepository, FeedRepository, ProviderRepository};
use crate::db::DbPool;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Semaphore;

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

/// Count entries within a date range (for batch tagging candidate estimation).
pub fn count_entries_by_date_range(pool: &DbPool, days: i64) -> Result<i64, String> {
    let repo = EntryRepository::new(pool.clone());
    repo.count_by_date_range(days).map_err(|e| e.to_string())
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

pub fn list_entries_by_tags(pool: &DbPool, tag_ids: Vec<i64>, match_mode: &str, page: i32, page_size: i32) -> Result<EntryPage, String> {
    let repo = crate::db::repository::EntryRepository::new(pool.clone());
    repo.list_by_tags(&tag_ids, match_mode, page, page_size).map_err(|e| e.to_string())
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

// ============================================================
// AI Tag Recommendations
// ============================================================

/// Generate tag recommendations for an article using AI.
/// Returns the generated recommendations.
pub async fn generate_tag_recommendations(
    pool: &DbPool,
    entry_id: i64,
    existing_tags: Vec<String>,
) -> Result<Vec<crate::db::model::TagRecommendation>, String> {
    // Get article title + summary (first 2000 chars only, no full content)
    let entry_repo = crate::db::repository::EntryRepository::new(pool.clone());
    let entry = entry_repo
        .find_by_id(entry_id)
        .map_err(|e| format!("读取文章失败: {}", e))?
        .ok_or_else(|| "文章不存在".to_string())?;

    let article_text = format!(
        "标题：{}\n摘要：{}",
        entry.title,
        entry.summary
    );
    let truncated: String = article_text.chars().take(2000).collect();

    // Get default provider
    let provider_repo = crate::db::repository::ProviderRepository::new(pool.clone());
    let provider = provider_repo
        .find_default()
        .map_err(|e| format!("读取 Provider 失败: {}", e))?
        .ok_or_else(|| "未配置默认 AI Provider".to_string())?;

    let models = provider_repo
        .list_models(provider.id)
        .map_err(|e| format!("读取模型列表失败: {}", e))?;

    let model = models
        .iter()
        .find(|m| m.is_default)
        .map(|m| m.model_name.clone())
        .or_else(|| models.first().map(|m| m.model_name.clone()))
        .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

    let api_key = crate::agent::crypto::decrypt(&provider.api_key_ref)
        .unwrap_or_else(|_| provider.api_key_ref.clone());

    // Inject existing tag library as vocabulary reference
    let tags_json = serde_json::to_string(&existing_tags).unwrap_or_else(|_| "[]".to_string());

    let system_prompt = "你是一个文章标签推荐助手。根据文章标题和摘要，推荐3-5个简洁精准的标签。优先使用已有标签库中的标签名。用JSON数组格式返回，如：[\"tag1\",\"tag2\"]。只输出JSON，不要其他内容。";

    let user_prompt = format!(
        "已有标签库：{}\n\n文章信息：{}",
        tags_json,
        truncated
    );

    // Call AI with timeout
    let client = crate::agent::client::AiClient::new();
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        client.chat(&provider.base_url, &api_key, &model, system_prompt, &user_prompt),
    )
    .await
    .map_err(|_| "AI 请求超时（60秒）".to_string())?
    .map_err(|e| format!("AI 调用失败: {}", e))?;

    // Parse JSON response
    let tag_names: Vec<String> = serde_json::from_str::<Vec<String>>(&response)
        .unwrap_or_else(|_| {
            // Fallback: try to extract JSON array from text
            if let Some(start) = response.find('[') {
                if let Some(end) = response.rfind(']') {
                    let json_part = &response[start..=end];
                    serde_json::from_str(json_part).unwrap_or_default()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        });

    let tag_names: Vec<String> = tag_names
        .into_iter()
        .filter(|s| !s.is_empty() && s.len() <= 50)
        .take(5)
        .collect();

    if tag_names.is_empty() {
        return Err("AI 未返回有效标签".to_string());
    }

    // Save to database
    let tag_repo = crate::db::repository::TagRepository::new(pool.clone());
    let recommendations: Vec<(String, String, f64)> = tag_names
        .iter()
        .map(|name| (name.clone(), "ai".to_string(), 0.8))
        .collect();
    tag_repo
        .save_recommendations(entry_id, &recommendations)
        .map_err(|e| format!("保存推荐失败: {}", e))?;

    // Record usage (stored but not shown in stats UI)
    let _ = crate::db::repository::LlmUsageRepository::new(pool.clone())
        .insert_event(&crate::db::model::LlmUsageEvent {
            id: 0,
            provider_id: provider.id,
            provider_name: provider.name.clone(),
            provider_base_url: provider.base_url.clone(),
            provider_host: provider.base_url.clone(),
            model_id: 0,
            model_name: model.clone(),
            agent_type: "tagging".to_string(),
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            request_status: "success".to_string(),
            timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            created_at: String::new(),
        });

    tag_repo
        .find_recommendations_by_entry_id(entry_id)
        .map_err(|e| format!("读取推荐失败: {}", e))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagProposal {
    pub tag_name: String,
    pub hit_count: i32,
    pub entry_count: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTagApplyResult {
    pub processed: i32,
    pub success: i32,
    pub failed: i32,
    pub tag_associations: i32,
    pub new_tags: i32,
    pub kept_proposals: i32,
    pub discarded_proposals: i32,
}

fn build_batch_tag_conditions(
    range: &str,
    skip_batch_tagged: bool,
    skip_tagged: bool,
) -> Result<(String, Option<i64>), String> {
    let mut clauses: Vec<String> = Vec::new();
    let mut days: Option<i64> = None;

    match range {
        "unread" => clauses.push("e.is_read = 0".to_string()),
        "all" => {}
        "1week" => {
            clauses.push("e.created_at >= datetime('now', '-' || ?1 || ' days')".to_string());
            days = Some(7);
        }
        "1month" => {
            clauses.push("e.created_at >= datetime('now', '-' || ?1 || ' days')".to_string());
            days = Some(30);
        }
        "3months" => {
            clauses.push("e.created_at >= datetime('now', '-' || ?1 || ' days')".to_string());
            days = Some(90);
        }
        "6months" => {
            clauses.push("e.created_at >= datetime('now', '-' || ?1 || ' days')".to_string());
            days = Some(180);
        }
        "1year" => {
            clauses.push("e.created_at >= datetime('now', '-' || ?1 || ' days')".to_string());
            days = Some(365);
        }
        _ => return Err("未知范围: " .to_string() + range),
    }

    if skip_batch_tagged {
        clauses.push("NOT EXISTS (SELECT 1 FROM entry_tags et WHERE et.entry_id = e.id AND et.source = 'batch')".to_string());
    }
    if skip_tagged {
        clauses.push("NOT EXISTS (SELECT 1 FROM entry_tags et WHERE et.entry_id = e.id)".to_string());
    }

    let where_clause = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };

    Ok((where_clause, days))
}

pub fn count_batch_tag_candidates(
    pool: &DbPool,
    range: &str,
    skip_batch_tagged: bool,
    skip_tagged: bool,
) -> Result<i64, String> {
    let conn = pool.get().map_err(|e| format!("数据库连接失败: {}", e))?;
    let (where_clause, days) = build_batch_tag_conditions(range, skip_batch_tagged, skip_tagged)?;
    let count_sql = format!("SELECT COUNT(*) FROM entries e {}", where_clause);

    let total: i64 = if let Some(days) = days {
        conn.query_row(&count_sql, rusqlite::params![days], |row| row.get(0))
            .map_err(|e| format!("查询候选文章数失败: {}", e))?
    } else {
        conn.query_row(&count_sql, [], |row| row.get(0))
            .map_err(|e| format!("查询候选文章数失败: {}", e))?
    };

    Ok(total)
}

pub fn get_batch_tag_candidate_entry_ids(
    pool: &DbPool,
    range: &str,
    skip_batch_tagged: bool,
    skip_tagged: bool,
) -> Result<Vec<i64>, String> {
    let conn = pool.get().map_err(|e| format!("数据库连接失败: {}", e))?;
    let (where_clause, days) = build_batch_tag_conditions(range, skip_batch_tagged, skip_tagged)?;
    let query = format!("SELECT e.id FROM entries e {} ORDER BY COALESCE(e.published_at, e.created_at) DESC", where_clause);
    let mut stmt = conn.prepare(&query).map_err(|e| format!("构建查询失败: {}", e))?;

    let rows = match days {
        Some(days) => {
            let rows = stmt
                .query_map(rusqlite::params![days], |row| row.get(0))
                .map_err(|e| format!("查询候选文章失败: {}", e))?;
            rows.collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| format!("查询候选文章失败: {}", e))
        }
        None => {
            let rows = stmt
                .query_map([], |row| row.get(0))
                .map_err(|e| format!("查询候选文章失败: {}", e))?;
            rows.collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| format!("查询候选文章失败: {}", e))
        }
    }?;

    Ok(rows)
}

/// Analyze entries for tag suggestions over a date range.
pub async fn analyze_entries_for_tags(
    pool: &DbPool,
    range: &str,
    skip_batch_tagged: bool,
    skip_tagged: bool,
    concurrency: i32,
) -> Result<Vec<TagProposal>, String> {
    let entries = {
        let conn = pool.get().map_err(|e| format!("数据库连接失败: {}", e))?;
        let (where_clause, days) = build_batch_tag_conditions(range, skip_batch_tagged, skip_tagged)?;
        let query = format!("SELECT e.id, e.title, e.summary FROM entries e {}", where_clause);
        let mut stmt = conn.prepare(&query).map_err(|e| format!("构建查询失败: {}", e))?;

        let entries = match days {
            Some(days) => {
                let rows = stmt
                    .query_map(rusqlite::params![days], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    })
                    .map_err(|e| format!("查询文章失败: {}", e))?;
                rows.collect::<Result<Vec<_>, rusqlite::Error>>()
                    .map_err(|e| format!("查询文章失败: {}", e))?
            }
            None => {
                let rows = stmt
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    })
                    .map_err(|e| format!("查询文章失败: {}", e))?;
                rows.collect::<Result<Vec<_>, rusqlite::Error>>()
                    .map_err(|e| format!("查询文章失败: {}", e))?
            }
        };

        entries
    };

    let provider_repo = ProviderRepository::new(pool.clone());
    let provider = provider_repo
        .find_default()
        .map_err(|e| format!("读取 Provider 失败: {}", e))?
        .ok_or_else(|| "未配置默认 AI Provider".to_string())?;

    let models = provider_repo
        .list_models(provider.id)
        .map_err(|e| format!("读取模型列表失败: {}", e))?;

    let model = models
        .iter()
        .find(|m| m.is_default)
        .map(|m| m.model_name.clone())
        .or_else(|| models.first().map(|m| m.model_name.clone()))
        .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

    let api_key = crate::agent::crypto::decrypt(&provider.api_key_ref)
        .unwrap_or_else(|_| provider.api_key_ref.clone());

    let tags_json = serde_json::to_string(&Vec::<String>::new()).unwrap_or_else(|_| "[]".to_string());
    let system_prompt = "你是一个文章标签推荐助手。根据文章标题和摘要，推荐3-5个简洁精准的标签。优先使用已有标签库中的标签名。用JSON数组格式返回，如:[\"tag1\",\"tag2\"]。只输出JSON，不要其他内容。";

    let concurrency = concurrency.clamp(1, 5) as usize;
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let mut handles = Vec::with_capacity(entries.len());
    for (entry_id, title, summary) in entries.into_iter() {
        let provider = provider.clone();
        let model = model.clone();
        let api_key = api_key.clone();
        let tags_json = tags_json.clone();
        let semaphore = semaphore.clone();
        let prompt = format!("已有标签库：{}\n\n文章信息：{}\n摘要：{}", tags_json, title, summary.chars().take(500).collect::<String>());

        let handle = tokio::spawn(async move {
            let _permit = semaphore
                .acquire_owned()
                .await
                .map_err(|e| format!("信号量获取失败: {}", e))?;

            let client = crate::agent::client::AiClient::new();
            let response = tokio::time::timeout(
                std::time::Duration::from_secs(60),
                client.chat(&provider.base_url, &api_key, &model, system_prompt, &prompt),
            )
            .await
            .map_err(|_| "AI 请求超时（60秒）".to_string())?
            .map_err(|e| format!("AI 调用失败: {}", e))?;

            let tag_names: Vec<String> = serde_json::from_str::<Vec<String>>(&response)
                .unwrap_or_else(|_| {
                    if let Some(start) = response.find('[') {
                        if let Some(end) = response.rfind(']') {
                            let json_part = &response[start..=end];
                            serde_json::from_str(json_part).unwrap_or_default()
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                });

            let tag_names = tag_names
                .into_iter()
                .map(|tag| tag.trim().to_string())
                .filter(|tag| !tag.is_empty() && tag.len() <= 50)
                .collect::<Vec<_>>();
            Ok::<(_, Vec<String>), String>((entry_id, tag_names))
        });

        handles.push(handle);
    }

    let mut hit_counts: HashMap<String, i32> = HashMap::new();
    let mut entry_counts: HashMap<String, i32> = HashMap::new();

    for handle in handles {
        match handle.await {
            Ok(Ok((_, tag_names))) => {
                let unique_tags: HashSet<String> = tag_names.iter().cloned().collect();
                for tag in tag_names {
                    *hit_counts.entry(tag.clone()).or_insert(0) += 1;
                }
                for tag in unique_tags {
                    *entry_counts.entry(tag).or_insert(0) += 1;
                }
            }
            Ok(Err(err)) => {
                tracing::warn!("单篇文章标签分析失败，已跳过: {}", err);
            }
            Err(err) => {
                tracing::warn!("单篇标签分析任务失败，已跳过: {}", err);
            }
        }
    }

    let mut proposals = hit_counts
        .into_iter()
        .map(|(tag_name, hit_count)| TagProposal {
            tag_name: tag_name.clone(),
            hit_count,
            entry_count: *entry_counts.get(&tag_name).unwrap_or(&0),
        })
        .collect::<Vec<_>>();

    proposals.sort_by(|a, b| {
        b.hit_count
            .cmp(&a.hit_count)
            .then_with(|| b.entry_count.cmp(&a.entry_count))
            .then_with(|| a.tag_name.cmp(&b.tag_name))
    });

    Ok(proposals)
}

pub async fn apply_batch_tags(
    pool: &DbPool,
    range: &str,
    skip_batch_tagged: bool,
    skip_tagged: bool,
    kept_tags: Vec<String>,
    total_proposals: usize,
) -> Result<BatchTagApplyResult, String> {
    let entry_ids = get_batch_tag_candidate_entry_ids(pool, range, skip_batch_tagged, skip_tagged)?;
    let repo = crate::db::repository::TagRepository::new(pool.clone());
    let mut new_tags = 0;
    let mut associations = 0;

    for tag_name in &kept_tags {
        let tag = match repo.find_by_name_or_alias(tag_name) {
            Ok(Some(existing)) => existing,
            Ok(None) => {
                new_tags += 1;
                repo.insert(tag_name, "#3b82f6").map_err(|e| format!("创建标签失败: {}", e))?
            }
            Err(e) => {
                tracing::warn!("查找标签失败: {}", e);
                continue;
            }
        };

        for entry_id in &entry_ids {
            match repo.add_tag_to_entry(*entry_id, tag.id, "batch", 0.0) {
                Ok(()) => associations += 1,
                Err(e) => tracing::warn!("为文章 {} 添加标签 {} 失败: {}", entry_id, tag_name, e),
            }
        }
    }

    Ok(BatchTagApplyResult {
        processed: entry_ids.len() as i32,
        success: entry_ids.len() as i32,
        failed: 0,
        tag_associations: associations,
        new_tags,
        kept_proposals: kept_tags.len() as i32,
        discarded_proposals: (total_proposals.saturating_sub(kept_tags.len())) as i32,
    })
}
