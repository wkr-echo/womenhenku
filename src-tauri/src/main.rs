fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("Womenhenku starting...");

    let db_path = womenhenku::db::default_db_path();
    let _pool = match womenhenku::db::initialize_database(&db_path) {
        Ok(pool) => {
            tracing::info!("Database initialized at: {}", db_path.display());
            pool
        }
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Womenhenku shutdown complete.");
}
