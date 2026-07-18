use rusqlite::params;

use crate::db::model::{Feed, FeedSummary};
use crate::db::DbPool;
use crate::db::error::RepositoryError;

/// Data access layer for the `feeds` table.
pub struct FeedRepository {
    pool: DbPool,
}

impl FeedRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn insert(&self, url: &str, title: &str) -> Result<Feed, RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO feeds (url, title) VALUES (?1, ?2)",
            params![url, title],
        )?;
        let id = conn.last_insert_rowid();
        self.find_by_id(id)?
            .ok_or(RepositoryError::NotFound("Feed not found after insert".into()))
    }

    pub fn insert_full(
        &self, url: &str, title: &str, description: &str, link: &str, feed_type: &str,
    ) -> Result<Feed, RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO feeds (url, title, description, link, feed_type) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![url, title, description, link, feed_type],
        )?;
        let id = conn.last_insert_rowid();
        self.find_by_id(id)?
            .ok_or(RepositoryError::NotFound("Feed not found after insert_full".into()))
    }

    pub fn find_by_id(&self, id: i64) -> Result<Option<Feed>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, url, title, description, link, feed_type, last_synced_at, created_at
             FROM feeds WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], map_feed)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn find_by_url(&self, url: &str) -> Result<Option<Feed>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, url, title, description, link, feed_type, last_synced_at, created_at
             FROM feeds WHERE url = ?1",
        )?;
        let mut rows = stmt.query_map(params![url], map_feed)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn find_all_with_unread_count(&self) -> Result<Vec<FeedSummary>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT f.id, f.title,
                    (SELECT COUNT(*) FROM entries e WHERE e.feed_id = f.id AND e.is_read = 0) AS unread_count
             FROM feeds f ORDER BY f.title COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FeedSummary { id: row.get(0)?, title: row.get(1)?, unread_count: row.get(2)? })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_all(&self) -> Result<Vec<Feed>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, url, title, description, link, feed_type, last_synced_at, created_at
             FROM feeds ORDER BY title COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], map_feed)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn update_sync_time(&self, id: i64, synced_at: &str) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE feeds SET last_synced_at = ?1 WHERE id = ?2",
            params![synced_at, id],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Feed id={} not found", id)));
        }
        Ok(())
    }

    pub fn update_title(&self, id: i64, title: &str) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE feeds SET title = ?1 WHERE id = ?2",
            params![title, id],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Feed id={} not found", id)));
        }
        Ok(())
    }

    pub fn update_title_and_link(&self, id: i64, title: &str, link: &str) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE feeds SET title = ?1, link = ?2 WHERE id = ?3",
            params![title, link, id],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Feed id={} not found", id)));
        }
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Feed id={} not found", id)));
        }
        Ok(())
    }
}

fn map_feed(row: &rusqlite::Row<'_>) -> rusqlite::Result<Feed> {
    Ok(Feed {
        id: row.get(0)?,
        url: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        link: row.get(4)?,
        feed_type: row.get(5)?,
        last_synced_at: row.get(6)?,
        created_at: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db_pool;

    fn new_repo() -> FeedRepository {
        let pool = open_test_db_pool().expect("Failed to create test pool");
        FeedRepository::new(pool)
    }

    #[test]
    fn test_insert_feed_success() {
        let repo = new_repo();
        let feed = repo.insert("https://example.com/feed.xml", "Example Blog").expect("insert failed");
        assert!(feed.id > 0);
        assert_eq!(feed.url, "https://example.com/feed.xml");
    }

    #[test]
    fn test_insert_duplicate_url_returns_error() {
        let repo = new_repo();
        repo.insert("https://dup.example.com/rss", "First").expect("first insert failed");
        let result = repo.insert("https://dup.example.com/rss", "Second");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_by_url_not_found_returns_none() {
        let repo = new_repo();
        let result = repo.find_by_url("https://no.example.com").expect("query failed");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_all_with_unread_count_initial_zero() {
        let repo = new_repo();
        repo.insert("https://a.example.com/feed", "Feed A").expect("insert failed");
        let summaries = repo.find_all_with_unread_count().expect("query failed");
        assert_eq!(summaries[0].unread_count, 0);
    }

    #[test]
    fn test_delete_feed_cascades_to_entries() {
        let repo = new_repo();
        let feed = repo.insert("https://del.example.com/rss", "Delete Test").expect("insert failed");
        let conn = repo.pool.get().expect("get conn failed");
        conn.execute(
            "INSERT INTO entries (feed_id, guid, title) VALUES (?1, ?2, ?3)",
            params![feed.id, "guid-1", "Entry Title"],
        ).expect("entry insert failed");

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM entries WHERE feed_id = ?1", params![feed.id], |r| r.get(0),
        ).expect("count failed");
        assert_eq!(count, 1);

        repo.delete(feed.id).expect("delete failed");

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM entries WHERE feed_id = ?1", params![feed.id], |r| r.get(0),
        ).expect("count failed");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_delete_nonexistent_returns_error() {
        let repo = new_repo();
        assert!(repo.delete(9999).is_err());
    }
}
