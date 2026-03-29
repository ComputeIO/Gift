# Native Binary Packages

This directory contains the npm package scaffolding for distributing the
`leaf` CLI binary as platform-specific npm packages.

## Packages

| Package | Platform |
|---------|----------|
| `@block/leaf-binary-darwin-arm64` | macOS Apple Silicon |
| `@block/leaf-binary-darwin-x64` | macOS Intel |
| `@block/leaf-binary-linux-arm64` | Linux ARM64 |
| `@block/leaf-binary-linux-x64` | Linux x64 |
| `@block/leaf-binary-win32-x64` | Windows x64 |

## Building

From the repository root:

```bash
# Build for all platforms (requires cross-compilation toolchains)
cargo build --release --target <target-triple> -p leaf-cli

# Example for Linux x64
cargo build --release --target x86_64-unknown-linux-gnu -p leaf-cli
```

The built binaries should be placed into `npm/leaf-binary-{platform}/bin/`.

## Publishing

```bash
# Publish all packages
for dir in npm/leaf-binary-*; do
  npm publish "$dir"
done
```

## Package Naming

All npm packages have been renamed from `goose-acp-server-*` to `leaf-binary-*`:
- Package names changed from `@block/goose-acp-server-*` to `@block/leaf-binary-*`
- Binary files changed from `bin/goose` to `bin/leaf`
- Repository URL changed from `block/goose` to `LeafAI/Leaf`
- The standalone `leaf-acp-server` binary was removed; use `leaf acp` subcommand instead
