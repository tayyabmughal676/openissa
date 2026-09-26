use serde::{Deserialize, Serialize};
use url::Url;

/// Represents an entry discovered in a sitemap.xml.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitemapEntry {
    pub loc: String,
    pub lastmod: Option<String>,
}

/// Discovered sitemap index pointing to sub-sitemaps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitemapIndexEntry {
    pub loc: String,
    pub lastmod: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SitemapDocument {
    UrlSet(Vec<SitemapEntry>),
    Index(Vec<SitemapIndexEntry>),
}

/// High-speed, robust XML parser for sitemap.xml and sitemap_index.xml.
pub struct SitemapParser;

impl SitemapParser {
    /// Parse raw XML string from a sitemap into either an UrlSet or SitemapIndex.
    pub fn parse(xml: &str) -> SitemapDocument {
        let is_index = xml.contains("<sitemapindex") || xml.contains("<sitemap>");

        if is_index {
            let mut sitemaps = Vec::new();
            for chunk in xml.split("<sitemap>") {
                if let Some(loc) = Self::extract_tag(chunk, "loc") {
                    let lastmod = Self::extract_tag(chunk, "lastmod");
                    sitemaps.push(SitemapIndexEntry { loc, lastmod });
                }
            }
            SitemapDocument::Index(sitemaps)
        } else {
            let mut urls = Vec::new();
            for chunk in xml.split("<url>") {
                if let Some(loc) = Self::extract_tag(chunk, "loc") {
                    let lastmod = Self::extract_tag(chunk, "lastmod");
                    urls.push(SitemapEntry { loc, lastmod });
                }
            }
            SitemapDocument::UrlSet(urls)
        }
    }

    /// Filter sitemap entries by path keyword or substring (e.g. "api", "docs", "guide").
    pub fn filter_entries(entries: &[SitemapEntry], keywords: &[String]) -> Vec<SitemapEntry> {
        if keywords.is_empty() {
            return entries.to_vec();
        }

        entries
            .iter()
            .filter(|e| {
                if let Ok(parsed) = Url::parse(&e.loc) {
                    let path = parsed.path().to_lowercase();
                    keywords.iter().any(|k| path.contains(&k.to_lowercase()))
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    fn extract_tag(source: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);

        let start = source.find(&open_tag)? + open_tag.len();
        let end = source[start..].find(&close_tag)?;
        let content = source[start..start + end].trim();

        if content.is_empty() {
            None
        } else {
            Some(content.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_urlset_sitemap() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <url>
                <loc>https://example.com/docs/api</loc>
                <lastmod>2026-09-01</lastmod>
            </url>
            <url>
                <loc>https://example.com/blog/intro</loc>
                <lastmod>2026-08-15</lastmod>
            </url>
        </urlset>"#;

        match SitemapParser::parse(xml) {
            SitemapDocument::UrlSet(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].loc, "https://example.com/docs/api");
                assert_eq!(entries[0].lastmod, Some("2026-09-01".to_string()));

                let filtered = SitemapParser::filter_entries(&entries, &["api".to_string()]);
                assert_eq!(filtered.len(), 1);
                assert_eq!(filtered[0].loc, "https://example.com/docs/api");
            }
            _ => panic!("Expected UrlSet"),
        }
    }

    #[test]
    fn test_parse_sitemap_index() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <sitemap>
                <loc>https://example.com/sitemap_docs.xml</loc>
                <lastmod>2026-09-20</lastmod>
            </sitemap>
        </sitemapindex>"#;

        match SitemapParser::parse(xml) {
            SitemapDocument::Index(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].loc, "https://example.com/sitemap_docs.xml");
            }
            _ => panic!("Expected Index"),
        }
    }
}
