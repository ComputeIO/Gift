# Justfile

# list all tasks
default:
  @just --list

# Run all style checks and formatting (precommit validation)
check-everything:
    @echo "🔧 RUNNING ALL STYLE CHECKS..."
    @echo "  → Formatting Rust code..."
    cargo fmt --all
    @echo "  → Running clippy linting..."
    cargo clippy --all-targets -- -D warnings
    @echo "  → Checking for banned TLS crates..."
    ./scripts/check-no-native-tls.sh
    @echo ""
    @echo "✅ All style checks passed!"

# Default release command
release-binary:
    @echo "Building release version..."
    cargo build --release
    @echo "Generating OpenAPI schema..."
    cargo run -p goose-server --bin generate_schema

# release-windows docker build command
win_docker_build_sh := '''rustup target add x86_64-pc-windows-gnu && \
	apt-get update && \
	apt-get install -y mingw-w64 protobuf-compiler cmake && \
	export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc && \
	export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++ && \
	export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar && \
	export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc && \
	export PKG_CONFIG_ALLOW_CROSS=1 && \
	export PROTOC=/usr/bin/protoc && \
	export PATH=/usr/bin:\$PATH && \
	protoc --version && \
	cargo build --release --target x86_64-pc-windows-gnu && \
	GCC_DIR=\$(ls -d /usr/lib/gcc/x86_64-w64-mingw32/*/ | head -n 1) && \
	cp \$GCC_DIR/libstdc++-6.dll /usr/src/myapp/target/x86_64-pc-windows-gnu/release/ && \
	cp \$GCC_DIR/libgcc_s_seh-1.dll /usr/src/myapp/target/x86_64-pc-windows-gnu/release/ && \
	cp /usr/x86_64-w64-mingw32/lib/libwinpthread-1.dll /usr/src/myapp/target/x86_64-pc-windows-gnu/release/'


# Build Windows executable
release-windows:
    #!/usr/bin/env sh
    if [ "$(uname)" = "Darwin" ] || [ "$(uname)" = "Linux" ]; then
        echo "Building Windows executable using Docker..."
        docker volume create goose-windows-cache || true
        docker run --rm \
            -v "$(pwd)":/usr/src/myapp \
            -v goose-windows-cache:/usr/local/cargo/registry \
            -w /usr/src/myapp \
            rust:latest \
            sh -c "{{win_docker_build_sh}}"
    else
        echo "Building Windows executable using Docker through PowerShell..."
        powershell.exe -Command "docker volume create goose-windows-cache; \
            docker run --rm \
                -v ${PWD}:/usr/src/myapp \
                -v goose-windows-cache:/usr/local/cargo/registry \
                -w /usr/src/myapp \
                rust:latest \
                sh -c '{{win_docker_build_sh}}'"
    fi
    echo "Windows executable and required DLLs created at ./target/x86_64-pc-windows-gnu/release/"

# Run Docusaurus server for documentation
run-docs:
    @echo "Running docs server..."
    cd documentation && yarn && yarn start

# Run server
run-server:
    @echo "Running server..."
    cargo run -p goose-server --bin goosed agent

# Generate OpenAPI specification
generate-openapi:
    @echo "Generating OpenAPI schema..."
    cargo run -p goose-server --bin generate_schema

# Generate manpages for the CLI
generate-manpages:
    @echo "Generating manpages..."
    cargo run -p goose-cli --bin generate_manpages
    @echo "Manpages generated at target/man/"

ensure-release-branch:
    #!/usr/bin/env bash
    branch=$(git rev-parse --abbrev-ref HEAD); \
    if [[ ! "$branch" == release/* ]]; then \
        echo "Error: You are not on a release branch (current: $branch)"; \
        exit 1; \
    fi

    # check that main is up to date with upstream main
    git fetch
    # @{u} refers to upstream branch of current branch
    if [ "$(git rev-parse HEAD)" != "$(git rev-parse @{u})" ]; then \
        echo "Error: Your branch is not up to date with the upstream branch"; \
        echo "  ensure your branch is up to date (git pull)"; \
        exit 1; \
    fi

# validate the version is semver, and not the current version
validate version:
    #!/usr/bin/env bash
    if [[ ! "{{ version }}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-.*)?$ ]]; then
      echo "[error]: invalid version '{{ version }}'."
      echo "  expected: semver format major.minor.patch or major.minor.patch-<suffix>"
      exit 1
    fi

    current_version=$(just get-tag-version)
    if [[ "{{ version }}" == "$current_version" ]]; then
      echo "[error]: current_version '$current_version' is the same as target version '{{ version }}'"
      echo "  expected: new version in semver format"
      exit 1
    fi

get-next-minor-version:
    @python -c "import sys; v=sys.argv[1].split('.'); print(f'{v[0]}.{int(v[1])+1}.0')" $(just get-tag-version)

get-next-patch-version:
    @python -c "import sys; v=sys.argv[1].split('.'); print(f'{v[0]}.{v[1]}.{int(v[2])+1}')" $(just get-tag-version)

# set cargo and app versions, must be semver
prepare-release version:
    @just validate {{ version }} || exit 1

    @git switch -c "release/{{ version }}"
    @uvx --from=toml-cli toml set --toml-path=Cargo.toml "workspace.package.version" {{ version }}

    # see --workspace flag https://doc.rust-lang.org/cargo/commands/cargo-update.html
    # used to update Cargo.lock after we've bumped versions in Cargo.toml
    @cargo update --workspace
    @cargo run --bin build_canonical_models
    @git add \
        Cargo.toml \
        Cargo.lock \
        crates/goose/src/providers/canonical/data/canonical_models.json \
        crates/goose/src/providers/canonical/data/provider_metadata.json
    @git commit --message "chore(release): release version {{ version }}"

# extract version from Cargo.toml
get-tag-version:
    @uvx --from=toml-cli toml get --toml-path=Cargo.toml "workspace.package.version"

# create the git tag from Cargo.toml, checking we're on a release branch
tag: ensure-release-branch
    git tag v$(just get-tag-version)

# create tag and push to origin (use this when release branch is merged to main)
tag-push: tag
    # this will kick of ci for release
    git push origin tag v$(just get-tag-version)

# generate release notes from git commits
release-notes old:
    #!/usr/bin/env bash
    git log --pretty=format:"- %s" {{ old }}..v$(just get-tag-version)

### s = file separator based on OS
s := if os() == "windows" { "\\" } else { "/" }

### testing/debugging
os:
  echo "{{os()}}"
  echo "{{s}}"

# Make just work on Window
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

### Build the core code
### profile = --release or "" for debug
### allparam = OR/AND/ANY/NONE --workspace --all-features --all-targets
win-bld profile allparam:
  cargo run {{profile}} -p goose-server --bin  generate_schema
  cargo build {{profile}} {{allparam}}

### Build just debug
win-bld-dbg:
  just win-bld " " " "

### Build debug and test, examples,...
win-bld-dbg-all:
  just win-bld " " "--workspace --all-targets --all-features"

### Build just release
win-bld-rls:
  just win-bld "--release" " "

### Build release and test, examples, ...
win-bld-rls-all:
  just win-bld "--release" "--workspace --all-targets --all-features"

build-test-tools:
  cargo build -p goose-test

record-mcp-tests: build-test-tools
  GOOSE_RECORD_MCP=1 cargo test --package goose --test mcp_integration_test
  git add crates/goose/tests/mcp_replays/
