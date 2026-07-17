use crate::db::error::RepositoryError;
use crate::db::model::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

/// Repository for managing LLM providers.
/// Mirrors Mercury's provider management functionality.
pub struct ProviderRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl ProviderRepository {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    /// Insert a new provider.
    pub fn insert(&self, provider: &NewProvider) -> Result<Provider, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "INSERT INTO providers (name, base_url, api_key_ref, is_default) VALUES (?1, ?2, ?3, ?4)",
        )?;

        stmt.execute(rusqlite::params![
            provider.name,
            provider.base_url,
            provider.api_key_ref,
            provider.is_default as i32,
        ])?;

        let id = conn.last_insert_rowid();
        self.find_by_id(id)?.ok_or(RepositoryError::NotFound("Provider not found".to_string()))
    }

    /// Find a provider by its ID.
    pub fn find_by_id(&self, id: i64) -> Result<Option<Provider>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, name, base_url, api_key_ref, is_default, created_at, updated_at FROM providers WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(Provider {
                id: row.get(0)?,
                name: row.get(1)?,
                base_url: row.get(2)?,
                api_key_ref: row.get(3)?,
                is_default: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        match rows.next() {
            Some(Ok(provider)) => Ok(Some(provider)),
            _ => Ok(None),
        }
    }

    /// List all providers.
    pub fn list_all(&self) -> Result<Vec<Provider>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, name, base_url, api_key_ref, is_default, created_at, updated_at FROM providers ORDER BY name",
        )?;

        let providers = stmt
            .query_map([], |row| {
                Ok(Provider {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    base_url: row.get(2)?,
                    api_key_ref: row.get(3)?,
                    is_default: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(providers)
    }

    /// Get the default provider.
    pub fn find_default(&self) -> Result<Option<Provider>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, name, base_url, api_key_ref, is_default, created_at, updated_at FROM providers WHERE is_default = 1 LIMIT 1",
        )?;

        let mut rows = stmt.query_map([], |row| {
            Ok(Provider {
                id: row.get(0)?,
                name: row.get(1)?,
                base_url: row.get(2)?,
                api_key_ref: row.get(3)?,
                is_default: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        match rows.next() {
            Some(Ok(provider)) => Ok(Some(provider)),
            _ => Ok(None),
        }
    }

    /// Update a provider. Only provided fields will be updated.
    pub fn update(&self, id: i64, update: &UpdateProvider) -> Result<Provider, RepositoryError> {
        let existing = self
            .find_by_id(id)?
            .ok_or(RepositoryError::NotFound("Provider not found".to_string()))?;

        let name = update.name.clone().unwrap_or(existing.name);
        let base_url = update.base_url.clone().unwrap_or(existing.base_url);
        let api_key_ref = update.api_key_ref.clone().unwrap_or(existing.api_key_ref);
        let is_default = update.is_default.unwrap_or(existing.is_default);

        // If setting this provider as default, unset any existing default first
        if is_default && !existing.is_default {
            self.clear_default()?;
        }

        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "UPDATE providers SET name = ?1, base_url = ?2, api_key_ref = ?3, is_default = ?4, updated_at = datetime('now') WHERE id = ?5",
        )?;

        stmt.execute(rusqlite::params![name, base_url, api_key_ref, is_default as i32, id])?;

        self.find_by_id(id)?.ok_or(RepositoryError::NotFound("Provider not found".to_string()))
    }

    /// Delete a provider by ID.
    pub fn delete(&self, id: i64) -> Result<bool, RepositoryError> {
        let conn = self.pool.get()?;
        let rows = conn.execute("DELETE FROM providers WHERE id = ?1", rusqlite::params![id])?;
        Ok(rows > 0)
    }

    /// Clear the default flag on all providers.
    fn clear_default(&self) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE providers SET is_default = 0, updated_at = datetime('now') WHERE is_default = 1",
            [],
        )?;
        Ok(())
    }

    // === Provider Model Methods ===

    /// Add a model to a provider.
    pub fn add_model(&self, model: &NewProviderModel) -> Result<ProviderModel, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "INSERT INTO provider_models (provider_id, model_name, is_default) VALUES (?1, ?2, ?3)",
        )?;

        stmt.execute(rusqlite::params![
            model.provider_id,
            model.model_name,
            model.is_default as i32,
        ])?;

        let id = conn.last_insert_rowid();
        self.find_model_by_id(id)?.ok_or(RepositoryError::NotFound("Provider not found".to_string()))
    }

    /// List all models for a given provider.
    pub fn list_models(&self, provider_id: i64) -> Result<Vec<ProviderModel>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, provider_id, model_name, is_default, created_at FROM provider_models WHERE provider_id = ?1 ORDER BY model_name",
        )?;

        let models = stmt
            .query_map(rusqlite::params![provider_id], |row| {
                Ok(ProviderModel {
                    id: row.get(0)?,
                    provider_id: row.get(1)?,
                    model_name: row.get(2)?,
                    is_default: row.get::<_, i32>(3)? != 0,
                    created_at: row.get(4)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(models)
    }

    /// Delete a model.
    pub fn delete_model(&self, id: i64) -> Result<bool, RepositoryError> {
        let conn = self.pool.get()?;
        let rows =
            conn.execute("DELETE FROM provider_models WHERE id = ?1", rusqlite::params![id])?;
        Ok(rows > 0)
    }

    fn find_model_by_id(&self, id: i64) -> Result<Option<ProviderModel>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, provider_id, model_name, is_default, created_at FROM provider_models WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(ProviderModel {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                model_name: row.get(2)?,
                is_default: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
            })
        })?;

        match rows.next() {
            Some(Ok(model)) => Ok(Some(model)),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_repo() -> (ProviderRepository, Pool<SqliteConnectionManager>) {
        let pool = db::open_test_db_pool().expect("Failed to open test DB");
        let repo = ProviderRepository::new(pool.clone());
        (repo, pool)
    }

    #[test]
    fn test_insert_and_find_provider() {
        let (repo, _pool) = setup_repo();

        let new = NewProvider {
            name: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key_ref: "key_openai".to_string(),
            is_default: true,
        };

        let provider = repo.insert(&new).expect("Failed to insert provider");
        assert_eq!(provider.name, "OpenAI");
        assert!(provider.is_default);
    }

    #[test]
    fn test_list_providers() {
        let (repo, _pool) = setup_repo();

        repo.insert(&NewProvider {
            name: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key_ref: "key1".to_string(),
            is_default: false,
        })
        .unwrap();

        repo.insert(&NewProvider {
            name: "DeepSeek".to_string(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            api_key_ref: "key2".to_string(),
            is_default: true,
        })
        .unwrap();

        let providers = repo.list_all().unwrap();
        assert_eq!(providers.len(), 2);
    }

    #[test]
    fn test_find_default() {
        let (repo, _pool) = setup_repo();

        repo.insert(&NewProvider {
            name: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key_ref: "key1".to_string(),
            is_default: false,
        })
        .unwrap();

        let default = repo.find_default().unwrap();
        assert!(default.is_none()); // No default yet

        repo.insert(&NewProvider {
            name: "DeepSeek".to_string(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            api_key_ref: "key2".to_string(),
            is_default: true,
        })
        .unwrap();

        let default = repo.find_default().unwrap();
        assert!(default.is_some());
        assert_eq!(default.unwrap().name, "DeepSeek");
    }

    #[test]
    fn test_update_provider() {
        let (repo, _pool) = setup_repo();

        let provider = repo
            .insert(&NewProvider {
                name: "Old Name".to_string(),
                base_url: "https://old.url/v1".to_string(),
                api_key_ref: "key".to_string(),
                is_default: true,
            })
            .unwrap();

        let updated = repo
            .update(
                provider.id,
                &UpdateProvider {
                    name: Some("New Name".to_string()),
                    base_url: None,
                    api_key_ref: None,
                    is_default: None,
                },
            )
            .unwrap();

        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.base_url, "https://old.url/v1");
    }

    #[test]
    fn test_delete_provider() {
        let (repo, _pool) = setup_repo();

        let provider = repo
            .insert(&NewProvider {
                name: "To Delete".to_string(),
                base_url: "https://x.url/v1".to_string(),
                api_key_ref: "key".to_string(),
                is_default: false,
            })
            .unwrap();

        let deleted = repo.delete(provider.id).unwrap();
        assert!(deleted);

        let found = repo.find_by_id(provider.id).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_add_and_list_models() {
        let (repo, _pool) = setup_repo();

        let provider = repo
            .insert(&NewProvider {
                name: "OpenAI".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                api_key_ref: "key".to_string(),
                is_default: true,
            })
            .unwrap();

        repo.add_model(&NewProviderModel {
            provider_id: provider.id,
            model_name: "gpt-4".to_string(),
            is_default: true,
        })
        .unwrap();

        repo.add_model(&NewProviderModel {
            provider_id: provider.id,
            model_name: "gpt-3.5-turbo".to_string(),
            is_default: false,
        })
        .unwrap();

        let models = repo.list_models(provider.id).unwrap();
        assert_eq!(models.len(), 2);
    }

    #[test]
    fn test_default_switch_clears_previous() {
        let (repo, _pool) = setup_repo();

        let p1 = repo
            .insert(&NewProvider {
                name: "First".to_string(),
                base_url: "https://first.url/v1".to_string(),
                api_key_ref: "k1".to_string(),
                is_default: true,
            })
            .unwrap();
        assert!(p1.is_default);

        let p2 = repo
            .insert(&NewProvider {
                name: "Second".to_string(),
                base_url: "https://second.url/v1".to_string(),
                api_key_ref: "k2".to_string(),
                is_default: true,
            })
            .unwrap();
        assert!(p2.is_default);

        // First should no longer be default (because of UNIQUE index on is_default=1)
        let p1_after = repo.find_by_id(p1.id).unwrap().unwrap();
        assert!(!p1_after.is_default);
    }
}
