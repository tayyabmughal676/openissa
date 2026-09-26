/// Parsed robots.txt representation for autonomous site discovery.
#[derive(Debug, Clone, Default)]
pub struct RobotsTxt {
    pub sitemaps: Vec<String>,
    pub disallowed_paths: Vec<String>,
}

impl RobotsTxt {
    /// Parse robots.txt string and extract all declared sitemaps.
    pub fn parse(content: &str) -> Self {
        let mut sitemaps = Vec::new();
        let mut disallowed_paths = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("Sitemap:") {
                let url = rest.trim();
                if !url.is_empty() {
                    sitemaps.push(url.to_string());
                }
            } else if let Some(rest) = trimmed.strip_prefix("sitemap:") {
                let url = rest.trim();
                if !url.is_empty() {
                    sitemaps.push(url.to_string());
                }
            } else if let Some(rest) = trimmed.strip_prefix("Disallow:") {
                let path = rest.trim();
                if !path.is_empty() {
                    disallowed_paths.push(path.to_string());
                }
            }
        }

        Self {
            sitemaps,
            disallowed_paths,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_robots_txt() {
        let text = r#"
User-agent: *
Disallow: /admin/
Disallow: /private/
Sitemap: https://example.com/sitemap.xml
Sitemap: https://example.com/sitemap_api.xml
"#;

        let parsed = RobotsTxt::parse(text);
        assert_eq!(parsed.sitemaps.len(), 2);
        assert_eq!(parsed.sitemaps[0], "https://example.com/sitemap.xml");
        assert_eq!(parsed.sitemaps[1], "https://example.com/sitemap_api.xml");
        assert_eq!(parsed.disallowed_paths.len(), 2);
    }
}
