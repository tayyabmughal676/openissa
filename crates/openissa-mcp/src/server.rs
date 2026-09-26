use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use openissa_core::error::{OpenIssaError, Result};
use openissa_fetcher::{DiscoveryEngine, FetchOptions, SitemapParser, SmartFetcher};
use openissa_investigate::{EndpointTestSpec, EndpointTester};
use openissa_search::{
    BraveSearchProvider, DuckDuckGoProvider, FederatedSearchEngine, GitHubSearchProvider,
    HackerNewsSearchProvider, TavilySearchProvider, WikipediaProvider,
};
use openissa_storage::StorageEngine;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub struct McpServer {
    fetcher: SmartFetcher,
    discovery: DiscoveryEngine,
    search: FederatedSearchEngine,
    investigate: EndpointTester,
    orchestrator: crate::orchestrator::ResearchOrchestrator,
    storage: Arc<StorageEngine>,
}

impl McpServer {
    pub fn new(storage: Arc<StorageEngine>) -> Result<Self> {
        let config = openissa_core::OpenIssaConfig::load();
        config.apply_to_env();

        let browser = Arc::new(openissa_browser::ChromiumSupervisor::new());
        let fetcher = SmartFetcher::new(FetchOptions::default())?.with_browser(browser);
        let discovery = DiscoveryEngine::new();
        let orchestrator = crate::orchestrator::ResearchOrchestrator::new(Arc::clone(&storage))?;
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
        let investigate = EndpointTester::new();

        Ok(Self {
            fetcher,
            discovery,
            search,
            investigate,
            orchestrator,
            storage,
        })
    }

    /// Process a single incoming JSON-RPC 2.0 message.
    pub async fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id.clone();

        match req.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "openissa",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "capabilities": {
                        "tools": {}
                    }
                });
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(result),
                    error: None,
                })
            }

            "notifications/initialized" => None,

            "tools/list" => {
                let tools = json!({
                    "tools": [
                        {
                            "name": "web_search",
                            "description": "Federated multi-provider web search with deduplication and domain scoring.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": { "type": "string", "description": "The search query" },
                                    "limit": { "type": "integer", "default": 10 }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "web_fetch",
                            "description": "Smart laddered fetcher. Automatically checks local SQLite cache, escalates to Tier 4 Chromium for dynamic CSR pages, and returns clean token-dense Markdown.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "url": { "type": "string", "description": "Target URL to fetch" },
                                    "force_browser": { "type": "boolean", "description": "Force Tier 4 headless Chromium JavaScript rendering" }
                                },
                                "required": ["url"]
                            }
                        },
                        {
                            "name": "web_inspect",
                            "description": "Deep network diagnostics for any URL or API endpoint: HTTP status, headers, and redirects.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "url": { "type": "string", "description": "Target URL to inspect" }
                                },
                                "required": ["url"]
                            }
                        },
                        {
                            "name": "web_test_endpoint",
                            "description": "Controlled, sandboxed HTTP experiment runner to test live REST/GraphQL APIs with methods and params.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "url": { "type": "string", "description": "Target endpoint URL" },
                                    "method": { "type": "string", "default": "GET" },
                                    "params": { "type": "object" },
                                    "body": { "type": "string" }
                                },
                                "required": ["url"]
                            }
                        },
                        {
                            "name": "web_index_site",
                            "description": "Autonomously map an entire documentation or API domain without search engines by discovering /llms.txt, XML sitemaps, and robots directives.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "url": { "type": "string", "description": "Target domain or base URL (e.g. https://docs.rs or https://stripe.com)" },
                                    "filter": { "type": "string", "description": "Optional keyword path filter (e.g. 'api', 'v2', 'guide')" }
                                },
                                "required": ["url"]
                            }
                        },
                        {
                            "name": "web_research",
                            "description": "Autonomous investigative engine. Executes a bounded research DAG: federates search queries, fetches via the Retrieval Ladder, verifies claims against primary documentation, and synthesizes an Evidence Graph.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "goal": { "type": "string", "description": "The research objective or technical question to investigate" },
                                    "budget_requests": { "type": "integer", "default": 5, "description": "Maximum number of page fetches allowed" }
                                },
                                "required": ["goal"]
                            }
                        }
                    ]
                });

                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(tools),
                    error: None,
                })
            }

            "tools/call" => {
                let params = req.params.unwrap_or(json!({}));
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                let content = match self.dispatch_tool(tool_name, args).await {
                    Ok(text) => json!({
                        "content": [{ "type": "text", "text": text }]
                    }),
                    Err(e) => json!({
                        "isError": true,
                        "content": [{ "type": "text", "text": format!("Error: {}", e) }]
                    }),
                };

                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(content),
                    error: None,
                })
            }

            _ => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(json!({
                    "code": -32601,
                    "message": format!("Method not found: {}", req.method)
                })),
            }),
        }
    }

    async fn dispatch_tool(&self, name: &str, args: Value) -> Result<String> {
        match name {
            "web_search" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

                let items = self.search.search(query, limit).await?;
                let mut out = format!("# Search Results for \"{}\"\n\n", query);

                // Check local BM25 index (Tier 2 Retrieval Ladder)
                if let Ok(local_hits) = self.storage.search_index(query, 3) {
                    if !local_hits.is_empty() {
                        out.push_str("### Local Matches (Offline BM25 Index)\n");
                        for hit in local_hits {
                            out.push_str(&format!(
                                "- [{}]({})\n  * **Snippet**: {}\n  * **Tier**: Local BM25 Index\n\n",
                                hit.title, hit.url, hit.snippet
                            ));
                        }
                        out.push_str("### Web Results\n");
                    }
                }

                for (idx, item) in items.iter().enumerate() {
                    out.push_str(&format!(
                        "{}. [{}]({})\n   - **Domain**: {}\n   - **Tier**: {}\n\n",
                        idx + 1,
                        item.title,
                        item.url,
                        item.domain,
                        item.tier.name()
                    ));
                }
                Ok(out)
            }

            "web_fetch" => {
                let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let force_browser = args
                    .get("force_browser")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // 1. Check local cache (0ms) unless force_browser is requested
                if !force_browser {
                    if let Ok(Some(cached_md)) = self.storage.get_cached_url(url) {
                        return Ok(format!(
                            "<untrusted_web_evidence url=\"{}\" cached=\"true\">\n{}\n</untrusted_web_evidence>",
                            url, cached_md
                        ));
                    }
                }

                // 2. Fetch direct via HTTP/2 (with auto CSR escalation) or forced Tier 4 Chromium
                let res = if force_browser {
                    self.fetcher.fetch_browser(url).await?
                } else {
                    self.fetcher.fetch(url).await?
                };

                // 3. Cache result in SQLite
                let _ = self
                    .storage
                    .set_cached_url(url, res.status, &res.markdown, 3600 * 24);

                Ok(format!(
                    "<untrusted_web_evidence url=\"{}\" status=\"{}\">\n{}\n</untrusted_web_evidence>",
                    url, res.status, res.markdown
                ))
            }

            "web_inspect" => {
                let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let res = self.fetcher.fetch(url).await?;
                let diag = json!({
                    "url": res.url,
                    "status": res.status,
                    "content_type": res.content_type,
                    "raw_bytes": res.raw_bytes_count,
                    "requires_browser": res.requires_browser,
                    "duration_ms": res.duration.as_millis()
                });
                Ok(serde_json::to_string_pretty(&diag).unwrap_or_default())
            }

            "web_test_endpoint" => {
                let url = args
                    .get("url")
                    .or_else(|| args.get("spec").and_then(|s| s.get("url")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let method = args
                    .get("method")
                    .or_else(|| args.get("spec").and_then(|s| s.get("method")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("GET")
                    .to_string();
                let body = args
                    .get("body")
                    .or_else(|| args.get("spec").and_then(|s| s.get("body")))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let spec = EndpointTestSpec {
                    url,
                    method,
                    headers: None,
                    params: None,
                    body,
                };

                let res = self.investigate.execute_test(spec).await?;
                let summary = json!({
                    "url": res.url,
                    "status": res.status,
                    "duration_ms": res.duration_ms,
                    "headers": res.response_headers,
                    "body_preview": if res.body.len() > 1000 {
                        format!("{}... [truncated]", &res.body[..1000])
                    } else {
                        res.body
                    }
                });
                Ok(serde_json::to_string_pretty(&summary).unwrap_or_default())
            }

            "web_index_site" => {
                let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let filter = args.get("filter").and_then(|v| v.as_str());

                let mut report = self.discovery.discover(url).await?;

                if let Some(f) = filter {
                    let kw = vec![f.to_string()];
                    report.sitemap_urls = SitemapParser::filter_entries(&report.sitemap_urls, &kw);
                    report.total_discovered = report.sitemap_urls.len()
                        + report.llms_txt.as_ref().map(|d| d.links.len()).unwrap_or(0);
                }

                let summary = json!({
                    "domain": report.domain,
                    "has_llms_txt": report.llms_txt.is_some(),
                    "llms_summary": report.llms_txt.as_ref().and_then(|d| d.summary.clone()),
                    "llms_full_text": report.llms_txt.as_ref().and_then(|d| d.full_text_url.clone()),
                    "curated_links_count": report.llms_txt.as_ref().map(|d| d.links.len()).unwrap_or(0),
                    "sitemap_urls_count": report.sitemap_urls.len(),
                    "sample_sitemap_urls": report.sitemap_urls.iter().take(25).map(|e| &e.loc).collect::<Vec<_>>(),
                    "sample_llms_links": report.llms_txt.as_ref().map(|d| d.links.iter().take(25).collect::<Vec<_>>())
                });
                Ok(serde_json::to_string_pretty(&summary).unwrap_or_default())
            }

            "web_research" => {
                let goal = args.get("goal").and_then(|v| v.as_str()).unwrap_or("");
                let budget = args
                    .get("budget_requests")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;

                let res = self.orchestrator.execute_research(goal, budget).await?;
                Ok(res.synthesis_markdown)
            }

            _ => Err(OpenIssaError::Internal(format!("Unknown tool: {}", name))),
        }
    }

    /// Run the stdio transport loop reading from stdin and writing to stdout.
    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
                if let Some(resp) = self.handle_request(req).await {
                    let out = serde_json::to_string(&resp)
                        .map_err(|e| OpenIssaError::ProtocolError(e.to_string()))?;
                    stdout
                        .write_all(out.as_bytes())
                        .await
                        .map_err(|e| OpenIssaError::Internal(e.to_string()))?;
                    stdout
                        .write_all(b"\n")
                        .await
                        .map_err(|e| OpenIssaError::Internal(e.to_string()))?;
                    stdout
                        .flush()
                        .await
                        .map_err(|e| OpenIssaError::Internal(e.to_string()))?;
                }
            }
        }

        Ok(())
    }
}
