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
    "p", "div", "h1", "h2", "h3", "h4", "h5", "h6",
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
/// Note: Readability natively preserves <pre> blocks but may strip whitespace
/// text nodes between inline elements (e.g. syntax-highlighting <span> tags).
/// To prevent code block corruption, we extract <pre> blocks BEFORE Readability
/// and re-inject them after.
pub fn extract(raw_html: &str, url: &str) -> String {
    // --- Phase 0: Extract <pre> blocks from raw HTML ---
    // Regex preserves ALL whitespace between tags (unlike Readability's
    // DOM-based processing which may collapse inter-element whitespace).
    let pre_re = regex::Regex::new(r"(?s)<pre[^>]*>(.*?)</pre>").unwrap();
    let mut pre_blocks: Vec<String> = Vec::new();

    let protected_html = pre_re.replace_all(raw_html, |caps: &regex::Captures| {
        let idx = pre_blocks.len();
        let inner = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        pre_blocks.push(inner);
        // Short placeholder — keeps Readability's text analysis intact
        format!("<p>PREBLOCK_{}_RESTORE</p>", idx)
    }).to_string();

    // --- Phase 1: Run Readability on protected HTML ---
    let mut cursor = Cursor::new(protected_html.as_bytes());
    let parsed_url = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => {
            tracing::warn!("Invalid URL '{}', using raw HTML", url);
            return raw_html.to_string();
        }
    };
    let extracted = match readability::extractor::extract(&mut cursor, &parsed_url) {
        Ok(product) => {
            tracing::debug!("Readability extracted content ({} chars)", product.content.len());
            product.content
        }
        Err(e) => {
            tracing::warn!("Readability extraction failed ({}), using raw HTML", e);
            return raw_html.to_string();
        }
    };

    // --- Phase 2: Restore <pre> blocks ---
    let restore_re = regex::Regex::new(
        r"PREBLOCK_(\d+)_RESTORE"
    ).unwrap();

    let restored = restore_re.replace_all(&extracted, |caps: &regex::Captures| {
        let idx: usize = caps[1].parse().unwrap_or(usize::MAX);
        if let Some(inner) = pre_blocks.get(idx) {
            // Strip HTML tags from inner content but preserve all whitespace
            let tag_re = regex::Regex::new(r"<[^>]*>").unwrap();
            let cleaned = tag_re.replace_all(inner, "");
            format!("<pre><code>{}</code></pre>", cleaned)
        } else {
            String::new()
        }
    }).to_string();

    restored
}

// ============================================================
// Step 2: HTML sanitization (whitelist-based)
// ============================================================

/// Sanitize HTML by stripping dangerous/irrelevant tags and attributes.
/// Uses a whitelist approach: only ALLOWED_TAGS and ALLOWED_ATTRS survive.
///
/// Note: <pre> block whitespace preservation is handled by extract().
/// After extract(), <pre> blocks contain clean tag-free text, so scraper
/// correctly preserves all whitespace.
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

    // Replace inline tags (multiline-aware)
    let re = regex::Regex::new(r"<(?:strong|b)>([\s\S]*?)</(?:strong|b)>").unwrap();
    md = re.replace_all(&md, "**$1**").to_string();

    let re = regex::Regex::new(r"<(?:em|i)>([\s\S]*?)</(?:em|i)>").unwrap();
    md = re.replace_all(&md, "*$1*").to_string();

    let re = regex::Regex::new(r"<(?:del|s)>([\s\S]*?)</(?:del|s)>").unwrap();
    md = re.replace_all(&md, "~~$1~~").to_string();

    // <a href="url">text</a> → [text](url)
    let re = regex::Regex::new(r#"<a href="([^"]*)">(.*?)</a>"#).unwrap();
    md = re.replace_all(&md, "[$2]($1)").to_string();

    // <img src="url" alt="text"> → ![text](url)
    let re = regex::Regex::new(r#"<img src="([^"]*)"(?: alt="([^"]*)")?>"#).unwrap();
    md = re.replace_all(&md, "![$2]($1)").to_string();

    // <pre><code>...</code></pre> → ```\n...\n```
    let re = regex::Regex::new(r"<pre>[\s]*<code>([\s\S]*?)</code>[\s]*</pre>").unwrap();
    md = re.replace_all(&md, "\n```\n$1\n```\n").to_string();

    // <pre>...</pre> without <code> wrapper → ```\n...\n```
    // Must run AFTER the <pre><code> variant so we don't double-process.
    let re = regex::Regex::new(r"<pre>([\s\S]*?)</pre>").unwrap();
    md = re.replace_all(&md, "\n```\n$1\n```\n").to_string();

    // <code>text</code> → `text` (inline code, only outside <pre>)
    let re = regex::Regex::new(r"<code>([\s\S]*?)</code>").unwrap();
    md = re.replace_all(&md, "`$1`").to_string();

    // Block-level tags
    // <h1> → # , <h2> → ## , etc.
    for level in (1..=6).rev() {
        let pattern = format!(r"<h{}>([\s\S]*?)</h{}>", level, level);
        let prefix = "#".repeat(level);
        let re = regex::Regex::new(&pattern).unwrap();
        md = re.replace_all(&md, format!("\n{} $1\n", prefix)).to_string();
    }

    // <li> → - item (multiline, with newlines between items)
    let re = regex::Regex::new(r"<li>([\s\S]*?)</li>").unwrap();
    md = re.replace_all(&md, "\n- $1\n").to_string();

    // <blockquote> → > text (multiline)
    let re = regex::Regex::new(r"<blockquote>([\s\S]*?)</blockquote>").unwrap();
    md = re.replace_all(&md, "\n> $1\n").to_string();

    // Strip <p> and <div> tags: treat both as paragraph breaks
    md = md.replace("</p>", "\n\n");
    md = md.replace("<p>", "");
    md = md.replace("</div>", "\n");
    md = md.replace("<div>", "");

    // Remove remaining block container tags (keep inner text)
    for tag in &["ul", "ol", "table", "thead", "tbody", "tr", "th", "td"] {
        md = md.replace(&format!("<{}>", tag), "");
        md = md.replace(&format!("</{}>", tag), "");
    }

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
    .reader-theme pre {{ background: var(--mercury-code-bg); padding: 1em; border-radius: 6px; overflow-x: auto; font-family: var(--reader-code-font, monospace); white-space: pre; }}
    .reader-theme pre code {{ background: none; padding: 0; border-radius: 0; font-size: inherit; white-space: pre; }}
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

    #[test]
    fn test_extract_preserves_pre_blocks() {
        let html = r#"<html><body>
    <div>
        <p>这是一段中文说明文字。</p>
        <p>看代码示例：</p>
        <pre><code>def add(a, b):
    return a + b

print(add(1, 2))
print(add("hello", "world"))</code></pre>
        <p>Python 在运行时才进行类型检查。</p>
    </div>
</body></html>"#;
        let result = extract(html, "https://example.com/code");
        eprintln!("=== EXTRACT RESULT ===");
        eprintln!("{}", &result[..result.len().min(600)]);
        assert!(result.contains("<pre>"), "pre tag should be preserved");
        assert!(result.contains("def add"), "code content should be preserved");
        assert!(result.contains("return a + b"), "code newlines should be preserved");
    }

    /// Test the FULL pipeline (without pre-block protection) 
    /// to see if pre blocks survive all steps.
    #[test]
    fn test_full_pipeline_preserves_pre_blocks() {
        let html = r#"<html><body>
    <div>
        <p>这是一段中文说明文字，解释了代码的工作原理。</p>
        <p>看代码示例。先看动态编程语言 Python 的：</p>
        <pre><code>def add(a, b):
    return a + b

print(add(1, 2))
print(add("hello", "world"))
print(add(1, "world"))   # TypeError</code></pre>
        <p>Python 在运行时才进行类型检查，a 和 b 可以是任何类型。</p>
    </div>
</body></html>"#;
        // Use raw readability first (skip our extract wrapper)
        let mut cursor = Cursor::new(html.as_bytes());
        let url = url::Url::parse("https://example.com/python").unwrap();
        let extracted = readability::extractor::extract(&mut cursor, &url)
            .unwrap().content;
        
        eprintln!("=== AFTER READABILITY ===");
        eprintln!("{}", &extracted[..extracted.len().min(300)]);
        
        let cleaned = sanitize(&extracted);
        eprintln!("=== AFTER SANITIZE ===");
        eprintln!("{}", &cleaned[..cleaned.len().min(300)]);
        
        let md = to_markdown(&cleaned);
        eprintln!("=== AFTER MARKDOWN ===");
        eprintln!("{}", &md[..md.len().min(300)]);
        
        let rendered = render(&md);
        eprintln!("=== AFTER RENDER ===");
        eprintln!("{}", &rendered[..rendered.len().min(300)]);
        
        assert!(rendered.contains("def add"), "Rendered output should contain code. Got: {}", &rendered[..rendered.len().min(500)]);
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

    // === Code block preservation tests ===

    /// Standard <pre><code> pattern — should work
    #[test]
    fn test_to_markdown_pre_code_standard() {
        let html = "<p>Intro</p><pre><code>def add(a, b):\n    return a + b\n\nprint(add(1, 2))\nprint(add(\"hello\", \"world\"))</code></pre><p>Outro</p>";
        let md = to_markdown(html);
        eprintln!("=== MARKDOWN (standard pre+code) ===\n{}", md);
        // Should contain fenced code block, not inline backticks
        assert!(md.contains("```"), "Should have fenced code block, got: {}", &md[..md.len().min(300)]);
        assert!(md.contains("def add"), "Should preserve code content");
        assert!(md.contains("return a + b"), "Should preserve newlines in code");
    }

    /// <pre> WITHOUT <code> wrapper — common real-world pattern
    #[test]
    fn test_to_markdown_pre_without_code() {
        let html = "<p>Intro</p><pre>def add(a, b):\n    return a + b\n\nprint(add(1, 2))</pre><p>Outro</p>";
        let md = to_markdown(html);
        eprintln!("=== MARKDOWN (pre without code) ===\n{}", md);
        assert!(md.contains("```"), "Should convert bare <pre> to fenced code block, got: {}", &md[..md.len().min(300)]);
        assert!(md.contains("def add"), "Should preserve code content");
        assert!(md.contains("return a + b"), "Should preserve newlines");
    }

    /// <pre> with whitespace+text between <pre> and <code>
    #[test]
    fn test_to_markdown_pre_with_leading_text() {
        let html = "<pre>  <code>line1\nline2\nline3</code></pre>";
        let md = to_markdown(html);
        eprintln!("=== MARKDOWN (pre with leading whitespace before code) ===\n{}", md);
        assert!(md.contains("```"), "Should produce fenced code block, got: {}", &md[..md.len().min(300)]);
        assert!(md.contains("line1"), "Should contain code content");
    }

    /// Full pipeline: <pre> with syntax-highlighting <span> children
    /// (real-world scenario from blogs using Prism/Highlight.js)
    #[test]
    fn test_full_pipeline_pre_with_syntax_highlight_spans() {
        let html = r#"<html><body><div>
<p>看代码示例。先看动态编程语言 Python 的：</p>
<pre class="language-python"><code class="language-python"><span class="token keyword">def</span> <span class="token function">add</span><span class="token punctuation">(</span><span class="token parameter">a</span><span class="token punctuation">,</span> <span class="token parameter">b</span><span class="token punctuation">)</span><span class="token punctuation">:</span>
    <span class="token keyword">return</span> <span class="token parameter">a</span> <span class="token operator">+</span> <span class="token parameter">b</span>

<span class="token function">add</span><span class="token punctuation">(</span><span class="token number">1</span><span class="token punctuation">,</span> <span class="token number">2</span><span class="token punctuation">)</span>    <span class="token comment"># &rarr; 3</span>
<span class="token function">add</span><span class="token punctuation">(</span><span class="token string">"hello"</span><span class="token punctuation">,</span> <span class="token string">"world"</span><span class="token punctuation">)</span>    <span class="token comment"># &rarr; "helloworld"</span>
<span class="token function">add</span><span class="token punctuation">(</span><span class="token number">1</span><span class="token punctuation">,</span> <span class="token string">"2"</span><span class="token punctuation">)</span>    <span class="token comment"># &rarr; TypeError</span></code></pre>
<p>Python 在运行时才进行类型检查。</p>
</div></body></html>"#;
        let result = run_full_pipeline(html, "https://example.com/python-syntax")
            .expect("Pipeline failed");

        // Debug: print each pipeline step
        eprintln!("=== EXTRACTED (after Readability) ===\n{}", &result.extracted_html[..result.extracted_html.len().min(500)]);
        eprintln!("=== CLEANED HTML ===\n{}", &result.cleaned_html[..result.cleaned_html.len().min(500)]);
        eprintln!("=== MARKDOWN ===\n{}", &result.markdown[..result.markdown.len().min(500)]);

        // Verify code content is preserved with spaces
        assert!(result.cleaned_html.contains("def add"), 
            "cleaned_html should contain 'def add' with space. Got: {}", 
            &result.cleaned_html[..result.cleaned_html.len().min(500)]);
        assert!(result.rendered_html.contains("return a + b"), 
            "Should contain 'return a + b' with spaces");
        assert!(result.rendered_html.contains("add(1, 2)"), 
            "Should contain 'add(1, 2)' with spaces");
        // Should NOT have compressed tokens
        assert!(!result.rendered_html.contains("defadd"), 
            "Should NOT have compressed 'defadd' without space");
        assert!(!result.rendered_html.contains("returna+b"), 
            "Should NOT have compressed 'returna+b'");
    }

    /// Full pipeline: <pre> only (no <code>), verify rendered output preserves newlines
    #[test]
    fn test_full_pipeline_pre_without_code_rendered() {
        let html = r#"<html><body><div>
<p>Some text</p>
<pre>def add(a, b):
    return a + b

print(add(1, 2))
print(add("hello", "world"))</pre>
<p>More text</p>
</div></body></html>"#;
        let result = run_full_pipeline(html, "https://example.com/code2")
            .expect("Pipeline failed");
        eprintln!("=== RENDERED (pre without code) ===\n{}", &result.rendered_html[..result.rendered_html.len().min(800)]);
        assert!(result.rendered_html.contains("def add"), "Rendered should contain code. Got: {}", &result.rendered_html[..result.rendered_html.len().min(500)]);
        assert!(result.rendered_html.contains("return a + b"), "Rendered should preserve multi-line code");
        // Should be wrapped in <pre> by comrak
        assert!(result.rendered_html.contains("<pre>") || result.rendered_html.contains("<code"), "Should have code markup");
    }

    /// List items should have newlines between them
    #[test]
    fn test_to_markdown_list_items_separated() {
        let html = "<ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>";
        let md = to_markdown(html);
        eprintln!("=== LIST MARKDOWN ===\n{}", md);
        assert!(md.contains("- Item 1"), "Item 1 missing");
        assert!(md.contains("- Item 2"), "Item 2 missing");
        assert!(md.contains("- Item 3"), "Item 3 missing");
        // Each item on its own line
        let items: Vec<&str> = md.lines().filter(|l| l.starts_with("- ")).collect();
        assert_eq!(items.len(), 3, "Should have 3 list items, got: {:?}", items);
    }

    /// <div> blocks should produce paragraph breaks
    #[test]
    fn test_to_markdown_div_block_breaks() {
        let html = "<div>Block 1</div><div>Block 2</div><div>Block 3</div>";
        let md = to_markdown(html);
        eprintln!("=== DIV MARKDOWN ===\n{}", md);
        assert!(md.contains("Block 1"), "Block 1 missing");
        assert!(md.contains("Block 2"), "Block 2 missing");
        assert!(md.contains("Block 3"), "Block 3 missing");
        let lines: Vec<&str> = md.lines().collect();
        assert!(lines.len() >= 2, "Should have at least 2 lines, got: {}", md);
    }

    /// Full pipeline: multiple paragraphs + list should preserve structure
    #[test]
    fn test_full_pipeline_paragraphs_and_lists() {
        let html = r#"<html><body><div>
<p>First paragraph.</p>
<p>Second paragraph.</p>
<ul><li>Feature one</li><li>Feature two</li><li>Feature three</li></ul>
<p>Third paragraph after list.</p>
<p>Fourth paragraph.</p>
</div></body></html>"#;
        let result = run_full_pipeline(html, "https://example.com/para-list")
            .expect("Pipeline failed");
        eprintln!("=== MARKDOWN ===\n{}", &result.markdown[..result.markdown.len().min(800)]);
        assert!(result.markdown.contains("First paragraph"), "First para missing");
        assert!(result.markdown.contains("Second paragraph"), "Second para missing");
        assert!(result.markdown.contains("Third paragraph"), "Third para missing");
        assert!(result.markdown.contains("Fourth paragraph"), "Fourth para missing");
        assert!(result.markdown.contains("Feature one"), "Feature one missing");
        assert!(result.markdown.contains("Feature two"), "Feature two missing");
        assert!(result.markdown.contains("Feature three"), "Feature three missing");
        let para_count = result.markdown.matches("\n\n").count();
        assert!(para_count >= 3, "Should have 3+ para breaks, got {}: \n{}", para_count, result.markdown);
    }
}
