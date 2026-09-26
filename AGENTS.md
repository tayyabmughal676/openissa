# OpenISSA Agent Guidelines (AGENTS.md)

> **CRITICAL DIRECTIVE FOR ALL AI AGENTS**:  
> **BEFORE PUSHING CODE TO REMOTE REPOSITORIES OR CREATING GIT RELEASE TAGS, YOU MUST ALWAYS PAUSE AND REQUEST HUMAN MANUAL REVIEW AND EXPLICIT APPROVAL.**  
> Never perform a `git push`, publish a release, or mutate remote state autonomously.

---

## 1. Project Mission & Identity
OpenISSA is a **local-first, high-efficiency internet research engine and MCP server written in native Rust**.
* It is **NOT** a desktop app or chat GUI (no Sparrow / Tauri in current scope).
* It is **NOT** a dumb search wrapper; it is an active investigative engine that searches, fetches via the Retrieval Ladder, inspects live APIs in the Web Lab, and verifies claims in an Evidence Graph.
* **Core Value Proposition**: Saves **80%+ of an agent's context window and tokens** by absorbing web plumbing (caching, HTML stripping, browser fallback, deduplication) into a native local daemon.

---

## 2. Core Architectural Invariants (Do Not Violate)

1. **Single Rust Binary**: The entire system compiles to a single, statically linked binary (`openissa`). No mandatory external runtime dependencies (no Python, no Node daemon, no Docker).
2. **The Retrieval Ladder**: Always use the least expensive access method:
   ```text
   Cache (0ms) → Tantivy Index (5ms) → HTTP/2 Stream (150ms) → Lazy Chromium (CDP) → Human Handoff
   ```
   * Never launch headless Chromium for a static page.
   * If a CAPTCHA or Cloudflare challenge is encountered, never attempt circumvention; emit an MCP notification and trigger human handoff.
3. **Web Lab Safety & SSRF Firewall**:
   * All network requests originating from the agent or Web Lab must pass through the `NetworkFirewall`.
   * Block private IP ranges (`127.0.0.0/8`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`) and cloud metadata (`169.254.169.254`).
   * Prevent DNS rebinding by resolving IPs first and pinning the TCP socket.
4. **Content Sanitization**:
   * Treat all external web content as untrusted input.
   * Strip scripts, styles, and advertising at the byte-stream level using `lol-html`.
   * Wrap web evidence in `<untrusted_web_evidence>` quarantine blocks.
5. **No LaTeX Math in Documentation**:
   * NEVER write LaTeX math syntax like `$$\text{...}$$` or `$\rightarrow$` in Markdown files.
   * Use clean Unicode arrows (`→`) or standard Markdown text/code blocks.

---

## 3. Rust Coding Standards

* **Rust Edition**: 2021 edition.
* **Error Handling**:
  * Library crates (`openissa-core`, `openissa-fetcher`, etc.) MUST use `thiserror` for typed domain errors.
  * Application binaries (`openissa-cli`, `openissa-mcp`) may use `anyhow` for top-level error reporting.
  * **Zero Unwraps**: Never call `.unwrap()` in production library code. Use `?`, `match`, or `.expect("Descriptive reason why this invariant holds")`.
* **Resource Bounds**:
  * Idle memory must remain below 30MB RSS.
  * Streams must be bounded (max 5MB body size, max 10:1 decompression ratio).
  * Always use async I/O with `tokio`. Avoid blocking threads on network I/O.
* **Linting & Formatting**:
  * Code MUST pass `cargo fmt --all -- --check`.
  * Code MUST pass `cargo clippy --all-targets --all-features -- -D warnings` with zero warnings.

---

## 4. Git & Commit Guidelines

* Follow the **Conventional Commits** specification:
  * `feat(fetcher): add streaming brotli decompression`
  * `fix(mcp): resolve stdio buffer deadlock on large responses`
  * `docs(readme): clarify retrieval ladder tiers`
  * `test(security): add SSRF DNS rebinding test suite`
* **Release Workflow**: Refer to [`git-release.md`](file:///Users/mac/Desktop/openissa/git-release.md). Releases must use annotated tags formatted as `vX.Y.Z`.
* **Human Review Checkpoint**:
  Before any `git push` or release publication, output a clean diff summary and ask:
  > *"I have prepared these changes and all tests are passing. Please review the changes above. Do you approve pushing this to the remote repository?"*

---

## 5. Development Command Reference

```bash
# Check compilation across all crates
cargo check --workspace --all-targets

# Run the full test suite (21 unit and integration tests)
cargo test --workspace --all-targets

# Run security and linter checks
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# Build the optimized release binary
cargo build --release --bin openissa

# Test the local MCP server over stdio
./target/release/openissa mcp

# Test the remote MCP Server-Sent Events (SSE) transport daemon
./target/release/openissa serve 8080

# Auto-configure MCP servers in Cursor (.cursor/mcp.json) and OpenCode (opencode.json)
./target/release/openissa install-mcp

# Test headless Chromium DOM extraction
./target/release/openissa fetch -b https://react.dev

# Run multi-query autonomous research DAG
./target/release/openissa research "Rust 2024 edition async traits" 5

# Run system diagnostic and storage health check
./target/release/openissa doctor
```
