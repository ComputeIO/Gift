# AGENTS Instructions

**Leaf** is an AI agent framework in Rust with **pure CLI interfaces**.

## Project Origin & Transformation

Leaf is a **fork of the block/goose project** with the following key transformations:

- **UI components removed**: All desktop/Electron UI code has been stripped
- **V8 dependencies removed**: No JavaScript/V8 integration
- **Naming changed**: All `goose`/`Goose` references renamed to `leaf`/`Leaf`
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

### Dependencies
- ❌ **NO V8/JavaScript**: No JavaScript execution or V8 integration
- ❌ **NO Web Frameworks**: No web servers for UI (except API endpoints for ACP)
- ✅ **Minimal Dependencies**: Keep binary size reasonable for CLI tool

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

### When Cherry-Picking from Goose

When adapting commits from the upstream Goose project:

1. **Skip UI commits**: Any commit touching `ui/desktop/` or `ui/text/` can be skipped
2. **Skip V8/code-mode**: Remove any code-mode or V8-related changes
3. **Rename imports**: Change `goose::` to `leaf::` throughout
4. **Update binary names**: Change `goose` to `leaf`, `goosed` to `leafd`
5. **Adapt provider changes**: Provider updates are usually safe to cherry-pick
6. **Review agent changes**: Core agent logic is usually applicable

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
"@block/goose-acp-server-linux-x64"
"@block/goose-acp-server-darwin-arm64"

// New (Leaf)
"@block/leaf-acp-server-linux-x64"
"@block/leaf-acp-server-darwin-arm64"
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
use goose::config::GooseMode;
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
