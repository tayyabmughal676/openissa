pub mod llms_txt;
pub mod robots;
pub mod sitemap;
pub mod spider;

pub use llms_txt::{LlmsTxtDocument, LlmsTxtLink, LlmsTxtParser};
pub use robots::RobotsTxt;
pub use sitemap::{SitemapDocument, SitemapEntry, SitemapParser};
pub use spider::SiteSpider;

use reqwest::Client;
use std::time::Duration;
use url::Url;

use openissa_core::error::{OpenIssaError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReport {
    pub domain: String,
    pub llms_txt: Option<LlmsTxtDocument>,
    pub sitemap_urls: Vec<SitemapEntry>,
    pub total_discovered: usize,
}

pub struct DiscoveryEngine {
    client: Client,
}

impl DiscoveryEngine {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .user_agent("OpenISSA/0.1 (+https://github.com/tayyabmughal676/openissa)")
                .build()
                .unwrap_or_default(),
        }
    }

    /// Autonomously probe a domain for AI-native llms.txt, XML sitemaps, and robots declarations.
    pub async fn discover(&self, target_url: &str) -> Result<DiscoveryReport> {
        let parsed =
            Url::parse(target_url).map_err(|e| OpenIssaError::InvalidUrl(e.to_string()))?;

        let host = parsed
            .host_str()
            .ok_or_else(|| OpenIssaError::InvalidUrl("Missing host in target URL".to_string()))?;

        let scheme = parsed.scheme();
        let base_origin = format!("{}://{}", scheme, host);

        let mut llms_txt = None;
        let mut sitemap_urls = Vec::new();

        // 1. Probe for /llms.txt (The AI-Native Documentation Standard)
        let llms_url = format!("{}/llms.txt", base_origin);
        if let Ok(resp) = self.client.get(&llms_url).send().await {
            if resp.status().is_success() {
                if let Ok(text) = resp.text().await {
                    let doc = LlmsTxtParser::parse(&text, &base_origin);
                    llms_txt = Some(doc);
                }
            }
        }

        // 2. Probe robots.txt for Sitemap directives
        let robots_url = format!("{}/robots.txt", base_origin);
        let mut declared_sitemaps = Vec::new();
        if let Ok(resp) = self.client.get(&robots_url).send().await {
            if resp.status().is_success() {
                if let Ok(text) = resp.text().await {
                    let robots = RobotsTxt::parse(&text);
                    declared_sitemaps = robots.sitemaps;
                }
            }
        }

        // If robots.txt had no sitemaps, fall back to standard /sitemap.xml
        if declared_sitemaps.is_empty() {
            declared_sitemaps.push(format!("{}/sitemap.xml", base_origin));
        }

        // 3. Fetch and parse discovered sitemaps
        for sitemap_url in declared_sitemaps.iter().take(3) {
            if let Ok(resp) = self.client.get(sitemap_url).send().await {
                if resp.status().is_success() {
                    if let Ok(xml) = resp.text().await {
                        match SitemapParser::parse(&xml) {
                            SitemapDocument::UrlSet(entries) => {
                                sitemap_urls.extend(entries);
                            }
                            SitemapDocument::Index(sub_sitemaps) => {
                                // Fetch first 2 child sitemaps
                                for sub in sub_sitemaps.iter().take(2) {
                                    if let Ok(sub_resp) = self.client.get(&sub.loc).send().await {
                                        if sub_resp.status().is_success() {
                                            if let Ok(sub_xml) = sub_resp.text().await {
                                                if let SitemapDocument::UrlSet(sub_entries) =
                                                    SitemapParser::parse(&sub_xml)
                                                {
                                                    sitemap_urls.extend(sub_entries);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let total_discovered =
            sitemap_urls.len() + llms_txt.as_ref().map(|d| d.links.len()).unwrap_or(0);

        Ok(DiscoveryReport {
            domain: host.to_string(),
            llms_txt,
            sitemap_urls,
            total_discovered,
        })
    }
}

impl Default for DiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}
