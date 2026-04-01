# AGENTS Instructions

**Leaf** is an AI agent framework in Rust with **pure CLI interfaces**.

## Project Origin & Transformation

Leaf is a **fork of the block/goose project** with the following key transformations:

- **UI components removed**: All desktop/Electron UI code has been stripped
- **V8 dependencies removed**: No JavaScript/V8 integration
- **Naming changed**: All `goose`/`Goose`/`GOOSE` references renamed to `leaf`/`Leaf`/`LEAF`
- **CLI-only focus**: Terminal-based interface only, no GUI

### What Was Removed
- `ui/desktop/` - Desktop application (Electron-based)
- `ui/text/` - Text UI components
- `vendor/v8/` - V8 JavaScript engine
- `code-mode` feature - Code execution via V8
- Desktop packaging (`.deb`, `.rpm`, `.flatpak`)

### What Was Kept
- Core agent logic (`crates/leaf`)
- CLI interface (`crates/leaf-cli`)
- ACP protocol (`crates/leaf-acp`)
- MCP extensions (`crates/leaf-mcp`)
- Server backend (`crates/leaf-server` → `leafd`)

## Branches

- **`cli`** — Current active development branch (main/master equivalent for CLI-only Leaf)
- **`main`** — Upstream block/goose reference; do not use directly

## Setup
```bash
source bin/activate-hermit
cargo build
```

## Commands

### Build
```bash
cargo build                   # debug
cargo build --release         # release  
just release-binary           # release + openapi
```

### Test
```bash
cargo test                   # all tests
cargo test -p leaf          # specific crate
cargo test --package leaf --test mcp_integration_test
just record-mcp-tests        # record MCP
```

### Lint/Format
```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
```

### Git
```bash
git commit -s                # required for DCO sign-off
```

## Structure
```
crates/
├── leaf              # core logic
├── leaf-acp          # Agent Client Protocol
├── leaf-acp-macros   # ACP proc macros
├── leaf-cli          # CLI entry
├── leaf-server       # backend (binary: leafd)
├── leaf-mcp          # MCP extensions
├── leaf-test         # test utilities
└── leaf-test-support # test helpers

evals/open-model-gym/  # benchmarking / evals
```

## Development Loop
```bash
# 1. source bin/activate-hermit
# 2. Make changes
# 3. cargo fmt
```

### Run these only if the user has asked you to build/test your changes:
```
# 1. cargo build
# 2. cargo test -p <crate>
# 3. cargo clippy --all-targets -- -D warnings
# 4. [if server] just generate-openapi
```

## Rules

Test: Prefer tests/ folder, e.g. crates/leaf/tests/
Test: When adding features, update leaf-self-test.yaml, rebuild, then run `leaf run --recipe leaf-self-test.yaml` to validate
Error: Use anyhow::Result
Provider: Implement Provider trait see providers/base.rs
MCP: Extensions in crates/leaf-mcp/
Server: Changes need just generate-openapi

## Code Quality

Comments: Write self-documenting code - prefer clear names over comments
Comments: Never add comments that restate what code does
Comments: Only comment for complex algorithms, non-obvious business logic, or "why" not "what"
Simplicity: Don't make things optional that don't need to be - the compiler will enforce
Simplicity: Booleans should default to false, not be optional
Errors: Don't add error context that doesn't add useful information (e.g., `.context("Failed to X")` when error already says it failed)
Simplicity: Avoid overly defensive code - trust Rust's type system
Logging: Clean up existing logs, don't add more unless for errors or security events

## Never

Never: Edit Cargo.toml use cargo add
Never: Skip cargo fmt
Never: Merge without running clippy
Never: Comment self-evident operations (`// Initialize`, `// Return result`), getters/setters, constructors, or standard Rust idioms

## Entry Points
- CLI: crates/leaf-cli/src/main.rs
- Server: crates/leaf-server/src/main.rs
- Agent: crates/leaf/src/agents/agent.rs

## CLI-Only Constraints

When contributing to Leaf, keep these CLI-only constraints in mind:

### UI Components
- ❌ **NO Desktop UI**: No React, Electron, or web-based components
- ❌ **NO GUI Elements**: No windows, dialogs, or visual interfaces
- ✅ **Terminal Only**: All interaction through command line and TUI (Terminal UI)

### Naming rules
- X  **NO GOOSE/Goose/goose**: All GOOSE/Goose/goose must be transformed to LEAF/Leaf/leaf directly

### Upstream block/goose
- ✅ **Upstream Synchronization**: An issue will be created to request merge upstream commits that can be fixed in PRs
- ✅ **Commits Processing**: Each upstream commit can be adopted or ignored(if not CLI related), PR MUST commit each adopted upstream commit independently.
- ✅ **Functional Focus**: NEVER cherry-pick any commit for simplicity, each upstream commit MUST be evaluated and merged carefully line by line

### Upstream Sync Workflow

When syncing from upstream block/goose:

1. **Create tracking issue** with commit range to analyze

    > ⚠️ **IMPORTANT**: Always create the issue on **LeafAI/Leaf** (origin), NOT on block/goose (upstream).
    > Verify the repo URL before creating: `git remote -v` shows origin → LeafAI/Leaf, upstream → block/goose.
2. **List all commits**: `git log upstream/main --oneline 928f4ac46..59a96c986`
3. **Analyze each commit**:
   - Check if CLI-relevant (affects `crates/leaf`, `crates/leaf-cli`, `crates/leaf-server`, `crates/leaf-acp`, `crates/leaf-mcp`, `download_cli.sh`)
   - Skip UI-only, desktop-only, CI-only, docs-only, version bump commits
   - Mark as "Apply", "Skip", or "Review"
4. **Update issue with analysis** before making changes
5. **Apply commits individually** with 1:1 mapping to upstream commits
6. **Rename goose→leaf** in any adapted code (scripts, env vars, binary names)
7. **Ask user to review** each commit before proceeding to next

### When to Stop and Ask

Stop and ask user for guidance when:
- Upstream commit modifies files that have been significantly refactored in Leaf
- A simple rename would be insufficient (requires redesign)
- Changes affect multiple crates with complex dependencies
- There's ambiguity about whether a change is "CLI-relevant"
- A commit requires additional dependencies that don't exist in Leaf

### Commit Message Format (Upstream Adoptions)

When adopting upstream commits:

```
<type>: <brief description matching upstream>

Upstream: <full commit hash>
Backport of upstream commit that explains what was adopted and any
Leaf-specific changes made during adaptation.

Co-authored-by: Original Author <email>
```

Example:
```
fix: gemini acp without an unexpected terminal window

Upstream: 59a96c986fa4b75335f27e129f405f9164d2f0ed
Backport of upstream commit that adds configure_subprocess() call
when spawning ACP processes. Also renamed GOOSED_CERT_FINGERPRINT
to LEAFD_CERT_FINGERPRINT in tls.rs.

Co-authored-by: Lifei Zhou <lifei@squareup.com>
Co-authored-by: Rick Hadeed <rick@block.xyz>
```

### Upstream Sync Verification

After each commit:
- [ ] `cargo fmt` passes
- [ ] `cargo check -p leaf -p leaf-cli -p leaf-server` passes
- [ ] Clippy warnings addressed (note pre-existing issues separately)

For feature flag changes:
- [ ] Check with `rustls-tls` feature (default)
- [ ] Check with `native-tls` feature if applicable

### External Interface References That MUST Remain as "goose"

When renaming goose→leaf, certain references MUST be preserved because they represent **external interfaces** that Leaf must remain compatible with:

| Category | Examples | Why Must Be Kept |
|----------|----------|------------------|
| **Databricks model names** | `goose_o3_placeholder-mini`, `kgoose_gpt_placeholder-4o`, `headless-goose_o3_placeholder-mini` | Registered with Databricks API - renaming would cause API failures |
| **ACP protocol method prefixes** | `_goose_placeholder/session/get`, `_goose_placeholder/session/list`, `_goose_placeholder/acp-aware` | Wire protocol defined by upstream - clients/servers expect exact names |
| **Upstream binary name** | `goose_acp_server_placeholder` | Belongs to upstream project - cannot rename a binary that isn't ours |
| **GitHub repository references** | `github_com_block_goose_placeholder` | Upstream project's actual repository URL |

**How to identify MUST-KEEP references:**
```bash
# These patterns represent external interfaces - verify before renaming:
grep -r "goose_o3_placeholder\|goose_placeholder\|_goose_placeholder/\|goose_acp_server_placeholder\|github_com_block_goose_placeholder" --include="*.rs"
```

### Dependency Version Pinning

When adding dependencies, be aware that `cargo install` may resolve semver ranges to the latest version:

```toml
# ❌ BAD: Allows cargo install to pull 1.3.0
rmcp = { version = "1.2.0", features = ["auth"] }

# ✅ GOOD: Pins exact version - cargo install cannot go higher
rmcp = { version = "=1.2.0", features = ["auth"] }
```

### Patch Sections and Optional Features

The `[patch.crates-io]` section in `Cargo.toml` causes Cargo to fetch git repositories **even when the patched dependency is not used**. This significantly slows down `cargo install` for users who don't need the optional feature.

**Before adding `[patch.crates-io]`:**
- Verify the dependency is only used by optional features (e.g., `telemetry`)
- If only used by optional features, consider removing the patch and using the crates.io version
- The `[patch]` section is inherited workspace-wide - moves to feature-specific Cargo.toml if possible

### Dependencies
- ❌ **NO V8/JavaScript**: No JavaScript execution or V8 integration
- ❌ **NO Web Frameworks**: No web servers for UI (except API endpoints for ACP)
- ✅ **Minimal Dependencies**: Keep binary size reasonable for CLI tool
- ✅ **LLM API Oriented**: No cloud deployment/inference implementation is allowed, only keep LLM API integration 

### Testing
- ✅ **Unit Tests**: Standard Rust unit tests in `tests/` folders
- ✅ **Integration Tests**: CLI behavior testing via recipes
- ❌ **NO UI Tests**: No browser-based or GUI testing

### Release Artifacts
- ✅ **CLI Binaries**: tar.bz2 (Linux/macOS), zip (Windows)
- ✅ **Install Scripts**: Shell scripts for easy installation
- ❌ **NO Desktop Packages**: No .deb, .rpm, .flatpak, or app bundles

### Binary Names
- **CLI**: `leaf` (main CLI binary)
- **Server**: `leafd` (daemon/server binary)
- **Legacy**: All references to `goose` or `goosed` renamed to `leaf`/`leafd`

### TLS Feature Flags

Leaf server supports two TLS backends:
- `rustls-tls` (default) - uses aws-lc-rs crypto provider
- `native-tls` - uses platform OpenSSL (or LibreSSL/BoringSSL)

When adding TLS-related changes:
- Always gate with `#[cfg(feature = "rustls-tls")]` or `#[cfg(feature = "native-tls")]`
- Use feature-gated type aliases for TLS config:
  ```rust
  #[cfg(feature = "rustls-tls")]
  pub type TlsConfig = axum_server::tls_rustls::RustlsConfig;
  #[cfg(feature = "native-tls")]
  pub type TlsConfig = axum_server::tls_openssl::OpenSSLConfig;
  ```
- Add compile_error macros to ensure exactly one backend is enabled:
  ```rust
  #[cfg(not(any(feature = "rustls-tls", feature = "native-tls")))]
  compile_error!("At least one of `rustls-tls` or `native-tls` features must be enabled");
  #[cfg(all(feature = "rustls-tls", feature = "native-tls"))]
  compile_error!("Features `rustls-tls` and `native-tls` are mutually exclusive");
  ```

### Install Script (download_cli.sh)

The `download_cli.sh` script has its own naming conventions:

**Environment variables:**
- `LEAF_BIN_DIR` (not `GOOSE_BIN_DIR`)
- `LEAF_VERSION`, `LEAF_PROVIDER`, `LEAF_MODEL`
- `LEAFD_CERT_FINGERPRINT` for TLS cert fingerprint output

**Repository references:**
- `https://github.com/LeafAI/Leaf` (not `block/goose`)
- `LeafAI/Leaf` as the repo identifier

When adapting upstream shell script changes, always rename:
- `GOOSE_BIN_DIR` → `LEAF_BIN_DIR`
- `GOOSED_CERT_FINGERPRINT` → `LEAFD_CERT_FINGERPRINT`
- `block/goose` → `LeafAI/Leaf`
- `goose` binary → `leaf` binary
- `goose.exe` → `leaf.exe`

### When Cherry-Picking from Leaf

When adapting commits from the upstream Leaf project:

1. **Skip UI commits**: Any commit touching `ui/desktop/` or `ui/text/` can be skipped
2. **Skip V8/code-mode**: Remove any code-mode or V8-related changes
3. **Rename imports**: Change `goose::` to `leaf::` throughout
4. **Update binary names**: Change `goose` to `leaf`, `goosed` to `leafd`
5. **Adapt provider changes**: Provider updates are usually safe to cherry-pick
6. **Review agent changes**: Core agent logic is usually applicable

### File and Folder Naming Constraints

**CRITICAL: NO files or folders should contain "goose" in their names.**

❌ **NEVER allow files/directories with "goose" in the name:**
- Directories like: `goose_apps/`, `goose_tools/`, `goose_data/`
- Files like: `goose_config.rs`, `goose_utils.py`, `.goosehints`
- Configuration files: `goose-self-test.yaml`, `goose-config.json`
- Scripts: `goose-db-helper.sh`, `goose_setup.py`

✅ **ALWAYS use "leaf" instead:**
- Directories: `leaf_apps/`, `leaf_tools/`, `leaf_data/`
- Files: `leaf_config.rs`, `leaf_utils.py`, `.leafhints`
- Configuration files: `leaf-self-test.yaml`, `leaf-config.json`
- Scripts: `leaf-db-helper.sh`, `leaf_setup.py`

**When you find files/folders with "goose" in the name:**
1. **Rename immediately** - Use `mv oldname newname` or `git mv`, commit in the same step
2. **Update all references** - Search for code references to the old name
3. **Update imports** - Change `crate::old_name` to `crate::new_name`
4. **Update include_str!** - Change `include_str!("../old_name/...")` to `include_str!("../new_name/...")`
5. **Test compilation** - Run `cargo check` to ensure nothing broke

**Common patterns to watch for:**
```bash
# Find files with leaf in the name
find . -type f -name "*goose*" | grep -v ".git"
find . -type d -name "*goose*" | grep -v ".git"

# Common files that often have goose names
*.snap files: goose__agents__*.snap → leaf__agents__*.snap
Docs: goose_doc_guide.md → leaf_doc_guide.md
Scripts: goose-*.sh → leaf-*.sh
Config: .goose* → .leaf*, goose-*.yaml → leaf-*.yaml
```

### Package and File Naming Conventions

All package names and references must be changed from `goose` to `leaf`:

**Rust Crate Names:**
```toml
# Old (Goose)
crates/goose
crates/goose-cli
crates/goose-server
crates/goose-acp

# New (Leaf)
crates/leaf
crates/leaf-cli
crates/leaf-server
crates/leaf-acp
```

**NPM Packages (npm/ directory):**
```json
// Old (Goose)
"@block/goose-binary-linux-x64"
"@block/goose-binary-darwin-arm64"

// New (Leaf)
"@leafai/leaf-binary-linux-x64"
"@leafai/leaf-binary-darwin-x64"
"@leafai/leaf-binary-linux-arm64"
"@leafai/leaf-binary-darwin-arm64"
"@leafai/leaf-binary-win32-x64"
```

**Binary Names:**
```
# Old (Goose)
bin/goose           # CLI binary
bin/goose.exe       # Windows CLI
goosed              # Server binary

# New (Leaf)
bin/leaf            # CLI binary
bin/leaf.exe        # Windows CLI
leafd               # Server binary
```

**Configuration Files:**
```yaml
# Old (Goose)
.goose/
.goose/config.yaml
goose-self-test.yaml

# New (Leaf)
.leaf/
.leaf/config.yaml
leaf-self-test.yaml
```

**Import Statements:**
```rust
// Old (Goose)
use goose::config::LeafMode;
use goose::providers::create;
use goose::agents::Agent;

// New (Leaf)
use leaf::config::LeafMode;
use leaf::providers::create;
use leaf::agents::Agent;
```

**GitHub Repository References:**
```
# Old (Goose)
https://github.com/block/goose

# New (Leaf)
https://github.com/LeafAI/Leaf
```

## OpenCode Provider System (Reference from upstream)

The OpenCode project uses a sophisticated provider system from models.dev. Understanding this is crucial for proper provider implementation.

### Provider vs Model npm (Engine)

**Provider npm** (`provider.npm`): The SDK/format used by the provider
- `@ai-sdk/openai-compatible` → OpenAI-compatible format (tool_calls with string arguments)
- `@ai-sdk/anthropic` → Anthropic format (tool_use with object input)
- Other npm packages indicate specific provider SDKs (e.g., `@ai-sdk/google-vertex/anthropic`)

**Model override** (`model.provider.npm`): Can override the provider's npm for specific models
- If set, this takes precedence over the provider's npm
- Used when a model expects a different format than the provider's default

### npm Resolution Order
```typescript
model.api.npm = model.provider?.npm ?? provider.npm ?? "@ai-sdk/openai-compatible"
```

### Request Flow in OpenCode Gateway

1. Request arrives at endpoint (e.g., `/zen/go/v1/messages`)
2. Endpoint has a default format (e.g., `format: "anthropic"`)
3. Handler looks up model and selects provider from model's provider list
4. Provider determines actual backend format
5. If endpoint format ≠ provider format, handler converts the request

### Format Conversion

OpenCode's handler uses `createBodyConverter` to transform requests:
- `fromAnthropicRequest`: Converts Anthropic format to CommonRequest
- `toOaCompatibleRequest`: Converts CommonRequest to OpenAI-compatible format

**Key difference in tool call format:**

OpenAI format (tool_calls with string arguments):
```json
{
  "tool_calls": [{
    "id": "call_123",
    "type": "function", 
    "function": {
      "name": "get_weather",
      "arguments": "{\"location\":\"Boston\"}"
    }
  }]
}
```

Anthropic format (tool_use with object input):
```json
{
  "content": [{
    "type": "tool_use",
    "id": "toolu_123",
    "name": "get_weather", 
    "input": {"location": "Boston"}
  }]
}
```

### Zen/go Endpoints

The `/zen/go/v1/*` endpoints are configured with `format: "anthropic"` but the handler converts requests to match the selected provider's format.

### Common Issues

**Error: "Input should be a valid dictionary"**

This error occurs when the API receives a tool_use.input that is not an object. Possible causes:
1. `tool_call.arguments` stored as string instead of object
2. Format conversion not handling the arguments properly
3. Provider selection mismatch for models with provider override

The fix requires ensuring that when using Anthropic format, `tool_use.input` is always a proper JSON object, not a string.

### Model Listing Behavior

OpenCode **does NOT provide a `/models` endpoint** on provider backends. Model data comes exclusively from `models.dev/api.json`.

#### How OpenCode Gets Models

1. **Primary source**: `https://models.dev/api.json` (or bundled snapshot at build time)
2. **Priority chain** (in `packages/opencode/src/provider/models.ts`):
   - Cache file (`~/.cache/opencode/models.json`)
   - Bundled snapshot (compiled at build time)
   - Remote fetch from `models.dev/api.json`
3. **No `/models` API calls to provider backends**

#### OpenCode Server Endpoints

| Endpoint | Purpose |
|----------|---------|
| `GET /provider` | Lists providers + models (from models.dev data) |
| No `/models` on provider backends | Models are embedded in provider response |

#### How Leaf Handles This

Leaf's `opencode.rs` fetches models from `models.dev/api.json` and registers them as preloaded models. In `OpenAiProvider::fetch_supported_models()`:

1. **For OpenCode providers** (name starts with `opencode-`): Skip `/models` API call, return preloaded models directly
2. **For other providers**: Try `/models` API first, fall back to preloaded models

This avoids unnecessary 404 errors when Leaf tries to call a non-existent `/models` endpoint on OpenCode provider backends.

#### Adding New OpenCode Providers

When adding providers from OpenCode's models.dev:

1. Provider data is automatically fetched from `models.dev/api.json` in `opencode.rs`
2. Models are grouped by engine (OpenAI or Anthropic) based on the `npm` field
3. Provider names follow the pattern: `opencode-{provider_id}-{engine}`
4. The `base_url` comes from the provider's `api` field in models.dev

#### Key Files

- `crates/leaf/src/providers/opencode.rs` - Fetches and registers OpenCode providers
- `crates/leaf/src/providers/openai.rs:399-420` - Skips `/models` for OpenCode providers
- `crates/leaf/src/config/declarative_providers.rs:422-428` - Maps `ProviderEngine::OpenAI` to `OpenAiProvider`
