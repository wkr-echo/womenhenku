use crate::db::model::Content;
use crate::db::repository::ContentRepository;
use crate::db::DbPool;
use crate::reader::pipeline;

/// Errors from the reader service layer.
#[derive(Debug, thiserror::Error)]
pub enum ReaderServiceError {
    #[error("Pipeline error: {0}")]
    Pipeline(#[from] pipeline::PipelineError),

    #[error("Database error: {0}")]
    Database(#[from] crate::db::error::RepositoryError),
}

/// Orchestrates the reader pipeline and persists results to the database.
pub struct ReaderService {
    pool: DbPool,
}

impl ReaderService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Process an entry's raw HTML through the full reader pipeline
    /// and store the results in the contents table.
    ///
    /// Expects that a content row already exists for this entry
    /// (created during feed ingestion with raw_html populated).
    /// If it doesn't exist, creates one first with empty raw_html.
    pub fn process_entry(&self, entry_id: i64, url: &str) -> Result<Content, ReaderServiceError> {
        let repo = ContentRepository::new(self.pool.clone());

        // Get existing raw HTML or create a placeholder
        let raw_html = match repo.find_by_entry_id(entry_id)? {
            Some(c) => c.raw_html,
            None => {
                tracing::warn!(
                    "No content row for entry_id={}, creating placeholder",
                    entry_id
                );
                repo.insert_raw(entry_id, "")?;
                String::new()
            }
        };

        if raw_html.is_empty() {
            return Err(ReaderServiceError::Database(
                crate::db::error::RepositoryError::InvalidInput(format!(
                    "No raw HTML for entry_id={}",
                    entry_id
                )),
            ));
        }

        let output = pipeline::run_full_pipeline(&raw_html, url)?;

        repo.update_cleaned(
            entry_id,
            &output.cleaned_html,
            &output.markdown,
            &output.rendered_html,
            1, // readability_version
        )?;

        repo.find_by_entry_id(entry_id)?
            .ok_or(ReaderServiceError::Database(
                crate::db::error::RepositoryError::NotFound(format!(
                    "Content for entry_id={} not found after update",
                    entry_id
                )),
            ))
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
