# OpenISSA — Open Internet Search, Investigation & Synthesis Agent

> **A privacy-first, local-first internet intelligence engine and MCP server for AI agents.**
>
> **Search deeper. Explore wider. Investigate. Verify. Keep control.**

Status: Implemented & Production-Verified Product Specification (v0.1.0 Baseline)  
Date: 2026-09-26  
Scope Decision: **Headless MCP-Only Architecture** (No desktop GUI/App; Sparrow companion postponed)

---

## 1. Executive Summary & The Core Problem

### The Problem: AI Agents Waste 80% of Their Context Window & Tokens on Plumbing
Today, if a developer using **Claude Code**, **Cursor**, **Windsurf**, or a custom autonomous agent wants the model to research the web, the agent is forced to juggle **4 to 5 fragmented, uncoordinated MCP servers**:
1. A **Search MCP** (e.g. Brave Search, SearXNG, Tavily)
2. A **Scraper/Fetch MCP** (e.g. Firecrawl, Puppeteer, Fetch)
3. A **Browser Automation MCP** (e.g. Microsoft Playwright, browser-use)
4. An **API/Testing MCP** (e.g. curl, REST tester)

```text
CURRENT FRAGMENTED REALITY (Massive Context & Token Waste):

[ AI Client Context Window (100-200k tokens) ]
┌────────────────────────────────────────────────────────────────────────┐
│ 80% WASTED ON DUMB PLUMBING:                                          │
│ • Calling Search MCP → getting 10 raw snippets                         │
│ • Calling Scrape MCP → dumping 40KB of messy HTML/boilerplate into ctx │
│ • Firing up heavy Chrome/Playwright for static pages (eating RAM/CPU)  │
│ • Fighting Cloudflare 403s, cookie banners, navigation menus           │
│ • Re-fetching the same URLs over and over across conversations         │
│ • Halting because of unverified hallucinations and conflicting blogs   │
├────────────────────────────────────────────────────────────────────────┤
│ 20% ACTUALLY LEFT FOR REASONING, CODING, AND SYNTHESIS                │
└────────────────────────────────────────────────────────────────────────┘
```

### The Solution: OpenISSA as the Unified Internet Substrate
OpenISSA collapses that entire messy stack into **one single, high-performance, local-first Rust MCP server**.

OpenISSA handles the retrieval ladder, caching, dynamic rendering, network diagnostics, and active hypothesis verification **under the hood**, returning clean, token-dense Markdown and evidence-backed claims:

```text
OPENISSA MCP SUBSTRATE (Maximum Efficiency & Grounded Truth):

Any AI Client (Claude Code / Cursor / Windsurf / Custom Agents)
                           │
                           ▼  MCP Tool Calls (stdio / SSE)
            ┌─────────────────────────────┐
            │       OpenISSA MCP          │
            │  (Single Local Rust Binary) │
            └──────────────┬──────────────┘
                           │
       ┌───────────────────┼───────────────────┐
       ▼                   ▼                   ▼
 The Retrieval Ladder   Web Lab / Active    Evidence Graph &
 (Cache → Tantivy →     Investigation       Verification
  HTTP → Browser)       (API tests, live    (Primary sources,
                        reproduction)       contradiction detection)
                           │
                           ▼
[ Clean, Token-Dense Markdown + Verified Evidence ] → 80%+ Context Window & Token Savings!
```

Instead of blindly operating as:
```text
query → search → fetch pages → dump into context → summarize
```

OpenISSA operates like an investigative engineer:
```text
research goal
    ↓
plan (DAG)
    ↓
discover (federated search & local index)
    ↓
inspect (headers, redirects, status, robots)
    ↓
fetch / browse / execute (via the cheapest reliable ladder tier)
    ↓
observe & test (Web Lab live reproduction)
    ↓
cross-check & verify (primary vs secondary sources)
    ↓
synthesize
    ↓
evidence-backed result with citations
```

---

## 2. Core Vision

### The Big Idea
Build an internet intelligence layer that an AI agent can use like a **second computer**.

The AI client should be able to make a single high-level tool call:
```json
{
  "tool": "web_research",
  "arguments": {
    "goal": "Verify whether the Stripe Invoicing API supports tax calculation on draft items",
    "budget": { "max_depth": 3, "max_requests": 25, "verify_live": true }
  }
}
```

Instead of having to micromanage:
- which search provider to query
- which page needs JavaScript vs. raw HTTP
- whether a headless browser must be spawned
- how to extract the article content from surrounding HTML junk
- how to verify if documentation is stale or contradictory
- how to test the actual endpoint

OpenISSA becomes the system that handles those decisions deterministically.

---

## 3. Product Identity & Strategic Decisions

### 3.1 OpenISSA
*   **Expansion**: Open Internet Search, Investigation & Synthesis Agent.
*   **Nature**: Single-binary, local-first, provider-agnostic MCP server written in Rust.
*   **Core Tagline**: *The open internet intelligence engine for AI agents.*

### 3.2 Strategic Decision: Pure MCP Server (No UI / Sparrow Postponed)
*   **Decision**: We will **NOT** build a desktop app, chat UI, or standalone frontend initially.
*   **Sparrow Postponed**: The proposed "Sparrow" companion app (Tauri/React desktop companion) is **officially dropped and postponed to the future if needed**.
*   **Rationale**: Any AI developer already has their preferred front-ends and agent clients (Claude Code, Cursor, Windsurf, Claude Desktop, autonomous terminal loops). OpenISSA’s fastest path to massive adoption is to be the **most indispensable, lightweight, single-binary MCP server** they plug into those existing clients.

---

## 4. What OpenISSA Is NOT

OpenISSA is:
- **NOT** a desktop chat GUI or Electron/Tauri app (Sparrow is dropped).
- **NOT** another Google or Perplexity clone.
- **NOT** another basic search API wrapper.
- **NOT** a dumb `search()` MCP that dumps search snippets into context.
- **NOT** a heavy, slow browser-only agent that launches Chrome for every URL.
- **NOT** a centralized, cloud-hosted proprietary database.

OpenISSA's value is the composition:
```text
Retrieval Ladder + Live Investigation + Evidence Graphs + Local-First Cache + MCP Native
```

---

## 5. Core Principles

### 5.1 Local-First
Data and processing remain on the user's machine. OpenISSA runs as a self-contained binary with embedded storage:
```text
~/.openissa/
├── openissa (binary)
├── openissa.db (SQLite + WAL)
├── tantivy_index/
├── blobs/ (content-addressed HTML/raw storage)
└── config.toml
```
No mandatory cloud accounts, no hosted vector DBs, no tracking.

### 5.2 Provider-Agnostic & Search-Free Discovery
Interchangeable search adapters & direct site discovery:
*   **Autonomous Discovery**: Native `/llms.txt`, `sitemap.xml`, and `robots.txt` indexer (100% provider-free, zero API cost)
*   **Federated Search**: Brave Search API, SearXNG, DuckDuckGo
*   **Site Spidering**: Domain-bounded internal link traversal and local Tantivy indexing

### 5.3 Deterministic Where Possible
Do not waste LLM tokens or inference latency on tasks a deterministic component can perform:
*   HTTP pooling, redirects, retries, compression, rate-limiting
*   Deduplication, canonical URL resolution, DOM cleaning, Markdown conversion
*   Tantivy BM25 indexing, caching, budget tracking

Reserve AI reasoning for:
*   Formulating research hypotheses
*   Selecting ambiguous research leads
*   Synthesizing evidence and resolving contradictions

### 5.4 Inspectable & Traceable
Every research operation produces a structured execution trace:
*   Requests made, response times, HTTP status codes
*   Sources considered vs. rejected
*   Claims extracted and their backing evidence
*   Token/bandwidth savings calculated

### 5.5 Controllable via Hard Budgets
Every operation has strict limits:
`max_requests`, `max_pages`, `max_depth`, `max_runtime_sec`, `max_bytes`, `allowed_domains`, `blocked_domains`.

### 5.6 Privacy-First
User research history does not leave the local machine. Web content is treated as untrusted input with strict policy boundaries.

### 5.7 Efficient by Default (The Retrieval Ladder)
Always use the least expensive reliable method first.

---

## 6. The OpenISSA Retrieval Ladder

The Retrieval Ladder is OpenISSA's core algorithmic cost-control mechanism:

```text
                  Research Request / URL
                             │
                             ▼
                    [ Tier 1: Local Cache ]
                      /                 \
                  Hit (0ms, 0 tokens)   Miss
                  │                      │
                  ▼                      ▼
                Return          [ Tier 2: Tantivy Index ]
                                  /                   \
                              Found (Local BM25)      Miss
                              │                        │
                              ▼                        ▼
                            Return              [ Tier 3: Direct HTTP/2 Fetch ]
                                                (reqwest + streaming HTML clean)
                                                       │
                                                JS Rendering Required?
                                                  /             \
                                                No              Yes
                                                │                │
                                                ▼                ▼
                                         Clean Markdown   [ Tier 4: Headless Browser ]
                                                          (Playwright / CDP)
                                                                 │
                                                          Interaction Needed?
                                                            /           \
                                                          No            Yes
                                                          │              │
                                                          ▼              ▼
                                                     DOM Extract   Agent Interaction
                                                                         │
                                                                 Access Blocked / CAPTCHA?
                                                                   /             \
                                                                 No              Yes
                                                                 │                │
                                                                 ▼                ▼
                                                              Extract     [ Tier 5: Human Handoff ]
                                                                          (Alert client / pause)
```

**Why this saves 80% of tokens and costs:**
*   90% of technical documentation and static blogs can be fetched via **Tier 3 (Direct HTTP)** in <200ms without touching a browser.
*   Repeated research hits **Tier 1 (Cache)** or **Tier 2 (Tantivy)** at zero network cost.
*   **Tier 4 (Browser)** is only launched when JavaScript rendering or dynamic interactions are strictly verified as necessary.

---

## 7. Deep Internet Research: Competitive Landscape Analysis

A comprehensive investigation of current open-source projects, tools, and commercial products reveals five distinct categories—and a massive structural white space that OpenISSA fills.

### 7.1 Category Breakdown

#### Category 1: Autonomous Deep Research Frameworks
*   **[GPT Researcher](https://github.com/assafelovic/gpt-researcher)**: High-popularity Python framework for deep web research. Uses multi-agent flows (LangGraph) to search 20+ sources and compile long research reports.
    *   *Limitations*: A monolithic Python application designed to generate PDF/markdown documents. It cannot function as a lightweight, low-latency MCP server for an external agent (like Claude Code) to use as a real-time investigation tool.
*   **[Hugging Face Open Deep Research](https://github.com/huggingface/smolagents)**: Built on Hugging Face’s `smolagents`. Uses Python code-agents in Docker/E2B sandboxes to browse and reason.
    *   *Limitations*: Heavyweight Python/Docker environment. Coupled to HF ecosystem. Focuses on full report synthesis rather than modular tool primitives.
*   **[Stanford STORM](https://github.com/stanford-oval/storm)**: Academic AI research system that synthesizes Wikipedia-style articles by simulating multi-perspective expert interviews.
    *   *Limitations*: Research paper writing tool, not an operational tool server for coding agents.
*   **[LangChain Open Deep Research](https://github.com/langchain-ai/open_deep_research)**: LangGraph-based research template.
    *   *Limitations*: Framework template, not a single deployable binary.

#### Category 2: LLM Web Crawlers & Scrapers
*   **[Firecrawl](https://github.com/firecrawl/firecrawl-mcp-server)** (Mendable): Market leader for web-to-markdown scraping. Has an official MCP server.
    *   *Limitations*: Primarily a paid cloud API. Self-hosting requires Docker, Redis, Postgres, and Playwright clusters. It is purely a scraper: no search federation, no research planning, no live endpoint testing, no claim verification.
*   **[Crawl4AI](https://github.com/sadiuysal/crawl4ai-mcp-server)**: Top open-source Python LLM crawler.
    *   *Limitations*: Python/Playwright/Docker dependency footprint. Lacks search, investigation DAGs, and local-first memory.
*   **[Spider-rs (`spider`)](https://github.com/spider-rs/spider)**: The fastest web crawler in existence, written in Rust. Contains a `spider_transformations` crate for fast markdown conversion.
    *   *Role in OpenISSA*: An outstanding candidate library to power OpenISSA's internal HTTP and crawler subsystems.
*   **[Jina Reader](https://github.com/jina-ai/reader)** (`r.jina.ai`): Cloud-based prefix proxy that converts URLs to markdown.
    *   *Limitations*: Third-party cloud dependency; privacy concern for enterprise codebases.

#### Category 3: Browser Automation MCPs
*   **[Microsoft Playwright MCP](https://github.com/microsoft/playwright-mcp)**: Microsoft's official MCP server exposing accessibility-tree snapshots and page interactions.
*   **[mcp-browser-use](https://github.com/Saik0s/mcp-browser-use)**: FastMCP server wrapping `browser-use`.
    *   *Limitations*: Spawns heavy Chromium for every single operation. Wastes massive CPU, RAM, and tokens on basic static pages. No retrieval ladder.

#### Category 4: Search & Meta-Search MCPs
*   **`mcp-searxng`**, **`brave-search-mcp`**, **`tavily-mcp`**, **`exa-mcp`**: Simple wrappers around individual search APIs that return 5–10 snippets.
    *   *Limitations*: Dumb snippet returns. Zero active verification, zero investigation, zero local caching.

#### Category 5: API Testing MCPs
*   **[Postman MCP](https://github.com/postmanlabs/postman-mcp-server)**, **[API Lab MCP](https://github.com/atototo/api-lab-mcp)**, **`mcp-rest-api`**: Expose HTTP request tools for developer API testing.
    *   *Limitations*: Built for developer QA workflows, not integrated into autonomous research or evidence verification.

---

### 7.2 Competitive Matrix

| Capability / Dimension | GPT Researcher | Firecrawl MCP | Browser-Use MCP | Brave/SearXNG MCP | **OpenISSA MCP** |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Architecture** | Python App | Cloud / Docker | Python / Node | Simple API Wrapper | **Single Rust Binary** |
| **Interface** | CLI / Web UI | MCP / REST | MCP | MCP | **MCP (stdio & SSE)** |
| **The Retrieval Ladder** | ❌ No | ❌ No | ❌ No (Browser only) | ❌ No | **✅ Yes (Cache → HTTP → Browser)** |
| **Active Web Investigation** | ❌ Passive only | ❌ Passive only | ⚠️ UI clicks only | ❌ No | **✅ Yes (Web Lab endpoint testing)** |
| **Evidence & Claims Graph** | ⚠️ Flat report | ❌ None | ❌ None | ❌ None | **✅ Yes (Primary vs Secondary DAG)** |
| **Local Index & Cache** | ❌ Ephemeral | ❌ Cloud / Redis | ❌ No | ❌ No | **✅ Yes (SQLite + Tantivy)** |
| **Context Window Savings** | Low (dumps full text) | Medium (Markdown) | Terrible (Vision/DOM) | Low (Snippets) | **Maximum (Saves 80%+ tokens)** |
| **Local Privacy** | ⚠️ Local Python | ❌ Cloud SaaS | ⚠️ Local Python | ❌ External API | **✅ 100% Local & Private** |

---

## 8. The OpenISSA MCP Tool Suite

OpenISSA exposes 6 cohesive, high-impact tools over the standard Model Context Protocol (stdio & SSE):

```json
[
  {
    "name": "web_search",
    "description": "Federated multi-provider web search with deduplication, domain scoring, and local index merging.",
    "inputSchema": {
      "type": "object",
      "properties": {
        "query": { "type": "string", "description": "The search query" },
        "providers": { "type": "array", "items": { "type": "string" }, "description": "Optional providers: ['brave', 'searxng', 'local']" },
        "freshness": { "type": "string", "enum": ["day", "week", "month", "year", "all"] },
        "limit": { "type": "integer", "default": 10 }
      },
      "required": ["query"]
    }
  },
  {
    "name": "web_index_site",
    "description": "Autonomously map an entire documentation domain without search engines by parsing /llms.txt, XML sitemaps, and robots directives.",
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
    "name": "web_fetch",
    "description": "Smart laddered fetcher. Automatically checks local cache, uses fast streaming HTTP, and escalates to headless browser only if JavaScript rendering is required. Returns clean, token-efficient Markdown.",
    "inputSchema": {
      "type": "object",
      "properties": {
        "url": { "type": "string", "description": "URL to fetch" },
        "force_browser": { "type": "boolean", "default": false },
        "wait_for_selector": { "type": "string" },
        "max_length_tokens": { "type": "integer", "default": 4000 }
      },
      "required": ["url"]
    }
  },
  {
    "name": "web_inspect",
    "description": "Network and architectural diagnostics for any URL or API endpoint: HTTP status, redirect chains, response headers, SSL/TLS details, robots.txt, sitemaps, and discovered API schemas.",
    "inputSchema": {
      "type": "object",
      "properties": {
        "url": { "type": "string", "description": "URL or endpoint to inspect" },
        "diagnostics": { "type": "array", "items": { "type": "string", "enum": ["headers", "redirects", "ssl", "robots", "sitemap", "endpoints"] } }
      },
      "required": ["url"]
    }
  },
  {
    "name": "web_test_endpoint",
    "description": "Controlled, sandboxed HTTP experiment runner. Safely test REST/GraphQL API endpoints with specific methods, headers, and payloads to verify live behavior, pagination, or errors.",
    "inputSchema": {
      "type": "object",
      "properties": {
        "url": { "type": "string", "description": "Target endpoint URL" },
        "method": { "type": "string", "enum": ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"], "default": "GET" },
        "headers": { "type": "object" },
        "params": { "type": "object" },
        "body": { "type": "string" }
      },
      "required": ["url"]
    }
  },
  {
    "name": "web_research",
    "description": "Autonomous investigative engine. Executes a bounded research DAG: searches across sources, fetches via ladder, verifies claims against primary documentation, detects contradictions, and returns an evidence-backed synthesis.",
    "inputSchema": {
      "type": "object",
      "properties": {
        "goal": { "type": "string", "description": "The research question or investigation objective" },
        "budget": {
          "type": "object",
          "properties": {
            "max_requests": { "type": "integer", "default": 20 },
            "max_runtime_sec": { "type": "integer", "default": 90 },
            "max_depth": { "type": "integer", "default": 3 }
          }
        },
        "require_primary_verification": { "type": "boolean", "default": true }
      },
      "required": ["goal"]
    }
  }
]
```

---

## 9. Subsystems Architecture

### 9.1 Web Fetching Engine
*   **Responsibilities**: Connection pooling, HTTP/2 multiplexing, streaming compression (brotli/gzip), conditional requests (`ETag`, `If-Modified-Since`), timeout handling, robots.txt compliance.
*   **Tech**: Rust (`reqwest`, `hyper`, `tokio`).
*   **Markdown Conversion**: `lol-html` / `scraper` / `spider_transformations` to strip scripts, styles, ads, and navigation boilerplate at the byte-stream level before converting to clean Markdown.

### 9.2 Browser Intelligence Layer
*   **Strategy**: Playwright / Chrome DevTools Protocol (CDP) bridge.
*   **Strict Escalation**: Only invoked if raw HTTP response indicates missing dynamic content (e.g. `<noscript>`, empty root divs, or specific JS frameworks).
*   **Human Handoff**: If Cloudflare Turnstile or CAPTCHA challenges are encountered, the server pauses the operation and sends an MCP progress/notification event alerting the user, rather than attempting circumvention.

### 9.3 Web Lab (Investigation & Active Testing)
*   **Core Philosophy**: *Hypothesize → Inspect → Execute → Observe → Verify.*
*   **Example Scenario**:
    1. Agent needs to verify: *"Does Endpoint X support pagination via `cursor` or `page`?"*
    2. OpenISSA searches documentation → identifies parameter.
    3. OpenISSA executes test request with `page=1` → inspects payload.
    4. OpenISSA executes test request with `page=2` → checks for data difference.
    5. Confirmed live → evidence saved to session DAG.
*   **Sandboxing & SSRF Defense**: Strict network boundaries. Private IPv4/IPv6 ranges (`10.0.0.0/8`, `127.0.0.1`, `192.168.0.0/16`, `169.254.169.254` AWS metadata) are blocked by default unless explicitly allowed.

### 9.4 Evidence Graph & Source Quality Hierarchy
*   **Hierarchy**:
    *   *Tier 1 (Primary)*: Official documentation, RFCs, GitHub repositories, official announcements, original datasets.
    *   *Tier 2 (Secondary)*: Verified engineering blogs, conference materials, established tech journalism.
    *   *Tier 3 (Community)*: Stack Overflow, Reddit, GitHub discussions.
    *   *Tier 4 (Low-Confidence)*: SEO aggregators, AI-generated summary farms.
*   **Contradiction Tracking**: Explicitly logs conflicts (e.g. *“Blog X claims feature is live, but Official Release Notes v3.2 state it was deprecated on 2026-04-01”*).

### 9.5 Local Storage & Memory
*   **SQLite + WAL**: Stores research sessions, evidence DAGs, request logs, and cached HTTP metadata.
*   **Tantivy**: Full-text search index over all previously crawled documents.
*   **Content-Addressed Blobs**: Compressed content hashes on disk for raw page snapshots.

---

## 10. Recommended Technology Stack

```text
OpenISSA Core (Rust Monorepo)
├── Runtime: Tokio (async I/O)
├── MCP Server: rmcp / custom stdio & SSE transport
├── HTTP Engine: reqwest, hyper, lol-html, spider_transformations
├── Browser: chromiumoxide / Playwright child worker (lazy loaded)
├── Embedded DB: rusqlite (SQLite + WAL mode)
├── Full-Text Search: Tantivy
└── Web API (optional local dashboard): Axum
```

**Why Rust?**
*   **Single Native Binary**: Distributable via `cargo install openissa`, Homebrew, or GitHub release binaries. No Python virtualenv hell or Node dependency explosion.
*   **Minimal Memory Footprint**: Runs silently in the background consuming <50MB RAM when idle (compared to 1GB+ for Python/Electron tools).
*   **Predictable High Concurrency**: Tokio handles hundreds of parallel streaming requests effortlessly.

---

## 11. Initial Repository Structure

```text
openissa/
├── Cargo.toml                  # Workspace definition
├── README.md                   # Quickstart, installation, MCP config
├── openissa-idea.md            # Architectural specification (this doc)
│
├── crates/
│   ├── openissa-cli/           # Binary entry point (`openissa mcp`, `openissa search`)
│   ├── openissa-core/          # Core types: Evidence, Claim, Source, Budget, DAG
│   ├── openissa-mcp/           # MCP protocol server (tool handlers, JSON-RPC, stdio/SSE)
│   ├── openissa-fetcher/       # The Retrieval Ladder: HTTP engine, stream cleaning, Markdown
│   ├── openissa-search/        # Federated search adapters (Brave, SearXNG, DuckDuckGo)
│   ├── openissa-browser/       # Lazy Chromium/CDP automation runner
│   ├── openissa-investigate/   # Web Lab test runner, API inspection, SSRF guards
│   └── openissa-storage/       # SQLite migrations, Tantivy indexing, disk cache
│
├── tests/
│   ├── ladder_tests.rs         # Escalation validation
│   └── mcp_conformance_tests.rs # MCP client compatibility
└── Makefile
```

---

## 12. Phased Roadmap (Pure-MCP Delivery)

### v0.1: The Lightweight Fast MCP (MVP) — [COMPLETED & VERIFIED]
*   Single Rust binary with `openissa mcp` stdio transport.
*   `web_search`: Federated search across DuckDuckGo, Hacker News, Wikipedia, GitHub (repos + issues/RFCs), Brave, and Tavily.
*   `web_fetch`: Smart HTTP fetcher with streaming HTML-to-Markdown cleaning (`lol-html`) and SQLite cache.
*   `web_inspect`: URL headers, redirects, and status inspector.
*   Zero-dependency TOML configuration loader for `~/.openissa/config.toml`.
*   Tested with **Claude Code**, **Cursor**, and **OpenCode**.

### v0.2: The Retrieval Ladder & Web Lab — [COMPLETED & VERIFIED]
*   `ChromiumSupervisor`: Lazy-spawned headless Chromium auto-discovery across macOS, Linux, and Windows (`--headless=new --dump-dom --virtual-time-budget=3000`).
*   `web_test_endpoint`: Sandboxed HTTP testing for live APIs with `NetworkFirewall` SSRF protection (blocking RFC1918, link-local metadata, loopback, and DNS rebinding).
*   Automatic client-side rendering (CSR) escalation heuristic.

### v0.3: Autonomous Discovery Engine & Local Memory — [COMPLETED & VERIFIED]
*   `web_index_site`: Autonomous discovery without search engines via `/llms.txt`, recursive XML sitemaps (`<urlset>` and `<sitemapindex>`), and domain-bounded `SiteSpider`.
*   Keyword path filtering to prevent token explosion.
*   SQLite FTS5 BM25 full-text search index (`doc_index`) for sub-millisecond local search.

### v0.4: Deep Evidence Engine & Remote SSE Daemon — [COMPLETED & VERIFIED]
*   `web_research`: Autonomous multi-query research DAG orchestrator with authority-weighted claim confidence scoring.
*   Contradiction detection and source quality tiering (Tier 1 Primary to Tier 4 Aggregators).
*   `openissa serve [port] [host]`: Multi-client HTTP/1.1 Server-Sent Events (SSE) daemon with `/sse`, `/message`, and `/health`.
*   21/21 passing automated unit and integration tests (`crates/openissa-cli/tests/e2e_research.rs`).

### v1.0: Enterprise & Multi-Node (Future)
*   Hybrid BM25 + embedded vector retrieval (e.g. LanceDB).
*   Multi-node distributed team cache deduplication.
*   Optional web trace visualizer companion.

---

## 13. Licensing & Commercial Model

OpenISSA adopts a **Fair Source / Community & Commercial Model**:
*   **Community Edition (Free)**: 100% free for individual developers (personal use and work tasks on personal/work machines), students, open-source projects, and small teams/startups (up to 5 users).
*   **Commercial Edition**: A commercial license is required for:
    1. Enterprise team deployments exceeding 5 users.
    2. Embedding, bundling, or redistributing OpenISSA inside commercial, revenue-generating products, SaaS platforms, or proprietary AI agents.
*   **Contact for Commercial Licensing**: `contact@datadaur.com` (future independent portal: `openissa.dev`)

---

## 14. North Star

An AI engineer types:
> *"Investigate the breaking changes in the latest Stripe API release and verify them against our endpoint."*

The AI client invokes `openissa mcp`.

OpenISSA:
1. Hits local cache → misses.
2. Federated searches official Stripe docs.
3. Streams direct HTTP → converts to token-dense Markdown.
4. Inspects endpoints → executes test requests.
5. Verifies claims → constructs evidence graph.
6. Returns clean, verified, grounded answer with exact citations.

**Result**: 
*   **80%+ savings in context window tokens and LLM spend.**
*   Zero hallucinated blog advice.
*   Executed entirely on the user's local machine with complete inspectability.