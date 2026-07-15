use crate::db::model::NewEntry;
use chrono::{DateTime, Utc};

/// Parsed feed metadata extracted from a feed document.
#[derive(Debug, Clone)]
pub struct ParsedFeed {
    pub title: String,
    pub description: String,
    pub link: String,
    pub feed_type: String,
    pub entries: Vec<NewEntry>,
}

/// Parse raw feed bytes (RSS, Atom, or JSON Feed) into a ParsedFeed.
/// Returns an error only for truly unparseable input; individual entry
/// parse failures are logged and skipped.
pub fn parse_feed(bytes: &[u8], feed_url: &str) -> Result<ParsedFeed, ParseError> {
    let feed = feed_rs::parser::parse(bytes).map_err(|e| {
        ParseError::ParseFailure(format!("feed-rs parse error: {}", e))
    })?;

    let feed_type = match feed.feed_type {
        feed_rs::model::FeedType::RSS2 => "rss",
        feed_rs::model::FeedType::Atom => "atom",
        feed_rs::model::FeedType::JSON => "json",
        _ => "rss",
    };

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    let description = feed
        .description
        .map(|d| d.content)
        .unwrap_or_default();

    let link = feed
        .links
        .first()
        .map(|l| l.href.clone())
        .unwrap_or_else(|| feed_url.to_string());

    let mut entries = Vec::new();

    for entry in &feed.entries {
        let guid = entry
            .id
            .clone();

        let entry_title = entry
            .title
            .as_ref()
            .map(|t| t.content.clone())
            .unwrap_or_default();

        let entry_link = entry
            .links
            .first()
            .map(|l| l.href.clone())
            .unwrap_or_default();

        let author = entry
            .authors
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_default();

        let summary = entry
            .summary
            .as_ref()
            .map(|s| s.content.clone())
            .unwrap_or_default();

        let published_at = entry
            .published
            .map(|dt| format_datetime_utc(dt));

        let updated_at = entry.updated.map(|dt| format_datetime_utc(dt));

        entries.push(NewEntry {
            feed_id: 0, // caller fills this in
            guid,
            title: entry_title,
            author,
            link: entry_link,
            summary,
            published_at,
            updated_at,
        });
    }

    Ok(ParsedFeed {
        title,
        description,
        link,
        feed_type: feed_type.to_string(),
        entries,
    })
}

fn format_datetime_utc(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Errors that can occur during feed parsing.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Failed to parse feed: {0}")]
    ParseFailure(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss2_feed() {
        let xml = include_str!("../../tests/fixtures/sample_rss2.xml");
        let result = parse_feed(xml.as_bytes(), "https://example.com/blog/rss")
            .expect("RSS parse failed");
        assert_eq!(result.feed_type, "rss");
        assert_eq!(result.title, "Example RSS Blog");
        assert_eq!(result.description, "A sample RSS 2.0 feed for testing");
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].title, "First Post");
        assert!(result.entries[0].guid.contains("first-post"));
        assert_eq!(result.entries[1].title, "Second Post");
    }

    #[test]
    fn test_parse_atom_feed() {
        let xml = include_str!("../../tests/fixtures/sample_atom.xml");
        let result = parse_feed(xml.as_bytes(), "https://example.com/atom")
            .expect("Atom parse failed");
        assert_eq!(result.feed_type, "atom");
        assert_eq!(result.title, "Example Atom Blog");
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].title, "Atom Entry One");
        assert_eq!(result.entries[1].title, "Atom Entry Two");
    }

    #[test]
    fn test_parse_json_feed() {
        let json = include_str!("../../tests/fixtures/sample_jsonfeed.json");
        let result = parse_feed(json.as_bytes(), "https://example.com/jsonfeed/feed.json")
            .expect("JSON Feed parse failed");
        assert_eq!(result.feed_type, "json");
        assert_eq!(result.title, "Example JSON Feed");
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].title, "JSON Feed Item One");
    }

    #[test]
    fn test_parse_garbage_returns_error() {
        let garbage = b"this is not a valid feed";
        let result = parse_feed(garbage, "https://example.com/bad");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_feed_entries() {
        let xml = r#"<?xml version="1.0"?>
        <rss version="2.0">
          <channel>
            <title>Empty Feed</title>
            <link>https://example.com/empty</link>
            <description>No items here</description>
          </channel>
        </rss>"#;
        let result = parse_feed(xml.as_bytes(), "https://example.com/empty")
            .expect("Empty feed parse failed");
        assert_eq!(result.entries.len(), 0);
    }
}
