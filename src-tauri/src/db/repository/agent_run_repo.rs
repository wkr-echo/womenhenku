use crate::db::error::RepositoryError;
use crate::db::model::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

/// Repository for tracking agent execution runs.
/// Stores the history and state of Summary/Translation agent runs.
pub struct AgentRunRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl AgentRunRepository {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    /// Create a new agent run record.
    pub fn create(&self, run: &NewAgentRun) -> Result<AgentRun, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "INSERT INTO agent_runs (entry_id, provider_id, task_kind, phase, target_language, detail_level)
             VALUES (?1, ?2, ?3, 'idle', ?4, ?5)",
        )?;

        stmt.execute(rusqlite::params![
            run.entry_id,
            run.provider_id,
            run.task_kind,
            run.target_language,
            run.detail_level,
        ])?;

        let id = conn.last_insert_rowid();
        self.find_by_id(id)?.ok_or(RepositoryError::NotFound("AgentRun not found".to_string()))
    }

    /// Find a run by its ID.
    pub fn find_by_id(&self, id: i64) -> Result<Option<AgentRun>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, entry_id, provider_id, task_kind, phase, target_language, detail_level,
                    output_text, prompt_tokens, completion_tokens, error_message,
                    started_at, completed_at, created_at
             FROM agent_runs WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(AgentRun {
                id: row.get(0)?,
                entry_id: row.get(1)?,
                provider_id: row.get(2)?,
                task_kind: row.get(3)?,
                phase: row.get(4)?,
                target_language: row.get(5)?,
                detail_level: row.get(6)?,
                output_text: row.get(7)?,
                prompt_tokens: row.get(8)?,
                completion_tokens: row.get(9)?,
                error_message: row.get(10)?,
                started_at: row.get(11)?,
                completed_at: row.get(12)?,
                created_at: row.get(13)?,
            })
        })?;

        match rows.next() {
            Some(Ok(run)) => Ok(Some(run)),
            _ => Ok(None),
        }
    }

    /// Find the latest run for a given entry and task kind.
    pub fn find_latest_by_entry_and_task(
        &self,
        entry_id: i64,
        task_kind: &str,
    ) -> Result<Option<AgentRun>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, entry_id, provider_id, task_kind, phase, target_language, detail_level,
                    output_text, prompt_tokens, completion_tokens, error_message,
                    started_at, completed_at, created_at
             FROM agent_runs
             WHERE entry_id = ?1 AND task_kind = ?2
             ORDER BY id DESC LIMIT 1",
        )?;

        let mut rows = stmt.query_map(rusqlite::params![entry_id, task_kind], |row| {
            Ok(AgentRun {
                id: row.get(0)?,
                entry_id: row.get(1)?,
                provider_id: row.get(2)?,
                task_kind: row.get(3)?,
                phase: row.get(4)?,
                target_language: row.get(5)?,
                detail_level: row.get(6)?,
                output_text: row.get(7)?,
                prompt_tokens: row.get(8)?,
                completion_tokens: row.get(9)?,
                error_message: row.get(10)?,
                started_at: row.get(11)?,
                completed_at: row.get(12)?,
                created_at: row.get(13)?,
            })
        })?;

        match rows.next() {
            Some(Ok(run)) => Ok(Some(run)),
            _ => Ok(None),
        }
    }

    /// Update the phase of a run.
    pub fn update_phase(&self, id: i64, phase: &str) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE agent_runs SET phase = ?1 WHERE id = ?2",
            rusqlite::params![phase, id],
        )?;
        Ok(())
    }

    /// Mark a run as started.
    pub fn mark_started(&self, id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE agent_runs SET phase = 'running', started_at = datetime('now') WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    /// Mark a run as completed with output.
    pub fn mark_completed(
        &self,
        id: i64,
        output_text: &str,
        prompt_tokens: Option<i64>,
        completion_tokens: Option<i64>,
    ) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE agent_runs SET phase = 'succeeded', output_text = ?1,
             prompt_tokens = ?2, completion_tokens = ?3, completed_at = datetime('now')
             WHERE id = ?4",
            rusqlite::params![output_text, prompt_tokens, completion_tokens, id],
        )?;
        Ok(())
    }

    /// Mark a run as failed with an error message.
    pub fn mark_failed(&self, id: i64, error_message: &str) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE agent_runs SET phase = 'failed', error_message = ?1, completed_at = datetime('now')
             WHERE id = ?2",
            rusqlite::params![error_message, id],
        )?;
        Ok(())
    }

    /// Mark a run as cancelled.
    pub fn mark_cancelled(&self, id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE agent_runs SET phase = 'cancelled', completed_at = datetime('now') WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    /// List runs for a given entry, ordered by most recent first.
    pub fn list_by_entry(&self, entry_id: i64) -> Result<Vec<AgentRun>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, entry_id, provider_id, task_kind, phase, target_language, detail_level,
                    output_text, prompt_tokens, completion_tokens, error_message,
                    started_at, completed_at, created_at
             FROM agent_runs WHERE entry_id = ?1 ORDER BY id DESC",
        )?;

        let runs = stmt
            .query_map(rusqlite::params![entry_id], |row| {
                Ok(AgentRun {
                    id: row.get(0)?,
                    entry_id: row.get(1)?,
                    provider_id: row.get(2)?,
                    task_kind: row.get(3)?,
                    phase: row.get(4)?,
                    target_language: row.get(5)?,
                    detail_level: row.get(6)?,
                    output_text: row.get(7)?,
                    prompt_tokens: row.get(8)?,
                    completion_tokens: row.get(9)?,
                    error_message: row.get(10)?,
                    started_at: row.get(11)?,
                    completed_at: row.get(12)?,
                    created_at: row.get(13)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(runs)
    }

    /// Delete runs for a given entry (e.g., when entry is deleted).
    pub fn delete_by_entry(&self, entry_id: i64) -> Result<u64, RepositoryError> {
        let conn = self.pool.get()?;
        let rows = conn.execute(
            "DELETE FROM agent_runs WHERE entry_id = ?1",
            rusqlite::params![entry_id],
        )?;
        Ok(rows as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_repo() -> (AgentRunRepository, Pool<SqliteConnectionManager>) {
        let pool = db::open_test_db_pool().expect("Failed to open test DB");
        let repo = AgentRunRepository::new(pool.clone());
        (repo, pool)
    }

    /// Helper: create a feed and entry in the DB for foreign key constraints.
    fn setup_entry(pool: &Pool<SqliteConnectionManager>) -> i64 {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO feeds (url, title) VALUES ('https://test.com/feed', 'Test Feed')",
            [],
        )
        .unwrap();
        let feed_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO entries (feed_id, guid, title) VALUES (?1, 'guid1', 'Test Entry')",
            rusqlite::params![feed_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    /// Helper: create a provider in the DB.
    fn setup_provider(pool: &Pool<SqliteConnectionManager>) -> i64 {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO providers (name, base_url, api_key_ref, is_default) VALUES ('TestProvider', 'https://test.com/v1', 'key', 1)",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn test_create_and_find_run() {
        let (repo, pool) = setup_repo();
        let entry_id = setup_entry(&pool);
        let provider_id = setup_provider(&pool);

        let run = repo
            .create(&NewAgentRun {
                entry_id,
                provider_id,
                task_kind: "summary".to_string(),
                target_language: "中文".to_string(),
                detail_level: Some("medium".to_string()),
            })
            .unwrap();

        assert_eq!(run.entry_id, entry_id);
        assert_eq!(run.task_kind, "summary");
        assert_eq!(run.phase, "idle");
    }

    #[test]
    fn test_mark_started() {
        let (repo, pool) = setup_repo();
        let entry_id = setup_entry(&pool);
        let provider_id = setup_provider(&pool);

        let run = repo
            .create(&NewAgentRun {
                entry_id,
                provider_id,
                task_kind: "translation".to_string(),
                target_language: "English".to_string(),
                detail_level: None,
            })
            .unwrap();

        repo.mark_started(run.id).unwrap();

        let updated = repo.find_by_id(run.id).unwrap().unwrap();
        assert_eq!(updated.phase, "running");
        assert!(updated.started_at.is_some());
    }

    #[test]
    fn test_mark_completed() {
        let (repo, pool) = setup_repo();
        let entry_id = setup_entry(&pool);
        let provider_id = setup_provider(&pool);

        let run = repo
            .create(&NewAgentRun {
                entry_id,
                provider_id,
                task_kind: "summary".to_string(),
                target_language: "中文".to_string(),
                detail_level: Some("short".to_string()),
            })
            .unwrap();

        repo.mark_completed(run.id, "这是摘要", Some(50), Some(30))
            .unwrap();

        let updated = repo.find_by_id(run.id).unwrap().unwrap();
        assert_eq!(updated.phase, "succeeded");
        assert_eq!(updated.output_text.unwrap(), "这是摘要");
    }

    #[test]
    fn test_mark_failed() {
        let (repo, pool) = setup_repo();
        let entry_id = setup_entry(&pool);
        let provider_id = setup_provider(&pool);

        let run = repo
            .create(&NewAgentRun {
                entry_id,
                provider_id,
                task_kind: "summary".to_string(),
                target_language: "中文".to_string(),
                detail_level: None,
            })
            .unwrap();

        repo.mark_failed(run.id, "API 调用超时").unwrap();

        let updated = repo.find_by_id(run.id).unwrap().unwrap();
        assert_eq!(updated.phase, "failed");
        assert!(updated.error_message.unwrap().contains("超时"));
    }

    #[test]
    fn test_find_latest_by_entry_and_task() {
        let (repo, pool) = setup_repo();
        let entry_id = setup_entry(&pool);
        let provider_id = setup_provider(&pool);

        // Create two runs for the same entry
        repo.create(&NewAgentRun {
            entry_id,
            provider_id,
            task_kind: "summary".to_string(),
            target_language: "中文".to_string(),
            detail_level: Some("short".to_string()),
        })
        .unwrap();

        repo.create(&NewAgentRun {
            entry_id,
            provider_id,
            task_kind: "summary".to_string(),
            target_language: "中文".to_string(),
            detail_level: Some("medium".to_string()),
        })
        .unwrap();

        let latest = repo
            .find_latest_by_entry_and_task(entry_id, "summary")
            .unwrap()
            .unwrap();
        assert_eq!(latest.detail_level.unwrap(), "medium");
    }

    #[test]
    fn test_list_by_entry() {
        let (repo, pool) = setup_repo();
        let entry_id = setup_entry(&pool);
        let provider_id = setup_provider(&pool);

        repo.create(&NewAgentRun {
            entry_id,
            provider_id,
            task_kind: "summary".to_string(),
            target_language: "中文".to_string(),
            detail_level: None,
        })
        .unwrap();

        repo.create(&NewAgentRun {
            entry_id,
            provider_id,
            task_kind: "translation".to_string(),
            target_language: "English".to_string(),
            detail_level: None,
        })
        .unwrap();

        let runs = repo.list_by_entry(entry_id).unwrap();
        assert_eq!(runs.len(), 2);
    }

    #[test]
    fn test_mark_cancelled() {
        let (repo, pool) = setup_repo();
        let entry_id = setup_entry(&pool);
        let provider_id = setup_provider(&pool);

        let run = repo
            .create(&NewAgentRun {
                entry_id,
                provider_id,
                task_kind: "summary".to_string(),
                target_language: "中文".to_string(),
                detail_level: None,
            })
            .unwrap();

        repo.mark_cancelled(run.id).unwrap();

        let updated = repo.find_by_id(run.id).unwrap().unwrap();
        assert_eq!(updated.phase, "cancelled");
    }
}
