# Native Binary Packages

This directory contains the npm package scaffolding for distributing the
`leaf-acp-server` Rust binary as platform-specific npm packages.

## Packages

| Package | Platform |
|---------|----------|
| `@block/leaf-acp-server-darwin-arm64` | macOS Apple Silicon |
| `@block/leaf-acp-server-darwin-x64` | macOS Intel |
| `@block/leaf-acp-server-linux-arm64` | Linux ARM64 |
| `@block/leaf-acp-server-linux-x64` | Linux x64 |
| `@block/leaf-acp-server-win32-x64` | Windows x64 |

## Building

From the repository root:

```bash
# Build for all platforms (requires cross-compilation toolchains)
cargo build --release --target <target-triple> -p leaf-acp

# Example for Linux x64
cargo build --release --target x86_64-unknown-linux-gnu -p leaf-acp
```

The built binaries should be placed into `npm/leaf-acp-server-{platform}/bin/`.

## Publishing

```bash
# Publish all packages
for dir in npm/leaf-acp-server-*; do
  npm publish "$dir"
done
```

## Package Naming

All npm packages have been renamed from `goose-acp-server-*` to `leaf-acp-server-*`:
- Package names changed from `@block/goose-acp-server-*` to `@block/leaf-acp-server-*`
- Binary files changed from `bin/goose` to `bin/leaf`
- Repository URL changed from `block/goose` to `LeafAI/Leaf`
