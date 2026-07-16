use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use quick_xml::events::Event;
use quick_xml::Reader;
use tokio::sync::Semaphore;

use crate::db::model::Feed;
use crate::db::repository::{EntryRepository, FeedRepository};
use crate::db::DbPool;

const BATCH_SIZE: usize = 24;
const SYNC_CONCURRENCY: usize = 6;

// ============================================================
// OPML parsing
// ============================================================

#[derive(Debug, thiserror::Error)]
pub enum OpmlError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Invalid OPML: {0}")]
    Invalid(String),
    #[error("Database error: {0}")]
    Database(#[from] crate::db::error::RepositoryError),
}

#[derive(Debug, Clone)]
pub struct OpmlOutline {
    pub title: String,
    pub xml_url: String,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub xml_url: String,
    pub title: String,
    pub success: bool,
    pub message: String,
}

pub fn parse_opml_file(path: &Path) -> Result<Vec<OpmlOutline>, OpmlError> {
    let content = fs::read_to_string(path)?;
    parse_opml(&content)
}

pub fn parse_opml(xml: &str) -> Result<Vec<OpmlOutline>, OpmlError> {
    let mut reader = Reader::from_str(xml);
    let mut outlines = Vec::new();
    let mut buf = Vec::new();
    let decoder = reader.decoder();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"outline" {
                    let mut title = String::new();
                    let mut xml_url = String::new();
                    let mut html_url = None;

                    for attr_result in e.attributes() {
                        let attr = attr_result.map_err(|e| OpmlError::Xml(e.into()))?;
                        match attr.key.as_ref() {
                            b"text" | b"title" => {
                                title = attr.decode_and_unescape_value(decoder).map_err(OpmlError::Xml)?.into_owned();
                            }
                            b"xmlUrl" => {
                                xml_url = attr.decode_and_unescape_value(decoder).map_err(OpmlError::Xml)?.into_owned();
                            }
                            b"htmlUrl" => {
                                html_url = Some(attr.decode_and_unescape_value(decoder).map_err(OpmlError::Xml)?.into_owned());
                            }
                            _ => {}
                        }
                    }

                    if !xml_url.is_empty() {
                        if title.is_empty() {
                            title = xml_url.clone();
                        }
                        outlines.push(OpmlOutline { title, xml_url, html_url });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OpmlError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    if outlines.is_empty() {
        return Err(OpmlError::Invalid("No feed outlines found".into()));
    }
    Ok(outlines)
}

// ============================================================
// Import with filtering, batching, title resolution, auto-sync
// ============================================================

pub fn import_feeds(pool: &DbPool, outlines: &[OpmlOutline]) -> Vec<ImportResult> {
    let mut results = Vec::new();
    let feed_repo = FeedRepository::new(pool.clone());
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("platinum/0.2 (RSS Reader)")
        .build()
        .expect("reqwest client");

    // Step 1-2: Filter HTTPS only
    let https_outlines: Vec<&OpmlOutline> = outlines.iter().filter(|o| {
        let ok = o.xml_url.starts_with("https://");
        if !ok {
            tracing::warn!("OPML import: skipping HTTP feed {}", o.xml_url);
            results.push(ImportResult {
                xml_url: o.xml_url.clone(), title: o.title.clone(),
                success: false, message: "HTTP not allowed (HTTPS required)".into(),
            });
        }
        ok
    }).collect();

    // Step 3: Batch 24
    for batch in https_outlines.chunks(BATCH_SIZE) {
        for outline in batch {
            // Resolve title first
            let title = resolve_feed_title(&client, &outline.xml_url)
                .unwrap_or_else(|| outline.title.clone());

            if let Ok(Some(existing)) = feed_repo.find_by_url(&outline.xml_url) {
                // Update existing feed's title and link
                let _ = feed_repo.update_title(existing.id, &title);
                results.push(ImportResult {
                    xml_url: outline.xml_url.clone(), title,
                    success: true, message: format!("Updated (id={})", existing.id),
                });
                continue;
            }

            match feed_repo.insert_full(&outline.xml_url, &title, "",
                outline.html_url.as_deref().unwrap_or(""), "rss"
            ) {
                Ok(feed) => {
                    tracing::info!("OPML import: {} ({})", title, outline.xml_url);
                    results.push(ImportResult {
                        xml_url: outline.xml_url.clone(), title,
                        success: true, message: format!("id={}", feed.id),
                    });
                }
                Err(e) => {
                    results.push(ImportResult {
                        xml_url: outline.xml_url.clone(), title: outline.title.clone(),
                        success: false, message: e.to_string(),
                    });
                }
            }
        }
    }

    // Step 5: Auto-sync concurrency 6
    let ok_urls: Vec<String> = results.iter()
        .filter(|r| r.success).map(|r| r.xml_url.clone()).collect();

    if !ok_urls.is_empty() {
        tracing::info!("OPML auto-sync: {} feeds (concurrency={})", ok_urls.len(), SYNC_CONCURRENCY);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all().build().expect("tokio runtime");
        rt.block_on(async { sync_feeds(pool, &ok_urls, SYNC_CONCURRENCY).await });
    }

    results
}

fn resolve_feed_title(client: &reqwest::Client, url: &str) -> Option<String> {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().ok()?;
    rt.block_on(async {
        let bytes = client.get(url).send().await.ok()?.bytes().await.ok()?;
        feed_rs::parser::parse(&bytes[..]).ok()?.title.map(|t| t.content)
    })
}

async fn sync_feeds(pool: &DbPool, urls: &[String], concurrency: usize) {
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("platinum/0.2")
        .build().expect("reqwest client");
    let mut handles = Vec::new();

    for url in urls {
        let permit = semaphore.clone().acquire_owned().await;
        let client = client.clone();
        let url = url.clone();
        let pool = pool.clone();

        handles.push(tokio::task::spawn(async move {
            let _permit = permit;
            let feed_repo = FeedRepository::new(pool.clone());
            let feed_id = match feed_repo.find_by_url(&url) {
                Ok(Some(f)) => f.id, _ => return,
            };

            let bytes = match client.get(&url).send().await {
                Ok(r) => match r.bytes().await { Ok(b) => b, Err(_) => return },
                Err(_) => return,
            };

            let parsed = match crate::feed::parser::parse_feed(&bytes, &url) {
                Ok(p) => p, Err(_) => return,
            };

            let _ = feed_repo.update_title(feed_id, &parsed.title);
            let entry_repo = EntryRepository::new(pool);
            let mut count = 0usize;
            for mut entry in parsed.entries {
                entry.feed_id = feed_id;
                if entry_repo.insert_or_ignore(&entry).unwrap_or(None).is_some() {
                    count += 1;
                }
            }
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
            let _ = feed_repo.update_sync_time(feed_id, &now);
            tracing::info!("OPML sync: {} → {} new entries", url, count);
        }));
    }
    for h in handles { let _ = h.await; }
}

// ============================================================
// OPML export
// ============================================================

pub fn export_opml_file(pool: &DbPool, path: &Path) -> Result<(), OpmlError> {
    let feed_repo = FeedRepository::new(pool.clone());
    let feeds = feed_repo.find_all()?;
    fs::write(path, generate_opml(&feeds))?;
    Ok(())
}

fn generate_opml(feeds: &[Feed]) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(r#"<opml version="2.0">"#);
    xml.push('\n');
    xml.push_str("  <head>\n    <title>Mercury Subscriptions</title>\n  </head>\n  <body>\n");
    for feed in feeds {
        let h = if feed.link.is_empty() { String::new() } else { format!(r#" htmlUrl="{}""#, escape_xml(&feed.link)) };
        xml.push_str(&format!(
            r#"    <outline text="{}" title="{}" type="rss" xmlUrl="{}"{} />"#,
            escape_xml(&feed.title), escape_xml(&feed.title), escape_xml(&feed.url), h
        ));
        xml.push('\n');
    }
    xml.push_str("  </body>\n</opml>\n");
    xml
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db_pool;

    #[test]
    fn test_parse_opml_https_only() {
        let xml = r#"<?xml version="1.0"?><opml version="2.0"><head/><body>
            <outline xmlUrl="http://bad.example.com/rss"/>
            <outline xmlUrl="https://good.example.com/feed"/>
        </body></opml>"#;
        let outlines = parse_opml(xml).expect("parse");
        assert_eq!(outlines.len(), 2); // parser extracts both

        let pool = open_test_db_pool().expect("pool");
        let results = import_feeds(&pool, &outlines);
        // HTTP should be rejected
        let http = results.iter().find(|r| r.xml_url.contains("http://"));
        assert!(http.is_some());
        assert!(!http.unwrap().success);
    }

    #[test]
    fn test_roundtrip() {
        let feeds = vec![Feed {
            id: 1, url: "https://x.example.com/rss".into(), title: "X Blog".into(),
            description: "".into(), link: "https://x.example.com".into(),
            feed_type: "rss".into(), last_synced_at: None, created_at: "".into(),
        }];
        let xml = generate_opml(&feeds);
        let outlines = parse_opml(&xml).expect("roundtrip");
        assert_eq!(outlines[0].title, "X Blog");
    }
}
