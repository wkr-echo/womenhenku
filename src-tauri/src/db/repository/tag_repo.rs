use rusqlite::params;

use crate::db::model::Tag;
use crate::db::DbPool;
use crate::db::error::RepositoryError;

pub struct TagRepository {
    pool: DbPool,
}

impl TagRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn insert(&self, name: &str, color: &str) -> Result<Tag, RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO tags (name, color) VALUES (?1, ?2)",
            params![name, color],
        )?;
        let id = conn.last_insert_rowid();
        self.find_by_id(id)?
            .ok_or(RepositoryError::NotFound("Tag not found after insert".into()))
    }

    pub fn find_by_id(&self, id: i64) -> Result<Option<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at FROM tags WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], map_tag)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn find_by_name(&self, name: &str) -> Result<Option<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at FROM tags WHERE name = ?1",
        )?;
        let mut rows = stmt.query_map(params![name], map_tag)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn find_all(&self) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at FROM tags ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], map_tag)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Tag id={} not found", id)));
        }
        Ok(())
    }

    pub fn update(&self, id: i64, name: &str, color: &str) -> Result<Tag, RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE tags SET name = ?1, color = ?2 WHERE id = ?3",
            params![name, color, id],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Tag id={} not found", id)));
        }
        self.find_by_id(id)?
            .ok_or(RepositoryError::NotFound("Tag not found after update".into()))
    }

    pub fn add_tag_to_entry(&self, entry_id: i64, tag_id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
            params![entry_id, tag_id],
        )?;
        Ok(())
    }

    pub fn remove_tag_from_entry(&self, entry_id: i64, tag_id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "DELETE FROM entry_tags WHERE entry_id = ?1 AND tag_id = ?2",
            params![entry_id, tag_id],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Entry-tag relation entry_id={}, tag_id={} not found",
                entry_id, tag_id
            )));
        }
        Ok(())
    }

    pub fn find_tags_by_entry_id(&self, entry_id: i64) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at
             FROM tags t
             JOIN entry_tags et ON t.id = et.tag_id
             WHERE et.entry_id = ?1
             ORDER BY t.created_at DESC",
        )?;
        let rows = stmt.query_map(params![entry_id], map_tag)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_entry_ids_by_tag_id(&self, tag_id: i64) -> Result<Vec<i64>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT entry_id FROM entry_tags WHERE tag_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![tag_id], |row| row.get(0))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_tags_with_entry_count(&self) -> Result<Vec<(Tag, i64)>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at,
                    COALESCE(COUNT(et.entry_id), 0) as count
             FROM tags t
             LEFT JOIN entry_tags et ON t.id = et.tag_id
             GROUP BY t.id
             ORDER BY count DESC, t.created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                },
                row.get(4)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_tag_entry_count(&self, tag_id: i64) -> Result<i64, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT COALESCE(COUNT(entry_id), 0) FROM entry_tags WHERE tag_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![tag_id], |row| row.get(0))?;
        match rows.next() {
            Some(result) => result.map_err(Into::into),
            None => Ok(0),
        }
    }
}

fn map_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        created_at: row.get(3)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db_pool;
    use rusqlite::params;

    fn setup() -> (TagRepository, i64) {
        let pool = open_test_db_pool().expect("Failed to create test pool");
        let repo = TagRepository::new(pool.clone());

        let conn = pool.get().expect("get conn");
        conn.execute(
            "INSERT INTO feeds (url, title) VALUES ('https://tag-test.example.com/rss', 'Test Feed')",
            [],
        ).expect("insert feed");
        let feed_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO entries (feed_id, guid, title) VALUES (?1, 'guid-tag-test', 'Tag Test Entry')",
            params![feed_id],
        ).expect("insert entry");
        let entry_id = conn.last_insert_rowid();

        (repo, entry_id)
    }

    #[test]
    fn test_insert_and_find_tag() {
        let (repo, _) = setup();
        let tag = repo.insert("news", "#ff0000").expect("insert failed");
        assert_eq!(tag.name, "news");
        assert_eq!(tag.color, "#ff0000");

        let found = repo.find_by_id(tag.id).expect("find failed");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "news");
    }

    #[test]
    fn test_find_by_name() {
        let (repo, _) = setup();
        repo.insert("tech", "#00ff00").expect("insert");
        let found = repo.find_by_name("tech").expect("find failed");
        assert!(found.is_some());
        assert_eq!(found.unwrap().color, "#00ff00");
    }

    #[test]
    fn test_find_all() {
        let (repo, _) = setup();
        repo.insert("tag1", "#111111").expect("insert 1");
        repo.insert("tag2", "#222222").expect("insert 2");
        let all = repo.find_all().expect("find all failed");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_update_tag() {
        let (repo, _) = setup();
        let tag = repo.insert("old-name", "#000000").expect("insert");
        let updated = repo.update(tag.id, "new-name", "#ffffff").expect("update failed");
        assert_eq!(updated.name, "new-name");
        assert_eq!(updated.color, "#ffffff");
    }

    #[test]
    fn test_delete_tag() {
        let (repo, _) = setup();
        let tag = repo.insert("to-delete", "#000000").expect("insert");
        repo.delete(tag.id).expect("delete failed");
        let found = repo.find_by_id(tag.id).expect("find failed");
        assert!(found.is_none());
    }

    #[test]
    fn test_add_and_remove_tag_from_entry() {
        let (repo, entry_id) = setup();
        let tag = repo.insert("test-tag", "#333333").expect("insert");

        repo.add_tag_to_entry(entry_id, tag.id).expect("add tag");
        let tags = repo.find_tags_by_entry_id(entry_id).expect("find tags");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].id, tag.id);

        repo.remove_tag_from_entry(entry_id, tag.id).expect("remove tag");
        let tags_after = repo.find_tags_by_entry_id(entry_id).expect("find tags after");
        assert_eq!(tags_after.len(), 0);
    }

    #[test]
    fn test_find_entry_ids_by_tag_id() {
        let (repo, entry_id) = setup();
        let tag = repo.insert("multi-tag", "#444444").expect("insert");
        repo.add_tag_to_entry(entry_id, tag.id).expect("add tag");

        let entries = repo.find_entry_ids_by_tag_id(tag.id).expect("find entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], entry_id);
    }

    #[test]
    fn test_find_tags_with_entry_count() {
        let (repo, entry_id) = setup();
        let tag1 = repo.insert("used-tag", "#555555").expect("insert");
        let tag2 = repo.insert("unused-tag", "#666666").expect("insert");
        repo.add_tag_to_entry(entry_id, tag1.id).expect("add tag");

        let result = repo.find_tags_with_entry_count().expect("find with count");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0.id, tag1.id);
        assert_eq!(result[0].1, 1);
        assert_eq!(result[1].0.id, tag2.id);
        assert_eq!(result[1].1, 0);
    }
}
