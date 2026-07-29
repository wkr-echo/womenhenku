-- 007_llm_usage.sql
-- Stage 6: LLM Token usage statistics.
--
-- llm_usage_events: Record every LLM call for analytics.
-- Includes provider/model snapshots to survive deletion.

CREATE TABLE IF NOT EXISTS llm_usage_events (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_id         INTEGER NOT NULL,
    provider_name       TEXT NOT NULL,
    provider_base_url   TEXT NOT NULL,
    provider_host       TEXT NOT NULL,
    model_id            INTEGER NOT NULL,
    model_name          TEXT NOT NULL,
    agent_type          TEXT NOT NULL,
    prompt_tokens       INTEGER NOT NULL DEFAULT 0,
    completion_tokens   INTEGER NOT NULL DEFAULT 0,
    total_tokens        INTEGER NOT NULL DEFAULT 0,
    request_status      TEXT NOT NULL,
    timestamp           TEXT NOT NULL,
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_llm_usage_events_timestamp ON llm_usage_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_llm_usage_events_provider_id ON llm_usage_events(provider_id);
CREATE INDEX IF NOT EXISTS idx_llm_usage_events_model_id ON llm_usage_events(model_id);
CREATE INDEX IF NOT EXISTS idx_llm_usage_events_agent_type ON llm_usage_events(agent_type);

CREATE TABLE IF NOT EXISTS settings (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    key         TEXT NOT NULL UNIQUE,
    value       TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO settings (key, value) VALUES ('llm_usage_retention_days', '180');
INSERT OR IGNORE INTO settings (key, value) VALUES ('app_language', 'zh');
