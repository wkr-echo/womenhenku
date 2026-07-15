use rusqlite::params;

use crate::db::model::Note;
use crate::db::DbPool;
use crate::db::error::RepositoryError;

/// Data access layer for the `notes` table.
pub struct NoteRepository {
    pool: DbPool,
}

impl NoteRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Save or update a note for an entry. One note per entry (upsert).
    pub fn save(&self, entry_id: i64, content: &str) -> Result<Note, RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO notes (entry_id, content, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(entry_id) DO UPDATE SET
                 content = excluded.content,
                 updated_at = datetime('now')",
            params![entry_id, content],
        )?;
        self.find_by_entry_id(entry_id)?
            .ok_or(RepositoryError::NotFound("Note not found after save".into()))
    }

    /// Get a note by entry_id.
    pub fn find_by_entry_id(&self, entry_id: i64) -> Result<Option<Note>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, entry_id, content, created_at, updated_at
             FROM notes WHERE entry_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![entry_id], map_note)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    /// Delete a note by entry_id.
    pub fn delete(&self, entry_id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "DELETE FROM notes WHERE entry_id = ?1",
            params![entry_id],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Note for entry_id={} not found", entry_id)));
        }
        Ok(())
    }

    /// List all notes for export (ordered by updated_at desc).
    pub fn list_all(&self) -> Result<Vec<Note>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, entry_id, content, created_at, updated_at
             FROM notes ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], map_note)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

fn map_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        entry_id: row.get(1)?,
        content: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db_pool;
    use rusqlite::params;

    fn setup() -> (NoteRepository, i64) {
        let pool = open_test_db_pool().expect("Failed to create test pool");
        let repo = NoteRepository::new(pool.clone());

        // Insert a feed and entry for FK reference
        let conn = pool.get().expect("get conn");
        conn.execute(
            "INSERT INTO feeds (url, title) VALUES ('https://note-test.example.com/rss', 'Test Feed')",
            [],
        ).expect("insert feed");
        let feed_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO entries (feed_id, guid, title) VALUES (?1, 'guid-note', 'Note Entry')",
            params![feed_id],
        ).expect("insert entry");
        let entry_id = conn.last_insert_rowid();

        (repo, entry_id)
    }

    #[test]
    fn test_save_and_retrieve_note() {
        let (repo, entry_id) = setup();
        let note = repo.save(entry_id, "# Hello\nThis is a note.").expect("save failed");
        assert_eq!(note.entry_id, entry_id);
        assert_eq!(note.content, "# Hello\nThis is a note.");
    }

    #[test]
    fn test_upsert_note() {
        let (repo, entry_id) = setup();
        repo.save(entry_id, "first version").expect("first save");
        let note = repo.save(entry_id, "second version").expect("upsert save");
        assert_eq!(note.content, "second version");
        // Should still be only one row
        let all = repo.list_all().expect("list failed");
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_delete_note() {
        let (repo, entry_id) = setup();
        repo.save(entry_id, "to delete").expect("save");
        repo.delete(entry_id).expect("delete failed");
        let result = repo.find_by_entry_id(entry_id).expect("find failed");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_by_entry_id_not_found_returns_none() {
        let (repo, _) = setup();
        let result = repo.find_by_entry_id(99999).expect("query failed");
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_nonexistent_returns_error() {
        let (repo, _) = setup();
        assert!(repo.delete(99999).is_err());
    }
}
