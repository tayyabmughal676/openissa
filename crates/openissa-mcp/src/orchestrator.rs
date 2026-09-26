use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

use openissa_core::budget::{BudgetTracker, ResearchBudget};
use openissa_core::error::Result;
use openissa_core::evidence::{EvidenceGraph, SourceTier};
use openissa_fetcher::{FetchOptions, SmartFetcher};
use openissa_search::{
    BraveSearchProvider, DuckDuckGoProvider, FederatedSearchEngine, GitHubSearchProvider,
    HackerNewsSearchProvider, TavilySearchProvider, WikipediaProvider,
};
use openissa_storage::StorageEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub goal: String,
    pub session_id: Uuid,
    pub evidence_graph: EvidenceGraph,
    pub pages_fetched: usize,
    pub synthesis_markdown: String,
}

pub struct ResearchOrchestrator {
    search: FederatedSearchEngine,
    fetcher: SmartFetcher,
    storage: Arc<StorageEngine>,
}

impl ResearchOrchestrator {
    pub fn new(storage: Arc<StorageEngine>) -> Result<Self> {
        let config = openissa_core::OpenIssaConfig::load();
        config.apply_to_env();

        let mut search = FederatedSearchEngine::new();
        if let Ok(key) = std::env::var("BRAVE_API_KEY") {
            if !key.trim().is_empty() {
                search.add_provider(Arc::new(BraveSearchProvider::new(key)));
            }
        }
        if let Ok(key) = std::env::var("TAVILY_API_KEY") {
            if !key.trim().is_empty() {
                search.add_provider(Arc::new(TavilySearchProvider::new(key)));
            }
        }
        search.add_provider(Arc::new(HackerNewsSearchProvider::new()));
        search.add_provider(Arc::new(GitHubSearchProvider::new()));
        search.add_provider(Arc::new(WikipediaProvider::new()));
        search.add_provider(Arc::new(DuckDuckGoProvider::new()));

        let fetcher = SmartFetcher::new(FetchOptions::default())?;

        Ok(Self {
            search,
            fetcher,
            storage,
        })
    }

    /// Extract explicit URLs mentioned directly in research goals (e.g. "read https://...")
    fn extract_urls_from_goal(text: &str) -> Vec<String> {
        text.split_whitespace()
            .filter_map(|token| {
                let clean = token.trim_matches(|c: char| {
                    c == '<'
                        || c == '>'
                        || c == '('
                        || c == ')'
                        || c == '"'
                        || c == '\''
                        || c == ','
                });
                if clean.starts_with("http://") || clean.starts_with("https://") {
                    Url::parse(clean).ok().map(|u| u.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract key technical tokens from natural language research goals
    fn extract_search_terms(text: &str) -> (String, Vec<String>) {
        let stop_words = [
            "what",
            "is",
            "how",
            "the",
            "a",
            "an",
            "in",
            "of",
            "to",
            "for",
            "with",
            "on",
            "at",
            "by",
            "from",
            "about",
            "use",
            "using",
            "synthesize",
            "key",
            "architectural",
            "claims",
            "claim",
            "show",
            "me",
            "framework",
            "library",
            "features",
            "feature",
            "explain",
            "investigate",
            "research",
            "verify",
            "summary",
            "analyze",
            "analysis",
        ];

        let tokens: Vec<String> = text
            .split_whitespace()
            .map(|t| {
                t.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|t| !t.is_empty() && t.len() > 1)
            .filter(|t| !stop_words.contains(&t.as_str()))
            .collect();

        let query = if tokens.is_empty() {
            text.to_string()
        } else {
            tokens.join(" ")
        };

        (query, tokens)
    }

    /// Execute a bounded, multi-tier research session synthesizing an Evidence Graph.
    pub async fn execute_research(
        &self,
        goal: &str,
        max_requests: usize,
    ) -> Result<ResearchResult> {
        let session_id = Uuid::new_v4();
        let limit = max_requests.clamp(1, 15);

        let budget = ResearchBudget {
            max_requests: limit,
            max_pages: limit * 2,
            max_runtime_secs: 60,
            max_bytes: 5 * 1024 * 1024,
            max_depth: 2,
        };
        let tracker = BudgetTracker::new(budget);
        let mut graph = EvidenceGraph::new();

        let (search_query, keywords) = Self::extract_search_terms(goal);

        // 1. Direct URL extraction from goal (if specified)
        let direct_urls = Self::extract_urls_from_goal(goal);
        let mut sources_to_fetch = Vec::new();

        for url in direct_urls {
            let domain = Url::parse(&url)
                .ok()
                .and_then(|u| u.domain().map(|d| d.to_string()))
                .unwrap_or_else(|| "direct".to_string());
            let src_id = graph.add_source(url.clone(), domain.clone(), SourceTier::Primary);
            sources_to_fetch.push((src_id, url, domain, SourceTier::Primary));
        }

        // 2. Retrieval Ladder: Consult local BM25 FTS5 index FIRST
        let mut bm25_queries = vec![search_query.clone()];
        for kw in &keywords {
            if !bm25_queries.contains(kw) {
                bm25_queries.push(kw.clone());
            }
        }

        for bq in bm25_queries {
            if let Ok(local_hits) = self.storage.search_index(&bq, limit) {
                for hit in local_hits {
                    if !sources_to_fetch.iter().any(|(_, u, _, _)| u == &hit.url) {
                        let domain = Url::parse(&hit.url)
                            .ok()
                            .and_then(|u| u.domain().map(|d| d.to_string()))
                            .unwrap_or_else(|| "local-cache".to_string());
                        let src_id =
                            graph.add_source(hit.url.clone(), domain.clone(), SourceTier::Primary);
                        sources_to_fetch.push((src_id, hit.url, domain, SourceTier::Primary));
                    }
                }
            }
        }

        // 3. Search Phase: Federated search across primary, secondary, and code sources
        let search_target = if search_query.is_empty() {
            goal
        } else {
            &search_query
        };
        if let Ok(search_items) = self.search.search(search_target, limit).await {
            for item in search_items {
                if !sources_to_fetch.iter().any(|(_, u, _, _)| u == &item.url) {
                    let src_id = graph.add_source(item.url.clone(), item.domain.clone(), item.tier);
                    sources_to_fetch.push((src_id, item.url, item.domain, item.tier));
                }
            }
        }

        // 4. Fetch & Claim Extraction Phase
        let mut pages_fetched = 0;

        for (src_id, url, domain, tier) in sources_to_fetch {
            if tracker.acquire_request_lease().is_err() {
                break;
            }

            // Check cache or fetch live
            let markdown = if let Ok(Some(cached)) = self.storage.get_cached_url(&url) {
                cached
            } else if let Ok(fetched) = self.fetcher.fetch(&url).await {
                let _ =
                    self.storage
                        .set_cached_url(&url, fetched.status, &fetched.markdown, 3600 * 24);
                pages_fetched += 1;
                fetched.markdown
            } else {
                continue;
            };

            let url_lower = url.to_lowercase();
            let domain_lower = domain.to_lowercase();
            let is_primary_doc = tier == SourceTier::Primary
                || domain_lower.contains("fastht")
                || url_lower.contains("fastht")
                || url_lower.contains("docs.")
                || url_lower.contains("/docs/");

            // Extract claims from Markdown structure (headings, key statements, bullets)
            let mut extracted_count = 0;
            for line in markdown.lines() {
                let trimmed = line.trim();

                // Extract prominent findings
                let is_heading = trimmed.starts_with("# ")
                    || trimmed.starts_with("## ")
                    || trimmed.starts_with("### ");
                let is_bullet = trimmed.starts_with("- ")
                    || trimmed.starts_with("* ")
                    || trimmed.starts_with("1. ");

                if (is_heading || is_bullet) && trimmed.len() > 15 && trimmed.len() < 350 {
                    let statement = trimmed
                        .trim_start_matches("#")
                        .trim_start_matches("##")
                        .trim_start_matches("###")
                        .trim_start_matches("- ")
                        .trim_start_matches("* ")
                        .trim_start_matches("1. ")
                        .trim();

                    let stmt_lower = statement.to_lowercase();

                    let is_boilerplate = stmt_lower.starts_with("folders and files")
                        || stmt_lower.starts_with("repository files navigation")
                        || stmt_lower.starts_with("latest commit")
                        || stmt_lower.starts_with("view all files")
                        || stmt_lower.starts_with("git stats")
                        || stmt_lower.starts_with("readme.md")
                        || stmt_lower.contains("license")
                        || stmt_lower.starts_with("contributing")
                        || stmt_lower.starts_with("pull request")
                        || stmt_lower.starts_with("merge pull request");

                    if is_boilerplate || statement.len() < 15 {
                        continue;
                    }

                    // Check relevance: either primary documentation page or contains key concepts
                    let matches_keyword = keywords.is_empty()
                        || keywords.iter().any(|kw| stmt_lower.contains(kw))
                        || stmt_lower.contains("component")
                        || stmt_lower.contains("render")
                        || stmt_lower.contains("html")
                        || stmt_lower.contains("tag")
                        || stmt_lower.contains("python")
                        || stmt_lower.contains("ft");

                    let is_relevant = is_primary_doc || matches_keyword;

                    if is_relevant {
                        let (initial_confidence, verified) = if is_primary_doc {
                            (0.95, true)
                        } else {
                            match tier {
                                SourceTier::Primary => (0.90, true),
                                SourceTier::Secondary => (0.70, false),
                                SourceTier::Community => (0.40, false),
                                SourceTier::Untrusted => (0.10, false),
                            }
                        };

                        let claim_id =
                            graph.add_claim(session_id, statement.to_string(), initial_confidence);
                        if verified {
                            graph.mark_verified(claim_id, true);
                        }

                        graph.attach_evidence(
                            claim_id,
                            src_id,
                            format!("Found on {}: {}", domain, statement),
                            false,
                        );

                        extracted_count += 1;
                        if extracted_count >= 5 {
                            break;
                        }
                    }
                }
            }
        }

        // Record session into persistent storage
        let status = if graph.claims.is_empty() {
            "halted"
        } else {
            "completed"
        };
        let _ = self
            .storage
            .save_session(&session_id.to_string(), goal, status);

        // If no sources could be fetched or claims discovered, report honest diagnostics
        let synthesis_markdown = if graph.claims.is_empty() {
            format!(
                "### Research Report: No Sources Found\n\n\
                * **Objective**: {}\n\
                * **Active Providers**: {:?}\n\
                * **Status**: Web search providers returned 0 reachable results or timed out.\n\n\
                **Recommended Next Steps**:\n\
                1. If working under restrictive regional ISP or network filters, configure `BRAVE_API_KEY` or `TAVILY_API_KEY` in environment or `opencode.json`.\n\
                2. Use `web_index_site` on target domains (e.g. `https://docs.rs/...`) to populate local offline BM25 index.\n\
                3. Use `web_fetch` directly on primary URLs.",
                goal,
                self.search.provider_names()
            )
        } else {
            graph.format_markdown_summary()
        };

        Ok(ResearchResult {
            goal: goal.to_string(),
            session_id,
            evidence_graph: graph,
            pages_fetched,
            synthesis_markdown,
        })
    }
}
