use std::collections::HashSet;
use std::sync::Arc;
use tracing::warn;

use crate::{SearchItem, SearchProvider};
use openissa_core::error::Result;

pub struct FederatedSearchEngine {
    providers: Vec<Arc<dyn SearchProvider>>,
}

impl FederatedSearchEngine {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Arc<dyn SearchProvider>) {
        self.providers.push(provider);
    }

    pub fn provider_names(&self) -> Vec<&'static str> {
        self.providers.iter().map(|p| p.name()).collect()
    }

    /// Run federated search concurrently across all registered providers, deduplicating URLs.
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchItem>> {
        let mut set = tokio::task::JoinSet::new();

        for provider in &self.providers {
            let p = Arc::clone(provider);
            let q = query.to_string();
            let name = p.name();
            set.spawn(async move {
                let res = p.search(&q, limit).await;
                (name, res)
            });
        }

        let mut all_results = Vec::new();
        let mut seen_urls = HashSet::new();
        let mut provider_errors: Vec<(&'static str, String)> = Vec::new();

        while let Some(res) = set.join_next().await {
            match res {
                Ok((name, Ok(items))) => {
                    let count = items.len();
                    for item in items {
                        if seen_urls.insert(item.url.clone()) {
                            all_results.push(item);
                        }
                    }
                    if count == 0 {
                        warn!(
                            "Search provider '{}' returned 0 results for query '{}'",
                            name, query
                        );
                    }
                }
                Ok((name, Err(err))) => {
                    warn!(
                        "Search provider '{}' failed for query '{}': {}",
                        name, query, err
                    );
                    eprintln!("[openissa-search] Provider '{}' failed: {}", name, err);
                    provider_errors.push((name, err.to_string()));
                }
                Err(join_err) => {
                    warn!("Search task panicked or was cancelled: {}", join_err);
                }
            }
        }

        if all_results.is_empty() && !provider_errors.is_empty() {
            eprintln!(
                "[openissa-search] All active search providers failed or returned 0 results for '{}'. Errors: {:?}",
                query, provider_errors
            );
        }

        // Sort by tier authority (Primary > Secondary > Community > Untrusted)
        all_results.sort_by_key(|a| a.tier);
        all_results.truncate(limit);

        Ok(all_results)
    }
}

impl Default for FederatedSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
