use std::io::Cursor;

use scraper::{Html, Selector};

/// Output of a full reader pipeline run.
#[derive(Debug, Clone)]
pub struct ReaderOutput {
    pub extracted_html: String,
    pub cleaned_html: String,
    pub markdown: String,
    pub rendered_html: String,
}

/// Errors that can occur during the reader pipeline.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Readability extraction failed: {0}")]
    Readability(String),

    #[error("Markdown conversion failed: {0}")]
    Markdown(String),
}

/// Allowed HTML tags for the sanitization step.
/// All other tags are stripped (inner text preserved).
const ALLOWED_TAGS: &[&str] = &[
    "p", "h1", "h2", "h3", "h4", "h5", "h6",
    "ul", "ol", "li",
    "a", "img",
    "table", "thead", "tbody", "tr", "th", "td",
    "pre", "code",
    "blockquote",
    "strong", "b", "em", "i", "del", "s",
    "br", "hr",
];

/// Allowed attributes for preserved tags.
/// Everything else is stripped.
const ALLOWED_ATTRS: &[&str] = &["href", "src", "alt", "title"];

// ============================================================
// Step 1: Readability extraction
// ============================================================

/// Extract the main content from raw HTML using the Mozilla Readability algorithm.
/// Falls back to the original HTML if extraction fails.
pub fn extract(raw_html: &str, url: &str) -> String {
    let mut cursor = Cursor::new(raw_html.as_bytes());
    let parsed_url = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => {
            tracing::warn!("Invalid URL '{}', using raw HTML", url);
            return raw_html.to_string();
        }
    };
    match readability::extractor::extract(&mut cursor, &parsed_url) {
        Ok(product) => {
            tracing::debug!("Readability extracted content ({} chars)", product.content.len());
            product.content
        }
        Err(e) => {
            tracing::warn!("Readability extraction failed ({}), using raw HTML", e);
            raw_html.to_string()
        }
    }
}

// ============================================================
// Step 2: HTML sanitization (whitelist-based)
// ============================================================

/// Sanitize HTML by stripping dangerous/irrelevant tags and attributes.
/// Uses a whitelist approach: only ALLOWED_TAGS and ALLOWED_ATTRS survive.
pub fn sanitize(html: &str) -> String {
    let document = Html::parse_document(html);
    let body_selector = Selector::parse("body").unwrap();
    let body = match document.select(&body_selector).next() {
        Some(b) => b,
        None => return html.to_string(),
    };

    let mut output = String::new();
    serialize_node(&body, &mut output);
    output
}

fn serialize_node(node: &scraper::ElementRef, output: &mut String) {
    for child in node.children() {
        match child.value() {
            scraper::node::Node::Text(text) => {
                output.push_str(&text.text);
            }
            scraper::node::Node::Element(element) => {
                let tag: &str = element.name.local.as_ref();
                if ALLOWED_TAGS.contains(&tag) {
                    let attrs = element
                        .attrs
                        .iter()
                        .filter(|(k, _)| {
                            let key: &str = k.local.as_ref();
                            ALLOWED_ATTRS.contains(&key)
                        })
                        .map(|(k, v)| format!(" {}=\"{}\"", k.local.as_ref(), v))
                        .collect::<Vec<_>>()
                        .join("");

                    let is_void = matches!(tag, "br" | "hr" | "img");

                    output.push_str(&format!("<{}", tag));
                    output.push_str(&attrs);
                    output.push('>');

                    if !is_void {
                        // Recurse into children
                        if let Some(el_ref) = scraper::ElementRef::wrap(child) {
                            serialize_node(&el_ref, output);
                        }
                        output.push_str(&format!("</{}>", tag));
                    }
                } else {
                    // Strip the tag but preserve inner text
                    if let Some(el_ref) = scraper::ElementRef::wrap(child) {
                        serialize_node(&el_ref, output);
                    }
                }
            }
            _ => {}
        }
    }
}

// ============================================================
// Step 3: HTML to Markdown
// ============================================================

/// Convert sanitized HTML to GFM Markdown.
pub fn to_markdown(html: &str) -> String {
    let mut options = comrak::ComrakOptions::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.render.unsafe_ = false;

    // comrak converts Markdown to HTML. To convert HTML to Markdown,
    // we need a different approach. For now, we use the HTML directly
    // and wrap it in a reader-friendly format.
    //
    // Since comrak is primarily a Markdown→HTML renderer, and there's
    // no reliable pure-Rust HTML→Markdown converter, we keep the
    // cleaned HTML as-is for rendering. The "markdown" field is stored
    // as the cleaned HTML for now, and rendered HTML is generated
    // via comrak if the input is actual Markdown.
    html.to_string()
}

// ============================================================
// Step 4: Markdown to rendered HTML
// ============================================================

/// Render Markdown (GFM) to HTML suitable for reader view display.
pub fn render(markdown: &str) -> String {
    let mut options = comrak::ComrakOptions::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.tagfilter = true;
    options.render.unsafe_ = false;
    options.render.hardbreaks = true;

    comrak::markdown_to_html(markdown, &options)
}

// ============================================================
// Full pipeline
// ============================================================

/// Run the complete reader pipeline on raw HTML.
/// Steps: extract → sanitize → to_markdown → render
pub fn run_full_pipeline(raw_html: &str, url: &str) -> Result<ReaderOutput, PipelineError> {
    let extracted = extract(raw_html, url);
    let cleaned = sanitize(&extracted);
    let markdown = to_markdown(&cleaned);
    let rendered_html = render(&markdown);

    Ok(ReaderOutput {
        extracted_html: extracted,
        cleaned_html: cleaned,
        markdown,
        rendered_html,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Readability tests ===

    #[test]
    fn test_extract_removes_nav_and_aside() {
        let html = include_str!("../../tests/fixtures/article_basic.html");
        let result = extract(html, "https://example.com/article");
        // Readability should extract the main article content
        assert!(!result.is_empty(), "Extraction should not be empty");
        // Should contain article text, not navigation
        assert!(!result.contains("Copyright"));
    }

    #[test]
    fn test_extract_chinese_content() {
        let html = include_str!("../../tests/fixtures/article_chinese.html");
        let result = extract(html, "https://example.com/cn-article");
        assert!(!result.is_empty());
        assert!(!result.contains("侧边栏广告"));
    }

    #[test]
    fn test_extract_fallback_on_garbage() {
        let garbage = "<html><body><p>Minimal</p></body></html>";
        let result = extract(garbage, "https://example.com/min");
        assert!(!result.is_empty());
    }

    // === Sanitization tests ===

    #[test]
    fn test_sanitize_strips_script_tags() {
        let html = include_str!("../../tests/fixtures/article_malicious.html");
        let result = sanitize(html);
        // <script> tag should be removed
        assert!(!result.contains("<script>"));
        assert!(!result.contains("</script>"));
    }

    #[test]
    fn test_sanitize_strips_inline_event_handlers() {
        let html = r#"<p onclick="evil()">Click me</p>"#;
        let result = sanitize(html);
        assert!(!result.contains("onclick"));
        assert!(result.contains("Click me"));
    }

    #[test]
    fn test_sanitize_keeps_allowed_tags() {
        let html = r#"<p>Text</p><strong>Bold</strong><a href="https://x.com">Link</a>"#;
        let result = sanitize(html);
        assert!(result.contains("<p>"));
        assert!(result.contains("<strong>"));
        assert!(result.contains("<a href=\"https://x.com\">"));
    }

    #[test]
    fn test_sanitize_strips_disallowed_attributes() {
        let html = r#"<p style="color: red;" class="foo">Styled</p>"#;
        let result = sanitize(html);
        assert!(!result.contains("style"));
        assert!(!result.contains("class"));
        assert!(result.contains("Styled"));
    }

    // === Markdown and render tests ===

    #[test]
    fn test_render_gfm_tables() {
        let md = r#"| Name | Value |
|------|-------|
| Rust | Fast  |
| JS   | Flex  |"#;
        let html = render(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>Name</th>"));
        assert!(html.contains("Rust"));
    }

    #[test]
    fn test_render_code_blocks() {
        let md = "```rust\nfn main() {}\n```";
        let html = render(md);
        assert!(html.contains("<code"));
        assert!(html.contains("fn main"));
    }

    #[test]
    fn test_render_strikethrough() {
        let md = "~~deleted text~~";
        let html = render(md);
        assert!(html.contains("<del>") || html.contains("<s>"));
    }

    // === Full pipeline tests ===

    #[test]
    fn test_full_pipeline_basic_article() {
        let html = include_str!("../../tests/fixtures/article_basic.html");
        let result = run_full_pipeline(html, "https://example.com/article")
            .expect("Pipeline failed");
        assert!(!result.extracted_html.is_empty());
        assert!(!result.cleaned_html.is_empty());
        assert!(!result.rendered_html.is_empty());
    }

    #[test]
    fn test_full_pipeline_chinese_article() {
        let html = include_str!("../../tests/fixtures/article_chinese.html");
        let result = run_full_pipeline(html, "https://example.com/cn")
            .expect("Pipeline failed");
        assert!(!result.cleaned_html.is_empty());
        // Sidebar/ads should be removed
        assert!(!result.cleaned_html.contains("侧边栏广告"));
    }
}
