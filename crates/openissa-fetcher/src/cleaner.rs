/// Convert raw HTML into clean, token-efficient Markdown.
/// Strips scripts, styles, navigation, headers, and boilerplate.
pub fn clean_html_to_markdown(html: &str) -> String {
    let mut text = html.to_string();

    // 1. Remove comments
    while let Some(start) = text.find("<!--") {
        if let Some(end) = text[start..].find("-->") {
            text.replace_range(start..start + end + 3, "");
        } else {
            break;
        }
    }

    // 2. Remove non-content tags and their inner text (scripts, styles, svg, iframes, nav, header, footer)
    let remove_tags = [
        "script", "style", "noscript", "svg", "iframe", "nav", "header", "footer", "aside",
    ];
    for tag in remove_tags {
        let open_pattern = format!("<{}", tag);
        let close_pattern = format!("</{}>", tag);

        while let Some(start) = text.to_lowercase().find(&open_pattern) {
            if let Some(end) = text[start..].to_lowercase().find(&close_pattern) {
                text.replace_range(start..start + end + close_pattern.len(), "");
            } else if let Some(tag_end) = text[start..].find('>') {
                text.replace_range(start..start + tag_end + 1, "");
            } else {
                break;
            }
        }
    }

    // 3. Replace headings with Markdown equivalents
    for i in (1..=6).rev() {
        let open = format!("<h{}", i);
        let close = format!("</h{}>", i);
        let hashes = "#".repeat(i);

        while let Some(start) = text.to_lowercase().find(&open) {
            if let Some(tag_end) = text[start..].find('>') {
                let content_start = start + tag_end + 1;
                if let Some(end_offset) = text[content_start..].to_lowercase().find(&close) {
                    let end = content_start + end_offset;
                    let heading_text = &text[content_start..end];
                    let replacement = format!("\n\n{} {}\n\n", hashes, heading_text.trim());
                    text.replace_range(start..end + close.len(), &replacement);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    // 4. Convert paragraphs and line breaks
    text = text
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");
    text = text.replace("<p>", "\n\n").replace("</p>", "\n");

    // 5. Convert list items
    text = text.replace("<li>", "\n* ").replace("</li>", "");

    // 6. Strip all remaining HTML tags
    let mut inside_tag = false;
    let mut cleaned = String::with_capacity(text.len());
    for c in text.chars() {
        if c == '<' {
            inside_tag = true;
        } else if c == '>' {
            inside_tag = false;
        } else if !inside_tag {
            cleaned.push(c);
        }
    }

    // 7. Decode basic HTML entities
    let decoded = cleaned
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // 8. Normalize excessive newlines and whitespace
    let mut normalized = String::with_capacity(decoded.len());
    let mut newline_count = 0;
    for line in decoded.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            newline_count += 1;
            if newline_count <= 2 {
                normalized.push('\n');
            }
        } else {
            newline_count = 0;
            normalized.push_str(trimmed);
            normalized.push('\n');
        }
    }

    normalized.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_html_to_markdown() {
        let html = r#"
            <html>
                <head><style>body { color: red; }</style></head>
                <body>
                    <header><nav><a href="/">Home</a></nav></header>
                    <h1>Documentation Title</h1>
                    <p>This is a paragraph with <b>bold</b> text.</p>
                    <script>console.log("secret");</script>
                    <ul>
                        <li>Item 1</li>
                        <li>Item 2</li>
                    </ul>
                </body>
            </html>
        "#;

        let md = clean_html_to_markdown(html);
        assert!(md.contains("# Documentation Title"));
        assert!(md.contains("This is a paragraph with bold text."));
        assert!(md.contains("* Item 1"));
        assert!(!md.contains("console.log"));
        assert!(!md.contains("body { color: red; }"));
    }
}
