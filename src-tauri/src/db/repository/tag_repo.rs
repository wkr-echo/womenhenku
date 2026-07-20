use rusqlite::params;

use crate::db::model::{Tag, TagAlias, TagRecommendation};
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
            "INSERT INTO tags (name, color, status, usage_count) VALUES (?1, ?2, 'permanent', 0)",
            params![name, color],
        )?;
        let id = conn.last_insert_rowid();
        self.find_by_id(id)?
            .ok_or(RepositoryError::NotFound("Tag not found after insert".into()))
    }

    pub fn insert_temporary(&self, name: &str, color: &str) -> Result<Tag, RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO tags (name, color, status, usage_count) VALUES (?1, ?2, 'temporary', 0)",
            params![name, color],
        )?;
        let id = conn.last_insert_rowid();
        if id == 0 {
            self.find_by_name(name)?
                .ok_or(RepositoryError::NotFound("Tag not found after insert".into()))
        } else {
            self.find_by_id(id)?
                .ok_or(RepositoryError::NotFound("Tag not found after insert".into()))
        }
    }

    pub fn find_by_id(&self, id: i64) -> Result<Option<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, status, usage_count, created_at FROM tags WHERE id = ?1",
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
            "SELECT id, name, color, status, usage_count, created_at FROM tags WHERE name = ?1",
        )?;
        let mut rows = stmt.query_map(params![name], map_tag)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn find_by_name_or_alias(&self, name: &str) -> Result<Option<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.status, t.usage_count, t.created_at
             FROM tags t
             LEFT JOIN tag_aliases ta ON t.id = ta.tag_id
             WHERE t.name = ?1 OR ta.alias = ?1",
        )?;
        let mut rows = stmt.query_map(params![name, name], map_tag)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn find_all(&self) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, status, usage_count, created_at FROM tags ORDER BY usage_count DESC, created_at DESC",
        )?;
        let rows = stmt.query_map([], map_tag)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_all_permanent(&self) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, status, usage_count, created_at FROM tags WHERE status = 'permanent' ORDER BY usage_count DESC",
        )?;
        let rows = stmt.query_map([], map_tag)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_all_temporary(&self) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, status, usage_count, created_at FROM tags WHERE status = 'temporary' ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], map_tag)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn search_by_prefix(&self, prefix: &str) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, status, usage_count, created_at 
             FROM tags WHERE name LIKE ?1 ORDER BY usage_count DESC LIMIT 10",
        )?;
        let rows = stmt.query_map(params![format!("{}%", prefix)], map_tag)?;
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

    pub fn update_status(&self, id: i64, status: &str) -> Result<Tag, RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE tags SET status = ?1 WHERE id = ?2",
            params![status, id],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Tag id={} not found", id)));
        }
        self.find_by_id(id)?
            .ok_or(RepositoryError::NotFound("Tag not found after update".into()))
    }

    pub fn increment_usage(&self, id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE tags SET usage_count = usage_count + 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn merge_tags(&self, source_id: i64, target_id: i64) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        tx.execute(
            "UPDATE entry_tags SET tag_id = ?1 WHERE tag_id = ?2",
            params![target_id, source_id],
        )?;

        tx.execute(
            "UPDATE tag_aliases SET tag_id = ?1 WHERE tag_id = ?2",
            params![target_id, source_id],
        )?;

        tx.execute(
            "DELETE FROM tags WHERE id = ?1",
            params![source_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn add_tag_to_entry(&self, entry_id: i64, tag_id: i64) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
            params![entry_id, tag_id],
        )?;

        tx.execute(
            "UPDATE tags SET usage_count = usage_count + 1 WHERE id = ?1",
            params![tag_id],
        )?;

        tx.execute(
            "UPDATE tags SET status = 'permanent' WHERE id = ?1 AND usage_count >= 2",
            params![tag_id],
        )?;

        tx.commit()?;
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

        conn.execute(
            "UPDATE tags SET usage_count = usage_count - 1 WHERE id = ?1 AND usage_count > 0",
            params![tag_id],
        )?;

        Ok(())
    }

    pub fn find_tags_by_entry_id(&self, entry_id: i64) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.status, t.usage_count, t.created_at
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
            "SELECT t.id, t.name, t.color, t.status, t.usage_count, t.created_at,
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
                    status: row.get(3)?,
                    usage_count: row.get(4)?,
                    created_at: row.get(5)?,
                },
                row.get(6)?,
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

    pub fn delete_unused_tags(&self) -> Result<usize, RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "DELETE FROM tags WHERE usage_count = 0 AND status = 'temporary'",
            [],
        )?;
        Ok(affected)
    }

    // === Alias Management ===

    pub fn add_alias(&self, tag_id: i64, alias: &str) -> Result<TagAlias, RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO tag_aliases (tag_id, alias) VALUES (?1, ?2)",
            params![tag_id, alias],
        )?;
        let id = conn.last_insert_rowid();
        if id == 0 {
            let mut stmt = conn.prepare(
                "SELECT id, tag_id, alias, created_at FROM tag_aliases WHERE tag_id = ?1 AND alias = ?2",
            )?;
            let mut rows = stmt.query_map(params![tag_id, alias], map_alias)?;
            Ok(rows.next().ok_or(RepositoryError::NotFound("Alias not found".into()))??)
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, tag_id, alias, created_at FROM tag_aliases WHERE id = ?1",
            )?;
            let mut rows = stmt.query_map(params![id], map_alias)?;
            Ok(rows.next().ok_or(RepositoryError::NotFound("Alias not found".into()))??)
        }
    }

    pub fn remove_alias(&self, tag_id: i64, alias: &str) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "DELETE FROM tag_aliases WHERE tag_id = ?1 AND alias = ?2",
            params![tag_id, alias],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Alias '{}' not found", alias)));
        }
        Ok(())
    }

    pub fn find_aliases_by_tag_id(&self, tag_id: i64) -> Result<Vec<TagAlias>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, tag_id, alias, created_at FROM tag_aliases WHERE tag_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![tag_id], map_alias)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // === Tag Recommendations ===

    pub fn save_recommendations(&self, entry_id: i64, recommendations: &[(String, String, f64)]) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        tx.execute(
            "DELETE FROM tag_recommendations WHERE entry_id = ?1",
            params![entry_id],
        )?;

        for (tag_name, source_type, confidence) in recommendations {
            tx.execute(
                "INSERT INTO tag_recommendations (entry_id, tag_name, source_type, confidence) VALUES (?1, ?2, ?3, ?4)",
                params![entry_id, tag_name, source_type, confidence],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn find_recommendations_by_entry_id(&self, entry_id: i64) -> Result<Vec<TagRecommendation>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, entry_id, tag_name, source_type, confidence, created_at 
             FROM tag_recommendations WHERE entry_id = ?1 ORDER BY confidence DESC",
        )?;
        let rows = stmt.query_map(params![entry_id], map_recommendation)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn delete_recommendations_by_entry_id(&self, entry_id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM tag_recommendations WHERE entry_id = ?1",
            params![entry_id],
        )?;
        Ok(())
    }
}

fn map_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        status: row.get(3)?,
        usage_count: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn map_alias(row: &rusqlite::Row<'_>) -> rusqlite::Result<TagAlias> {
    Ok(TagAlias {
        id: row.get(0)?,
        tag_id: row.get(1)?,
        alias: row.get(2)?,
        created_at: row.get(3)?,
    })
}

fn map_recommendation(row: &rusqlite::Row<'_>) -> rusqlite::Result<TagRecommendation> {
    Ok(TagRecommendation {
        id: row.get(0)?,
        entry_id: row.get(1)?,
        tag_name: row.get(2)?,
        source_type: row.get(3)?,
        confidence: row.get(4)?,
        created_at: row.get(5)?,
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
        assert_eq!(tag.status, "permanent");
        assert_eq!(tag.usage_count, 0);

        let found = repo.find_by_id(tag.id).expect("find failed");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "news");
    }

    #[test]
    fn test_insert_temporary() {
        let (repo, _) = setup();
        let tag = repo.insert_temporary("temp-tag", "#ffff00").expect("insert failed");
        assert_eq!(tag.status, "temporary");
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
    fn test_update_status() {
        let (repo, _) = setup();
        let tag = repo.insert_temporary("temp", "#000000").expect("insert");
        assert_eq!(tag.status, "temporary");
        let updated = repo.update_status(tag.id, "permanent").expect("update failed");
        assert_eq!(updated.status, "permanent");
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

        let found_tag = repo.find_by_id(tag.id).expect("find tag");
        assert_eq!(found_tag.unwrap().usage_count, 1);

        repo.remove_tag_from_entry(entry_id, tag.id).expect("remove tag");
        let tags_after = repo.find_tags_by_entry_id(entry_id).expect("find tags after");
        assert_eq!(tags_after.len(), 0);
    }

    #[test]
    fn test_merge_tags() {
        let (repo, entry_id) = setup();
        let source_tag = repo.insert("source", "#ff0000").expect("insert source");
        let target_tag = repo.insert("target", "#00ff00").expect("insert target");

        repo.add_tag_to_entry(entry_id, source_tag.id).expect("add tag");

        repo.merge_tags(source_tag.id, target_tag.id).expect("merge");

        let tags = repo.find_tags_by_entry_id(entry_id).expect("find tags");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].id, target_tag.id);

        let found_source = repo.find_by_id(source_tag.id).expect("find source");
        assert!(found_source.is_none());
    }

    #[test]
    fn test_add_and_remove_alias() {
        let (repo, _) = setup();
        let tag = repo.insert("rust", "#dea584").expect("insert");

        let alias = repo.add_alias(tag.id, "rust-lang").expect("add alias");
        assert_eq!(alias.alias, "rust-lang");

        let aliases = repo.find_aliases_by_tag_id(tag.id).expect("find aliases");
        assert_eq!(aliases.len(), 1);

        repo.remove_alias(tag.id, "rust-lang").expect("remove alias");
        let aliases_after = repo.find_aliases_by_tag_id(tag.id).expect("find aliases after");
        assert_eq!(aliases_after.len(), 0);
    }

    #[test]
    fn test_save_and_find_recommendations() {
        let (repo, entry_id) = setup();

        let recs = &[
            ("Rust".to_string(), "nlp".to_string(), 0.9),
            ("Tauri".to_string(), "ai".to_string(), 0.8),
        ];
        repo.save_recommendations(entry_id, recs).expect("save recs");

        let found = repo.find_recommendations_by_entry_id(entry_id).expect("find recs");
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].tag_name, "Rust");
        assert_eq!(found[0].source_type, "nlp");
    }
}
