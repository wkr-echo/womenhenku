use std::time::Duration;

use crate::db::model::Content;
use crate::db::repository::ContentRepository;
use crate::db::DbPool;
use crate::reader::pipeline;

/// Errors from the reader service layer.
#[derive(Debug, thiserror::Error)]
pub enum ReaderServiceError {
    #[error("Pipeline error: {0}")]
    Pipeline(#[from] pipeline::PipelineError),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] crate::db::error::RepositoryError),

    #[error("No article URL for entry_id={0}")]
    NoUrl(i64),
}

/// Orchestrates the reader pipeline and persists results to the database.
pub struct ReaderService {
    pool: DbPool,
    client: reqwest::Client,
}

impl ReaderService {
    pub fn new(pool: DbPool) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("platinum/0.2 (RSS Reader)")
            .build()
            .expect("Failed to build reqwest client");
        Self { pool, client }
    }

    /// Process an entry through the full reader pipeline and store results.
    ///
    /// Three-tier logic:
    /// 1. Cache hit (cleaned_html exists) → return immediately.
    /// 2. raw_html exists in DB → run pipeline from cached raw HTML.
    /// 3. No raw_html → fetch article from URL, store, then run pipeline.
    pub fn process_entry(&self, entry_id: i64, url: &str) -> Result<Content, ReaderServiceError> {
        let repo = ContentRepository::new(self.pool.clone());

        tracing::debug!("process_entry: entry_id={}, url={}", entry_id, url);

        // Tier 1: Return cached content if already processed
        if let Some(c) = repo.find_by_entry_id(entry_id)? {
            if c.cleaned_html.as_ref().is_some_and(|h| !h.is_empty()) {
                tracing::debug!("Content cache hit for entry_id={}", entry_id);
                return Ok(c);
            }
        }

        // Tier 2 & 3: Get raw HTML (from DB cache or fetch from URL)
        let raw_html = match repo.find_by_entry_id(entry_id)? {
            Some(c) if !c.raw_html.is_empty() => {
                tracing::debug!("Using cached raw_html for entry_id={}", entry_id);
                c.raw_html
            }
            _ => {
                if url.is_empty() {
                    tracing::warn!("No article URL for entry_id={}, cannot fetch", entry_id);
                    return Err(ReaderServiceError::NoUrl(entry_id));
                }
                tracing::info!("Fetching article from {} for entry_id={}", url, entry_id);
                let html = self.fetch_article(url)?;
                tracing::info!("Fetched {} chars from {}", html.len(), url);
                repo.upsert_raw(entry_id, &html)?;
                html
            }
        };

        // Run the pipeline
        let output = pipeline::run_full_pipeline(&raw_html, url)?;

        // Store cleaned results
        repo.update_cleaned(
            entry_id,
            &output.cleaned_html,
            &output.markdown,
            &output.rendered_html,
            1,
        )?;

        repo.find_by_entry_id(entry_id)?
            .ok_or(ReaderServiceError::Database(
                crate::db::error::RepositoryError::NotFound(format!(
                    "Content for entry_id={} not found after update",
                    entry_id
                )),
            ))
    }

    /// Fetch article HTML from URL.
    fn fetch_article(&self, url: &str) -> Result<String, ReaderServiceError> {
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

        // Try UTF-8 first, fall back to lossy conversion
        let text = String::from_utf8(bytes.to_vec())
            .unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).into_owned());

        tracing::debug!("Fetched article from {} ({} chars)", url, text.len());
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db_pool;
    use crate::db::repository::{EntryRepository, FeedRepository};
    use crate::db::model::NewEntry;

    #[test]
    fn test_process_entry_stores_pipeline_result() {
        let pool = open_test_db_pool().expect("Failed to create test pool");

        // Setup: create feed + entry + content
        let feed_repo = FeedRepository::new(pool.clone());
        let feed = feed_repo
            .insert("https://reader-test.example.com/feed", "Reader Test")
            .expect("feed insert");

        let entry_repo = EntryRepository::new(pool.clone());
        let entry = entry_repo
            .insert_or_ignore(&NewEntry {
                feed_id: feed.id,
                guid: "reader-test-guid".into(),
                title: "Reader Test Entry".into(),
                author: "".into(),
                link: "https://example.com/test".into(),
                summary: "".into(),
                published_at: None,
                updated_at: None,
            })
            .expect("entry insert")
            .expect("duplicate");

        let content_repo = ContentRepository::new(pool.clone());
        let html = r#"<html><body>
            <nav>Menu</nav>
            <article>
                <h1>Test Article</h1>
                <p>This is a <strong>test</strong> paragraph.</p>
                <p>Second paragraph with a <a href="https://x.com">link</a>.</p>
            </article>
            <aside>Sidebar ad</aside>
        </body></html>"#;
        content_repo
            .insert_raw(entry.id, html)
            .expect("content insert");

        // Process
        let service = ReaderService::new(pool);
        let result = service
            .process_entry(entry.id, "https://example.com/test")
            .expect("process_entry failed");

        // Validate
        assert!(!result.cleaned_html.as_deref().unwrap_or("").is_empty());
        assert!(!result.rendered_html.as_deref().unwrap_or("").is_empty());
        assert!(!result.cleaned_html.as_deref().unwrap_or("").contains("nav"));
        assert!(result.updated_at.is_some());
    }
}
