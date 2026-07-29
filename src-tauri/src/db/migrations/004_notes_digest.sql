-- 004_notes_digest.sql
-- Stage 4: Notes and digest export system.
--
-- notes: per-entry Markdown notes (one per entry, upsert by entry_id).
-- digest_templates: user-defined export templates for digest rendering.

CREATE TABLE IF NOT EXISTS notes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id    INTEGER NOT NULL UNIQUE REFERENCES entries(id) ON DELETE CASCADE,
    content     TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_notes_entry_id ON notes(entry_id);

CREATE TABLE IF NOT EXISTS digest_templates (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    body        TEXT NOT NULL DEFAULT '',
    format      TEXT NOT NULL DEFAULT 'markdown',  -- 'markdown' | 'html' | 'plaintext'
    is_default  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);