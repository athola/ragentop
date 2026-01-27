# ragentop

A terminal-first monitoring tool for AI coding agents, written in Rust.

Track sessions, metrics, and command history across Claude Code and other AI assistants.

## Why ragentop?

When working with AI coding agents via SSH or in terminal multiplexers, you need visibility into what's happening without leaving your workflow. ragentop provides:

- **Real-time metrics** — Token usage, cost estimates, session duration
- **Multi-agent support** — Claude Code, Codex, Copilot, Gemini Code Assist, Qwen, GLM
- **Command history** — Tool calls with configurable depth (tool-only → full conversation)
- **Versioned state** — Merkle DAG storage for session history and replay
- **TUI dashboard** — SSH-friendly interface built with ratatui
- **Multiplexer aware** — Tmux and Zellij session detection

## Status

**Current version**: v0.1.0-dev (MVP in progress)

| Component | Status |
|-----------|--------|
| ragentop-core | ✓ Types, traits, DAG, multiplexer, protocol |
| adapter-claude | ✓ Detection + metrics |
| adapter-codex | ✓ Detection scaffold |
| adapter-copilot | ✓ Detection scaffold |
| adapter-gemini | ✓ Detection scaffold |
| adapter-qwen | ✓ Detection scaffold |
| adapter-glm | ✓ Detection scaffold |
| ragentop-daemon | ✓ Socket API, sessions, tmux/zellij |
| ragentop-tui | ✓ Dashboard + input handling |
| ragentop-cli | ✓ daemon/tui subcommands |
| ragentop-web | Scaffold (deferred to v0.2.0) |

## Installation

```bash
git clone https://github.com/alext/ragentop
cd ragentop
make build && make install
```

Requires Rust 1.75+ (uses `async fn` in traits).

## Usage

```bash
# Start the background daemon
ragentop daemon start

# Launch the TUI dashboard
ragentop tui

# List active sessions
ragentop list

# Get help
ragentop --help
```

## Architecture

Hybrid Hexagonal + Functional Core design:

```
┌─────────────────────────────────────────────┐
│           IMPERATIVE SHELL                  │
│  cli · daemon · tui · web                   │
├─────────────────────────────────────────────┤
│           HEXAGONAL BOUNDARY                │
│  adapter-claude · adapter-codex · ...       │
├─────────────────────────────────────────────┤
│           FUNCTIONAL CORE                   │
│  ragentop-core (pure functions, no I/O)     │
│  types · traits · DAG ops · config          │
└─────────────────────────────────────────────┘
```

**Key principle**: Business logic stays pure in `ragentop-core`. Adapters implement the `AgentAdapter` trait. Shell orchestrates I/O.

## Development

```bash
make check        # Type check all crates
make test         # Run test suite
make lint         # Clippy lints
make pre-commit   # Format + lint + test
make demo         # Run functionality demo
```

See [Makefile Guide](docs/MAKEFILE.md) for all targets.

## Documentation

- [Architecture Decision Record](docs/adr/001-hybrid-hexagonal-functional-core-architecture.md)
- [MVP Design](docs/plans/2025-01-25-ragentop-mvp-design.md)
- [Implementation Plan](docs/plans/2025-01-25-ragentop-implementation-plan.md)
- [MVP Scope Decisions](docs/plans/2026-01-26-war-room-mvp-decisions.md)
- [Makefile Guide](docs/MAKEFILE.md)

## License

MIT
