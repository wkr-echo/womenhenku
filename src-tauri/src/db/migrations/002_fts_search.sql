-- 002_fts_search.sql
-- Stage 2: Full-text search on entry titles and summaries.
-- Uses SQLite FTS5 for fast, relevance-ranked text search.
-- The content table is 'entries' so the FTS index stays in sync via triggers
-- (or manual rebuild after inserts).

CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
    title,
    summary,
    content='entries',
    content_rowid='id'
);

-- Triggers to keep FTS index in sync with the entries table.
-- These are CREATE TRIGGER IF NOT EXISTS to be migration-safe.

CREATE TRIGGER IF NOT EXISTS entries_fts_insert AFTER INSERT ON entries BEGIN
    INSERT INTO entries_fts(rowid, title, summary)
    VALUES (new.id, new.title, new.summary);
END;

CREATE TRIGGER IF NOT EXISTS entries_fts_delete AFTER DELETE ON entries BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, title, summary)
    VALUES ('delete', old.id, old.title, old.summary);
END;

CREATE TRIGGER IF NOT EXISTS entries_fts_update AFTER UPDATE ON entries BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, title, summary)
    VALUES ('delete', old.id, old.title, old.summary);
    INSERT INTO entries_fts(rowid, title, summary)
    VALUES (new.id, new.title, new.summary);
END;
