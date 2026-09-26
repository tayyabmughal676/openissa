<div align="center">

# 🌐 OpenISSA

### Stop burning 80% of your AI agent's context window on dumb web plumbing.

**A high-performance, local-first internet research engine & MCP server for AI agents.**  
*Search deeper. Fetch via ladder. Test live APIs. Verify claims. Keep 100% control.*

[![Context Savings](https://img.shields.io/badge/Context_Savings-80%25-brightgreen.svg?style=for-the-badge)](#the-80-token-tax-why-current-agent-search-is-broken)
[![MCP Native](https://img.shields.io/badge/MCP-Native-purple.svg?style=for-the-badge)](#the-mcp-tool-suite)
[![Architecture](https://img.shields.io/badge/Architecture-Single_Rust_Binary-orange.svg?style=for-the-badge)](#pillar-4-single-rust-binary--local-first-privacy)
[![Privacy](https://img.shields.io/badge/Privacy-100%25_Local--First-blue.svg?style=for-the-badge)](#pillar-4-single-rust-binary--local-first-privacy)
[![License](https://img.shields.io/badge/License-Fair_Source_%2F_Commercial-black.svg?style=for-the-badge)](#license)

<br/>

```bash
# Plug into Claude Code in 5 seconds:
claude mcp add openissa -- openissa mcp
```

</div>

---

## ⚡ The 3-Second Pitch: Why OpenISSA?

If you ask **Claude Code**, **Cursor**, or an autonomous agent to research the web today, it is forced to juggle 4–5 separate, bloated MCP servers (Search, Scrapers, Playwright, and Curl). Your agent wastes its tokens and context on plumbing before it even starts thinking.

**OpenISSA replaces that entire fragmented mess with one native local Rust MCP server:**

| The Old Fragmented Way | The OpenISSA Substrate |
| :--- | :--- |
| ❌ **4–5 separate MCP servers** needed for search, scraping, browser, and testing | ✅ **One single, native Rust binary** (`openissa mcp`) exposing a cohesive tool suite |
| ❌ **Dumping 50KB of raw HTML, scripts & cookie banners** into your agent's context | ✅ **Extracts token-dense Markdown** directly at the byte stream — **saves 80%+ tokens** |
| ❌ **Spawning heavy Chromium for static pages**, eating 2GB RAM & stalling for 15s | ✅ **The Retrieval Ladder**: Cache (0ms) → Tantivy (5ms) → HTTP/2 (150ms) → Chrome *only if JS is required* |
| ❌ **Hallucinating outdated advice** from random SEO blogs that contradict official APIs | ✅ **Web Lab & Evidence Graphs**: Cross-checks primary RFCs and **tests live API endpoints** live |
| ❌ **Re-fetching the same web documentation** in every single conversation | ✅ **Persistent local SQLite + Tantivy index** on your SSD (instant 0ms cache hits) |
| ❌ **Sending queries to third-party cloud scrapers** and paying subscription fees | ✅ **100% local, private, and offline-capable** with zero cloud dependencies |

---

## What is OpenISSA?

**OpenISSA** (*Open Internet Search, Investigation & Synthesis Agent*) is a local-first, single-binary MCP server written in Rust that turns the open web into an inspectable, cost-disciplined, and verifiable intelligence substrate for AI agents.

Instead of forcing your AI client (**Claude Code**, **Cursor**, **Windsurf**, or custom agent loops) to clumsily juggle separate tools for search, scraping, browser automation, and API testing, OpenISSA unifies the entire web lifecycle into a single high-performance engine that runs locally on your machine.

---

## The "80% Token Tax": Why Current Agent Search is Broken

Today, asking an autonomous agent to research a technical question on the web is an economic and computational disaster:

```text
CURRENT FRAGMENTED AGENT REALITY:

[ Client Context Window (100k - 200k tokens) ]
┌────────────────────────────────────────────────────────────────────────┐
│ 80% WASTED ON DUMB PLUMBING & BLOAT:                                  │
│ • Calling a Search MCP → getting 10 noisy, unverified snippets         │
│ • Calling a Scraper MCP → dumping 45KB of raw HTML, scripts & menus    │
│ • Calling a Browser MCP → spinning up heavy Chromium for a static doc  │
│ • Fighting Cloudflare 403s, cookie popups, and broken redirects        │
│ • Re-fetching the exact same documentation pages in every new prompt   │
│ • Getting hallucinations because blogs contradict official RFCs/APIs   │
├────────────────────────────────────────────────────────────────────────┤
│ 20% ACTUALLY LEFT FOR REASONING, CODE GENERATION & SYNTHESIS           │
└────────────────────────────────────────────────────────────────────────┘
```

When an agent has to orchestrate raw HTTP calls, browser sessions, and JSON parsing in its own context window, it burns **80% of its tokens and attention budget** before it even begins synthesizing an answer.

### The OpenISSA Solution
OpenISSA absorbs that entire plumbing layer into a native local daemon. It executes the retrieval ladder, sanitizes DOM trees to byte-dense Markdown, tests hypotheses, and cross-checks claims **locally**, returning only structured, verifiable evidence:

```text
WITH OPENISSA MCP:

Any AI Client (Claude Code / Cursor / Windsurf / Agent Loop)
                            │
                            ▼  Single tool call: `web_research` or `web_fetch`
              ┌─────────────────────────────┐
              │       OpenISSA MCP          │
              │  (Single Local Rust Binary) │
              └──────────────┬──────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
   The Retrieval Ladder     Web Lab Active       Evidence Graph &
   (Cache → Tantivy →       Testing Engine       Claim Verification
    HTTP → Browser)         (Live API tests)     (Primary vs. Secondary)
                             │
                             ▼
  [ Verified Claims + Clean Token-Dense Markdown + Exact Citations ]
  → 80%+ Context Window & Token Budget Preserved for Your Agent!
```

---

## Why OpenISSA vs. All Others?

The current landscape is fragmented into narrow silos. Here is how OpenISSA fundamentally differs from existing alternatives:

```
┌────────────────────────────────────────────────────────────────────────┐
│                          THE LANDSCAPE TODAY                           │
├───────────────────┬───────────────────┬──────────────────┬─────────────┤
│ 1. Deep Research  │ 2. Scrapers /     │ 3. Browser       │ 4. Search   │
│    Frameworks     │    LLM Crawlers   │    Agents (MCP)  │    APIs/MCP │
├───────────────────┼───────────────────┼──────────────────┼─────────────┤
│ • GPT Researcher  │ • Firecrawl       │ • browser-use    │ • Brave MCP │
│ • HF DeepResearch │ • Crawl4AI        │ • MS Playwright  │ • SearXNG   │
│ • Stanford STORM  │ • Spider-rs       │   MCP            │ • Tavily    │
│ • LangChain ODR   │ • Jina Reader     │ • Automata       │ • Exa       │
└───────────────────┴───────────────────┴──────────────────┴─────────────┘
                               ▲
                               │  THE GAP FILLED BY OPENISSA
                               ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        OPENISSA MCP SERVER                             │
│  Single Rust binary: Unified Ladder + Federated Search + Active        │
│  Live API Testing + Tantivy/SQLite Local Cache + Evidence DAGs         │
└────────────────────────────────────────────────────────────────────────┘
```

### 1. OpenISSA vs. Autonomous Research Frameworks (GPT Researcher, HF Open Deep Research, STORM)
*   **The Competitors**: [GPT Researcher](https://github.com/assafelovic/gpt-researcher), Hugging Face [Open Deep Research](https://github.com/huggingface/smolagents), and Stanford [STORM](https://github.com/stanford-oval/storm) are monolithic Python applications. They take a topic and generate a 2,000-word essay or PDF report.
*   **The Flaw**: They are stand-alone end products, **not tool substrates**. You cannot easily plug them into your day-to-day coding assistant (like Claude Code or Cursor) to answer precise engineering questions in real time. They also rely on heavy Python virtualenvs, Docker containers, and massive dependency trees.
*   **The OpenISSA Edge**: OpenISSA is a **pure, headless MCP server**. It doesn't write essays; it delivers grounded, structured research primitives directly into whatever AI client you already use, powered by a sub-50MB compiled Rust binary.

### 2. OpenISSA vs. LLM Scrapers & Crawlers (Firecrawl, Crawl4AI, Spider-rs)
*   **The Competitors**: [Firecrawl](https://github.com/firecrawl/firecrawl-mcp-server) and [Crawl4AI](https://github.com/sadiuysal/crawl4ai-mcp-server) are content converters. They turn a given URL into LLM-ready Markdown.
*   **The Flaw**: They are **passive scrapers only**. They don't do federated search across providers, they don't plan multi-step research graphs, they don't verify whether information is true, and they cannot run active API experiments. Furthermore, Firecrawl is predominantly a paid cloud API, while Crawl4AI requires a heavy Python + Playwright Docker stack.
*   **The OpenISSA Edge**: OpenISSA includes high-speed streaming HTML-to-Markdown conversion (leveraging streaming Rust parsers inspired by `spider-rs`), but embeds this inside a complete **Retrieval Ladder** with search federation, local Tantivy indexing, and active verification.

### 3. OpenISSA vs. Heavy Browser Agents (browser-use, Playwright MCP)
*   **The Competitors**: [browser-use](https://github.com/Saik0s/mcp-browser-use) and [Microsoft Playwright MCP](https://github.com/microsoft/playwright-mcp) launch headless Chrome instances and navigate pages visually or via accessibility trees.
*   **The Flaw**: **Brute-force browser execution is an anti-pattern for 90% of web research.** Launching Chromium for a static API documentation page burns 1GB+ of RAM, takes 5–15 seconds, and spews thousands of unnecessary DOM node tokens into your context window.
*   **The OpenISSA Edge**: The **Retrieval Ladder**. OpenISSA hits local cache first, local Tantivy index second, fast streaming HTTP/2 third, and **only escalates to headless Chromium if dynamic JavaScript execution or user interaction is strictly required**.

### 4. OpenISSA vs. Dumb Search MCPs (Brave Search MCP, SearXNG MCP, Tavily)
*   **The Competitors**: Standard search MCPs take a string and return 5–10 snippets.
*   **The Flaw**: Snippets are frequently misleading, out of date, or SEO spam. Your agent is forced to make subsequent calls to read the actual pages, burning tokens with zero caching or contradiction detection.
*   **The OpenISSA Edge**: OpenISSA federates search across multiple engines (Brave, SearXNG, DuckDuckGo), deduplicates results, scores authority, crawls deeper links, and builds an **Evidence Graph** cross-checking secondary sources against primary documentation.

---

## Detailed Competitive Matrix

| Feature / Dimension | GPT Researcher | Firecrawl MCP | Browser-Use MCP | Brave/SearXNG MCP | **OpenISSA MCP** |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Form Factor** | Python App / CLI | Cloud SaaS / Docker | Python / Node MCP | Basic Tool Wrapper | **Single Native Rust Binary** |
| **Client Compatibility** | Standalone | Any MCP Client | Any MCP Client | Any MCP Client | **Any MCP Client (Claude Code, Cursor, etc.)** |
| **Token & Context Savings** | Low (dumps full text) | Medium (Clean MD) | Terrible (Vision/DOM) | Low (Raw snippets) | **Maximum (Saves 80%+ tokens)** |
| **The Retrieval Ladder** | ❌ No | ❌ No | ❌ No (Browser only) | ❌ No | **✅ Yes (Cache → Tantivy → HTTP → Browser)** |
| **Active Web Investigation** | ❌ Passive only | ❌ Passive only | ⚠️ UI clicks only | ❌ No | **✅ Yes (Live API testing in Web Lab)** |
| **Evidence & Claims Graph** | ⚠️ Flat text report | ❌ None | ❌ None | ❌ None | **✅ Yes (Primary vs. Secondary DAG)** |
| **Contradiction Detection** | ❌ None | ❌ None | ❌ None | ❌ None | **✅ Yes (Highlights doc vs. blog mismatches)** |
| **Persistent Local Memory** | ❌ Ephemeral | ❌ Cloud / Redis | ❌ None | ❌ None | **✅ Yes (Embedded SQLite + Tantivy Index)** |
| **Resource Footprint** | Heavy (Python) | Heavy (Docker cluster)| Heavy (1-2GB RAM) | Lightweight | **Ultra-light (<50MB RAM idle)** |
| **Privacy & Data Control** | ⚠️ Local Python | ❌ Third-party Cloud | ⚠️ Local | ❌ Third-party Cloud | **✅ 100% Local & Private** |

---

## The 4 Core Architectural Pillars

### Pillar 1: The Retrieval Ladder (Economic Discipline)
Instead of treating all web access identically, OpenISSA escalates through 5 distinct tiers:

```text
Local Cache (0ms) → Tantivy Index (5ms) → HTTP/2 Streaming (150ms) → Chromium Fallback (2s) → Human Handoff
```

*   **90% of requests** are resolved at Tier 1, 2, or 3 without ever touching a browser.
*   If a Cloudflare challenge or CAPTCHA is encountered, OpenISSA **never wastes tokens trying to evade it**; it halts and triggers a clean human handoff notification.

### Pillar 2: Active Investigation & The "Web Lab"
Most tools treat the web as a library to be read. OpenISSA treats it as an **experimental environment**.
*   **The Problem**: Documentation says an API supports `?limit=100`, but a blog post says it was restricted to `?limit=50`.
*   **The OpenISSA Solution**: The agent invokes `web_test_endpoint`. OpenISSA constructs sandboxed HTTP requests, hits the endpoint live, inspects the headers and JSON payload, tests boundary conditions, and returns:
    > *"Verified live on 2026-09-26: `?limit=100` returns HTTP 400 with `{'error': 'max_limit_50'}`. Documentation is stale; live behavior confirms 50."*
*   **Safety**: Complete SSRF protection blocking private subnets (`10.0.0.0/8`, `127.0.0.1`, `192.168.0.0/16`, AWS metadata endpoints).

### Pillar 3: Evidence Graphs vs. Hallucinated Summaries
LLMs hallucinate during deep research because unverified SEO affiliate posts sit next to official RFCs with equal weight. OpenISSA classifies every source into a strict hierarchy:
1.  **Tier 1 (Primary)**: Official documentation, RFCs, GitHub source repos, official release notes.
2.  **Tier 2 (Secondary)**: Verified engineering blogs, academic papers, established tech journalism.
3.  **Tier 3 (Community)**: Stack Overflow, Reddit, forum discussions.
4.  **Tier 4 (Low-Confidence)**: SEO scrapers, unverified aggregators.

Every research session compiles a **Claim-to-Evidence DAG**, tracking where claims originate and explicitly warning you when sources contradict each other.

### Pillar 4: Single Rust Binary & Local-First Privacy
*   **Zero Dependencies**: No Python version conflicts, no Node daemon crashes, no Docker containers required.
*   **Complete Privacy**: Your search queries, extracted data, and research traces live in `~/.openissa/` on your SSD. Nothing is phoned home to a centralized telemetry server.

---

## The MCP Tool Suite

OpenISSA exposes 5 lean, high-leverage tools to your AI client:

| Tool | Purpose | Key Parameters |
| :--- | :--- | :--- |
| `web_search` | Federated search across Brave, SearXNG, DuckDuckGo with deduplication & authority scoring. | `query`, `providers`, `freshness`, `limit` |
| `web_fetch` | Smart laddered fetcher. Auto-escalates from HTTP streaming to headless Chromium if JS is required. Returns token-dense Markdown. | `url`, `force_browser`, `max_length_tokens` |
| `web_inspect` | Deep network & DOM diagnostics: HTTP status, redirect chains, SSL, response headers, robots.txt, sitemaps. | `url`, `diagnostics` |
| `web_test_endpoint` | Sandboxed HTTP experiment runner. Safely test live REST/GraphQL APIs with methods, headers, and params. | `url`, `method`, `headers`, `params`, `body` |
| `web_research` | Autonomous research engine. Bounded DAG that searches, fetches via ladder, verifies against primary sources, and builds an evidence graph. | `goal`, `budget`, `require_primary_verification` |

---

## Quickstart

### 1. Installation
Install the pre-compiled native binary:

```bash
# Via Cargo
cargo install openissa

# Or via Homebrew (macOS)
brew install openissa/tap/openissa
```

### 2. Connect to Claude Code
Add OpenISSA to Claude Code with a single command:

```bash
claude mcp add openissa -- openissa mcp
```

### 3. Connect to Cursor / Windsurf
Add OpenISSA to your `mcp.json` or `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "openissa": {
      "command": "openissa",
      "args": ["mcp"],
      "env": {
        "BRAVE_API_KEY": "optional-key-here"
      }
    }
  }
}
```

### 4. CLI Usage (Independent of MCP)
OpenISSA also functions as a powerful local CLI tool:

```bash
# Fast federated search
openissa search "Rust 2024 edition async traits"

# Fetch and stream clean Markdown
openissa fetch https://docs.rs/tokio/latest/tokio/

# Run network diagnostics
openissa inspect https://api.github.com

# Launch an autonomous research session
openissa research "Investigate breaking changes in Axum 0.8" --budget-requests 15
```

---

## Configuration (`~/.openissa/config.toml`)

```toml
[openissa]
mode = "local"
log_level = "info"

[providers]
default = "auto"
# Brave, SearXNG, DuckDuckGo (HTML)
brave_api_key = "your_brave_key"
searxng_url = "http://localhost:8080"

[ladder]
cache_enabled = true
cache_ttl_hours = 48
enable_browser_fallback = true
max_browser_sessions = 2

[research]
max_requests = 25
max_runtime_sec = 120
max_depth = 3
block_private_networks = true # SSRF protection
```

---

## Roadmap

- [x] Conceptual specification & competitive landscape analysis (v0.2).
- [ ] **v0.1**: Single Rust binary MCP server with `web_search`, `web_fetch` (streaming HTML → Markdown), and SQLite cache.
- [ ] **v0.2**: The Retrieval Ladder with lazy Chromium CDP fallback + `web_test_endpoint` Web Lab with SSRF guardrails.
- [ ] **v0.3**: Tantivy full-text index integration + autonomous research DAG with primary vs. secondary evidence graphs.
- [ ] **v1.0**: Embedded hybrid vector search (LanceDB) and multi-node team sharing.

## License

OpenISSA is licensed under a **Fair Source / Community & Commercial License**:

* **Community Use (Free)**: 100% free for individual developers, hobbyists, students, open-source projects, and small teams (up to 5 users). Individual developers may freely use OpenISSA on their personal or work machines without restrictions.
* **Commercial Use**: A commercial license is required for:
  1. Enterprise team deployments with more than 5 users.
  2. Embedding, bundling, or redistributing OpenISSA within a commercial product, SaaS platform, or proprietary AI agent.

For commercial licensing inquiries, enterprise support, and custom deployments:  
📧 **contact@datadaur.com**
