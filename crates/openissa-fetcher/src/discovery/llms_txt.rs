use serde::{Deserialize, Serialize};

/// An individual link entry parsed from an llms.txt file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmsTxtLink {
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub section: Option<String>,
}

/// Parsed structure of a standard /llms.txt file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmsTxtDocument {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub links: Vec<LlmsTxtLink>,
    pub full_text_url: Option<String>,
}

pub struct LlmsTxtParser;

impl LlmsTxtParser {
    /// Parse raw Markdown text from an llms.txt file.
    pub fn parse(content: &str, base_url: &str) -> LlmsTxtDocument {
        let mut title = None;
        let mut summary = None;
        let mut links = Vec::new();
        let mut current_section = None;
        let mut full_text_url = None;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Extract main title
            if trimmed.starts_with("# ") && title.is_none() {
                title = Some(trimmed[2..].trim().to_string());
                continue;
            }

            // Extract summary blockquote
            if let Some(rest) = trimmed.strip_prefix("> ") {
                if summary.is_none() {
                    summary = Some(rest.trim().to_string());
                }
                continue;
            }

            // Section headings (e.g. ## Documentation, ## API)
            if let Some(rest) = trimmed.strip_prefix("## ") {
                current_section = Some(rest.trim().to_string());
                continue;
            }

            // Extract Markdown link: - [Title](url): optional description
            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                if let Some((link_title, raw_url, desc)) = Self::parse_markdown_bullet(trimmed) {
                    let absolute_url = Self::resolve_url(base_url, &raw_url);
                    if raw_url.ends_with("llms-full.txt") || raw_url.contains("full") {
                        full_text_url = Some(absolute_url.clone());
                    }

                    links.push(LlmsTxtLink {
                        title: link_title,
                        url: absolute_url,
                        description: desc,
                        section: current_section.clone(),
                    });
                }
            }
        }

        LlmsTxtDocument {
            title,
            summary,
            links,
            full_text_url,
        }
    }

    fn parse_markdown_bullet(line: &str) -> Option<(String, String, Option<String>)> {
        let start_bracket = line.find('[')?;
        let end_bracket = line[start_bracket..].find(']')? + start_bracket;
        let title = line[start_bracket + 1..end_bracket].trim().to_string();

        let start_paren = line[end_bracket..].find('(')? + end_bracket;
        let end_paren = line[start_paren..].find(')')? + start_paren;
        let url = line[start_paren + 1..end_paren].trim().to_string();

        let remainder = line[end_paren + 1..].trim();
        let description = if let Some(rest) = remainder.strip_prefix(':') {
            Some(rest.trim().to_string())
        } else if !remainder.is_empty() {
            Some(remainder.to_string())
        } else {
            None
        };

        Some((title, url, description))
    }

    fn resolve_url(base: &str, target: &str) -> String {
        if target.starts_with("http://") || target.starts_with("https://") {
            target.to_string()
        } else if let Ok(base_url) = url::Url::parse(base) {
            base_url
                .join(target)
                .map(|u| u.to_string())
                .unwrap_or_else(|_| target.to_string())
        } else {
            target.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_llms_txt() {
        let content = r#"
# FastHTML Documentation
> FastHTML is a new, next-generation web framework for building modern web apps.

## Key Docs
- [Quickstart](/docs/quickstart.md): Getting started guide for new developers
- [API Reference](/docs/api.md): Full reference documentation
- [Full Documentation](/llms-full.txt): Complete single-file documentation
"#;

        let doc = LlmsTxtParser::parse(content, "https://fastht.ml");
        assert_eq!(doc.title, Some("FastHTML Documentation".to_string()));
        assert_eq!(doc.links.len(), 3);
        assert_eq!(doc.links[0].title, "Quickstart");
        assert_eq!(doc.links[0].url, "https://fastht.ml/docs/quickstart.md");
        assert_eq!(
            doc.links[0].description,
            Some("Getting started guide for new developers".to_string())
        );
        assert_eq!(
            doc.full_text_url,
            Some("https://fastht.ml/llms-full.txt".to_string())
        );
    }
}
