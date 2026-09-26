use anyhow::Result;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use openissa_fetcher::{FetchOptions, SmartFetcher};
use openissa_mcp::McpServer;
use openissa_search::{
    BraveSearchProvider, DuckDuckGoProvider, FederatedSearchEngine, GitHubSearchProvider,
    HackerNewsSearchProvider, TavilySearchProvider, WikipediaProvider,
};
use openissa_storage::StorageEngine;

fn get_storage_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".openissa");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("openissa.db")
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = openissa_core::OpenIssaConfig::load();
    config.apply_to_env();

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "mcp" => {
            // Stdio transport: Logs must go to stderr so stdout remains pure JSON-RPC
            eprintln!(
                "OpenISSA MCP Server v{} started over stdio",
                env!("CARGO_PKG_VERSION")
            );
            let storage = Arc::new(StorageEngine::open(get_storage_path())?);
            let server = McpServer::new(storage)?;
            server.run_stdio().await?;
        }

        "serve" => {
            let port = args
                .get(2)
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(8080);
            let host = args.get(3).map(|s| s.as_str()).unwrap_or("127.0.0.1");
            let addr_str = format!("{}:{}", host, port);
            let addr: std::net::SocketAddr = addr_str
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid bind address '{}': {}", addr_str, e))?;

            println!("==> OpenISSA Remote SSE Transport Daemon");
            println!("    Version:  v{}", env!("CARGO_PKG_VERSION"));
            println!("    Endpoint: http://{}/sse", addr);
            println!("    Messages: http://{}/message?sessionId=<uuid>", addr);
            println!("    Health:   http://{}/health", addr);
            println!("    Storage:  {:?}", get_storage_path());
            println!();

            let storage = Arc::new(StorageEngine::open(get_storage_path())?);
            let mcp = Arc::new(McpServer::new(storage)?);
            let sse = openissa_mcp::SseServer::new(mcp);
            sse.run(addr).await?;
        }

        "search" => {
            let query = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if query.is_empty() {
                eprintln!("Usage: openissa search <query>");
                std::process::exit(1);
            }

            println!("==> Searching for: \"{}\"", query);

            // 1. Check local offline BM25 index (Tier 2 Retrieval Ladder, 1ms)
            if let Ok(storage) = StorageEngine::open(get_storage_path()) {
                if let Ok(local_hits) = storage.search_index(query, 3) {
                    if !local_hits.is_empty() {
                        println!("==> [Tier 2 Local BM25 Cache Hits (1ms)]");
                        for (idx, hit) in local_hits.iter().enumerate() {
                            println!("  [Local] {}. {} ({})", idx + 1, hit.title, hit.url);
                            println!("          Snippet: {}", hit.snippet);
                        }
                        println!();
                    }
                }
            }

            let mut engine = FederatedSearchEngine::new();
            if let Ok(key) = env::var("BRAVE_API_KEY") {
                if !key.trim().is_empty() {
                    engine.add_provider(Arc::new(BraveSearchProvider::new(key)));
                }
            }
            if let Ok(key) = env::var("TAVILY_API_KEY") {
                if !key.trim().is_empty() {
                    engine.add_provider(Arc::new(TavilySearchProvider::new(key)));
                }
            }
            engine.add_provider(Arc::new(HackerNewsSearchProvider::new()));
            engine.add_provider(Arc::new(GitHubSearchProvider::new()));
            engine.add_provider(Arc::new(WikipediaProvider::new()));
            engine.add_provider(Arc::new(DuckDuckGoProvider::new()));

            let items = engine.search(query, 10).await?;
            for (idx, item) in items.iter().enumerate() {
                println!("{}. {} ({})", idx + 1, item.title, item.url);
                println!("   Tier: {} | Domain: {}", item.tier.name(), item.domain);
            }
        }

        "fetch" => {
            let arg2 = args.get(2).map(|s| s.as_str()).unwrap_or("");
            let (url, force_browser) = if arg2 == "--browser" || arg2 == "-b" {
                (args.get(3).map(|s| s.as_str()).unwrap_or(""), true)
            } else {
                (arg2, false)
            };

            if url.is_empty() {
                eprintln!("Usage: openissa fetch [--browser] <url>");
                std::process::exit(1);
            }

            println!("==> Fetching: {} (force_browser={})", url, force_browser);
            let storage = StorageEngine::open(get_storage_path())?;

            if !force_browser {
                if let Ok(Some(cached)) = storage.get_cached_url(url) {
                    println!("==> [Cache Hit 0ms]");
                    println!("{}", cached);
                    return Ok(());
                }
            }

            let browser = Arc::new(openissa_browser::ChromiumSupervisor::new());
            let fetcher = SmartFetcher::new(FetchOptions::default())?.with_browser(browser);
            let res = if force_browser {
                fetcher.fetch_browser(url).await?
            } else {
                fetcher.fetch(url).await?
            };

            println!(
                "==> [HTTP Status: {} | Duration: {:?}]",
                res.status, res.duration
            );
            println!("{}", res.markdown);

            let _ = storage.set_cached_url(url, res.status, &res.markdown, 3600 * 24);
        }

        "discover" => {
            let url = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if url.is_empty() {
                eprintln!("Usage: openissa discover <url> [filter]");
                std::process::exit(1);
            }
            let filter = args.get(3).map(|s| s.as_str());

            println!(
                "==> Discovering documentation and sitemap structure for: {}",
                url
            );
            let engine = openissa_fetcher::DiscoveryEngine::new();
            let mut report = engine.discover(url).await?;

            if let Some(f) = filter {
                let kw = vec![f.to_string()];
                report.sitemap_urls =
                    openissa_fetcher::SitemapParser::filter_entries(&report.sitemap_urls, &kw);
                report.total_discovered = report.sitemap_urls.len()
                    + report.llms_txt.as_ref().map(|d| d.links.len()).unwrap_or(0);
            }

            println!("    Domain: {}", report.domain);
            if let Some(llms) = &report.llms_txt {
                println!("    [+] /llms.txt Found!");
                if let Some(title) = &llms.title {
                    println!("        Title:   {}", title);
                }
                if let Some(summary) = &llms.summary {
                    println!("        Summary: {}", summary);
                }
                if let Some(full) = &llms.full_text_url {
                    println!("        Full Doc: {}", full);
                }
                println!("        Curated Links: {}", llms.links.len());
                for link in llms.links.iter().take(5) {
                    println!("          - {} ({})", link.title, link.url);
                }
            } else {
                println!("    [-] No /llms.txt detected");
            }

            println!("    Discovered Sitemap URLs: {}", report.sitemap_urls.len());
            for entry in report.sitemap_urls.iter().take(10) {
                println!("      * {}", entry.loc);
            }
            if report.sitemap_urls.len() > 10 {
                println!(
                    "      ... and {} more entries",
                    report.sitemap_urls.len() - 10
                );
            }
        }

        "doctor" => {
            println!("==> OpenISSA System Diagnostic");
            println!("    Version:          v{}", env!("CARGO_PKG_VERSION"));
            println!("    Storage Database: {:?}", get_storage_path());
            println!(
                "    Configuration:    {:?}",
                openissa_core::OpenIssaConfig::default_path()
            );
            let cfg_exists = openissa_core::OpenIssaConfig::default_path().exists();
            println!(
                "    Config Status:    {}",
                if cfg_exists {
                    "Found (~/.openissa/config.toml)"
                } else {
                    "Default fallback"
                }
            );
            let chrome = openissa_browser::ChromiumSupervisor::detect_executable();
            println!(
                "    Tier 4 Chromium:  {}",
                chrome
                    .map(|p| format!("Available ({:?})", p))
                    .unwrap_or_else(|| "Not found (Install Chrome/Brave/Edge)".to_string())
            );
            println!("    Architecture:     Single-binary native Rust");
            println!("    Status:           Healthy and operational");
        }

        "install-mcp" => {
            let current_exe = env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "openissa".to_string());

            println!("==> OpenISSA Model Context Protocol (MCP) Setup\n");
            println!("1. Claude Code (Terminal Command):");
            println!("   claude mcp add openissa -- {} mcp\n", current_exe);
            println!("2. Cursor (.cursor/mcp.json):");
            println!(
                "   {{\n     \"mcpServers\": {{\n       \"openissa\": {{\n         \"command\": \"{}\",\n         \"args\": [\"mcp\"]\n       }}\n     }}\n   }}\n",
                current_exe
            );
            println!("3. Claude Desktop Configuration:");
            println!(
                "   \"openissa\": {{\n     \"command\": \"{}\",\n     \"args\": [\"mcp\"]\n   }}",
                current_exe
            );
        }

        "research" => {
            let goal = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if goal.is_empty() {
                eprintln!("Usage: openissa research <goal> [max_requests]");
                std::process::exit(1);
            }
            let max_requests = args
                .get(3)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(5);

            println!("==> Initiating Autonomous Research DAG");
            println!("    Goal:   \"{}\"", goal);
            println!("    Budget: {} requests", max_requests);
            println!();

            let storage = Arc::new(StorageEngine::open(get_storage_path())?);
            let orchestrator = openissa_mcp::ResearchOrchestrator::new(storage)?;
            let res = orchestrator.execute_research(goal, max_requests).await?;

            println!("{}", res.synthesis_markdown);
        }

        "--version" | "-v" => {
            println!("openissa {}", env!("CARGO_PKG_VERSION"));
        }

        _ => {
            println!("OpenISSA — Local-First Internet Intelligence Substrate for AI Agents");
            println!();
            println!("Usage:");
            println!("  openissa mcp                    Run the MCP server over stdio for Claude Code / Cursor");
            println!("  openissa serve [port] [host]    Run the remote MCP SSE transport daemon (default 127.0.0.1:8080)");
            println!(
                "  openissa research <goal> [num]  Execute autonomous research DAG synthesizing Evidence Graph"
            );
            println!("  openissa install-mcp            Print MCP setup snippets for Claude Code and Cursor");
            println!("  openissa search <query>         Execute federated web search");
            println!(
                "  openissa fetch [-b] <url>       Fetch URL via Retrieval Ladder (use -b for Tier 4 Chromium)"
            );
            println!(
                "  openissa discover <url> [fltr]  Autonomously discover /llms.txt and XML sitemaps"
            );
            println!(
                "  openissa doctor                 Check local environment and storage health"
            );
            println!("  openissa --version              Print version");
        }
    }

    Ok(())
}
