use autumnus::{FormatterOption, Options};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options as CmarkOptions, Parser, Tag, TagEnd};

/// Renders markdown to HTML with syntax highlighting for code blocks.
/// Uses CSS classes (HtmlLinked) for dynamic light/dark theme switching.
pub fn render(markdown_input: &str) -> String {
    let mut options = CmarkOptions::empty();
    options.insert(CmarkOptions::ENABLE_TABLES);
    options.insert(CmarkOptions::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown_input, options);
    let mut html_output = String::new();
    let mut code_buffer = String::new();
    let mut current_lang: Option<&str> = None;
    let mut in_code_block = false;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_buffer.clear();
                current_lang = match &kind {
                    CodeBlockKind::Fenced(lang) => parse_language(lang),
                    CodeBlockKind::Indented => None,
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let highlighted = highlight_code(&code_buffer, current_lang);
                html_output.push_str(&highlighted);
                current_lang = None;
            }
            Event::Text(text) if in_code_block => {
                code_buffer.push_str(&text);
            }
            _ => {
                let mut single_event = vec![event];
                pulldown_cmark::html::push_html(&mut html_output, single_event.drain(..));
            }
        }
    }

    html_output
}

fn parse_language(lang: &CowStr) -> Option<&'static str> {
    let lang_str = lang.as_ref().split_whitespace().next().unwrap_or("");
    match lang_str {
        "rust" | "rs" => Some("rust"),
        "python" | "py" => Some("python"),
        "javascript" | "js" => Some("javascript"),
        "typescript" | "ts" => Some("typescript"),
        "json" => Some("json"),
        "html" => Some("html"),
        "css" => Some("css"),
        "bash" | "sh" | "shell" | "zsh" => Some("bash"),
        "toml" => Some("toml"),
        "yaml" | "yml" => Some("yaml"),
        "xml" => Some("xml"),
        _ => None,
    }
}

fn highlight_code(code: &str, lang: Option<&str>) -> String {
    let lang_class = lang.unwrap_or("text");

    match lang {
        Some(language) => {
            let options = Options {
                lang_or_file: Some(language),
                formatter: FormatterOption::HtmlLinked {
                    pre_class: Some(lang_class),
                    highlight_lines: None,
                    header: None,
                },
            };
            autumnus::highlight(code, options)
        }
        None => {
            let escaped = escape_html(code);
            format!(
                "<pre class=\"athl\"><code class=\"language-{}\">{}</code></pre>\n",
                lang_class, escaped
            )
        }
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_plain_text() {
        let md = "Hello, world!";
        let html = render(md);
        assert!(html.contains("<p>Hello, world!</p>"));
    }

    #[test]
    fn test_render_code_block_with_highlighting() {
        let md = r#"```rust
fn main() {
    println!("Hello");
}
```"#;
        let html = render(md);
        assert!(html.contains("language-rust"));
        assert!(html.contains("<span")); // Syntax highlighting spans
    }

    #[test]
    fn test_render_code_block_without_lang() {
        let md = r#"```
plain code
```"#;
        let html = render(md);
        assert!(html.contains("language-text"));
        assert!(html.contains("plain code"));
    }

    #[test]
    fn test_parse_language_aliases() {
        assert_eq!(parse_language(&"rust".into()), Some("rust"));
        assert_eq!(parse_language(&"rs".into()), Some("rust"));
        assert_eq!(parse_language(&"python".into()), Some("python"));
        assert_eq!(parse_language(&"py".into()), Some("python"));
        assert_eq!(parse_language(&"js".into()), Some("javascript"));
        assert_eq!(parse_language(&"sh".into()), Some("bash"));
        assert_eq!(parse_language(&"xml".into()), Some("xml"));
        assert!(parse_language(&"unknown".into()).is_none());
    }

    #[test]
    fn test_render_gfm_table() {
        let md = r#"| Col A | Col B |
|-------|-------|
| foo   | bar   |"#;
        let html = render(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<th>Col A</th>"));
        assert!(html.contains("foo"));
    }

    #[test]
    fn test_render_gfm_strikethrough() {
        let md = "This is ~~deleted~~ text.";
        let html = render(md);
        assert!(html.contains("<del>deleted</del>"));
    }
}
