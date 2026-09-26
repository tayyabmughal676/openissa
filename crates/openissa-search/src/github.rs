use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::time::Duration;
use tracing::warn;

use crate::{SearchItem, SearchProvider};
use openissa_core::error::Result;
use openissa_core::evidence::SourceTier;

pub struct GitHubSearchProvider {
    client: Client,
}

impl GitHubSearchProvider {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        if let Ok(token) = std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("GH_TOKEN")) {
            if !token.trim().is_empty() {
                if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token.trim())) {
                    headers.insert(AUTHORIZATION, val);
                }
            }
        }

        Self {
            client: Client::builder()
                .timeout(Duration::from_millis(5000))
                .user_agent("OpenISSA/0.1 (+https://github.com/tayyabmughal676/openissa)")
                .default_headers(headers)
                .build()
                .unwrap_or_default(),
        }
    }

    /// Extract key technical keywords from a natural language query
    fn generate_query_variants(query: &str) -> Vec<String> {
        let mut variants = Vec::new();

        let stop_words = [
            "the",
            "in",
            "and",
            "a",
            "an",
            "of",
            "to",
            "for",
            "with",
            "on",
            "at",
            "by",
            "from",
            "about",
            "into",
            "through",
            "after",
            "features",
            "feature",
            "what",
            "how",
            "why",
            "where",
            "when",
            "who",
            "which",
            "latest",
            "new",
            "release",
            "overview",
            "explain",
            "guide",
            "docs",
            "documentation",
            "synthesize",
            "key",
            "architectural",
            "architecture",
            "claims",
            "claim",
            "show",
            "me",
            "framework",
            "library",
            "investigate",
            "verify",
            "research",
            "summary",
            "analyze",
            "analysis",
            "frameworks",
            "components",
            "component",
        ];

        let tokens: Vec<&str> = query
            .split_whitespace()
            .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|t| !t.is_empty() && t.len() > 1)
            .filter(|t| !stop_words.contains(&t.to_lowercase().as_str()))
            .collect();

        // 1. Top subject keyword (e.g. "fasthtml")
        if let Some(first) = tokens.first() {
            variants.push(first.to_string());
        }

        // 2. Combined technical tokens (e.g. "fasthtml server")
        if tokens.len() >= 2 {
            let combined = tokens[..tokens.len().min(3)].join(" ");
            if !variants.contains(&combined) {
                variants.push(combined);
            }
        }

        // 3. Raw query if short (<= 3 words)
        let word_count = query.split_whitespace().count();
        if word_count <= 3 && !variants.contains(&query.to_string()) {
            variants.push(query.to_string());
        }

        variants
    }
}

impl Default for GitHubSearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct GitHubRepoResponse {
    items: Option<Vec<GitHubRepoItem>>,
}

#[derive(Deserialize)]
struct GitHubRepoItem {
    html_url: String,
    full_name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct GitHubIssueResponse {
    items: Option<Vec<GitHubIssueItem>>,
}

#[derive(Deserialize)]
struct GitHubIssueItem {
    html_url: String,
    title: String,
    body: Option<String>,
}

#[async_trait]
impl SearchProvider for GitHubSearchProvider {
    fn name(&self) -> &'static str {
        "github"
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchItem>> {
        let variants = Self::generate_query_variants(query);
        let mut items = Vec::new();
        let mut seen_urls = HashSet::new();

        for target_query in variants {
            if items.len() >= limit {
                break;
            }

            // 1. Search Repositories
            let repo_endpoint = format!(
                "https://api.github.com/search/repositories?q={}&per_page={}",
                url::form_urlencoded::byte_serialize(target_query.as_bytes()).collect::<String>(),
                limit.min(6)
            );

            if let Ok(res) = self.client.get(&repo_endpoint).send().await {
                let status = res.status();
                if status == 403 || status == 429 {
                    warn!(
                        "GitHub search returned status {} (rate limited). Export GITHUB_TOKEN for higher limits.",
                        status
                    );
                } else if status.is_success() {
                    if let Ok(data) = res.json::<GitHubRepoResponse>().await {
                        if let Some(repo_items) = data.items {
                            for item in repo_items {
                                if seen_urls.insert(item.html_url.clone()) {
                                    items.push(SearchItem {
                                        url: item.html_url,
                                        title: format!("{} (GitHub Repo)", item.full_name),
                                        snippet: item.description.unwrap_or_default(),
                                        domain: "github.com".to_string(),
                                        tier: SourceTier::Primary,
                                        score: 0.95,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // 2. Search Issues/RFCs/Discussions
            if items.len() < limit {
                let needed = limit - items.len();
                let issue_endpoint = format!(
                    "https://api.github.com/search/issues?q={}&per_page={}",
                    url::form_urlencoded::byte_serialize(target_query.as_bytes())
                        .collect::<String>(),
                    needed.min(6)
                );

                if let Ok(res) = self.client.get(&issue_endpoint).send().await {
                    if res.status().is_success() {
                        if let Ok(data) = res.json::<GitHubIssueResponse>().await {
                            if let Some(issue_items) = data.items {
                                let q_lower = target_query.to_lowercase();
                                for issue in issue_items {
                                    let title_lower = issue.title.to_lowercase();
                                    let url_lower = issue.html_url.to_lowercase();
                                    let body_lower =
                                        issue.body.as_deref().unwrap_or_default().to_lowercase();

                                    // Require relevance to target subject
                                    if !title_lower.contains(&q_lower)
                                        && !url_lower.contains(&q_lower)
                                        && !body_lower.contains(&q_lower)
                                    {
                                        continue;
                                    }

                                    if seen_urls.insert(issue.html_url.clone()) {
                                        let snippet = issue
                                            .body
                                            .map(|b| b.chars().take(200).collect::<String>())
                                            .unwrap_or_default();

                                        items.push(SearchItem {
                                            url: issue.html_url,
                                            title: format!("{} (GitHub RFC/Issue)", issue.title),
                                            snippet,
                                            domain: "github.com".to_string(),
                                            tier: SourceTier::Primary,
                                            score: 0.90,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        items.truncate(limit);
        Ok(items)
    }
}
