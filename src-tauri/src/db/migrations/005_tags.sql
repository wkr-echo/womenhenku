-- 005_tags.sql
-- Stage 5: Tag system for entry organization.
--
-- tags: tag definitions (name, color).
-- entry_tags: many-to-many relationship between entries and tags.

CREATE TABLE IF NOT EXISTS tags (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    color       TEXT NOT NULL DEFAULT '#3b82f6',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);

CREATE TABLE IF NOT EXISTS entry_tags (
    entry_id    INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    tag_id      INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (entry_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_entry_tags_entry_id ON entry_tags(entry_id);
CREATE INDEX IF NOT EXISTS idx_entry_tags_tag_id ON entry_tags(tag_id);
