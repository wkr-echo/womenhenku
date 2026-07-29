-- 001_initial_schema.sql
-- Stage 1: Core tables for feeds, entries, and contents.
--
-- Design notes:
--   - All timestamps are ISO 8601 text strings (chrono::NaiveDateTime -> SQLite TEXT).
--   - ON DELETE CASCADE ensures referential integrity when a feed is removed.
--   - UNIQUE(feed_id, guid) prevents duplicate entries within the same feed.
--   - contents.rendered_html caches the final reader output.
--   - readability_version allows cache invalidation when the extraction pipeline changes.

CREATE TABLE IF NOT EXISTS feeds (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    url             TEXT NOT NULL UNIQUE,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    link            TEXT NOT NULL DEFAULT '',
    feed_type       TEXT NOT NULL DEFAULT 'rss',  -- 'rss' | 'atom' | 'json'
    last_synced_at  TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS entries (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id         INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    guid            TEXT NOT NULL,
    title           TEXT NOT NULL DEFAULT '',
    author          TEXT NOT NULL DEFAULT '',
    link            TEXT NOT NULL DEFAULT '',
    summary         TEXT NOT NULL DEFAULT '',
    published_at    TEXT,
    updated_at      TEXT,
    is_read         INTEGER NOT NULL DEFAULT 0,
    is_starred      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(feed_id, guid)
);

-- Index for fast feed-scoped entry listing
CREATE INDEX IF NOT EXISTS idx_entries_feed_id ON entries(feed_id);

-- Index for listing entries sorted by publish date
CREATE INDEX IF NOT EXISTS idx_entries_published_at ON entries(published_at);

-- Index for unread filtering within a feed
CREATE INDEX IF NOT EXISTS idx_entries_feed_read ON entries(feed_id, is_read);

CREATE TABLE IF NOT EXISTS contents (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id            INTEGER NOT NULL UNIQUE REFERENCES entries(id) ON DELETE CASCADE,
    raw_html            TEXT NOT NULL DEFAULT '',
    cleaned_html        TEXT,
    cleaned_markdown    TEXT,
    rendered_html       TEXT,
    readability_version INTEGER NOT NULL DEFAULT 1,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT
);
