-- 009_usage_events: Token usage recording (replaces simplified llm_usage_events)
-- Records every LLM call with snapshots for historical reporting.

CREATE TABLE IF NOT EXISTS usage_events (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    task_run_id                 INTEGER,
    entry_id                    INTEGER,
    task_type                   TEXT NOT NULL,
    provider_id                 INTEGER,
    model_id                    INTEGER,
    provider_base_url_snapshot  TEXT NOT NULL,
    provider_host_snapshot      TEXT,
    provider_name_snapshot      TEXT,
    model_name_snapshot         TEXT NOT NULL,
    request_phase               TEXT NOT NULL DEFAULT 'normal',
    request_status              TEXT NOT NULL,
    prompt_tokens               INTEGER,
    completion_tokens           INTEGER,
    total_tokens                INTEGER,
    usage_availability          TEXT NOT NULL DEFAULT 'missing',
    started_at                  TEXT,
    finished_at                 TEXT,
    created_at                  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_usage_task_type_created ON usage_events(task_type, created_at);
CREATE INDEX IF NOT EXISTS idx_usage_provider_created ON usage_events(provider_id, created_at);
