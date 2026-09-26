use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

use openissa_core::OpenIssaConfig;
use openissa_investigate::NetworkFirewall;
use openissa_mcp::{JsonRpcRequest, McpServer, ResearchOrchestrator};
use openissa_storage::StorageEngine;

struct TestDbGuard {
    path: PathBuf,
}

impl Drop for TestDbGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn setup_test_db() -> (TestDbGuard, Arc<StorageEngine>) {
    let filename = format!("openissa_e2e_{}.db", uuid::Uuid::new_v4());
    let path = std::env::temp_dir().join(filename);
    let storage = Arc::new(StorageEngine::open(&path).expect("Failed to open test storage"));
    (TestDbGuard { path }, storage)
}

#[tokio::test]
async fn test_e2e_mcp_lifecycle() {
    let (_guard, storage) = setup_test_db();
    let server = McpServer::new(storage).expect("Failed to initialize McpServer");

    // 1. Initialize Handshake
    let init_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "e2e-test", "version": "1.0" }
        })),
    };

    let init_res = server.handle_request(init_req).await;
    assert!(init_res.is_some(), "Server must reply to initialize");
    let init_res = init_res.unwrap();
    assert_eq!(init_res.jsonrpc, "2.0");
    assert!(init_res.error.is_none());

    let res_val = init_res.result.expect("Must have result object");
    assert_eq!(res_val["serverInfo"]["name"], "openissa");

    // 2. Tools List Inspection
    let tools_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/list".to_string(),
        params: None,
    };

    let tools_res = server.handle_request(tools_req).await.unwrap();
    let tools_val = tools_res.result.unwrap();
    let tools_arr = tools_val["tools"]
        .as_array()
        .expect("Tools must be an array");

    let tool_names: Vec<&str> = tools_arr
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(tool_names.contains(&"web_search"), "Missing web_search");
    assert!(tool_names.contains(&"web_fetch"), "Missing web_fetch");
    assert!(tool_names.contains(&"web_inspect"), "Missing web_inspect");
    assert!(
        tool_names.contains(&"web_test_endpoint"),
        "Missing web_test_endpoint"
    );
    assert!(
        tool_names.contains(&"web_index_site"),
        "Missing web_index_site"
    );
    assert!(tool_names.contains(&"web_research"), "Missing web_research");
}

#[tokio::test]
async fn test_e2e_storage_retrieval_ladder() {
    let (_guard, storage) = setup_test_db();

    // Tier 1: Cache Insertion and Retrieval (0ms)
    let url = "https://openissa.internal/test-spec";
    let markdown_body = "# OpenISSA Specification\n\n- Low latency\n- Token preservation";
    storage
        .set_cached_url(url, 200, markdown_body, 3600)
        .expect("Cache insert should succeed");

    let cached = storage
        .get_cached_url(url)
        .expect("Cache query should succeed")
        .expect("Must return cached body");
    assert_eq!(cached, markdown_body);

    // Tier 2: BM25 Full-Text Indexing and Search (1ms)
    storage
        .index_document(
            "https://openissa.internal/architecture",
            "OpenISSA System Architecture",
            "The architecture implements a 4-tier Retrieval Ladder with SQLite WAL caching and lazy Chromium fallback.",
        )
        .expect("Indexing should succeed");

    let hits = storage
        .search_index("Retrieval Ladder", 5)
        .expect("BM25 search should succeed");
    assert!(
        !hits.is_empty(),
        "Expected BM25 hits for 'Retrieval Ladder'"
    );
    assert_eq!(hits[0].title, "OpenISSA System Architecture");
    assert!(hits[0].snippet.contains("Retrieval"));
}

#[test]
fn test_e2e_security_firewall_ssrf_guard() {
    // Must block RFC 1918 private IPs
    assert!(!NetworkFirewall::is_ip_allowed(
        &"127.0.0.1".parse().unwrap()
    ));
    assert!(!NetworkFirewall::is_ip_allowed(
        &"10.0.0.1".parse().unwrap()
    ));
    assert!(!NetworkFirewall::is_ip_allowed(
        &"192.168.1.1".parse().unwrap()
    ));
    assert!(!NetworkFirewall::is_ip_allowed(
        &"172.16.0.1".parse().unwrap()
    ));

    // Must block AWS/Cloud metadata service
    assert!(!NetworkFirewall::is_ip_allowed(
        &"169.254.169.254".parse().unwrap()
    ));

    // Must block localhost / loopback URLs
    assert!(NetworkFirewall::validate_url("http://localhost:8080/admin").is_err());
    assert!(NetworkFirewall::validate_url("http://127.0.0.1/private").is_err());
    assert!(NetworkFirewall::validate_url("http://169.254.169.254/latest/meta-data").is_err());

    // Public domains pass initial syntax validation
    assert!(NetworkFirewall::validate_url("https://api.github.com/zen").is_ok());
}

#[tokio::test]
async fn test_e2e_research_orchestrator_flow() {
    let (_guard, storage) = setup_test_db();

    // Seed local BM25 index with offline document for deterministic testing
    storage
        .index_document(
            "https://docs.rust-lang.org/edition-guide/rust-2024/async-closures.html",
            "Async Closures in Rust 2024",
            "# Async Closures\n\nAsync closures use AsyncFn traits.\n\n- Stabilized in Rust 1.85\n- Implements AsyncFn, AsyncFnMut, and AsyncFnOnce\n- Proper capture lifetimes without manual boxing",
        )
        .expect("Seed document indexing should succeed");

    let orchestrator =
        ResearchOrchestrator::new(storage).expect("Failed to create ResearchOrchestrator");

    // Execute research with goal targeting seeded knowledge
    let result = orchestrator
        .execute_research("Async Closures in Rust 2024", 2)
        .await
        .expect("Research execution should succeed");

    assert_eq!(result.goal, "Async Closures in Rust 2024");
    assert!(!result.synthesis_markdown.is_empty());
    assert!(
        result.synthesis_markdown.contains("Async Closures")
            || result.synthesis_markdown.contains("Research")
    );
}

#[test]
fn test_e2e_config_loader_and_env_application() {
    let sample = r#"
        [openissa]
        mode = "local"
        log_level = "info"

        [providers]
        brave_api_key = "e2e_test_brave_key"
        tavily_api_key = "e2e_test_tavily_key"

        [ladder]
        cache_enabled = true
        cache_ttl_hours = 24
    "#;

    let cfg = OpenIssaConfig::from_toml_str(sample);
    assert_eq!(
        cfg.providers.brave_api_key.as_deref(),
        Some("e2e_test_brave_key")
    );
    assert_eq!(
        cfg.providers.tavily_api_key.as_deref(),
        Some("e2e_test_tavily_key")
    );
    assert_eq!(cfg.ladder.cache_ttl_hours, 24);
}
