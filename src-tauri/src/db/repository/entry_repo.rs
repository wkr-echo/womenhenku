use rusqlite::params;

use crate::db::model::{Entry, EntryListItem, EntryPage, NewEntry};
use crate::db::DbPool;
use crate::db::error::RepositoryError;

pub struct EntryRepository {
    pool: DbPool,
}

impl EntryRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn insert_or_ignore(&self, entry: &NewEntry) -> Result<Option<Entry>, RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "INSERT OR IGNORE INTO entries (feed_id, guid, title, author, link, summary, published_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.feed_id, entry.guid, entry.title, entry.author,
                entry.link, entry.summary, entry.published_at, entry.updated_at,
            ],
        )?;
        if affected == 0 {
            return Ok(None);
        }
        let id = conn.last_insert_rowid();
        self.find_by_id(id)?
            .ok_or(RepositoryError::NotFound("Entry not found after insert".into()))
            .map(Some)
    }

    pub fn find_by_id(&self, id: i64) -> Result<Option<Entry>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, guid, title, author, link, summary,
                    published_at, updated_at, is_read, is_starred, created_at
             FROM entries WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], map_entry)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn find_by_feed_and_guid(
        &self, feed_id: i64, guid: &str,
    ) -> Result<Option<Entry>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, guid, title, author, link, summary,
                    published_at, updated_at, is_read, is_starred, created_at
             FROM entries WHERE feed_id = ?1 AND guid = ?2",
        )?;
        let mut rows = stmt.query_map(params![feed_id, guid], map_entry)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn list_by_feed(
        &self, feed_id: i64, page: i32, page_size: i32, filter: Option<&str>,
    ) -> Result<EntryPage, RepositoryError> {
        let conn = self.pool.get()?;
        let offset = (page - 1) * page_size;
        let where_clause = match filter {
            Some("unread") => "WHERE e.feed_id = ?1 AND e.is_read = 0",
            Some("starred") => "WHERE e.feed_id = ?1 AND e.is_starred = 1",
            _ => "WHERE e.feed_id = ?1",
        };

        let count_sql = format!("SELECT COUNT(*) FROM entries e {}", where_clause);
        let total: i64 = conn.query_row(&count_sql, params![feed_id], |row| row.get(0))?;

        let list_sql = format!(
            "SELECT e.id, e.feed_id, e.title, e.author, e.summary, e.published_at, e.is_read
             FROM entries e {} ORDER BY COALESCE(e.published_at, e.created_at) DESC LIMIT ?2 OFFSET ?3",
            where_clause
        );
        let mut stmt = conn.prepare(&list_sql)?;
        let entries: Vec<EntryListItem> = stmt
            .query_map(params![feed_id, page_size, offset], |row| {
                Ok(EntryListItem {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    author: row.get(3)?,
                    summary: row.get::<_, String>(4).unwrap_or_default(),
                    published_at: row.get(5)?,
                    is_read: row.get::<_, i32>(6)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(EntryPage { entries, total, page, page_size })
    }

    pub fn list_all(
        &self, page: i32, page_size: i32, filter: Option<&str>,
    ) -> Result<EntryPage, RepositoryError> {
        let conn = self.pool.get()?;
        let offset = (page - 1) * page_size;
        let where_clause = match filter {
            Some("unread") => "WHERE e.is_read = 0",
            Some("starred") => "WHERE e.is_starred = 1",
            _ => "",
        };

        let count_sql = format!("SELECT COUNT(*) FROM entries e {}", where_clause);
        let total: i64 = conn.query_row(&count_sql, [], |row| row.get(0))?;

        let list_sql = format!(
            "SELECT e.id, e.feed_id, e.title, e.author, e.summary, e.published_at, e.is_read
             FROM entries e {} ORDER BY COALESCE(e.published_at, e.created_at) DESC LIMIT ?1 OFFSET ?2",
            where_clause
        );
        let mut stmt = conn.prepare(&list_sql)?;
        let entries: Vec<EntryListItem> = stmt
            .query_map(params![page_size, offset], |row| {
                Ok(EntryListItem {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    author: row.get(3)?,
                    summary: row.get::<_, String>(4).unwrap_or_default(),
                    published_at: row.get(5)?,
                    is_read: row.get::<_, i32>(6)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(EntryPage { entries, total, page, page_size })
    }

    /// Search entries by title and summary using FTS5 (Stage 2).
    /// Falls back to LIKE search if FTS table is not available.
    pub fn search(
        &self, query: &str, page: i32, page_size: i32,
    ) -> Result<EntryPage, RepositoryError> {
        let conn = self.pool.get()?;
        let offset = (page - 1) * page_size;

        let fts_available = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries_fts'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        if fts_available {
            let total: i64 = conn.query_row(
                "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH ?1",
                params![query],
                |row| row.get(0),
            )?;

            let mut stmt = conn.prepare(
                "SELECT e.id, e.feed_id, e.title, e.author, e.summary, e.published_at, e.is_read
                 FROM entries_fts f
                 JOIN entries e ON e.id = f.rowid
                 WHERE entries_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2 OFFSET ?3",
            )?;
            let entries: Vec<EntryListItem> = stmt
                .query_map(params![query, page_size, offset], |row| {
                    Ok(EntryListItem {
                        id: row.get(0)?,
                        feed_id: row.get(1)?,
                        title: row.get(2)?,
                        author: row.get(3)?,
                    summary: row.get::<_, String>(4).unwrap_or_default(),
                        published_at: row.get(5)?,
                        is_read: row.get::<_, i32>(6)? != 0,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(EntryPage { entries, total, page, page_size })
        } else {
            let like_pattern = format!("%{}%", query);
            let total: i64 = conn.query_row(
                "SELECT COUNT(*) FROM entries e WHERE e.title LIKE ?1 OR e.summary LIKE ?1",
                params![like_pattern],
                |row| row.get(0),
            )?;

            let mut stmt = conn.prepare(
                "SELECT e.id, e.feed_id, e.title, e.author, e.summary, e.published_at, e.is_read
                 FROM entries e
                 WHERE e.title LIKE ?1 OR e.summary LIKE ?1
                 ORDER BY COALESCE(e.published_at, e.created_at) DESC
                 LIMIT ?2 OFFSET ?3",
            )?;
            let entries: Vec<EntryListItem> = stmt
                .query_map(params![like_pattern, page_size, offset], |row| {
                    Ok(EntryListItem {
                        id: row.get(0)?,
                        feed_id: row.get(1)?,
                        title: row.get(2)?,
                        author: row.get(3)?,
                    summary: row.get::<_, String>(4).unwrap_or_default(),
                        published_at: row.get(5)?,
                        is_read: row.get::<_, i32>(6)? != 0,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(EntryPage { entries, total, page, page_size })
        }
    }

    pub fn mark_read(&self, id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute("UPDATE entries SET is_read = 1 WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Entry id={} not found", id)));
        }
        Ok(())
    }

    pub fn mark_unread(&self, id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute("UPDATE entries SET is_read = 0 WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Entry id={} not found", id)));
        }
        Ok(())
    }

    pub fn mark_all_read_in_feed(&self, feed_id: i64) -> Result<usize, RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE entries SET is_read = 1 WHERE feed_id = ?1 AND is_read = 0",
            params![feed_id],
        )?;
        Ok(affected)
    }

    pub fn list_by_tag(
        &self, tag_id: i64, page: i32, page_size: i32,
    ) -> Result<EntryPage, RepositoryError> {
        let conn = self.pool.get()?;
        let offset = (page - 1) * page_size;

        let count_sql = "SELECT COUNT(*) FROM entries e JOIN entry_tags et ON e.id = et.entry_id WHERE et.tag_id = ?1";
        let total: i64 = conn.query_row(count_sql, params![tag_id], |row| row.get(0))?;

        let list_sql = "SELECT e.id, e.feed_id, e.title, e.author, e.summary, e.published_at, e.is_read
                        FROM entries e
                        JOIN entry_tags et ON e.id = et.entry_id
                        WHERE et.tag_id = ?1
                        ORDER BY COALESCE(e.published_at, e.created_at) DESC
                        LIMIT ?2 OFFSET ?3";
        let mut stmt = conn.prepare(list_sql)?;
        let entries: Vec<EntryListItem> = stmt
            .query_map(params![tag_id, page_size, offset], |row| {
                Ok(EntryListItem {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    author: row.get(3)?,
                    summary: row.get::<_, String>(4).unwrap_or_default(),
                    published_at: row.get(5)?,
                    is_read: row.get::<_, i32>(6)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(EntryPage { entries, total, page, page_size })
    }

    pub fn list_by_tags(
        &self, tag_ids: &[i64], match_mode: &str, page: i32, page_size: i32,
    ) -> Result<EntryPage, RepositoryError> {
        let conn = self.pool.get()?;
        let offset = (page - 1) * page_size;

        if tag_ids.is_empty() {
            return self.list_all(page, page_size, None);
        }

        let placeholders: Vec<String> = tag_ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
        let placeholders_str = placeholders.join(", ");

        let (count_sql, list_sql) = if match_mode == "and" {
            let tag_count = tag_ids.len() as i64;
            (
                format!("SELECT COUNT(*) FROM entries e JOIN entry_tags et ON e.id = et.entry_id WHERE et.tag_id IN ({}) GROUP BY e.id HAVING COUNT(DISTINCT et.tag_id) = {}", placeholders_str, tag_count),
                format!("SELECT e.id, e.feed_id, e.title, e.author, e.summary, e.published_at, e.is_read FROM entries e JOIN entry_tags et ON e.id = et.entry_id WHERE et.tag_id IN ({}) GROUP BY e.id HAVING COUNT(DISTINCT et.tag_id) = {} ORDER BY COALESCE(e.published_at, e.created_at) DESC LIMIT ?{} OFFSET ?{}", placeholders_str, tag_count, tag_ids.len() + 1, tag_ids.len() + 2),
            )
        } else {
            (
                format!("SELECT COUNT(DISTINCT e.id) FROM entries e JOIN entry_tags et ON e.id = et.entry_id WHERE et.tag_id IN ({})", placeholders_str),
                format!("SELECT DISTINCT e.id, e.feed_id, e.title, e.author, e.summary, e.published_at, e.is_read FROM entries e JOIN entry_tags et ON e.id = et.entry_id WHERE et.tag_id IN ({}) ORDER BY COALESCE(e.published_at, e.created_at) DESC LIMIT ?{} OFFSET ?{}", placeholders_str, tag_ids.len() + 1, tag_ids.len() + 2),
            )
        };

        let mut params: Vec<rusqlite::types::ToSqlOutput<'_>> = tag_ids.iter().map(|&id| id.into()).collect();
        let total: i64 = conn.query_row(&count_sql, rusqlite::params_from_iter(&params), |row| row.get(0))?;

        params.push(page_size.into());
        params.push(offset.into());

        let mut stmt = conn.prepare(&list_sql)?;
        let entries: Vec<EntryListItem> = stmt
            .query_map(rusqlite::params_from_iter(&params), |row| {
                Ok(EntryListItem {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    author: row.get(3)?,
                    summary: row.get::<_, String>(4).unwrap_or_default(),
                    published_at: row.get(5)?,
                    is_read: row.get::<_, i32>(6)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(EntryPage { entries, total, page, page_size })
    }
}

fn map_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        feed_id: row.get(1)?,
        guid: row.get(2)?,
        title: row.get(3)?,
        author: row.get(4)?,
        link: row.get(5)?,
        summary: row.get(6)?,
        published_at: row.get(7)?,
        updated_at: row.get(8)?,
        is_read: row.get::<_, i32>(9)? != 0,
        is_starred: row.get::<_, i32>(10)? != 0,
        created_at: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::model::NewEntry;
    use crate::db::open_test_db_pool;
    use crate::db::repository::FeedRepository;

    fn setup() -> (EntryRepository, i64) {
        let pool = open_test_db_pool().expect("Failed to create test pool");
        let feed_repo = FeedRepository::new(pool.clone());
        let feed = feed_repo
            .insert("https://test.example.com/feed", "Test Feed")
            .expect("feed insert failed");
        (EntryRepository::new(pool), feed.id)
    }

    fn seed_entry(repo: &EntryRepository, feed_id: i64, guid: &str, title: &str) -> Entry {
        let entry = NewEntry {
            feed_id,
            guid: guid.to_string(),
            title: title.to_string(),
            author: "Test Author".into(),
            link: "https://example.com/1".into(),
            summary: "A test entry".into(),
            published_at: Some("2026-07-15T10:00:00".into()),
            updated_at: None,
        };
        repo.insert_or_ignore(&entry)
            .expect("insert failed")
            .expect("entry was duplicate")
    }

    #[test]
    fn test_insert_entry_success() {
        let (repo, feed_id) = setup();
        let entry = seed_entry(&repo, feed_id, "unique-guid-1", "My Entry");
        let found = repo.find_by_id(entry.id).expect("find failed").expect("not found");
        assert_eq!(found.title, "My Entry");
        assert!(!found.is_read);
    }

    #[test]
    fn test_insert_duplicate_entry_ignored() {
        let (repo, feed_id) = setup();
        let entry = NewEntry {
            feed_id,
            guid: "dup-guid".into(),
            title: "First".into(),
            author: "".into(),
            link: "".into(),
            summary: "".into(),
            published_at: None,
            updated_at: None,
        };
        let first = repo.insert_or_ignore(&entry).expect("insert failed");
        assert!(first.is_some());
        let second = repo.insert_or_ignore(&entry).expect("second insert failed");
        assert!(second.is_none());
    }

    #[test]
    fn test_list_by_feed_pagination() {
        let (repo, feed_id) = setup();
        for i in 1..=5 {
            seed_entry(&repo, feed_id, &format!("guid-{}", i), &format!("Entry {}", i));
        }
        let page = repo.list_by_feed(feed_id, 1, 3, None).expect("list failed");
        assert_eq!(page.entries.len(), 3);
        assert_eq!(page.total, 5);
    }

    #[test]
    fn test_mark_read_changes_status() {
        let (repo, feed_id) = setup();
        let entry = seed_entry(&repo, feed_id, "guid-rw", "Read/Write");
        repo.mark_read(entry.id).expect("mark_read failed");
        let found = repo.find_by_id(entry.id).expect("find failed").expect("not found");
        assert!(found.is_read);
    }

    #[test]
    fn test_mark_unread_reverts_status() {
        let (repo, feed_id) = setup();
        let entry = seed_entry(&repo, feed_id, "guid-ur", "Unread Test");
        repo.mark_read(entry.id).expect("mark_read failed");
        repo.mark_unread(entry.id).expect("mark_unread failed");
        let found = repo.find_by_id(entry.id).expect("find failed").expect("not found");
        assert!(!found.is_read);
    }

    #[test]
    fn test_mark_all_read_in_feed() {
        let (repo, feed_id) = setup();
        for i in 1..=3 {
            seed_entry(&repo, feed_id, &format!("guid-ar-{}", i), "All Read");
        }
        let affected = repo.mark_all_read_in_feed(feed_id).expect("mark_all failed");
        assert_eq!(affected, 3);
        let page = repo.list_by_feed(feed_id, 1, 10, Some("unread")).expect("list failed");
        assert_eq!(page.total, 0);
    }

    #[test]
    fn test_filter_unread_only() {
        let (repo, feed_id) = setup();
        let e1 = seed_entry(&repo, feed_id, "guid-u1", "Unread");
        seed_entry(&repo, feed_id, "guid-u2", "To Read");
        repo.mark_read(e1.id).expect("mark_read failed");
        let page = repo.list_by_feed(feed_id, 1, 10, Some("unread")).expect("list failed");
        assert_eq!(page.total, 1);
    }

    #[test]
    fn test_search_by_title() {
        let (repo, feed_id) = setup();
        seed_entry(&repo, feed_id, "guid-s1", "Rust Programming Guide");
        seed_entry(&repo, feed_id, "guid-s2", "Python Tips");
        let page = repo.search("Rust", 1, 10).expect("search failed");
        assert_eq!(page.total, 1);
        assert_eq!(page.entries[0].title, "Rust Programming Guide");
    }
}
