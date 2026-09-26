use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::time::Duration;
use url::Url;

use crate::{SearchItem, SearchProvider};
use openissa_core::error::{OpenIssaError, Result};
use openissa_core::evidence::SourceTier;

pub struct HackerNewsSearchProvider {
    client: Client,
}

impl HackerNewsSearchProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_millis(2500))
                .user_agent("OpenISSA/0.1 (+https://github.com/tayyabmughal676/openissa)")
                .build()
                .unwrap_or_default(),
        }
    }

    /// Generate smart fallback query variations for natural language / composite queries
    fn generate_query_variants(query: &str) -> Vec<String> {
        let mut variants = Vec::new();
        variants.push(query.to_string());

        let stop_words = [
            "the", "in", "and", "a", "an", "of", "to", "for", "with", "on", "at", "by", "from",
            "about", "into", "through", "after", "features", "feature", "what", "how", "why",
            "where", "when", "who", "which", "latest", "new", "overview", "explain", "guide",
        ];

        let words: Vec<&str> = query
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| {
                    c == ',' || c == ':' || c == ';' || c == '?' || c == '!' || c == '"'
                })
            })
            .filter(|w| !w.is_empty())
            .filter(|w| !stop_words.contains(&w.to_lowercase().as_str()))
            .collect();

        // 1. If there is a version token (e.g. "1.85", "v0.1"), try query WITHOUT the version
        let no_version: Vec<&str> = words
            .iter()
            .copied()
            .filter(|w| {
                !(w.chars().any(|c| c.is_ascii_digit()) && (w.contains('.') || w.starts_with('v')))
            })
            .collect();

        if no_version.len() >= 2 && no_version.len() < words.len() {
            let s = no_version.join(" ");
            if !variants.contains(&s) {
                variants.push(s);
            }
        }

        // 2. Try first anchor word + version (e.g. "rust 1.85")
        if let Some(first) = words.first() {
            if let Some(ver) = words.iter().find(|w| {
                w.chars().any(|c| c.is_ascii_digit()) && (w.contains('.') || w.starts_with('v'))
            }) {
                let ver_query = format!("{} {}", first, ver);
                if !variants.contains(&ver_query) {
                    variants.push(ver_query);
                }
            }
        }

        // 3. Try top 3 significant keywords
        let top_kw = words.iter().copied().take(3).collect::<Vec<_>>().join(" ");
        if !top_kw.is_empty() && !variants.contains(&top_kw) {
            variants.push(top_kw);
        }

        variants
    }

    async fn query_algolia(&self, query_str: &str, limit: usize) -> Result<Vec<HnHit>> {
        let endpoint = format!(
            "https://hn.algolia.com/api/v1/search?query={}&tags=story&hitsPerPage={}",
            url::form_urlencoded::byte_serialize(query_str.as_bytes()).collect::<String>(),
            limit.min(15)
        );

        let response =
            self.client
                .get(&endpoint)
                .send()
                .await
                .map_err(|e| OpenIssaError::FetchError {
                    url: endpoint.clone(),
                    message: e.to_string(),
                })?;

        if !response.status().is_success() {
            return Err(OpenIssaError::FetchError {
                url: endpoint,
                message: format!("HN Search returned status: {}", response.status()),
            });
        }

        let data = response
            .json::<HnSearchResponse>()
            .await
            .map_err(|e| OpenIssaError::ProtocolError(e.to_string()))?;

        Ok(data.hits)
    }
}

impl Default for HackerNewsSearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct HnSearchResponse {
    hits: Vec<HnHit>,
}

#[derive(Deserialize)]
struct HnHit {
    title: Option<String>,
    url: Option<String>,
    #[serde(rename = "objectID")]
    object_id: String,
    points: Option<i64>,
    story_text: Option<String>,
}

#[async_trait]
impl SearchProvider for HackerNewsSearchProvider {
    fn name(&self) -> &'static str {
        "hackernews"
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchItem>> {
        let variants = Self::generate_query_variants(query);
        let mut items = Vec::new();
        let mut seen_urls = HashSet::new();

        for variant in variants {
            if items.len() >= limit {
                break;
            }

            let hits = self
                .query_algolia(&variant, limit)
                .await
                .unwrap_or_default();

            for hit in hits {
                if let Some(title) = hit.title {
                    let final_url =
                        hit.url
                            .filter(|u| u.starts_with("http"))
                            .unwrap_or_else(|| {
                                format!("https://news.ycombinator.com/item?id={}", hit.object_id)
                            });

                    if !seen_urls.insert(final_url.clone()) {
                        continue;
                    }

                    let domain = Url::parse(&final_url)
                        .ok()
                        .and_then(|u| u.domain().map(|d| d.to_string()))
                        .unwrap_or_else(|| "news.ycombinator.com".to_string());

                    let tier = if domain.contains("rust-lang.org")
                        || domain.contains("github.com")
                        || domain.contains("docs.")
                        || domain.contains("arxiv.org")
                        || domain.contains(".org")
                    {
                        SourceTier::Primary
                    } else if domain == "news.ycombinator.com" {
                        SourceTier::Community
                    } else {
                        SourceTier::Secondary
                    };

                    let snippet = hit.story_text.unwrap_or_else(|| {
                        format!("Story on Hacker News ({} points)", hit.points.unwrap_or(0))
                    });

                    items.push(SearchItem {
                        url: final_url,
                        title,
                        snippet,
                        domain,
                        tier,
                        score: 0.92,
                    });

                    if items.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(items)
    }
}
