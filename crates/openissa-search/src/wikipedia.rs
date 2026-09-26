use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

use crate::{SearchItem, SearchProvider};
use openissa_core::error::{OpenIssaError, Result};
use openissa_core::evidence::SourceTier;

pub struct WikipediaProvider {
    client: Client,
}

impl WikipediaProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_millis(5000))
                .user_agent("OpenISSA/0.1 (+https://github.com/tayyabmughal676/openissa)")
                .build()
                .unwrap_or_default(),
        }
    }
}

impl Default for WikipediaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchProvider for WikipediaProvider {
    fn name(&self) -> &'static str {
        "wikipedia"
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchItem>> {
        let endpoint = format!(
            "https://en.wikipedia.org/w/api.php?action=opensearch&search={}&limit={}&format=json",
            url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>(),
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

        let data = response
            .json::<Value>()
            .await
            .map_err(|e| OpenIssaError::ProtocolError(e.to_string()))?;

        let mut items = Vec::new();

        // Wikipedia OpenSearch format: [query, [titles...], [descriptions...], [urls...]]
        if let (Some(titles), Some(urls)) = (
            data.get(1).and_then(|v| v.as_array()),
            data.get(3).and_then(|v| v.as_array()),
        ) {
            for (title_val, url_val) in titles.iter().zip(urls.iter()) {
                if let (Some(title), Some(url)) = (title_val.as_str(), url_val.as_str()) {
                    items.push(SearchItem {
                        url: url.to_string(),
                        title: title.to_string(),
                        snippet: format!("Wikipedia entry for {}", title),
                        domain: "wikipedia.org".to_string(),
                        tier: SourceTier::Primary,
                        score: 0.9,
                    });
                }
            }
        }

        Ok(items)
    }
}
