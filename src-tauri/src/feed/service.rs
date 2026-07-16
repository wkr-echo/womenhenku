use std::time::Duration;

use crate::db::model::{Feed, FeedSummary};
use crate::db::repository::{EntryRepository, FeedRepository};
use crate::db::DbPool;
use crate::feed::parser::{self, ParsedFeed};

/// Result type for feed service operations.
pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Feed parse error: {0}")]
    Parse(#[from] parser::ParseError),

    #[error("Database error: {0}")]
    Database(#[from] crate::db::error::RepositoryError),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Feed not found: {0}")]
    NotFound(String),
}

/// Core service for feed lifecycle: add, remove, refresh, list.
pub struct FeedService {
    pool: DbPool,
    client: reqwest::Client,
}

impl FeedService {
    pub fn new(pool: DbPool) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("womenhenku/0.1 (RSS Reader)")
            .build()
            .expect("Failed to build reqwest client");

        Self { pool, client }
    }

    // ---- Public API ----

    /// Add a new feed subscription: fetch URL, parse, store feed + entries.
    /// Returns the created Feed.
    pub fn add_feed(&self, url: &str) -> ServiceResult<Feed> {
        // Validate URL
        self.validate_url(url)?;

        let parsed = self.fetch_and_parse(url)?;

        let feed_repo = FeedRepository::new(self.pool.clone());
        let entry_repo = EntryRepository::new(self.pool.clone());

        // Check if feed URL already exists
        if let Some(existing) = feed_repo.find_by_url(url)? {
            return Err(ServiceError::InvalidUrl(format!(
                "Feed already subscribed: {} (id={})",
                existing.title, existing.id
            )));
        }

        // Insert feed
        let feed = feed_repo.insert_full(
            url,
            &parsed.title,
            &parsed.description,
            &parsed.link,
            &parsed.feed_type,
        )?;

        // Insert entries (deduplicated by guid)
        let mut inserted = 0;
        for mut entry in parsed.entries {
            entry.feed_id = feed.id;
            if entry_repo.insert_or_ignore(&entry)?.is_some() {
                inserted += 1;
            }
        }

        // Update sync timestamp
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        feed_repo.update_sync_time(feed.id, &now)?;

        tracing::info!(
            "Feed added: {} ({}), {} new entries",
            feed.title, url, inserted
        );

        Ok(feed)
    }

    /// Remove a feed and all its entries + contents (cascade).
    pub fn remove_feed(&self, id: i64) -> ServiceResult<()> {
        let feed_repo = FeedRepository::new(self.pool.clone());
        // find_by_id to verify it exists before delete
        feed_repo.find_by_id(id)?.ok_or(ServiceError::NotFound(format!("Feed id={}", id)))?;
        feed_repo.delete(id)?;
        tracing::info!("Feed removed: id={}", id);
        Ok(())
    }

    /// Refresh a single feed: re-fetch, parse, insert new entries only.
    /// Returns the number of new entries found.
    pub fn refresh_feed(&self, id: i64) -> ServiceResult<usize> {
        let feed_repo = FeedRepository::new(self.pool.clone());
        let entry_repo = EntryRepository::new(self.pool.clone());

        let feed = feed_repo
            .find_by_id(id)?
            .ok_or(ServiceError::NotFound(format!("Feed id={}", id)))?;

        let parsed = self.fetch_and_parse(&feed.url)?;

        // Update feed metadata in case title/description changed
        feed_repo.update_title(id, &parsed.title)?;

        let mut new_count = 0;
        for mut entry in parsed.entries {
            entry.feed_id = id;
            if entry_repo.insert_or_ignore(&entry)?.is_some() {
                new_count += 1;
            }
        }

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        feed_repo.update_sync_time(id, &now)?;

        tracing::info!("Feed refreshed: id={}, {} new entries", id, new_count);
        Ok(new_count)
    }

    /// List all feeds with unread entry counts.
    pub fn list_feeds(&self) -> ServiceResult<Vec<FeedSummary>> {
        let feed_repo = FeedRepository::new(self.pool.clone());
        Ok(feed_repo.find_all_with_unread_count()?)
    }

    /// Get a single feed by id.
    pub fn get_feed(&self, id: i64) -> ServiceResult<Feed> {
        let feed_repo = FeedRepository::new(self.pool.clone());
        feed_repo.find_by_id(id)?.ok_or(ServiceError::NotFound(format!("Feed id={}", id)))
    }

    // ---- Internal helpers ----

    /// Fetch a URL and parse the response body as a feed.
    fn fetch_and_parse(&self, url: &str) -> ServiceResult<ParsedFeed> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        let bytes = rt.block_on(async {
            self.client
                .get(url)
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await
        })?;

        let parsed = parser::parse_feed(&bytes, url)?;
        Ok(parsed)
    }

    /// Validate that a URL is well-formed and has an HTTP(S) scheme.
    fn validate_url(&self, url: &str) -> ServiceResult<()> {
        let parsed = url::Url::parse(url).map_err(|_| {
            ServiceError::InvalidUrl(format!("Malformed URL: {}", url))
        })?;
        match parsed.scheme() {
            "http" | "https" => Ok(()),
            other => Err(ServiceError::InvalidUrl(format!(
                "Unsupported URL scheme '{}': must be http or https",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db_pool;

    fn setup_service() -> FeedService {
        let pool = open_test_db_pool().expect("Failed to create test pool");
        FeedService::new(pool)
    }

    #[test]
    fn test_validate_url_accepts_https() {
        let svc = setup_service();
        assert!(svc.validate_url("https://example.com/feed.xml").is_ok());
    }

    #[test]
    fn test_validate_url_rejects_ftp() {
        let svc = setup_service();
        let result = svc.validate_url("ftp://example.com/feed.xml");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_url_rejects_malformed() {
        let svc = setup_service();
        let result = svc.validate_url("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_feeds_initially_empty() {
        let svc = setup_service();
        let feeds = svc.list_feeds().expect("list_feeds failed");
        assert!(feeds.is_empty());
    }
}
