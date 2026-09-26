use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;
use url::Url;

use crate::{SearchItem, SearchProvider};
use openissa_core::error::{OpenIssaError, Result};
use openissa_core::evidence::SourceTier;

pub struct DuckDuckGoProvider {
    client: Client,
}

impl DuckDuckGoProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_millis(6000))
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .build()
                .unwrap_or_default(),
        }
    }

    fn extract_target_url(raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        if trimmed.starts_with("http") {
            Some(trimmed.to_string())
        } else if let Some(idx) = trimmed.find("uddg=") {
            let encoded = &trimmed[idx + 5..];
            let end = encoded.find('&').unwrap_or(encoded.len());
            let query_str = format!("u={}", &encoded[..end]);
            url::form_urlencoded::parse(query_str.as_bytes())
                .find(|(k, _)| k == "u")
                .map(|(_, v)| v.into_owned())
        } else {
            None
        }
    }
}

impl Default for DuckDuckGoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchProvider for DuckDuckGoProvider {
    fn name(&self) -> &'static str {
        "duckduckgo"
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchItem>> {
        // Try html.duckduckgo.com first with real form body
        let endpoints = [
            "https://html.duckduckgo.com/html/",
            "https://lite.duckduckgo.com/lite/",
        ];

        let mut last_err = None;

        for endpoint in &endpoints {
            let res = self
                .client
                .post(*endpoint)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .form(&[("q", query), ("b", "")])
                .send()
                .await;

            let response = match res {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    last_err = Some(OpenIssaError::FetchError {
                        url: endpoint.to_string(),
                        message: format!("HTTP {}", r.status()),
                    });
                    continue;
                }
                Err(e) => {
                    last_err = Some(OpenIssaError::FetchError {
                        url: endpoint.to_string(),
                        message: e.to_string(),
                    });
                    continue;
                }
            };

            let html = match response.text().await {
                Ok(t) => t,
                Err(e) => {
                    last_err = Some(OpenIssaError::FetchError {
                        url: endpoint.to_string(),
                        message: e.to_string(),
                    });
                    continue;
                }
            };

            let mut items = Vec::new();

            // Extract results
            for part in html.split("<a class=\"result__url\" href=\"") {
                if let Some(url_end) = part.find('"') {
                    let raw = &part[..url_end];
                    if let Some(target_url) = Self::extract_target_url(raw) {
                        let domain = Url::parse(&target_url)
                            .ok()
                            .and_then(|u| u.domain().map(|d| d.to_string()))
                            .unwrap_or_default();

                        let tier = if domain.contains("github.com") || domain.contains("docs.") {
                            SourceTier::Primary
                        } else if domain.contains("stackoverflow.com")
                            || domain.contains("reddit.com")
                        {
                            SourceTier::Community
                        } else {
                            SourceTier::Secondary
                        };

                        items.push(SearchItem {
                            url: target_url,
                            title: domain.clone(),
                            snippet: String::new(),
                            domain,
                            tier,
                            score: 0.8,
                        });

                        if items.len() >= limit {
                            break;
                        }
                    }
                }
            }

            if !items.is_empty() {
                return Ok(items);
            }
        }

        if let Some(err) = last_err {
            Err(err)
        } else {
            Ok(Vec::new())
        }
    }
}
