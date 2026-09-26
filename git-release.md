# OpenISSA Git Release & Installer Deployment Guide

> **MANDATORY POLICY**: Every release of OpenISSA must be tied to a verified Git release tag (`vX.Y.Z`).  
> **CRITICAL RULE**: Before pushing any code, creating tags, or publishing releases, **ALWAYS pause and ask for human manual review and explicit approval**.

---

## 1. Release Philosophy & Versioning Scheme

OpenISSA strictly follows [Semantic Versioning 2.0.0](https://semver.org/):
* **MAJOR (`vX.0.0`)**: Incompatible API or MCP tool schema changes, breaking configuration changes.
* **MINOR (`v0.X.0`)**: New backwards-compatible capabilities (e.g. adding a new MCP tool or search provider).
* **PATCH (`v0.0.X`)**: Backwards-compatible bug fixes, performance improvements, and security patches.

**Tag Format**: Always prefix version tags with `v` (e.g. `v0.1.0`, `v0.1.1`, `v0.2.0`). Never use plain numbers like `0.1.0`.

---

## 2. Pre-Release Checklist (The "Zero-Panic" Gate)

Every release candidate must pass this checklist locally before requesting human sign-off:

```bash
# 1. Format check (must return 0 exit code with no diffs)
cargo fmt --all -- --check

# 2. Strict linter check (zero warnings permitted)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Complete test suite execution
cargo test --workspace --all-features

# 4. Security audit on dependencies
cargo audit

# 5. Build release binary locally and test basic execution
cargo build --release --bin openissa
./target/release/openissa --version
./target/release/openissa doctor
```

---

## 3. Mandatory Human Review Gate

Before running any `git push` or `gh release` commands, the agent or developer MUST stop and present the following summary to the user:

```text
==============================================================================
                    MANDATORY HUMAN REVIEW CHECKPOINT
==============================================================================
Target Tag:        v0.1.0
Target Branch:     main
Commit Hash:       abcdef1234567890
Changes Summary:   - Federated search engine (DuckDuckGo, HackerNews, Wikipedia, GitHub, Brave, Tavily)
                   - Streaming HTML-to-Markdown fetcher (lol-html) & SQLite WAL cache
                   - Tier 4 Chromium CDP headless fallback with virtual time budget
                   - Web Lab declarative testing with SSRF firewall (DNS pinning, private IP blocking)
                   - Autonomous Discovery Engine (/llms.txt, recursive sitemaps, spider, path filtering)
                   - SQLite FTS5 BM25 full-text search index
                   - Dual MCP transport: Stdio JSON-RPC 2.0 + HTTP/1.1 SSE daemon (openissa serve)
                   - Auto-installer for Cursor (.cursor/mcp.json) and OpenCode (opencode.json)
Breaking Changes:  None
Test Status:       All 21 unit & E2E integration tests passing (100% green)
Security Status:   SSRF firewall active; zero cargo audit vulnerabilities
==============================================================================
Do you approve pushing this commit and publishing release tag v0.1.0? [y/N]
==============================================================================
```

**Never proceed with `git push` or tagging without explicit human confirmation.**

---

## 4. Release Execution Workflow

Once the human reviewer has approved the release, execute the following sequence:

### Step 1: Update Version Numbers
Update the version string across all `Cargo.toml` files in the workspace:
* Root `Cargo.toml` (`[workspace.package] version = "0.1.0"`)
* Individual crate manifests if independently versioned.

### Step 2: Commit and Tag
```bash
# Stage modified manifests
git add Cargo.toml Cargo.lock

# Commit with standard release message
git commit -m "chore(release): prepare v0.1.0"

# Create an annotated and signed Git tag
git tag -a v0.1.0 -m "Release v0.1.0"
```

### Step 3: Push Commit and Tag (Only after human approval)
```bash
# Push main branch
git push origin main

# Push the release tag
git push origin v0.1.0
```

---

## 5. Artifact Compilation & Cross-Target Matrix

The automated GitHub Actions release workflow builds pre-compiled, optimized native binaries for the following targets:

| Target Triple | OS / Architecture | Artifact Name |
| :--- | :--- | :--- |
| `aarch64-apple-darwin` | macOS Apple Silicon (M1/M2/M3/M4) | `openissa-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| `x86_64-apple-darwin` | macOS Intel | `openissa-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| `x86_64-unknown-linux-gnu` | Linux x86_64 (glibc 2.31+) | `openissa-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| `x86_64-unknown-linux-musl` | Linux x86_64 (Fully static musl) | `openissa-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz` |
| `aarch64-unknown-linux-gnu` | Linux ARM64 | `openissa-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz` |
| `x86_64-pc-windows-msvc` | Windows x86_64 | `openissa-vX.Y.Z-x86_64-pc-windows-msvc.zip` |

### Cargo Release Profile Optimization
All release binaries are built with maximum optimization, stripped symbols, and LTO:
```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"
strip = true
```

---

## 6. Checksums & Verification

Every GitHub release must include an authoritative `SHA256SUMS.txt` file containing cryptographic hashes of all binary tarballs:

```bash
# Generate checksums on build host
sha256sum openissa-v*.tar.gz openissa-v*.zip > SHA256SUMS.txt

# Verify checksums
sha256sum -c SHA256SUMS.txt
```

---

## 7. Universal Installer Script (`install.sh`)

Users can install any release tag with a single command via the universal install script:

```bash
# Install the latest release tag:
curl -fsSL https://raw.githubusercontent.com/tayyabmughal676/openissa/main/install.sh | bash

# Or install a specific pinned release tag:
curl -fsSL https://raw.githubusercontent.com/tayyabmughal676/openissa/main/install.sh | bash -s -- --version v0.1.0
```

The installer:
1. Detects OS and CPU architecture (`uname -s`, `uname -m`).
2. Fetches the matching release tarball and `SHA256SUMS.txt` from GitHub releases.
3. Verifies the SHA-256 hash before unpacking.
4. Installs the standalone `openissa` binary into `~/.local/bin` (or `/usr/local/bin` if root).
5. Tests execution by running `openissa --version`.

---

## 8. Rollback & Emergency Hotfix Protocol

If a critical security flaw or regression is discovered in a release:
1. **Never delete a published Git tag**: Deleting tags breaks reproducibility and package caches.
2. **Issue an immediate patch tag**:
   * Create a hotfix branch from the tag: `git checkout -b hotfix/v0.1.1 v0.1.0`.
   * Apply the minimal fix, write regression tests, and verify.
   * Obtain human approval.
   * Tag `v0.1.1` and push.
3. **Mark the broken release as deprecated/yanked** on GitHub Releases with a prominent warning directing users to the hotfix tag.
