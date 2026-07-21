-- 008_tag_schema_enhance.sql
-- Stage 5 enhancement: Add missing fields per v2-features-spec.md
--
-- Add normalized_name to tags
-- Add source and confidence to entry_tags
-- Add normalized_alias to tag_aliases

ALTER TABLE tags ADD COLUMN normalized_name TEXT;
ALTER TABLE tags ADD COLUMN is_provisional INTEGER NOT NULL DEFAULT 1;

ALTER TABLE entry_tags ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
ALTER TABLE entry_tags ADD COLUMN confidence REAL NOT NULL DEFAULT 0.0;

ALTER TABLE tag_aliases ADD COLUMN normalized_alias TEXT;

UPDATE tags SET normalized_name = LOWER(name), is_provisional = CASE WHEN status = 'permanent' THEN 0 ELSE 1 END WHERE normalized_name IS NULL;

CREATE INDEX IF NOT EXISTS idx_tags_normalized_name ON tags(normalized_name);
CREATE INDEX IF NOT EXISTS idx_entry_tags_source ON entry_tags(source);
CREATE INDEX IF NOT EXISTS idx_tag_aliases_normalized_alias ON tag_aliases(normalized_alias);