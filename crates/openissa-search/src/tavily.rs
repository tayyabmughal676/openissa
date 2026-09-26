use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

use crate::{SearchItem, SearchProvider};
use openissa_core::error::{OpenIssaError, Result};
use openissa_core::evidence::SourceTier;

pub struct TavilySearchProvider {
    client: Client,
    api_key: String,
}

impl TavilySearchProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(4))
                .user_agent("OpenISSA/0.1 (+https://github.com/tayyabmughal676/openissa)")
                .build()
                .unwrap_or_default(),
            api_key: api_key.into(),
        }
    }
}

#[derive(Serialize)]
struct TavilyRequest<'a> {
    api_key: &'a str,
    query: &'a str,
    max_results: usize,
    search_depth: &'a str,
}

#[derive(Deserialize)]
struct TavilyResponse {
    results: Option<Vec<TavilyResultItem>>,
}

#[derive(Deserialize)]
struct TavilyResultItem {
    title: String,
    url: String,
    content: String,
    score: Option<f32>,
}

#[async_trait]
impl SearchProvider for TavilySearchProvider {
    fn name(&self) -> &'static str {
        "tavily"
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchItem>> {
        let endpoint = "https://api.tavily.com/search";
        let payload = TavilyRequest {
            api_key: &self.api_key,
            query,
            max_results: limit.min(10),
            search_depth: "basic",
        };

        let response = self
            .client
            .post(endpoint)
            .json(&payload)
            .send()
            .await
            .map_err(|e| OpenIssaError::FetchError {
                url: endpoint.to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(OpenIssaError::FetchError {
                url: endpoint.to_string(),
                message: format!("Tavily returned HTTP {}", response.status()),
            });
        }

        let data = response
            .json::<TavilyResponse>()
            .await
            .map_err(|e| OpenIssaError::ProtocolError(e.to_string()))?;

        let mut items = Vec::new();
        if let Some(res_items) = data.results {
            for item in res_items {
                let domain = Url::parse(&item.url)
                    .ok()
                    .and_then(|u| u.domain().map(|d| d.to_string()))
                    .unwrap_or_default();

                let tier = if domain.contains("github.com")
                    || domain.contains("docs.")
                    || domain.contains(".org")
                {
                    SourceTier::Primary
                } else if domain.contains("reddit.com") || domain.contains("stackoverflow.com") {
                    SourceTier::Community
                } else {
                    SourceTier::Secondary
                };

                items.push(SearchItem {
                    url: item.url,
                    title: item.title,
                    snippet: item.content,
                    domain,
                    tier,
                    score: item.score.unwrap_or(0.9),
                });
            }
        }

        Ok(items)
    }
}
