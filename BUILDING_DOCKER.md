# Building and Running leaf with Docker

This guide covers building Docker images for leaf CLI for production use, CI/CD pipelines, and local development.

## Quick Start

### Using Pre-built Images

The easiest way to use leaf with Docker is to pull the pre-built image from GitHub Container Registry:

```bash
# Pull the latest image
docker pull ghcr.io/block/leaf:latest

# Run leaf CLI
docker run --rm ghcr.io/block/leaf:latest --version

# Run with LLM configuration
docker run --rm \
  -e LEAF_PROVIDER=openai \
  -e LEAF_MODEL=gpt-4o \
  -e OPENAI_API_KEY=$OPENAI_API_KEY \
  ghcr.io/block/leaf:latest run -t "Hello, world!"
```

## Building from Source

### Prerequisites

- Docker 20.10 or later
- Docker Buildx (for multi-platform builds)
- Git

### Build the Image

1. Clone the repository:
```bash
git clone https://github_com_block_leaf_placeholder.git
cd leaf
```

2. Build the Docker image:
```bash
docker build -t leaf:local .
```

The build process:
- Uses a multi-stage build to minimize final image size
- Compiles with optimizations (LTO, stripping, size optimization)
- Results in a ~340MB image containing the `leaf` CLI binary

### Build Options

For a development build with debug symbols:
```bash
docker build --build-arg CARGO_PROFILE_RELEASE_STRIP=false -t leaf:dev .
```

For multi-platform builds:
```bash
docker buildx build --platform linux/amd64,linux/arm64 -t leaf:multi .
```

## Running leaf in Docker

### CLI Mode

Basic usage:
```bash
# Show help
docker run --rm leaf:local --help

# Run a command
docker run --rm \
  -e LEAF_PROVIDER=openai \
  -e LEAF_MODEL=gpt-4o \
  -e OPENAI_API_KEY=$OPENAI_API_KEY \
  leaf:local run -t "Explain Docker containers"
```

With volume mounts for file access:
```bash
docker run --rm \
  -v $(pwd):/workspace \
  -w /workspace \
  -e LEAF_PROVIDER=openai \
  -e LEAF_MODEL=gpt-4o \
  -e OPENAI_API_KEY=$OPENAI_API_KEY \
  leaf:local run -t "Analyze the code in this directory"
```

Interactive session mode with Databricks:
```bash
docker run -it --rm \
  -e LEAF_PROVIDER=databricks \
  -e LEAF_MODEL=databricks-dbrx-instruct \
  -e DATABRICKS_HOST="$DATABRICKS_HOST" \
  -e DATABRICKS_TOKEN="$DATABRICKS_TOKEN" \
  leaf:local session
```



### Docker Compose

Create a `docker-compose.yml`:

```yaml
version: '3.8'

services:
  leaf:
    image: ghcr.io/block/leaf:latest
    environment:
      - LEAF_PROVIDER=${LEAF_PROVIDER:-openai}
      - LEAF_MODEL=${LEAF_MODEL:-gpt-4o}
      - OPENAI_API_KEY=${OPENAI_API_KEY}
    volumes:
      - ./workspace:/workspace
      - leaf-config:/home/leaf/.config/leaf
    working_dir: /workspace
    stdin_open: true
    tty: true

volumes:
  leaf-config:
```

Run with:
```bash
docker-compose run --rm leaf session
```

## Configuration

### Environment Variables

The Docker image accepts all standard leaf environment variables:

- `LEAF_PROVIDER`: LLM provider (openai, anthropic, google, etc.)
- `LEAF_MODEL`: Model to use (gpt-4o, claude-sonnet-4, etc.)
- Provider-specific API keys (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.)

### Persistent Configuration

Mount the configuration directory to persist settings:
```bash
docker run --rm \
  -v ~/.config/leaf:/home/leaf/.config/leaf \
  leaf:local configure
```

### Installing Additional Tools

The image runs as a non-root user by default. To install additional packages:

```bash
# Run as root to install packages
docker run --rm \
  -u root \
  --entrypoint bash \
  leaf:local \
  -c "apt-get update && apt-get install -y vim && leaf --version"

# Or create a custom Dockerfile
FROM ghcr.io/block/leaf:latest
USER root
RUN apt-get update && apt-get install -y \
    vim \
    tmux \
    && rm -rf /var/lib/apt/lists/*
USER leaf
```

## CI/CD Integration

### GitHub Actions

```yaml
jobs:
  analyze:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/block/leaf:latest
      env:
        LEAF_PROVIDER: openai
        LEAF_MODEL: gpt-4o
        OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
    steps:
      - uses: actions/checkout@v4
      - name: Run leaf analysis
        run: |
          leaf run -t "Review this codebase for security issues"
```

### GitLab CI

```yaml
analyze:
  image: ghcr.io/block/leaf:latest
  variables:
    LEAF_PROVIDER: openai
    LEAF_MODEL: gpt-4o
  script:
    - leaf run -t "Generate documentation for this project"
```

## Image Details

### Size and Optimization

- **Base image**: Debian Bookworm Slim (minimal runtime dependencies)
- **Final size**: ~340MB
- **Optimizations**: Link-Time Optimization (LTO), binary stripping, size optimization
- **Binary included**: `/usr/local/bin/leaf` (32MB)

### Security

- Runs as non-root user `leaf` (UID 1000)
- Minimal attack surface with only essential runtime dependencies
- Regular security updates via automated builds

### Included Tools

The image includes essential tools for leaf operation:
- `git` - Version control operations
- `curl` - HTTP requests
- `ca-certificates` - SSL/TLS support
- Basic shell utilities

## Troubleshooting

### Permission Issues

If you encounter permission errors when mounting volumes:
```bash
# Ensure the mounted directory is accessible
docker run --rm \
  -v $(pwd):/workspace \
  -u $(id -u):$(id -g) \
  leaf:local run -t "List files"
```

### API Key Issues

If API keys aren't being recognized:
1. Ensure environment variables are properly set
2. Check that quotes are handled correctly in your shell
3. Use `docker run --env-file .env` for multiple environment variables

### Network Issues

For accessing local services from within the container:
```bash
# Use host network mode
docker run --rm --network host leaf:local
```

## Advanced Usage

### Custom Entrypoint

Override the default entrypoint for debugging:
```bash
docker run --rm -it --entrypoint bash leaf:local
```

### Resource Limits

Set memory and CPU limits:
```bash
docker run --rm \
  --memory="2g" \
  --cpus="2" \
  leaf:local
```

### Multi-stage Development

For development with hot reload:
```bash
# Mount source code
docker run --rm \
  -v $(pwd):/usr/src/leaf \
  -w /usr/src/leaf \
  rust:1.82-bookworm \
  cargo watch -x run
```

## Building for Production

For production deployments:

1. Use specific image tags instead of `latest`
2. Use secrets management for API keys
3. Set up logging and monitoring
4. Configure resource limits and auto-scaling

Example production Dockerfile:
```dockerfile
FROM ghcr.io/block/leaf:v1.6.0
# Add any additional tools needed for your use case
USER root
RUN apt-get update && apt-get install -y your-tools && rm -rf /var/lib/apt/lists/*
USER leaf
```

## Contributing

When contributing Docker-related changes:

1. Test builds on multiple platforms (amd64, arm64)
2. Verify image size remains reasonable
3. Update this documentation
4. Consider CI/CD implications
5. Test with various LLM providers

## Related Documentation

- [leaf in Docker Tutorial](documentation/docs/tutorials/leaf-in-docker.md) - Step-by-step tutorial
- [Installation Guide](https://block.github.io/leaf/docs/getting-started/installation) - All installation methods
- [Configuration Guide](https://block.github.io/leaf/docs/guides/config-files) - Detailed configuration options
