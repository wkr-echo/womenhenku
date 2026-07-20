use rusqlite::params;

use crate::db::model::{LlmUsageEvent, LlmUsageStats, DailyUsage, ProviderUsage, ModelUsage, AgentUsage, Setting};
use crate::db::DbPool;
use crate::db::error::RepositoryError;

pub struct LlmUsageRepository {
    pool: DbPool,
}

impl LlmUsageRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn insert_event(&self, event: &LlmUsageEvent) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO llm_usage_events (provider_id, provider_name, provider_base_url, provider_host, model_id, model_name, agent_type, prompt_tokens, completion_tokens, total_tokens, request_status, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                event.provider_id,
                event.provider_name,
                event.provider_base_url,
                event.provider_host,
                event.model_id,
                event.model_name,
                event.agent_type,
                event.prompt_tokens,
                event.completion_tokens,
                event.total_tokens,
                event.request_status,
                event.timestamp,
            ],
        )?;
        Ok(())
    }

    pub fn get_stats(&self, days: i64, agent_type: Option<&str>) -> Result<LlmUsageStats, RepositoryError> {
        let conn = self.pool.get()?;

        let (total_tokens, prompt_tokens, completion_tokens, request_count, success_count) = match agent_type {
            Some(at) => {
                let sql = format!(
                    "SELECT COALESCE(SUM(total_tokens), 0), COALESCE(SUM(prompt_tokens), 0), COALESCE(SUM(completion_tokens), 0), COUNT(*), COALESCE(SUM(CASE WHEN request_status = 'success' THEN 1 ELSE 0 END), 0) FROM llm_usage_events WHERE timestamp >= datetime('now', '-{} days') AND agent_type = ?",
                    days
                );
                let mut stmt = conn.prepare(&sql)?;
                let mut rows = stmt.query_map(params![at], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                })?;
                rows.next().unwrap_or(Ok((0, 0, 0, 0, 0)))?
            }
            None => {
                let sql = format!(
                    "SELECT COALESCE(SUM(total_tokens), 0), COALESCE(SUM(prompt_tokens), 0), COALESCE(SUM(completion_tokens), 0), COUNT(*), COALESCE(SUM(CASE WHEN request_status = 'success' THEN 1 ELSE 0 END), 0) FROM llm_usage_events WHERE timestamp >= datetime('now', '-{} days')",
                    days
                );
                let mut stmt = conn.prepare(&sql)?;
                let mut rows = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                })?;
                rows.next().unwrap_or(Ok((0, 0, 0, 0, 0)))?
            }
        };

        let success_rate = if request_count > 0 {
            (success_count as f64 / request_count as f64) * 100.0
        } else {
            0.0
        };

        let avg_tokens_per_request = if request_count > 0 {
            total_tokens as f64 / request_count as f64
        } else {
            0.0
        };

        Ok(LlmUsageStats {
            total_tokens,
            prompt_tokens,
            completion_tokens,
            request_count,
            success_rate,
            avg_tokens_per_request,
        })
    }

    pub fn get_daily_usage(&self, days: i64, agent_type: Option<&str>) -> Result<Vec<DailyUsage>, RepositoryError> {
        let conn = self.pool.get()?;

        if let Some(at) = agent_type {
            let sql = format!(
                "SELECT DATE(timestamp) as date, COALESCE(SUM(total_tokens), 0), COALESCE(SUM(prompt_tokens), 0), COALESCE(SUM(completion_tokens), 0), COUNT(*) FROM llm_usage_events WHERE timestamp >= datetime('now', '-{} days') AND agent_type = ? GROUP BY DATE(timestamp) ORDER BY date",
                days
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![at], map_daily_usage)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        } else {
            let sql = format!(
                "SELECT DATE(timestamp) as date, COALESCE(SUM(total_tokens), 0), COALESCE(SUM(prompt_tokens), 0), COALESCE(SUM(completion_tokens), 0), COUNT(*) FROM llm_usage_events WHERE timestamp >= datetime('now', '-{} days') GROUP BY DATE(timestamp) ORDER BY date",
                days
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map([], map_daily_usage)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
        }
    }

    pub fn get_provider_usage(&self, days: i64) -> Result<Vec<ProviderUsage>, RepositoryError> {
        let conn = self.pool.get()?;

        let sql = format!(
            "SELECT provider_id, provider_name, COALESCE(SUM(total_tokens), 0), COUNT(*) FROM llm_usage_events WHERE timestamp >= datetime('now', '-{} days') GROUP BY provider_id ORDER BY SUM(total_tokens) DESC",
            days
        );
        let mut stmt = conn.prepare(&sql)?;

        let rows = stmt.query_map([], |row| {
            Ok(ProviderUsage {
                provider_id: row.get(0)?,
                provider_name: row.get(1)?,
                total_tokens: row.get(2)?,
                request_count: row.get(3)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_model_usage(&self, days: i64) -> Result<Vec<ModelUsage>, RepositoryError> {
        let conn = self.pool.get()?;

        let sql = format!(
            "SELECT model_id, model_name, COALESCE(SUM(total_tokens), 0), COUNT(*) FROM llm_usage_events WHERE timestamp >= datetime('now', '-{} days') GROUP BY model_id ORDER BY SUM(total_tokens) DESC",
            days
        );
        let mut stmt = conn.prepare(&sql)?;

        let rows = stmt.query_map([], |row| {
            Ok(ModelUsage {
                model_id: row.get(0)?,
                model_name: row.get(1)?,
                total_tokens: row.get(2)?,
                request_count: row.get(3)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_agent_usage(&self, days: i64) -> Result<Vec<AgentUsage>, RepositoryError> {
        let conn = self.pool.get()?;

        let sql = format!(
            "SELECT agent_type, COALESCE(SUM(total_tokens), 0), COUNT(*) FROM llm_usage_events WHERE timestamp >= datetime('now', '-{} days') GROUP BY agent_type ORDER BY SUM(total_tokens) DESC",
            days
        );
        let mut stmt = conn.prepare(&sql)?;

        let rows = stmt.query_map([], |row| {
            Ok(AgentUsage {
                agent_type: row.get(0)?,
                total_tokens: row.get(1)?,
                request_count: row.get(2)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn cleanup_old_events(&self, retention_days: i64) -> Result<usize, RepositoryError> {
        let conn = self.pool.get()?;
        let sql = format!(
            "DELETE FROM llm_usage_events WHERE timestamp < datetime('now', '-{} days')",
            retention_days
        );
        let affected = conn.execute(&sql, [])?;
        Ok(affected)
    }

    pub fn delete_all_events(&self) -> Result<usize, RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute("DELETE FROM llm_usage_events", [])?;
        Ok(affected)
    }
}

pub struct SettingsRepository {
    pool: DbPool,
}

impl SettingsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
        match rows.next() {
            Some(result) => result.map(Some).map_err(Into::into),
            None => Ok(None),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<Vec<Setting>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, key, value, created_at, updated_at FROM settings ORDER BY key")?;
        let rows = stmt.query_map([], |row| {
            Ok(Setting {
                id: row.get(0)?,
                key: row.get(1)?,
                value: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, RepositoryError> {
        self.get_setting(key)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), RepositoryError> {
        self.set_setting(key, value)
    }

    pub fn delete(&self, key: &str) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(())
    }
}

fn map_daily_usage(row: &rusqlite::Row<'_>) -> rusqlite::Result<DailyUsage> {
    Ok(DailyUsage {
        date: row.get(0)?,
        total_tokens: row.get(1)?,
        prompt_tokens: row.get(2)?,
        completion_tokens: row.get(3)?,
        request_count: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db_pool;

    #[test]
    fn test_insert_and_get_stats() {
        let pool = open_test_db_pool().expect("Failed to create test pool");
        let repo = LlmUsageRepository::new(pool);

        let event = LlmUsageEvent {
            id: 0,
            provider_id: 1,
            provider_name: "OpenAI".to_string(),
            provider_base_url: "https://api.openai.com/v1".to_string(),
            provider_host: "api.openai.com".to_string(),
            model_id: 1,
            model_name: "gpt-4".to_string(),
            agent_type: "summary".to_string(),
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            request_status: "success".to_string(),
            timestamp: "2026-07-20 10:00:00".to_string(),
            created_at: "2026-07-20 10:00:00".to_string(),
        };

        repo.insert_event(&event).expect("insert event");

        let stats = repo.get_stats(30, None).expect("get stats");
        assert_eq!(stats.total_tokens, 150);
        assert_eq!(stats.request_count, 1);
        assert_eq!(stats.success_rate, 100.0);
    }

    #[test]
    fn test_get_daily_usage() {
        let pool = open_test_db_pool().expect("Failed to create test pool");
        let repo = LlmUsageRepository::new(pool);

        let event = LlmUsageEvent {
            id: 0,
            provider_id: 1,
            provider_name: "OpenAI".to_string(),
            provider_base_url: "https://api.openai.com/v1".to_string(),
            provider_host: "api.openai.com".to_string(),
            model_id: 1,
            model_name: "gpt-4".to_string(),
            agent_type: "translation".to_string(),
            prompt_tokens: 200,
            completion_tokens: 100,
            total_tokens: 300,
            request_status: "success".to_string(),
            timestamp: "2026-07-20 10:00:00".to_string(),
            created_at: "2026-07-20 10:00:00".to_string(),
        };

        repo.insert_event(&event).expect("insert event");

        let daily = repo.get_daily_usage(30, None).expect("get daily");
        assert_eq!(daily.len(), 1);
        assert_eq!(daily[0].date, "2026-07-20");
        assert_eq!(daily[0].total_tokens, 300);
    }

    #[test]
    fn test_settings_crud() {
        let pool = open_test_db_pool().expect("Failed to create test pool");
        let repo = SettingsRepository::new(pool);

        repo.set_setting("test_key", "test_value").expect("set setting");

        let value = repo.get_setting("test_key").expect("get setting");
        assert_eq!(value, Some("test_value".to_string()));

        repo.set_setting("test_key", "updated_value").expect("update setting");

        let value = repo.get_setting("test_key").expect("get updated setting");
        assert_eq!(value, Some("updated_value".to_string()));
    }
}
