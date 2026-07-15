// Digest export service layer — Stage 4
//
// Exports single or multiple entries as digest strings in Markdown, HTML, or plaintext.
// Uses the repository layer to fetch entry metadata, content, and notes.

use crate::db::model::{Content, Entry, Note};
use crate::db::repository::{ContentRepository, EntryRepository, NoteRepository};
use crate::db::DbPool;

/// Digest output format.
#[derive(Debug, Clone, PartialEq)]
pub enum DigestFormat {
    Markdown,
    Html,
    Plaintext,
}

impl DigestFormat {
    /// Parse format from a string (case-insensitive).
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(DigestFormat::Markdown),
            "html" => Ok(DigestFormat::Html),
            "plaintext" | "plain" | "txt" => Ok(DigestFormat::Plaintext),
            other => Err(format!("Unknown digest format: {}. Supported: markdown, html, plaintext", other)),
        }
    }
}

/// Holds the data needed to render a single entry in the digest.
struct DigestEntry {
    entry: Entry,
    content: Option<Content>,
    note: Option<Note>,
}

// ============================================================
// Public API
// ============================================================

/// Export a single entry as digest.
pub fn export_single(pool: &DbPool, entry_id: i64, format: &DigestFormat) -> Result<String, String> {
    let entry = fetch_entry(pool, entry_id)?;
    let content = fetch_content(pool, entry_id);
    let note = fetch_note(pool, entry_id);

    match format {
        DigestFormat::Markdown => render_single_markdown(&entry, content.as_ref(), note.as_ref()),
        DigestFormat::Html => render_single_html(&entry, content.as_ref(), note.as_ref()),
        DigestFormat::Plaintext => render_single_plaintext(&entry, content.as_ref(), note.as_ref()),
    }
}

/// Export multiple entries as a combined digest.
pub fn export_multi(pool: &DbPool, entry_ids: &[i64], format: &DigestFormat) -> Result<String, String> {
    let mut digest_entries: Vec<DigestEntry> = Vec::new();

    for &id in entry_ids {
        let entry = fetch_entry(pool, id)?;
        let content = fetch_content(pool, id);
        let note = fetch_note(pool, id);
        digest_entries.push(DigestEntry { entry, content, note });
    }

    match format {
        DigestFormat::Markdown => render_multi_markdown(&digest_entries),
        DigestFormat::Html => render_multi_html(&digest_entries),
        DigestFormat::Plaintext => render_multi_plaintext(&digest_entries),
    }
}

// ============================================================
// Data fetching helpers
// ============================================================

fn fetch_entry(pool: &DbPool, entry_id: i64) -> Result<Entry, String> {
    let repo = EntryRepository::new(pool.clone());
    repo.find_by_id(entry_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Entry id={} not found", entry_id))
}

fn fetch_content(pool: &DbPool, entry_id: i64) -> Option<Content> {
    let repo = ContentRepository::new(pool.clone());
    repo.find_by_entry_id(entry_id).ok().flatten()
}

fn fetch_note(pool: &DbPool, entry_id: i64) -> Option<Note> {
    let repo = NoteRepository::new(pool.clone());
    repo.find_by_entry_id(entry_id).ok().flatten()
}

// ============================================================
// Single-entry renderers
// ============================================================

fn render_single_markdown(entry: &Entry, content: Option<&Content>, note: Option<&Note>) -> Result<String, String> {
    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", entry.title));

    if !entry.author.is_empty() {
        md.push_str(&format!("**作者**: {}\n\n", entry.author));
    }
    if let Some(ref pub_date) = entry.published_at {
        md.push_str(&format!("**发布日期**: {}\n\n", pub_date));
    }
    md.push_str(&format!("**原文链接**: {}\n\n", entry.link));
    md.push_str("---\n\n");

    if let Some(c) = content {
        if let Some(ref cleaned) = c.cleaned_markdown {
            md.push_str(cleaned);
            md.push_str("\n\n");
        } else if !c.raw_html.is_empty() {
            md.push_str("> (原始 HTML 内容，尚未经过清洗)\n\n");
        }
    }

    if let Some(n) = note {
        if !n.content.is_empty() {
            md.push_str("---\n\n## 笔记\n\n");
            md.push_str(&n.content);
            md.push('\n');
        }
    }

    Ok(md)
}

fn render_single_html(entry: &Entry, content: Option<&Content>, note: Option<&Note>) -> Result<String, String> {
    let mut html = String::from("<!DOCTYPE html>\n<html>\n<head><meta charset=\"utf-8\">\n");
    html.push_str(&format!("<title>{}</title>\n", escape_html(&entry.title)));
    html.push_str("</head>\n<body>\n");

    html.push_str(&format!("<h1>{}</h1>\n", escape_html(&entry.title)));

    if !entry.author.is_empty() {
        html.push_str(&format!("<p><strong>作者:</strong> {}</p>\n", escape_html(&entry.author)));
    }
    if let Some(ref pub_date) = entry.published_at {
        html.push_str(&format!("<p><strong>发布日期:</strong> {}</p>\n", escape_html(pub_date)));
    }
    html.push_str(&format!("<p><a href=\"{}\">原文链接</a></p>\n", escape_html(&entry.link)));
    html.push_str("<hr>\n");

    if let Some(c) = content {
        if let Some(ref rendered) = c.rendered_html {
            html.push_str(rendered);
            html.push('\n');
        } else if let Some(ref cleaned) = c.cleaned_html {
            html.push_str(cleaned);
            html.push('\n');
        } else if !c.raw_html.is_empty() {
            html.push_str(&format!("<pre>{}</pre>\n", escape_html(&c.raw_html)));
        }
    }

    if let Some(n) = note {
        if !n.content.is_empty() {
            html.push_str("<hr>\n<h2>笔记</h2>\n");
            html.push_str(&format!("<pre>{}</pre>\n", escape_html(&n.content)));
        }
    }

    html.push_str("</body>\n</html>\n");
    Ok(html)
}

fn render_single_plaintext(entry: &Entry, content: Option<&Content>, note: Option<&Note>) -> Result<String, String> {
    let mut txt = String::new();
    txt.push_str(&format!("{}\n", entry.title));
    txt.push_str(&format!("{}\n\n", "=".repeat(entry.title.chars().count())));

    if !entry.author.is_empty() {
        txt.push_str(&format!("作者: {}\n", entry.author));
    }
    if let Some(ref pub_date) = entry.published_at {
        txt.push_str(&format!("发布日期: {}\n", pub_date));
    }
    txt.push_str(&format!("原文链接: {}\n\n", entry.link));

    if let Some(c) = content {
        if let Some(ref cleaned) = c.cleaned_markdown {
            txt.push_str(cleaned);
            txt.push_str("\n\n");
        } else if !c.raw_html.is_empty() {
            txt.push_str("(原始 HTML 内容，尚未经过清洗)\n\n");
        }
    }

    if let Some(n) = note {
        if !n.content.is_empty() {
            txt.push_str("--- 笔记 ---\n");
            txt.push_str(&n.content);
            txt.push('\n');
        }
    }

    Ok(txt)
}

// ============================================================
// Multi-entry renderers
// ============================================================

fn render_multi_markdown(entries: &[DigestEntry]) -> Result<String, String> {
    let mut md = String::from("# 文摘合集\n\n");
    md.push_str(&format!("共 {} 篇文章\n\n---\n\n", entries.len()));

    for de in entries {
        md.push_str(&format!("## {}\n\n", de.entry.title));
        if let Some(ref c) = de.content {
            if let Some(ref cleaned) = c.cleaned_markdown {
                md.push_str(cleaned);
                md.push_str("\n\n");
            }
        }
        if let Some(ref n) = de.note {
            if !n.content.is_empty() {
                md.push_str(&format!("> 笔记: {}\n\n", n.content));
            }
        }
        md.push_str("---\n\n");
    }

    Ok(md)
}

fn render_multi_html(entries: &[DigestEntry]) -> Result<String, String> {
    let mut html = String::from("<!DOCTYPE html>\n<html>\n<head><meta charset=\"utf-8\">\n");
    html.push_str("<title>文摘合集</title>\n</head>\n<body>\n");
    html.push_str(&format!("<h1>文摘合集</h1>\n<p>共 {} 篇文章</p>\n<hr>\n", entries.len()));

    for de in entries {
        html.push_str(&format!("<h2>{}</h2>\n", escape_html(&de.entry.title)));
        if let Some(ref c) = de.content {
            if let Some(ref rendered) = c.rendered_html {
                html.push_str(rendered);
                html.push('\n');
            } else if let Some(ref cleaned) = c.cleaned_html {
                html.push_str(cleaned);
                html.push('\n');
            }
        }
        if let Some(ref n) = de.note {
            if !n.content.is_empty() {
                html.push_str(&format!("<blockquote>笔记: {}</blockquote>\n", escape_html(&n.content)));
            }
        }
        html.push_str("<hr>\n");
    }

    html.push_str("</body>\n</html>\n");
    Ok(html)
}

fn render_multi_plaintext(entries: &[DigestEntry]) -> Result<String, String> {
    let mut txt = String::from("文摘合集\n========\n\n");
    txt.push_str(&format!("共 {} 篇文章\n\n", entries.len()));

    for de in entries {
        txt.push_str(&format!("{}\n", de.entry.title));
        txt.push_str(&format!("{}\n", "-".repeat(de.entry.title.chars().count())));
        if let Some(ref c) = de.content {
            if let Some(ref cleaned) = c.cleaned_markdown {
                txt.push_str(cleaned);
                txt.push_str("\n");
            }
        }
        if let Some(ref n) = de.note {
            if !n.content.is_empty() {
                txt.push_str(&format!("[笔记] {}\n", n.content));
            }
        }
        txt.push('\n');
    }

    Ok(txt)
}

// ============================================================
// Utility
// ============================================================

/// Minimal HTML escaping for safe output.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}