<div align="center">

# Leaf

_a pure CLI AI agent forked from Leaf, focused on automation through the command line_

<p align="center">
  <a href="https://opensource.org/licenses/Apache-2.0"
    ><img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg"></a>
  <a href="https://github.com/LeafAI/Leaf/actions/workflows/ci.yml"
     ><img src="https://img.shields.io/github/actions/workflow/status/LeafAI/Leaf/ci.yml?branch=cli" alt="CI"></a>
</p>
</div>

Leaf is a **pure CLI AI agent** forked from the [Leaf](https://github_com_block_leaf_placeholder) project by Block. While Leaf provides both desktop and CLI interfaces, Leaf focuses exclusively on command-line automation for developers who prefer terminal-based workflows.

## Key Differences from Leaf

- **Pure CLI**: No desktop UI or Electron components - just a fast, terminal-based experience
- **Lightweight**: Removed V8 dependencies and UI components for a smaller footprint
- **Agent Protocol (ACP)**: Full support for Agent Client Protocol for multi-agent workflows
- **MCP Integration**: Seamless integration with Model Context Protocol (MCP) servers
- **Multi-Model**: Works with any LLM (OpenAI, Anthropic, local models, etc.)

## What Leaf Can Do

Leaf is your on-machine AI agent, capable of automating complex development tasks:

- **Code Generation**: Build projects from scratch, write and refactor code
- **Debugging**: Analyze errors and fix issues autonomously
- **Workflow Automation**: Execute complex multi-step engineering pipelines
- **External Integrations**: Interact with APIs and external services via MCP
- **Orchestration**: Delegate tasks to subagents with independent contexts

## Quick Start

### Installation

```bash
# Build from source
cargo build --release --package leaf-cli

# The binary will be at:
./target/release/leaf
```

### Setup

```bash
# Configure your provider
leaf configure

# Start an interactive session
leaf session

# Or run a recipe
leaf run --recipe my-recipe.yaml
```

### Example Usage

```bash
# Ask Leaf to help with a task
leaf session

# Run with a specific provider
leaf session --provider openai

# Use recipes for repeatable workflows
leaf run --recipe deploy.yaml
```

## Architecture

```
crates/
├── leaf              # Core agent logic
├── leaf-acp          # Agent Client Protocol implementation
├── leaf-cli          # Command-line interface
├── leaf-mcp          # Model Context Protocol extensions
└── leaf-server       # ACP server (leafd)
```

## Project Status

Leaf is a community fork focused on CLI-only workflows. It maintains compatibility with:
- **MCP Servers**: All standard Model Context Protocol servers
- **Recipes**: YAML-based automation workflows
- **Extensions**: Dynamic tool loading via MCP

## Documentation

- [Getting Started](#getting-started) - Installation and first steps
- [Configuration](#configuration) - Setting up providers and extensions
- [Recipes](#recipes) - Automation workflows
- [MCP Extensions](#mcp-extensions) - Available tools and integrations

## Development

```bash
# Setup
source bin/activate-hermit

# Build
cargo build --release

# Test
cargo test --package leaf-cli

# Lint
cargo clippy --all-targets -- -D warnings
cargo fmt
```

## Contributing

We welcome contributions! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a PR

## License

Apache License 2.0 - see [LICENSE](LICENSE) file.

## Acknowledgments

Leaf is a fork of [Leaf](https://github_com_block_leaf_placeholder) by Block. We're grateful for the solid foundation they built.

---

<div align="center">

**Pure CLI. Maximum Control. No UI Overhead.**

</div>
