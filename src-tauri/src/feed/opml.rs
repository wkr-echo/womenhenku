use std::fs;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::db::model::Feed;
use crate::db::repository::FeedRepository;
use crate::db::DbPool;

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

/// Parse an OPML file and extract feed outlines.
pub fn parse_opml_file(path: &Path) -> Result<Vec<OpmlOutline>, OpmlError> {
    let content = fs::read_to_string(path)?;
    parse_opml(&content)
}

/// Parse OPML XML content and extract feed outlines.
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
                                title = attr
                                    .decode_and_unescape_value(decoder)
                                    .map_err(OpmlError::Xml)?
                                    .into_owned();
                            }
                            b"xmlUrl" => {
                                xml_url = attr
                                    .decode_and_unescape_value(decoder)
                                    .map_err(OpmlError::Xml)?
                                    .into_owned();
                            }
                            b"htmlUrl" => {
                                html_url = Some(
                                    attr.decode_and_unescape_value(decoder)
                                        .map_err(OpmlError::Xml)?
                                        .into_owned(),
                                );
                            }
                            _ => {}
                        }
                    }

                    if !xml_url.is_empty() {
                        if title.is_empty() {
                            title = xml_url.clone();
                        }
                        outlines.push(OpmlOutline {
                            title,
                            xml_url,
                            html_url,
                        });
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
        return Err(OpmlError::Invalid("No feed outlines found in OPML file".into()));
    }

    Ok(outlines)
}

/// Export all feeds to an OPML file.
pub fn export_opml_file(pool: &DbPool, path: &Path) -> Result<(), OpmlError> {
    let feed_repo = FeedRepository::new(pool.clone());
    let feeds = feed_repo.find_all()?;
    let xml = generate_opml(&feeds);
    fs::write(path, xml)?;
    Ok(())
}

/// Generate OPML XML string from a list of feeds.
fn generate_opml(feeds: &[Feed]) -> String {
    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(r#"<opml version="2.0">"#);
    xml.push('\n');
    xml.push_str("  <head>\n");
    xml.push_str("    <title>Womenhenku Subscriptions</title>\n");
    xml.push_str("  </head>\n");
    xml.push_str("  <body>\n");

    for feed in feeds {
        let html_url_attr = if feed.link.is_empty() {
            String::new()
        } else {
            format!(r#" htmlUrl="{}""#, escape_xml(&feed.link))
        };
        xml.push_str(&format!(
            r#"    <outline text="{}" type="rss" xmlUrl="{}"{} />"#,
            escape_xml(&feed.title),
            escape_xml(&feed.url),
            html_url_attr,
        ));
        xml.push('\n');
    }

    xml.push_str("  </body>\n");
    xml.push_str("</opml>\n");
    xml
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db_pool;
    use crate::db::repository::FeedRepository;

    const SAMPLE_OPML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <head><title>Test Subscriptions</title></head>
  <body>
    <outline text="Blog A" type="rss" xmlUrl="https://a.example.com/feed.xml" htmlUrl="https://a.example.com/"/>
    <outline text="Blog B" type="rss" xmlUrl="https://b.example.com/rss"/>
    <outline text="Folder">
      <outline text="Blog C" type="rss" xmlUrl="https://c.example.com/atom.xml"/>
    </outline>
  </body>
</opml>"#;

    #[test]
    fn test_parse_opml_extracts_outlines() {
        let outlines = parse_opml(SAMPLE_OPML).expect("Parse failed");
        assert_eq!(outlines.len(), 3);
        assert_eq!(outlines[0].title, "Blog A");
        assert_eq!(outlines[0].xml_url, "https://a.example.com/feed.xml");
        assert_eq!(outlines[0].html_url.as_deref(), Some("https://a.example.com/"));
        assert_eq!(outlines[1].title, "Blog B");
        assert_eq!(outlines[1].html_url, None);
        assert_eq!(outlines[2].title, "Blog C");
    }

    #[test]
    fn test_parse_opml_empty_returns_error() {
        let xml = r#"<?xml version="1.0"?><opml version="2.0"><head/><body/></opml>"#;
        assert!(parse_opml(xml).is_err());
    }

    #[test]
    fn test_parse_opml_skips_folders_without_xmlurl() {
        let xml = r#"<?xml version="1.0"?>
        <opml version="2.0">
          <head/><body>
            <outline text="Just a folder"/>
            <outline text="Real Feed" type="rss" xmlUrl="https://real.example.com/feed"/>
          </body>
        </opml>"#;
        let outlines = parse_opml(xml).expect("Parse failed");
        assert_eq!(outlines.len(), 1);
        assert_eq!(outlines[0].xml_url, "https://real.example.com/feed");
    }

    #[test]
    fn test_generate_and_parse_roundtrip() {
        let feeds = vec![
            Feed {
                id: 1, url: "https://x.example.com/rss".into(), title: "X Blog".into(),
                description: "".into(), link: "https://x.example.com".into(),
                feed_type: "rss".into(), last_synced_at: None, created_at: "".into(),
            },
            Feed {
                id: 2, url: "https://y.example.com/atom".into(), title: "Y Blog".into(),
                description: "".into(), link: "".into(), feed_type: "atom".into(),
                last_synced_at: None, created_at: "".into(),
            },
        ];

        let xml = generate_opml(&feeds);
        let outlines = parse_opml(&xml).expect("Roundtrip parse failed");
        assert_eq!(outlines.len(), 2);
        assert_eq!(outlines[0].title, "X Blog");
        assert_eq!(outlines[0].xml_url, "https://x.example.com/rss");
        assert_eq!(outlines[1].title, "Y Blog");
        assert_eq!(outlines[1].xml_url, "https://y.example.com/atom");
    }

    #[test]
    fn test_export_opml_to_file() {
        let pool = open_test_db_pool().expect("Failed to create test pool");
        let feed_repo = FeedRepository::new(pool.clone());
        feed_repo.insert_full(
            "https://export.example.com/rss", "Export Test", "Test desc",
            "https://export.example.com", "rss",
        ).expect("Insert failed");

        let tmp = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        export_opml_file(&pool, tmp.path()).expect("Export failed");

        let outlines = parse_opml_file(tmp.path()).expect("Parse failed");
        assert_eq!(outlines.len(), 1);
        assert_eq!(outlines[0].title, "Export Test");
        assert_eq!(outlines[0].xml_url, "https://export.example.com/rss");
    }

    #[test]
    fn test_escape_xml_special_chars() {
        let escaped = escape_xml(r#"AT&T "Blog" <cool> & 'fun'"#);
        assert!(!escaped.contains('<'));
        assert!(escaped.contains("&amp;"));
        assert!(escaped.contains("&quot;"));
    }
}
