-- 003_providers.sql
-- Stage 3: Provider and Agent run tracking tables.
--
-- Design notes:
--   - api_key is stored as encrypted text using Tauri secure-store.
--     The secure store reference (api_key_ref) is stored here instead of the raw key.
--   - Each provider has a unique name for display in the UI.
--   - provider_models stores available models for each provider.
--   - agent_runs tracks the history and state of all agent executions.
--
-- Migration number 003 intentionally skips to reserve space between Stage 1 (002) and Stage 4 (004).

CREATE TABLE IF NOT EXISTS providers (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL UNIQUE,
    base_url        TEXT NOT NULL,
    api_key_ref     TEXT NOT NULL DEFAULT '',  -- Reference key for secure store
    is_default      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT
);

-- Only one provider can be default at a time
CREATE UNIQUE INDEX IF NOT EXISTS idx_providers_is_default 
    ON providers(is_default) WHERE is_default = 1;

CREATE TABLE IF NOT EXISTS provider_models (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_id     INTEGER NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    model_name      TEXT NOT NULL,
    is_default      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(provider_id, model_name)
);

CREATE TABLE IF NOT EXISTS agent_runs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id        INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    provider_id     INTEGER NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    task_kind       TEXT NOT NULL,  -- 'summary' | 'translation'
    phase           TEXT NOT NULL DEFAULT 'idle',  -- 'idle' | 'running' | 'succeeded' | 'failed' | 'cancelled'
    target_language TEXT NOT NULL DEFAULT '中文',
    detail_level    TEXT,  -- 'short' | 'medium' | 'detailed' (for summary only)
    output_text     TEXT,
    prompt_tokens   INTEGER,
    completion_tokens INTEGER,
    error_message   TEXT,
    started_at      TEXT,
    completed_at    TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for querying runs by entry
CREATE INDEX IF NOT EXISTS idx_agent_runs_entry_id ON agent_runs(entry_id);

-- Index for querying runs by entry + task kind (most common query)
CREATE INDEX IF NOT EXISTS idx_agent_runs_entry_task ON agent_runs(entry_id, task_kind);
