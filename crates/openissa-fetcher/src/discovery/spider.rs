use std::collections::HashSet;
use url::Url;

pub struct SiteSpider;

impl SiteSpider {
    /// Extract all internal, same-domain links from an HTML document.
    pub fn extract_internal_links(html: &str, base_url: &str) -> Vec<String> {
        let Ok(parsed_base) = Url::parse(base_url) else {
            return Vec::new();
        };

        let Some(base_host) = parsed_base.host_str() else {
            return Vec::new();
        };

        let mut discovered = HashSet::new();

        // Search for <a ... href="..." ...>
        for part in html.split("<a ") {
            if let Some(href_idx) = part.find("href=\"") {
                let after_href = &part[href_idx + 6..];
                if let Some(quote_idx) = after_href.find('"') {
                    let raw_link = &after_href[..quote_idx].trim();
                    if let Ok(mut resolved) = parsed_base.join(raw_link) {
                        // Strip fragment (#section)
                        resolved.set_fragment(None);

                        // Domain boundary check (same host only)
                        if let Some(host) = resolved.host_str() {
                            if host == base_host {
                                let clean_url = resolved.to_string();
                                // Exclude common non-HTML files
                                if !Self::is_ignored_extension(&clean_url) {
                                    discovered.insert(clean_url);
                                }
                            }
                        }
                    }
                }
            }
        }

        discovered.into_iter().collect()
    }

    fn is_ignored_extension(url_str: &str) -> bool {
        let ignored = [
            ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".pdf", ".zip", ".tar.gz", ".css",
            ".js", ".woff", ".woff2",
        ];
        ignored.iter().any(|ext| url_str.ends_with(ext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_internal_links() {
        let html = r#"
            <p>Check out our <a href="/docs/getting-started">Getting Started</a> guide.</p>
            <p>See our <a href="https://example.com/api/v1#section-auth">API</a> page.</p>
            <p>Visit <a href="https://external.com/article">External</a> blog.</p>
            <p>Download <a href="/asset/image.png">Logo</a> image.</p>
        "#;

        let links = SiteSpider::extract_internal_links(html, "https://example.com/home");
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://example.com/docs/getting-started".to_string()));
        assert!(links.contains(&"https://example.com/api/v1".to_string()));
        assert!(!links.contains(&"https://external.com/article".to_string()));
    }
}
