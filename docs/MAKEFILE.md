# ragentop Makefile Guide

## Overview

The ragentop Makefile provides a comprehensive interface for building, testing, and demonstrating the AI agent monitoring tool. It follows standard Rust conventions while adding project-specific targets for the workspace architecture.

## Quick Reference

```bash
# Get help
make help

# Development workflow
make check          # Quick type check
make fmt-check      # Verify formatting
make lint           # Run clippy
make test-quiet     # Run tests
make pre-commit     # All pre-commit checks

# Building
make build          # Release build
make build-dev      # Dev build
make install        # Install CLI binary

# Demonstration
make demo           # Show project overview
make demo-architecture  # Show architecture diagram
make demo-adapters      # List all adapters

# CI/CD
make ci             # Full CI pipeline
make ci-full        # CI with verbose tests
```

## Target Categories

### General Targets

| Target | Description |
|--------|-------------|
| `help` | Display help message with all targets |
| `all` | Build all workspace members (alias for build) |

### Development Targets

| Target | Description |
|--------|-------------|
| `check` | Quick check (parse + type check) |
| `check-verbose` | Check with detailed output |
| `build` | Build release binaries |
| `build-dev` | Build dev binaries |
| `watch` | Watch for changes (requires cargo-watch) |

### Testing Targets

| Target | Description |
|--------|-------------|
| `test` | Run all tests |
| `test-quiet` | Run tests with minimal output |
| `test-verbose` | Run tests with captured output |
| `test-core` | Test only ragentop-core |
| `test-adapters` | Test all adapters |
| `coverage` | Generate coverage report (requires cargo-llvm-cov) |

### Quality Targets

| Target | Description |
|--------|-------------|
| `lint` | Run clippy with `-D warnings` |
| `fmt` | Format all code with rustfmt |
| `fmt-check` | Check if code is formatted |
| `audit` | Security audit (requires cargo-audit) |
| `outdated` | Check outdated deps (requires cargo-outdated) |

### Cleaning Targets

| Target | Description |
|--------|-------------|
| `clean` | Clean build artifacts |
| `clean-docs` | Clean generated docs |
| `deep-clean` | Clean everything |

### Documentation Targets

| Target | Description |
|--------|-------------|
| `docs` | Generate documentation |
| `docs-open` | Generate and open docs in browser |
| `readme-deps` | List README dependencies |

### Installation Targets

| Target | Description |
|--------|-------------|
| `install` | Build and install CLI binary |
| `install-release` | Install release-optimized binary |

### Demonstration Targets

| Target | Description |
|--------|-------------|
| `demo` | Run full demonstration |
| `demo-adapters` | Show adapter capabilities |
| `demo-architecture` | Display architecture overview |

### CI/CD Targets

| Target | Description |
|--------|-------------|
| `ci` | Run CI locally (fmt + lint + test) |
| `ci-full` | Full CI with verbose tests |
| `pre-commit` | Quick pre-commit checks |
| `pre-push` | Full pre-push validation |

### Utility Targets

| Target | Description |
|--------|-------------|
| `tree` | Show workspace tree |
| `deps` | Show dependency tree |
| `size` | Show binary sizes |
| `version` | Show version info |
| `dry-run-build` | Show what would be built |
| `dry-run-test` | Show what tests would run |

## Architecture Integration

The Makefile is aware of ragentop's hexagonal architecture:

```
FUNCTIONAL CORE (ragentop-core)
├── Pure functions only
├── Types, traits, DAG operations
└── Test target: 90% coverage

HEXAGONAL BOUNDARY
├── Ports = traits in core
└── Adapters = implementations

IMPERATIVE SHELL
├── daemon: Background collector
├── tui: Terminal UI (ratatui)
├── web: Web UI (leptos)
└── cli: Command-line interface
```

## Workspace Members

The Makefile tracks these workspace crates:
- `ragentop-core` - Functional core with types and traits
- `ragentop-daemon` - Background collection daemon
- `ragentop-tui` - Terminal UI
- `ragentop-web` - Web UI
- `ragentop-cli` - Command-line interface

## Adapters

The Makefile is aware of all agent adapters:
- `adapter-claude` - Claude Code
- `adapter-codex` - OpenAI Codex
- `adapter-gemini` - Google Gemini
- `adapter-copilot` - GitHub Copilot
- `adapter-qwen` - Alibaba Qwen
- `adapter-glm` -智谱 GLM

## Best Practices

1. **Pre-commit workflow**: Run `make pre-commit` before committing
2. **Pre-push workflow**: Run `make pre-push` before pushing
3. **Continuous development**: Use `make watch` for automatic rebuilds
4. **Quality gates**: Always run `make ci` before merging
5. **Format first**: Run `make fmt` to auto-format code

## Extending the Makefile

To add new targets:

1. Follow the naming convention: `category-action` (e.g., `test-integration`)
2. Add `.PHONY` declaration at the top
3. Use `@echo` for user feedback
4. Add help comment with `## Description`
5. Test with `make -n target` for dry-run

Example:

```makefile
.PHONY: test-integration

test-integration: ## Run integration tests
	@echo "Running integration tests..."
	$(CARGO) test --workspace --test '*_integration'
```

## Environment Variables

The Makefile respects these environment variables:

- `CARGO` - Path to cargo binary (default: `cargo`)
- `CARGO_WATCH` - Path to cargo-watch (default: `cargo-watch`)
- `RUSTFLAGS` - Additional rustc flags

## Troubleshooting

### "cargo-watch not installed"
```bash
cargo install cargo-watch
```

### "cargo-llvm-cov not installed"
```bash
cargo install cargo-llvm-cov
```

### "cargo-audit not installed"
```bash
cargo install cargo-audit
```

### "cargo-outdated not installed"
```bash
cargo install cargo-outdated
```

## Integration with Git Hooks

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/sh
make pre-commit
```

Add to `.git/hooks/pre-push`:

```bash
#!/bin/sh
make pre-push
```

## See Also

- [CLAUDE.md](../CLAUDE.md) - Project guidelines
- [Cargo.toml](../Cargo.toml) - Workspace configuration
- [Implementation Plan](./plans/2025-01-25-ragentop-implementation-plan.md) - Development roadmap
