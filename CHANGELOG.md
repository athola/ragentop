# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-15

Initial MVP release. Terminal-first monitoring for AI coding agents.

### Added

- **ragentop-core**: Pure functional core with types, traits, DAG storage, config,
  alerts, pricing, burn-rate tracking, stats, and normalization modules
- **adapter-claude**: Session detection from `~/.claude/projects/`, stats-cache
  parsing, metrics extraction
- **adapter-codex**: Detection and history parsing for Codex CLI
- **adapter-copilot**: Detection for GitHub Copilot CLI
- **adapter-gemini**: Detection for Gemini CLI
- **adapter-qwen**: Detection and parsing for Qwen Code
- **ragentop-daemon**: Background collector with Unix socket API, session tracking,
  tmux/zellij multiplexer integration via `tmux-interface` crate
- **ragentop-tui**: Terminal dashboard with session list and detail panels,
  keyboard and mouse input handling (ratatui)
- **ragentop-cli**: Entry point with `detect`, `status`, `tui`, `web`, and
  `daemon` subcommands (clap)
- **ragentop-web**: Axum HTTP server with automatic browser launch
- Hybrid Hexagonal + Functional Core architecture with strict layer boundaries
- Workspace-level Cargo configuration with ultra-strict clippy lints
- GitHub Actions CI workflow

[0.1.0]: https://github.com/alext/ragentop/releases/tag/v0.1.0
