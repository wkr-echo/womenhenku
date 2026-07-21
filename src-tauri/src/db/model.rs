use serde::{Deserialize, Serialize};

// === Feed ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct FeedSummary {
    pub id: i64,
    pub title: String,
    pub unread_count: i64,
}

// === Entry ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct EntryListItem {
    pub id: i64,
    pub feed_id: i64,
    pub title: String,
    pub author: String,
    pub summary: String,
    pub published_at: Option<String>,
    pub is_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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

// === Note ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: i64,
    pub entry_id: i64,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

// === DigestTemplate ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DigestTemplate {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub body: String,
    pub format: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

// === Provider ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: i64,
    pub name: String,
    pub base_url: String,
    pub api_key_ref: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProvider {
    pub name: String,
    pub base_url: String,
    pub api_key_ref: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProvider {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_key_ref: Option<String>,
    pub is_default: Option<bool>,
}

// === ProviderModel ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModel {
    pub id: i64,
    pub provider_id: i64,
    pub model_name: String,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProviderModel {
    pub provider_id: i64,
    pub model_name: String,
    pub is_default: bool,
}

// === AgentRun ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRun {
    pub id: i64,
    pub entry_id: i64,
    pub provider_id: i64,
    pub task_kind: String,
    pub phase: String,
    pub target_language: String,
    pub detail_level: Option<String>,
    pub output_text: Option<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAgentRun {
    pub entry_id: i64,
    pub provider_id: i64,
    pub task_kind: String,
    pub target_language: String,
    pub detail_level: Option<String>,
}

// === Tags ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub normalized_name: String,
    pub color: String,
    pub is_provisional: bool,
    pub usage_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryTag {
    pub entry_id: i64,
    pub tag_id: i64,
    pub source: String,
    pub confidence: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagAlias {
    pub id: i64,
    pub tag_id: i64,
    pub alias: String,
    pub normalized_alias: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagRecommendation {
    pub id: i64,
    pub entry_id: i64,
    pub tag_name: String,
    pub source_type: String,
    pub confidence: f64,
    pub created_at: String,
}

// === LLM Usage ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmUsageEvent {
    pub id: i64,
    pub provider_id: i64,
    pub provider_name: String,
    pub provider_base_url: String,
    pub provider_host: String,
    pub model_id: i64,
    pub model_name: String,
    pub agent_type: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub request_status: String,
    pub timestamp: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmUsageStats {
    pub total_tokens: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub request_count: i64,
    pub success_rate: f64,
    pub avg_tokens_per_request: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyUsage {
    pub date: String,
    pub total_tokens: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub request_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUsage {
    pub provider_id: i64,
    pub provider_name: String,
    pub total_tokens: i64,
    pub request_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub model_id: i64,
    pub model_name: String,
    pub total_tokens: i64,
    pub request_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUsage {
    pub agent_type: String,
    pub total_tokens: i64,
    pub request_count: i64,
}

// === Settings ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}
