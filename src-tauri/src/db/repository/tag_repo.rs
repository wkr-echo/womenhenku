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
        let normalized_name = normalize_tag_name(name);
        if normalized_name.is_empty() {
            return Err(RepositoryError::InvalidInput("Tag name cannot be empty".into()));
        }

        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO tags (name, normalized_name, color, is_provisional, usage_count) VALUES (?1, ?2, ?3, 1, 0)",
            params![name, normalized_name, color],
        )?;
        let id = conn.last_insert_rowid();
        self.find_by_id(id)?
            .ok_or(RepositoryError::NotFound("Tag not found after insert".into()))
    }

    pub fn insert_temporary(&self, name: &str, color: &str) -> Result<Tag, RepositoryError> {
        let normalized_name = normalize_tag_name(name);
        if normalized_name.is_empty() {
            return Err(RepositoryError::InvalidInput("Tag name cannot be empty".into()));
        }

        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO tags (name, normalized_name, color, is_provisional, usage_count) VALUES (?1, ?2, ?3, 1, 0)",
            params![name, normalized_name, color],
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
            "SELECT id, name, normalized_name, color, is_provisional, usage_count, created_at FROM tags WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], map_tag)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn find_by_name(&self, name: &str) -> Result<Option<Tag>, RepositoryError> {
        let normalized_name = normalize_tag_name(name);
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, normalized_name, color, is_provisional, usage_count, created_at FROM tags WHERE normalized_name = ?1",
        )?;
        let mut rows = stmt.query_map(params![normalized_name], map_tag)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn find_by_name_or_alias(&self, name: &str) -> Result<Option<Tag>, RepositoryError> {
        let normalized_name = normalize_tag_name(name);
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.normalized_name, t.color, t.is_provisional, t.usage_count, t.created_at
             FROM tags t
             LEFT JOIN tag_aliases ta ON t.id = ta.tag_id
             WHERE t.normalized_name = ?1 OR ta.normalized_alias = ?1",
        )?;
        let mut rows = stmt.query_map(params![normalized_name, normalized_name], map_tag)?;
        match rows.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn find_all(&self) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, normalized_name, color, is_provisional, usage_count, created_at FROM tags ORDER BY usage_count DESC, created_at DESC",
        )?;
        let rows = stmt.query_map([], map_tag)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_all_permanent(&self) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, normalized_name, color, is_provisional, usage_count, created_at FROM tags WHERE is_provisional = 0 ORDER BY usage_count DESC",
        )?;
        let rows = stmt.query_map([], map_tag)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_all_temporary(&self) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, normalized_name, color, is_provisional, usage_count, created_at FROM tags WHERE is_provisional = 1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], map_tag)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_unused(&self) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, normalized_name, color, is_provisional, usage_count, created_at FROM tags WHERE usage_count = 0 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], map_tag)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn search_by_prefix(&self, prefix: &str) -> Result<Vec<Tag>, RepositoryError> {
        let normalized_prefix = normalize_tag_name(prefix);
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, normalized_name, color, is_provisional, usage_count, created_at 
             FROM tags WHERE normalized_name LIKE ?1 ORDER BY usage_count DESC LIMIT 10",
        )?;
        let rows = stmt.query_map(params![format!("{}%", normalized_prefix)], map_tag)?;
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
        let normalized_name = normalize_tag_name(name);
        if normalized_name.is_empty() {
            return Err(RepositoryError::InvalidInput("Tag name cannot be empty".into()));
        }

        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE tags SET name = ?1, normalized_name = ?2, color = ?3 WHERE id = ?4",
            params![name, normalized_name, color, id],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Tag id={} not found", id)));
        }
        self.find_by_id(id)?
            .ok_or(RepositoryError::NotFound("Tag not found after update".into()))
    }

    pub fn update_status(&self, id: i64, is_provisional: bool) -> Result<Tag, RepositoryError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE tags SET is_provisional = ?1 WHERE id = ?2",
            params![is_provisional, id],
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
            "UPDATE tags SET usage_count = usage_count + 1, is_provisional = CASE WHEN usage_count + 1 >= 2 THEN 0 ELSE is_provisional END WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn decrement_usage(&self, id: i64) -> Result<(), RepositoryError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE tags SET usage_count = usage_count - 1, is_provisional = CASE WHEN usage_count - 1 < 2 THEN 1 ELSE is_provisional END WHERE id = ?1 AND usage_count > 0",
            params![id],
        )?;
        Ok(())
    }

    pub fn merge_tags(&self, source_id: i64, target_id: i64) -> Result<(), RepositoryError> {
        if source_id == target_id {
            return Err(RepositoryError::InvalidInput("Cannot merge a tag with itself".into()));
        }

        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        let source_tag = self.find_by_id(source_id)?.ok_or(RepositoryError::NotFound(format!("Source tag id={} not found", source_id)))?;

        tx.execute(
            "UPDATE entry_tags SET tag_id = ?1 WHERE tag_id = ?2",
            params![target_id, source_id],
        )?;

        tx.execute(
            "UPDATE tag_aliases SET tag_id = ?1 WHERE tag_id = ?2",
            params![target_id, source_id],
        )?;

        tx.execute(
            "INSERT OR IGNORE INTO tag_aliases (tag_id, alias, normalized_alias) VALUES (?1, ?2, ?3)",
            params![target_id, source_tag.name, source_tag.normalized_name],
        )?;

        tx.execute(
            "DELETE FROM tags WHERE id = ?1",
            params![source_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn add_tag_to_entry(&self, entry_id: i64, tag_id: i64, source: &str, confidence: f64) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id, source, confidence) VALUES (?1, ?2, ?3, ?4)",
            params![entry_id, tag_id, source, confidence],
        )?;

        tx.execute(
            "UPDATE tags SET usage_count = usage_count + 1, is_provisional = CASE WHEN usage_count + 1 >= 2 THEN 0 ELSE is_provisional END WHERE id = ?1",
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

        self.decrement_usage(tag_id)?;
        Ok(())
    }

    pub fn find_tags_by_entry_id(&self, entry_id: i64) -> Result<Vec<Tag>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.normalized_name, t.color, t.is_provisional, t.usage_count, t.created_at
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
            "SELECT t.id, t.name, t.normalized_name, t.color, t.is_provisional, t.usage_count, t.created_at,
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
                    normalized_name: row.get(2)?,
                    color: row.get(3)?,
                    is_provisional: row.get(4)?,
                    usage_count: row.get(5)?,
                    created_at: row.get(6)?,
                },
                row.get(7)?,
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
            "DELETE FROM tags WHERE usage_count = 0",
            [],
        )?;
        Ok(affected)
    }

    // === Alias Management ===

    pub fn add_alias(&self, tag_id: i64, alias: &str) -> Result<TagAlias, RepositoryError> {
        let normalized_alias = normalize_tag_name(alias);
        if normalized_alias.is_empty() {
            return Err(RepositoryError::InvalidInput("Alias cannot be empty".into()));
        }

        let tag = self.find_by_id(tag_id)?.ok_or(RepositoryError::NotFound(format!("Tag id={} not found", tag_id)))?;
        if normalized_alias == tag.normalized_name {
            return Err(RepositoryError::InvalidInput("Alias cannot be the same as tag name".into()));
        }

        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO tag_aliases (tag_id, alias, normalized_alias) VALUES (?1, ?2, ?3)",
            params![tag_id, alias, normalized_alias],
        )?;
        let id = conn.last_insert_rowid();
        if id == 0 {
            let mut stmt = conn.prepare(
                "SELECT id, tag_id, alias, normalized_alias, created_at FROM tag_aliases WHERE tag_id = ?1 AND normalized_alias = ?2",
            )?;
            let mut rows = stmt.query_map(params![tag_id, normalized_alias], map_alias)?;
            Ok(rows.next().ok_or(RepositoryError::NotFound("Alias not found".into()))??)
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, tag_id, alias, normalized_alias, created_at FROM tag_aliases WHERE id = ?1",
            )?;
            let mut rows = stmt.query_map(params![id], map_alias)?;
            Ok(rows.next().ok_or(RepositoryError::NotFound("Alias not found".into()))??)
        }
    }

    pub fn remove_alias(&self, tag_id: i64, alias: &str) -> Result<(), RepositoryError> {
        let normalized_alias = normalize_tag_name(alias);
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "DELETE FROM tag_aliases WHERE tag_id = ?1 AND normalized_alias = ?2",
            params![tag_id, normalized_alias],
        )?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(format!("Alias '{}' not found", alias)));
        }
        Ok(())
    }

    pub fn find_aliases_by_tag_id(&self, tag_id: i64) -> Result<Vec<TagAlias>, RepositoryError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, tag_id, alias, normalized_alias, created_at FROM tag_aliases WHERE tag_id = ?1 ORDER BY created_at DESC",
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

    // === Duplicate Detection ===

    pub fn find_potential_duplicates(&self) -> Result<Vec<(Tag, Tag, String)>, RepositoryError> {
        let all_tags = self.find_all()?;
        let mut duplicates = Vec::new();

        for i in 0..all_tags.len() {
            for j in (i + 1)..all_tags.len() {
                let tag_a = &all_tags[i];
                let tag_b = &all_tags[j];

                if let Some(reason) = detect_duplicate(tag_a, tag_b) {
                    duplicates.push((tag_a.clone(), tag_b.clone(), reason));
                }
            }
        }

        Ok(duplicates)
    }
}

pub fn normalize_tag_name(name: &str) -> String {
    let trimmed = name.trim().to_lowercase();
    let folded: String = trimmed
        .chars()
        .fold(Vec::new(), |mut acc, c| {
            if c == '-' || c == '_' || c == '.' || c.is_whitespace() {
                if let Some(last) = acc.last() {
                    if *last != ' ' {
                        acc.push(' ');
                    }
                }
            } else {
                acc.push(c);
            }
            acc
        })
        .into_iter()
        .collect();
    folded.trim().to_string()
}

fn detect_duplicate(tag_a: &Tag, tag_b: &Tag) -> Option<String> {
    let name_a = &tag_a.normalized_name;
    let name_b = &tag_b.normalized_name;

    let singular_a = to_singular(name_a);
    let singular_b = to_singular(name_b);
    if singular_a == singular_b {
        return Some("plural_variant".to_string());
    }

    let no_space_a: String = name_a.chars().filter(|c| !c.is_whitespace()).collect();
    let no_space_b: String = name_b.chars().filter(|c| !c.is_whitespace()).collect();
    if no_space_a == no_space_b && name_a != name_b {
        return Some("naming_variant".to_string());
    }

    if name_a.len() >= 5 && name_b.len() >= 5 && 
       name_a.chars().next() == name_b.chars().next() &&
       name_a.chars().last() == name_b.chars().last() &&
       levenshtein_distance(name_a, name_b) <= 2 {
        return Some("spelling_variant".to_string());
    }

    None
}

fn to_singular(word: &str) -> String {
    if word.ends_with("ies") && word.len() > 3 {
        word[..word.len() - 3].to_string() + "y"
    } else if word.ends_with("s") && !word.ends_with("ss") && !word.ends_with("us") && word.len() > 2 {
        word[..word.len() - 1].to_string()
    } else {
        word.to_string()
    }
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_len][b_len]
}

fn map_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        normalized_name: row.get(2)?,
        color: row.get(3)?,
        is_provisional: row.get(4)?,
        usage_count: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn map_alias(row: &rusqlite::Row<'_>) -> rusqlite::Result<TagAlias> {
    Ok(TagAlias {
        id: row.get(0)?,
        tag_id: row.get(1)?,
        alias: row.get(2)?,
        normalized_alias: row.get(3)?,
        created_at: row.get(4)?,
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
    fn test_normalize_tag_name() {
        assert_eq!(normalize_tag_name("  AI  "), "ai");
        assert_eq!(normalize_tag_name("Machine-Learning"), "machine learning");
        assert_eq!(normalize_tag_name("foo...bar"), "foo bar");
        assert_eq!(normalize_tag_name("hello   world"), "hello world");
        assert_eq!(normalize_tag_name("   "), "");
        assert_eq!(normalize_tag_name("Rust-Lang"), "rust lang");
    }

    #[test]
    fn test_insert_and_find_tag() {
        let (repo, _) = setup();
        let tag = repo.insert("news", "#ff0000").expect("insert failed");
        assert_eq!(tag.name, "news");
        assert_eq!(tag.normalized_name, "news");
        assert_eq!(tag.color, "#ff0000");
        assert!(tag.is_provisional);
        assert_eq!(tag.usage_count, 0);

        let found = repo.find_by_id(tag.id).expect("find failed");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "news");
    }

    #[test]
    fn test_find_by_name_normalized() {
        let (repo, _) = setup();
        repo.insert("Rust", "#dea584").expect("insert");

        let found = repo.find_by_name("rust").expect("find failed");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Rust");
    }

    #[test]
    fn test_insert_temporary() {
        let (repo, _) = setup();
        let tag = repo.insert_temporary("temp-tag", "#ffff00").expect("insert failed");
        assert!(tag.is_provisional);
    }

    #[test]
    fn test_add_tag_to_entry_with_source() {
        let (repo, entry_id) = setup();
        let tag = repo.insert("test-tag", "#333333").expect("insert");

        repo.add_tag_to_entry(entry_id, tag.id, "manual", 0.0).expect("add tag");
        let tags = repo.find_tags_by_entry_id(entry_id).expect("find tags");
        assert_eq!(tags.len(), 1);

        let found_tag = repo.find_by_id(tag.id).expect("find tag").unwrap();
        assert_eq!(found_tag.usage_count, 1);
        assert!(found_tag.is_provisional);

        repo.add_tag_to_entry(entry_id, tag.id, "batch", 0.8).expect("add tag again");
        let found_tag2 = repo.find_by_id(tag.id).expect("find tag").unwrap();
        assert_eq!(found_tag2.usage_count, 2);
        assert!(!found_tag2.is_provisional);
    }

    #[test]
    fn test_merge_tags() {
        let (repo, entry_id) = setup();
        let source_tag = repo.insert("source", "#ff0000").expect("insert source");
        let target_tag = repo.insert("target", "#00ff00").expect("insert target");

        repo.add_tag_to_entry(entry_id, source_tag.id, "manual", 0.0).expect("add tag");

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
        assert_eq!(alias.normalized_alias, "rust lang");

        let aliases = repo.find_aliases_by_tag_id(tag.id).expect("find aliases");
        assert_eq!(aliases.len(), 1);

        repo.remove_alias(tag.id, "Rust-Lang").expect("remove alias");
        let aliases_after = repo.find_aliases_by_tag_id(tag.id).expect("find aliases after");
        assert_eq!(aliases_after.len(), 0);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("Rust", "rust"), 1);
        assert_eq!(levenshtein_distance("Python", "Pytohn"), 2);
        assert_eq!(levenshtein_distance("MacOS", "Mac OS"), 1);
    }

    #[test]
    fn test_detect_duplicate() {
        let tag1 = Tag { id: 1, name: "Databases".to_string(), normalized_name: "databases".to_string(), color: "#000".to_string(), is_provisional: false, usage_count: 10, created_at: "2026-01-01".to_string() };
        let tag2 = Tag { id: 2, name: "Database".to_string(), normalized_name: "database".to_string(), color: "#000".to_string(), is_provisional: false, usage_count: 5, created_at: "2026-01-02".to_string() };
        assert_eq!(detect_duplicate(&tag1, &tag2), Some("plural_variant".to_string()));

        let tag3 = Tag { id: 3, name: "MacOS".to_string(), normalized_name: "macos".to_string(), color: "#000".to_string(), is_provisional: false, usage_count: 10, created_at: "2026-01-01".to_string() };
        let tag4 = Tag { id: 4, name: "Mac OS".to_string(), normalized_name: "mac os".to_string(), color: "#000".to_string(), is_provisional: false, usage_count: 5, created_at: "2026-01-02".to_string() };
        assert_eq!(detect_duplicate(&tag3, &tag4), Some("naming_variant".to_string()));

        let tag5 = Tag { id: 5, name: "Programing".to_string(), normalized_name: "programing".to_string(), color: "#000".to_string(), is_provisional: false, usage_count: 10, created_at: "2026-01-01".to_string() };
        let tag6 = Tag { id: 6, name: "Programming".to_string(), normalized_name: "programming".to_string(), color: "#000".to_string(), is_provisional: false, usage_count: 5, created_at: "2026-01-02".to_string() };
        assert_eq!(detect_duplicate(&tag5, &tag6), Some("spelling_variant".to_string()));
    }
}