use rusqlite::params;

use crate::db::model::Content;
use crate::db::DbPool;
use crate::db::error::RepositoryError;

/// Data access layer for the `contents` table.
pub struct ContentRepository {
    pool: DbPool,
}

impl ContentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn insert_raw(&self, entry_id: i64, raw_html: &str) -> Result<Content, RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO contents (entry_id, raw_html) VALUES (?1, ?2)",
            params![entry_id, raw_html],
        )?;
        self.find_by_entry_id(entry_id)?
            .ok_or(RepositoryError::NotFound("Content not found after insert".into()))
    }

    /// Insert or replace raw HTML row for an entry.
    /// If a row already exists for this entry_id, update its raw_html.
    pub fn upsert_raw(&self, entry_id: i64, raw_html: &str) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO contents (entry_id, raw_html) VALUES (?1, ?2)
             ON CONFLICT(entry_id) DO UPDATE SET raw_html = excluded.raw_html, updated_at = datetime('now')",
            params![entry_id, raw_html],
        )?;
        Ok(())
    }

    pub fn find_by_entry_id(&self, entry_id: i64) -> Result<Option<Content>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, entry_id, raw_html, cleaned_html, cleaned_markdown,
                    rendered_html, readability_version, created_at, updated_at
             FROM contents WHERE entry_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![entry_id], map_content)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn update_cleaned(
        &self,
        entry_id: i64,
        cleaned_html: &str,
        cleaned_markdown: &str,
        rendered_html: &str,
        readability_version: i32,
    ) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE contents
             SET cleaned_html = ?1, cleaned_markdown = ?2, rendered_html = ?3,
                 readability_version = ?4, updated_at = datetime('now')
             WHERE entry_id = ?5",
            params![cleaned_html, cleaned_markdown, rendered_html, readability_version, entry_id],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Content for entry_id={} not found", entry_id
            )));
        }
        Ok(())
    }
}

fn map_content(row: &rusqlite::Row<'_>) -> rusqlite::Result<Content> {
    Ok(Content {
        id: row.get(0)?,
        entry_id: row.get(1)?,
        raw_html: row.get(2)?,
        cleaned_html: row.get(3)?,
        cleaned_markdown: row.get(4)?,
        rendered_html: row.get(5)?,
        readability_version: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db_pool;
    use crate::db::repository::{EntryRepository, FeedRepository};
    use crate::db::model::NewEntry;

    fn setup() -> (ContentRepository, i64) {
        let pool = open_test_db_pool().expect("Failed to create test pool");
        let feed_repo = FeedRepository::new(pool.clone());
        let feed = feed_repo.insert("https://ct.example.com/feed", "Content Test").expect("feed insert");

        let entry_repo = EntryRepository::new(pool.clone());
        let entry = entry_repo.insert_or_ignore(&NewEntry {
            feed_id: feed.id,
            guid: "content-test-guid".into(),
            title: "Content Test Entry".into(),
            author: "".into(), link: "".into(), summary: "".into(),
            published_at: None, updated_at: None,
        }).expect("entry insert").expect("duplicate");

        (ContentRepository::new(pool), entry.id)
    }

    #[test]
    fn test_insert_content_success() {
        let (repo, entry_id) = setup();
        let content = repo.insert_raw(entry_id, "<html><body><p>Hello</p></body></html>")
            .expect("insert failed");
        assert!(content.id > 0);
        assert_eq!(content.entry_id, entry_id);
        assert!(content.raw_html.contains("Hello"));
        assert!(content.cleaned_html.is_none());
    }

    #[test]
    fn test_update_cleaned_content_persists() {
        let (repo, entry_id) = setup();
        repo.insert_raw(entry_id, "<html><p>Raw</p></html>").expect("insert failed");
        repo.update_cleaned(entry_id, "<p>Clean</p>", "Clean MD", "<p>Rendered</p>", 1)
            .expect("update failed");
        let content = repo.find_by_entry_id(entry_id).expect("find failed").expect("not found");
        assert_eq!(content.cleaned_html.as_deref(), Some("<p>Clean</p>"));
        assert_eq!(content.rendered_html.as_deref(), Some("<p>Rendered</p>"));
        assert!(content.updated_at.is_some());
    }

    #[test]
    fn test_find_nonexistent_content_returns_none() {
        let (repo, _) = setup();
        let result = repo.find_by_entry_id(9999).expect("find failed");
        assert!(result.is_none());
    }
}
