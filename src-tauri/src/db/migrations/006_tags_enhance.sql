-- 006_tags_enhance.sql
-- Stage 5 enhancement: Tag system improvements.
--
-- Add status field to tags (temporary/permanent).
-- Create tag_aliases table for synonym management.
-- Create tag_recommendations table for AI suggestions.

ALTER TABLE tags ADD COLUMN status TEXT NOT NULL DEFAULT 'permanent';
ALTER TABLE tags ADD COLUMN usage_count INTEGER NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS tag_aliases (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tag_id      INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    alias       TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(tag_id, alias)
);

CREATE INDEX IF NOT EXISTS idx_tag_aliases_tag_id ON tag_aliases(tag_id);
CREATE INDEX IF NOT EXISTS idx_tag_aliases_alias ON tag_aliases(alias);

CREATE TABLE IF NOT EXISTS tag_recommendations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id        INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    tag_name        TEXT NOT NULL,
    source_type     TEXT NOT NULL,
    confidence      REAL NOT NULL DEFAULT 0.5,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_tag_recommendations_entry_id ON tag_recommendations(entry_id);
CREATE INDEX IF NOT EXISTS idx_tag_recommendations_tag_name ON tag_recommendations(tag_name);
