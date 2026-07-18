use crate::db::DbPool;
use rusqlite::Connection;

/// Embedded migration scripts.
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "001_initial_schema",
        sql: include_str!("migrations/001_initial_schema.sql"),
    },
    Migration {
        version: 2,
        name: "002_fts_search",
        sql: include_str!("migrations/002_fts_search.sql"),
    },
    Migration {
        version: 3,
        name: "003_providers",
        sql: include_str!("migrations/003_providers.sql"),
    },
    Migration {
        version: 4,
        name: "004_notes_digest",
        sql: include_str!("migrations/004_notes_digest.sql"),
    },
];

struct Migration {
    version: i32,
    name: &'static str,
    sql: &'static str,
}

/// Run all pending migrations using a connection pool.
/// Each migration runs in its own transaction (atomic: all-or-nothing).
/// Already-applied versions are skipped.
pub fn run_migrations(pool: &DbPool) -> Result<(), anyhow::Error> {
    let conn = pool.get()?;
    run_migrations_on_conn(&conn)
}

/// Run migrations on a single connection. Used by pool initialization and tests.
pub fn run_migrations_on_conn(conn: &Connection) -> Result<(), anyhow::Error> {
    // Create schema_version table if it does not exist yet
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version     INTEGER PRIMARY KEY,
            applied_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for migration in MIGRATIONS {
        if migration.version > current_version {
            tracing::info!(
                "Applying migration {}: {}",
                migration.version,
                migration.name
            );

            // Wrap each migration in a transaction for atomicity
            let tx = conn.unchecked_transaction()?;
            tx.execute_batch(migration.sql)?;
            tx.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [migration.version],
            )?;
            tx.commit()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_migrations_creates_tables() {
        let conn = crate::db::open_test_connection().expect("Failed to open test connection");

        // Verify schema_version table exists and has record
        let version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .expect("Failed to query schema_version");
        assert_eq!(version, 4);

        // Verify core tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .expect("Failed to prepare statement")
            .query_map([], |row| row.get(0))
            .expect("Failed to query tables")
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"feeds".to_string()));
        assert!(tables.contains(&"entries".to_string()));
        assert!(tables.contains(&"contents".to_string()));
        assert!(tables.contains(&"notes".to_string()));
        assert!(tables.contains(&"digest_templates".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().expect("Failed to open in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .expect("Failed to enable foreign keys");

        // Run twice — second run should be a no-op
        run_migrations_on_conn(&conn).expect("First migration failed");
        run_migrations_on_conn(&conn).expect("Second migration failed");

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))
            .expect("Failed to count schema_version");
        assert_eq!(count, 4); // 1, 2, 3, 4 — four migrations
    }
}
