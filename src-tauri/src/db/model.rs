use serde::{Deserialize, Serialize};

// === Feed ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub description: String,
    pub link: String,
    pub feed_type: String,
    pub last_synced_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSummary {
    pub id: i64,
    pub title: String,
    pub unread_count: i64,
}

// === Entry ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub author: String,
    pub link: String,
    pub summary: String,
    pub published_at: Option<String>,
    pub updated_at: Option<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryListItem {
    pub id: i64,
    pub feed_id: i64,
    pub title: String,
    pub author: String,
    pub published_at: Option<String>,
    pub is_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPage {
    pub entries: Vec<EntryListItem>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct NewEntry {
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub author: String,
    pub link: String,
    pub summary: String,
    pub published_at: Option<String>,
    pub updated_at: Option<String>,
}

// === Content ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub id: i64,
    pub entry_id: i64,
    pub raw_html: String,
    pub cleaned_html: Option<String>,
    pub cleaned_markdown: Option<String>,
    pub rendered_html: Option<String>,
    pub readability_version: i32,
    pub created_at: String,
    pub updated_at: Option<String>,
}
