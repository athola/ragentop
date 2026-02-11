# ragentop - Claude Code Guidelines

## Project Overview

ragentop is a Rust monitoring tool for AI coding agents. It uses a **Hybrid Hexagonal + Functional Core** architecture.

## Architecture Rules

### Layer Boundaries (ENFORCED)

```
FUNCTIONAL CORE (ragentop-core)
├── Pure functions only - NO I/O, NO side effects
├── Types, traits (ports), DAG operations, business logic
└── Test target: 90% coverage with unit tests

HEXAGONAL BOUNDARY
├── Ports = traits defined in core (AgentAdapter, DagStore)
└── Adapters = implementations in adapter-* crates

IMPERATIVE SHELL (daemon, cli, tui, web)
├── Orchestration, I/O, retries, events
└── Calls core functions, interprets results
```

### Dependency Direction

```
adapters/* → ragentop-core  ✓
ragentop-daemon → ragentop-core, adapters/*  ✓
ragentop-core → adapters/*  ✗ FORBIDDEN
ragentop-core → external I/O  ✗ FORBIDDEN
```

### Code Review Checklist

Before merging, verify:
- [ ] Business logic is in `ragentop-core`, not shell
- [ ] Core functions are pure (no I/O)
- [ ] New adapters implement `AgentAdapter` trait
- [ ] Tests exist for new functionality

## Crate Responsibilities

| Crate | Purpose | Purity |
|-------|---------|--------|
| `ragentop-core` | Types, traits, DAG ops, config | Pure |
| `ragentop-daemon` | Background collector, socket API | Impure |
| `ragentop-tui` | Terminal UI (ratatui) | Impure |
| `ragentop-web` | Web UI (leptos) | Impure |
| `ragentop-cli` | CLI entry point (clap) | Impure |
| `adapter-*` | Agent-specific data extraction | Impure |

## Development Commands

```bash
# Using Makefile (preferred)
make check           # Check all crates
make test            # Run all tests
make lint            # Run clippy
make pre-commit      # Format + lint + test

# Direct cargo commands
cargo check --workspace
cargo test --workspace
cargo fmt --all && cargo clippy --workspace -- -D warnings
```

See [docs/MAKEFILE.md](docs/MAKEFILE.md) for full target reference.

## Adding New Adapters

1. Create `crates/adapters/adapter-<name>/`
2. Implement `AgentAdapter` trait from `ragentop-core`
3. Add to workspace members in root `Cargo.toml`
4. Add contract tests validating trait behavior
5. Register in `ragentop-daemon` adapter registry

## Testing Strategy

- **Core**: Unit tests with pure functions, property-based tests
- **Adapters**: Contract tests + integration tests with fixtures
- **Shell**: Integration tests, minimal unit tests

## Key Files

- `docs/adr/001-*.md` - Architecture decision record
- `docs/plans/2025-01-25-ragentop-mvp-design.md` - Full specification
- `docs/plans/2025-01-25-ragentop-implementation-plan.md` - TDD task breakdown
- `docs/plans/2026-01-26-war-room-mvp-decisions.md` - MVP scope decisions
- `docs/MAKEFILE.md` - Build system reference
