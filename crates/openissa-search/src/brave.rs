use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::{SearchItem, SearchProvider};
use openissa_core::error::{OpenIssaError, Result};
use openissa_core::evidence::SourceTier;

pub struct BraveSearchProvider {
    api_key: String,
    client: Client,
}

impl BraveSearchProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(4))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[derive(Deserialize)]
struct BraveApiResponse {
    web: Option<BraveWebResults>,
}

#[derive(Deserialize)]
struct BraveWebResults {
    results: Option<Vec<BraveResultItem>>,
}

#[derive(Deserialize)]
struct BraveResultItem {
    url: String,
    title: String,
    description: Option<String>,
}

#[async_trait]
impl SearchProvider for BraveSearchProvider {
    fn name(&self) -> &'static str {
        "brave"
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchItem>> {
        if self.api_key.trim().is_empty() {
            return Err(OpenIssaError::Internal(
                "Brave API key is not configured".to_string(),
            ));
        }

        let endpoint = format!(
            "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
            urlencoding::encode(query),
            limit.min(20)
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Subscription-Token",
            HeaderValue::from_str(&self.api_key)
                .map_err(|e| OpenIssaError::Internal(e.to_string()))?,
        );
        headers.insert("Accept", HeaderValue::from_static("application/json"));

        let response = self
            .client
            .get(&endpoint)
            .headers(headers)
            .send()
            .await
            .map_err(|e| OpenIssaError::FetchError {
                url: endpoint.clone(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(OpenIssaError::FetchError {
                url: endpoint,
                message: format!("Brave API returned status {}", response.status()),
            });
        }

        let data = response
            .json::<BraveApiResponse>()
            .await
            .map_err(|e| OpenIssaError::ProtocolError(e.to_string()))?;

        let mut items = Vec::new();
        if let Some(web) = data.web {
            if let Some(results) = web.results {
                for r in results {
                    let domain = Url::parse(&r.url)
                        .ok()
                        .and_then(|u| u.domain().map(|d| d.to_string()))
                        .unwrap_or_default();

                    let tier = if domain.contains("github.com")
                        || domain.contains("docs.")
                        || domain.contains("ietf.org")
                        || domain.contains("w3.org")
                    {
                        SourceTier::Primary
                    } else if domain.contains("stackoverflow.com") || domain.contains("reddit.com")
                    {
                        SourceTier::Community
                    } else {
                        SourceTier::Secondary
                    };

                    items.push(SearchItem {
                        url: r.url,
                        title: r.title,
                        snippet: r.description.unwrap_or_default(),
                        domain,
                        tier,
                        score: 1.0,
                    });
                }
            }
        }

        Ok(items)
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
