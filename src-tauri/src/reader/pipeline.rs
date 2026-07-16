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

/// Convert sanitized HTML to GFM Markdown using simple regex-based conversion.
/// Handles the common tags from our whitelist: h1-h6, p, a, strong, em, del,
/// ul, ol, li, blockquote, pre/code, br, img.
pub fn to_markdown(html: &str) -> String {
    let mut md = html.to_string();

    // Replace inline tags (order matters: nested tags handled from inside out)
    // <strong> / <b> → **text**
    let re = regex::Regex::new(r"<(?:strong|b)>(.*?)</(?:strong|b)>").unwrap();
    md = re.replace_all(&md, "**$1**").to_string();

    // <em> / <i> → *text*
    let re = regex::Regex::new(r"<(?:em|i)>(.*?)</(?:em|i)>").unwrap();
    md = re.replace_all(&md, "*$1*").to_string();

    // <del> / <s> → ~~text~~
    let re = regex::Regex::new(r"<(?:del|s)>(.*?)</(?:del|s)>").unwrap();
    md = re.replace_all(&md, "~~$1~~").to_string();

    // <a href="url">text</a> → [text](url)
    let re = regex::Regex::new(r#"<a href="([^"]*)">(.*?)</a>"#).unwrap();
    md = re.replace_all(&md, "[$2]($1)").to_string();

    // <img src="url" alt="text"> → ![text](url)
    let re = regex::Regex::new(r#"<img src="([^"]*)"(?: alt="([^"]*)")?>"#).unwrap();
    md = re.replace_all(&md, "![$2]($1)").to_string();

    // <code>text</code> → `text` (inline code, not inside pre)
    let re = regex::Regex::new(r"<code>(.*?)</code>").unwrap();
    md = re.replace_all(&md, "`$1`").to_string();

    // <pre><code>...</code></pre> → ```\n...\n```
    let re = regex::Regex::new(r"<pre>\s*<code>(.*?)</code>\s*</pre>").unwrap();
    md = re.replace_all(&md, "\n```\n$1\n```\n").to_string();

    // Block-level tags
    // <h1> → # , <h2> → ## , etc.
    for level in (1..=6).rev() {
        let pattern = format!(r"<h{}>(.*?)</h{}>", level, level);
        let prefix = "#".repeat(level);
        let re = regex::Regex::new(&pattern).unwrap();
        md = re.replace_all(&md, format!("\n{} $1\n", prefix)).to_string();
    }

    // <li> → - item
    let re = regex::Regex::new(r"<li>(.*?)</li>").unwrap();
    md = re.replace_all(&md, "- $1").to_string();

    // <blockquote> → > text
    let re = regex::Regex::new(r"<blockquote>(.*?)</blockquote>").unwrap();
    md = re.replace_all(&md, "\n> $1\n").to_string();

    // Remove remaining block tags (keep inner text)
    for tag in &["p", "ul", "ol", "table", "thead", "tbody", "tr", "th", "td"] {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        md = md.replace(&open, "");
        md = md.replace(&close, "");
    }
    md = md.replace("</p>", "\n\n");

    // <br> → \n, <hr> → ---
    md = md.replace("<br>", "\n");
    md = md.replace("<hr>", "\n---\n");

    // Clean up: collapse multiple blank lines, trim
    let re = regex::Regex::new(r"\n{3,}").unwrap();
    md = re.replace_all(&md, "\n\n").to_string();
    md = md.trim().to_string();

    md
}

// ============================================================
// Step 4: Markdown to rendered HTML
// ============================================================

/// Render Markdown (GFM) to HTML wrapped in a reader theme container.
/// Injects CSS custom properties for light/dark theme support.
pub fn render(markdown: &str) -> String {
    let mut options = comrak::ComrakOptions::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.tagfilter = true;
    options.render.unsafe_ = false;
    options.render.hardbreaks = true;

    let body = comrak::markdown_to_html(markdown, &options);

    format!(
        r#"<div class="reader-theme" style="
  --mercury-bg-primary: var(--bg-primary);
  --mercury-bg-secondary: var(--bg-secondary);
  --mercury-text-primary: var(--text-primary);
  --mercury-text-secondary: var(--text-secondary);
  --mercury-link-color: var(--link-color);
  --mercury-border-color: var(--border-color);
  --mercury-code-bg: var(--bg-tertiary);
  --mercury-blockquote-border: var(--accent-color);
  background: var(--mercury-bg-primary);
  color: var(--mercury-text-primary);
  font-family: var(--reader-font, system-ui);
  line-height: 1.8;
  max-width: 720px;
  margin: 0 auto;
  padding: 2rem 1rem;
">
  <style>
    .reader-theme h1, .reader-theme h2, .reader-theme h3, .reader-theme h4, .reader-theme h5, .reader-theme h6 {{ font-family: inherit; margin-top: 1.5em; margin-bottom: 0.5em; }}
    .reader-theme p {{ margin-bottom: 1em; }}
    .reader-theme a {{ color: var(--mercury-link-color); }}
    .reader-theme pre {{ background: var(--mercury-code-bg); padding: 1em; border-radius: 6px; overflow-x: auto; font-family: var(--reader-code-font, monospace); }}
    .reader-theme code {{ background: var(--mercury-code-bg); padding: 0.2em 0.4em; border-radius: 3px; font-size: 0.9em; font-family: var(--reader-code-font, monospace); }}
    .reader-theme blockquote {{ border-left: 3px solid var(--mercury-blockquote-border); padding-left: 1em; margin-left: 0; color: var(--mercury-text-secondary); }}
    .reader-theme table {{ border-collapse: collapse; width: 100%; margin-bottom: 1em; }}
    .reader-theme th, .reader-theme td {{ border: 1px solid var(--mercury-border-color); padding: 0.5em 0.75em; text-align: left; }}
    .reader-theme th {{ background: var(--mercury-bg-secondary); }}
    .reader-theme img {{ max-width: 100%; height: auto; }}
    .reader-theme ul, .reader-theme ol {{ padding-left: 1.5em; margin-bottom: 1em; }}
    .reader-theme hr {{ border: none; border-top: 1px solid var(--mercury-border-color); margin: 2em 0; }}
  </style>
  {body}
</div>"#
    )
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
