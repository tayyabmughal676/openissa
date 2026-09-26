# OpenISSA Gemini Guidelines (GEMINI.md)

This file contains repository-specific instructions for Gemini (including Antigravity, Gemini CLI, and Google DeepMind agent environments).

---

## 🚨 MANDATORY DIRECTIVE: HUMAN REVIEW BEFORE PUSHING
**BEFORE PUSHING ANY CODE TO REMOTE REPOSITORIES OR CREATING GIT RELEASE TAGS, YOU MUST ALWAYS PAUSE AND ASK FOR HUMAN MANUAL REVIEW AND EXPLICIT APPROVAL.**  
Never run `git push`, publish release tags, or modify remote branches without direct, explicit human confirmation.

---

## Project Overview & Core Mission
* **Name**: OpenISSA (*Open Internet Search, Investigation & Synthesis Agent*)
* **Repository**: `https://github.com/tayyabmughal676/openissa`
* **Architecture**: Single compiled Rust binary MCP server (`openissa mcp`).
* **License**: Fair Source / Community & Commercial (`contact@datadaur.com`).
* **Core Value**: Saves **80%+ of an agent's context window and tokens** by absorbing web plumbing (caching, HTML stripping, browser fallback, deduplication) into a native local daemon.

---

## Architectural Rules (Do Not Violate)
1. **The Retrieval Ladder**: Always escalate from Cache (0ms) → Tantivy Index (5ms) → HTTP/2 Stream (150ms) → Lazy Chromium (CDP) → Human Handoff. Never launch Chromium for static pages.
2. **Web Lab Safety & SSRF Guard**: All outbound network requests must pass through `NetworkFirewall`. Block private IP ranges (`127.0.0.0/8`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `169.254.169.254`) and prevent DNS rebinding.
3. **No LaTeX Math in Markdown**: Never write `$$\text{...}$$` or `$\rightarrow$` in Markdown documentation. Use clean Unicode arrows (`→`) or standard Markdown code blocks.
4. **Local-First & Zero Cloud**: Data lives in `~/.openissa/` on the local machine (SQLite + Tantivy).

---

## Command Reference

```bash
# Compilation & Check
cargo check --workspace --all-targets

# Full test suite (21 unit & integration tests passing)
cargo test --workspace --all-targets

# Code formatting check
cargo fmt --all -- --check

# Strict linting (zero warnings permitted)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Release build
cargo build --release --bin openissa

# Test stdio MCP server
./target/release/openissa mcp

# Test remote MCP SSE daemon
./target/release/openissa serve 8080

# Auto-configure MCP servers in Cursor and OpenCode
./target/release/openissa install-mcp

# Test headless Chromium DOM extraction
./target/release/openissa fetch -b https://react.dev

# Run multi-query autonomous research DAG
./target/release/openissa research "Rust 2024 edition async traits" 5

# Run system diagnostic and storage health check
./target/release/openissa doctor
```

---

## Documentation & Communication Style
* Use clickable markdown links with `file://` scheme when referencing local files (e.g. [`README.md`](file:///Users/mac/Desktop/openissa/README.md), [`ARCHITECTURE.md`](file:///Users/mac/Desktop/openissa/ARCHITECTURE.md)).
* Keep code idiomatic to Rust 2021 edition with `thiserror` for typed errors and zero `unwrap()` in library crates.
* Follow [`git-release.md`](file:///Users/mac/Desktop/openissa/git-release.md) for all release tagging procedures.
