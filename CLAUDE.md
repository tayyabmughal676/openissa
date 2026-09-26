# OpenISSA Claude Guidelines (CLAUDE.md)

This file contains repository-specific instructions for Claude (including Claude Code, Claude Desktop, and Anthropic API agents).

---

## 🚨 MANDATORY RULE: HUMAN REVIEW BEFORE PUSHING
**BEFORE PUSHING ANY CODE TO REMOTE REPOSITORIES OR CREATING GIT RELEASE TAGS, YOU MUST ALWAYS PAUSE AND ASK FOR HUMAN MANUAL REVIEW AND EXPLICIT APPROVAL.**
Never run `git push`, `gh release create`, or push tags autonomously.

---

## Project Overview
* **Name**: OpenISSA (*Open Internet Search, Investigation & Synthesis Agent*)
* **Repository**: `https://github.com/tayyabmughal676/openissa`
* **Type**: Single-binary local-first Rust MCP server (stdio & SSE)
* **License**: Fair Source / Community & Commercial (`contact@datadaur.com`)
* **Primary Selling Point**: Saves 80%+ of an AI agent's context window and tokens by absorbing web plumbing (caching, HTML stripping, browser fallback) into a local Rust daemon.

---

## Architectural Invariants
1. **The Retrieval Ladder**: Cache (0ms) → Tantivy Index (5ms) → HTTP/2 Stream (150ms) → Lazy Chromium (CDP) → Human Handoff. Never launch Chromium for static pages.
2. **Web Lab Safety**: All outbound requests must pass through `NetworkFirewall`. Block private IPs (RFC1918, `127.0.0.1`, `169.254.169.254`) and prevent DNS rebinding.
3. **No LaTeX Math in Markdown**: Never write `$$\text{...}$$` or `$\rightarrow$` in docs. Use clean Unicode arrows (`→`) or standard code blocks.
4. **Zero Cloud Dependencies**: All data lives locally in `~/.openissa/` (SQLite + Tantivy).

---

## Common Development Commands

```bash
# Compilation & Check
cargo check --workspace --all-targets

# Run all unit, integration, and E2E security tests (21 tests passing)
cargo test --workspace --all-targets

# Code formatting (must be 100% clean)
cargo fmt --all -- --check

# Strict linting (zero warnings permitted)
cargo clippy --workspace --all-targets -- -D warnings

# Build optimized release binary
cargo build --release --bin openissa

# Test MCP server over stdio
./target/release/openissa mcp

# Test remote MCP SSE daemon
./target/release/openissa serve 8080

# Auto-configure MCP servers in Cursor and OpenCode
./target/release/openissa install-mcp

# Test headless Chromium CDP rendering
./target/release/openissa fetch -b https://react.dev

# Run multi-query autonomous research DAG
./target/release/openissa research "Rust 2024 edition async closures" 5

# Run system and storage health diagnostics
./target/release/openissa doctor
```

---

## Rust Code Style
* **Edition**: Rust 2021.
* **Error Handling**: Use `thiserror` in library crates (`openissa-*`) and `anyhow` in `openissa-cli`.
* **Safety**: Zero `unwrap()` in production library code. Always handle errors or use `.expect("invariant")`.
* **Memory & Performance**: Keep idle memory under 30MB RSS; stream large HTML bodies; respect 10:1 decompression ratio limit.

---

## Release Procedures
Always follow [`git-release.md`](file:///Users/mac/Desktop/openissa/git-release.md). Releases require:
1. Version bump in `Cargo.toml`.
2. Clean `cargo test`, `cargo clippy`, and `cargo fmt`.
3. **Human manual review and explicit approval**.
4. Annotated Git tag (`git tag -a vX.Y.Z -m "Release vX.Y.Z"`).
