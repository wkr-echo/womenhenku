// Standalone tool: test OPML import flow end-to-end.
// Usage: cargo run --example test_opml_import
//
// Creates a test OPML file, parses it, writes feeds directly to DB.

use std::path::PathBuf;

use womenhenku_lib::db;
use womenhenku_lib::db::repository::FeedRepository;
use womenhenku_lib::feed::opml;

fn main() {
    tracing_subscriber::fmt::init();

    let db_path = db::default_db_path();
    let pool = db::initialize_database(&db_path).expect("Failed to init DB");

    // Step 1: Create a test OPML file
    let opml_path = std::env::temp_dir().join("test.opml");
    let opml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <head><title>Test Subscriptions</title></head>
  <body>
    <outline text="RSS 2.0 Test" type="rss" xmlUrl="https://feeds.example.com/rss" htmlUrl="https://example.com/"/>
    <outline text="Atom Test Blog" type="rss" xmlUrl="https://feeds.example.com/atom.xml"/>
    <outline text="JSON Feed Test" type="rss" xmlUrl="https://feeds.example.com/feed.json"/>
  </body>
</opml>"#;

    std::fs::write(&opml_path, opml_content).expect("Failed to write test OPML");
    println!("[1] Created test OPML: {}", opml_path.display());

    // Step 2: Parse OPML
    let outlines = opml::parse_opml_file(&opml_path).expect("Failed to parse OPML");
    println!("[2] Parsed {} feed outlines:", outlines.len());
    for o in &outlines {
        println!("    - {} ({})", o.title, o.xml_url);
    }

    // Step 3: Insert feeds directly (bypass HTTP fetch)
    let feed_repo = FeedRepository::new(pool.clone());
    let mut inserted = 0;
    for outline in &outlines {
        match feed_repo.insert_full(
            &outline.xml_url, &outline.title, "",
            outline.html_url.as_deref().unwrap_or(""), "rss",
        ) {
            Ok(feed) => {
                println!("[3] Inserted: id={} title={}", feed.id, feed.title);
                inserted += 1;
            }
            Err(e) => println!("[3] Error: {}", e),
        }
    }
    println!("[3] Total inserted: {}", inserted);

    // Step 4: Show all feeds
    let all = feed_repo.find_all().expect("Failed to list feeds");
    println!("\n[4] Database contents ({} feeds):", all.len());
    for f in &all {
        println!("    id={}  title=\"{}\"  url={}", f.id, f.title, f.url);
    }

    let summaries = feed_repo.find_all_with_unread_count().expect("Failed");
    println!("\n[5] Feed summaries (sidebar view):");
    for s in &summaries {
        println!("    id={}  title=\"{}\"  unread={}", s.id, s.title, s.unread_count);
    }

    println!("\n  DB path: {}", db_path.display());
    println!("  Inspect: sqlite3 {} \".tables\"", db_path.display());
    println!("  Data:    sqlite3 {} \"SELECT * FROM feeds;\"", db_path.display());

    std::fs::remove_file(&opml_path).ok();
    println!("\nDone.");
}
