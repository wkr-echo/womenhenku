// Token usage event recorder — records every LLM call.
//
// Called after each LLM request (summary, translation, tagging)
// regardless of success/failure/cancellation/timeout.

use crate::db::DbPool;
use rusqlite::params;

/// Context for a single usage event to be recorded.
pub struct UsageEventContext {
    pub task_type: String,           // "summary" | "translation" | "tagging"
    pub entry_id: Option<i64>,
    pub provider_id: Option<i64>,
    pub model_id: Option<i64>,
    pub provider_base_url: String,
    pub provider_host: Option<String>,
    pub provider_name: Option<String>,
    pub model_name: String,
    pub request_phase: String,       // "normal" | "retry"
    pub request_status: String,      // "succeeded" | "failed" | "cancelled" | "timed_out"
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

/// Record a single usage event. Called from agent code after each LLM call.
pub fn record_usage_event(pool: &DbPool, ctx: UsageEventContext) -> Result<(), String> {
    let total = match (ctx.prompt_tokens, ctx.completion_tokens) {
        (Some(p), Some(c)) => Some(p + c),
        _ => None,
    };
    let availability = if total.is_some() && total.unwrap_or(0) > 0 {
        "actual"
    } else {
        "missing"
    };

    let conn = pool.get().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO usage_events
            (entry_id, task_type, provider_id, model_id,
             provider_base_url_snapshot, provider_host_snapshot,
             provider_name_snapshot, model_name_snapshot,
             request_phase, request_status,
             prompt_tokens, completion_tokens, total_tokens,
             usage_availability, started_at, finished_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            ctx.entry_id,
            ctx.task_type,
            ctx.provider_id,
            ctx.model_id,
            ctx.provider_base_url,
            ctx.provider_host,
            ctx.provider_name,
            ctx.model_name,
            ctx.request_phase,
            ctx.request_status,
            ctx.prompt_tokens,
            ctx.completion_tokens,
            total,
            availability,
            ctx.started_at,
            ctx.finished_at,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Delete usage events older than the given number of days.
pub fn cleanup_old_events(pool: &DbPool, retention_days: i64) -> Result<usize, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    let deleted = conn
        .execute(
            "DELETE FROM usage_events WHERE created_at < datetime('now', ?1)",
            params![format!("-{} days", retention_days)],
        )
        .map_err(|e| e.to_string())?;
    Ok(deleted)
}

/// Delete all usage events.
pub fn clear_all_events(pool: &DbPool) -> Result<usize, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    let deleted = conn
        .execute("DELETE FROM usage_events", [])
        .map_err(|e| e.to_string())?;
    Ok(deleted)
}
