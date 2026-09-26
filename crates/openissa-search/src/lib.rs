//! OpenISSA Search: Federated search engine supporting Brave, Tavily, HackerNews, GitHub, DuckDuckGo, and Wikipedia.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use openissa_core::error::Result;
use openissa_core::evidence::SourceTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub domain: String,
    pub tier: SourceTier,
    pub score: f32,
}

#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchItem>>;
}

pub mod brave;
pub mod duckduckgo;
pub mod federator;
pub mod github;
pub mod hackernews;
pub mod tavily;
pub mod wikipedia;

pub use brave::BraveSearchProvider;
pub use duckduckgo::DuckDuckGoProvider;
pub use federator::FederatedSearchEngine;
pub use github::GitHubSearchProvider;
pub use hackernews::HackerNewsSearchProvider;
pub use tavily::TavilySearchProvider;
pub use wikipedia::WikipediaProvider;
