# ragentop Makefile
# Rust monitoring tool for AI coding agents

.PHONY: help all check check-verbose build build-dev watch \
        test test-quiet test-verbose test-core test-adapters coverage bench \
        lint fmt fmt-check audit outdated \
        clean clean-docs deep-clean \
        docs docs-open readme-deps \
        install install-release \
        demo demo-adapters demo-architecture demo-all demo-types \
        demo-detection demo-metrics demo-history demo-dag \
        demo-multiplexer demo-protocol demo-tracking \
        demo-daemon-start demo-cleanup dogfood \
        ci ci-full pre-commit pre-push githooks \
        tree deps size version dry-run-build dry-run-test update-deps

# Default target
.DEFAULT_GOAL := help

# Fail fast on errors in shell commands
SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

# Rust binaries
CARGO := cargo
CARGO_WATCH := cargo-watch

# Project metadata
WORKSPACE_MEMBERS := ragentop-core ragentop-daemon ragentop-tui ragentop-web ragentop-cli
ADAPTERS := adapter-claude adapter-codex adapter-gemini adapter-copilot adapter-qwen

##@ General

help: ## Display this help message
	@echo "ragentop - Rust monitoring tool for AI coding agents"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*##"; printf "  %-20s %s\n", "Target", "Description"} /^[%/a-zA-Z_-]+:.*?##/ { printf "  %-20s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
	@echo ""
	@echo "Project Structure:"
	@echo "  Workspace: $(WORKSPACE_MEMBERS)"
	@echo "  Adapters: $(ADAPTERS)"

all: build ## Build all workspace members

##@ Development

check: ## Quick check all crates (parse + type check)
	@echo "Checking workspace..."
	$(CARGO) check --workspace

check-verbose: ## Check with detailed output
	$(CARGO) check --workspace --verbose

build: ## Build all crates in release mode
	@echo "Building workspace (release)..."
	$(CARGO) build --workspace --release

build-dev: ## Build all crates in dev mode
	$(CARGO) build --workspace

watch: ## Watch for changes and rebuild (requires cargo-watch)
	@if command -v $(CARGO_WATCH) >/dev/null 2>&1; then \
		$(CARGO_WATCH) -x build; \
	else \
		echo "cargo-watch not installed. Install with: cargo install cargo-watch"; \
		exit 1; \
	fi

##@ Testing

test: ## Run all tests
	@echo "Running tests..."
	$(CARGO) test --workspace

test-quiet: ## Run tests with minimal output
	$(CARGO) test --workspace --quiet

test-verbose: ## Run tests with output
	$(CARGO) test --workspace -- --nocapture

test-core: ## Run only core library tests
	$(CARGO) test -p ragentop-core

test-adapters: ## Run all adapter tests
	@failed=0; \
	for adapter in $(ADAPTERS); do \
		echo "Testing $$adapter..."; \
		$(CARGO) test -p $$adapter || failed=1; \
	done; \
	exit $$failed

coverage: ## Run tests with coverage (requires cargo-llvm-cov)
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		cargo llvm-cov --workspace; \
	else \
		echo "cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	fi

bench: ## Run benchmarks
	$(CARGO) bench --workspace

##@ Quality

lint: ## Run clippy lints
	@echo "Running linter..."
	$(CARGO) clippy --workspace -- -D warnings

fmt: ## Format all code
	@echo "Formatting code..."
	$(CARGO) fmt --all

fmt-check: ## Check if code is formatted
	$(CARGO) fmt --all -- --check

audit: ## Security audit dependencies
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		echo "cargo-audit not installed. Install with: cargo install cargo-audit"; \
	fi

outdated: ## Check for outdated dependencies
	@if command -v cargo-outdated >/dev/null 2>&1; then \
		cargo outdated --workspace; \
	else \
		echo "cargo-outdated not installed. Install with: cargo install cargo-outdated"; \
	fi

##@ Cleaning

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	$(CARGO) clean

clean-docs: ## Clean generated documentation
	@echo "Cleaning documentation..."
	$(RM) -rf target/doc

deep-clean: clean clean-docs ## Clean everything including documentation
	@echo "Deep clean complete"

##@ Documentation

docs: ## Generate documentation
	@echo "Generating documentation..."
	$(CARGO) doc --workspace --no-deps

docs-open: docs ## Generate and open documentation in browser
	$(CARGO) doc --workspace --no-deps --open

readme-deps: ## List README dependencies (for context optimization)
	@echo "Project README dependencies:"
	@cat README.md 2>/dev/null || echo "README.md not found"

##@ Installation

install: ## Build and install CLI binary
	@echo "Installing ragentop CLI..."
	$(CARGO) install --path crates/ragentop-cli

install-release: ## Install release-optimized binary
	$(CARGO) install --path crates/ragentop-cli --release

##@ Demonstration

# Daemon lifecycle for demos
demo-daemon-start: build
	@echo "┌─ Starting daemon for demo... ─────────────────────────┐"
	@./target/release/ragentop daemon start & sleep 1
	@echo "│  ✓ Daemon started                                     │"
	@echo "└───────────────────────────────────────────────────────┘"

demo-cleanup:
	@echo "┌─ Cleaning up demo daemon... ──────────────────────────┐"
	@./target/release/ragentop daemon stop 2>/dev/null || true
	@rm -f /tmp/ragentop.sock
	@echo "│  ✓ Daemon stopped                                     │"
	@echo "└───────────────────────────────────────────────────────┘"

demo: build ## Run demonstration of core functionality
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║            ragentop — Agent Monitor                   ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "┌─ Binary Metadata ────────────────────────────────────┐"
	@if [ -f target/release/ragentop ]; then \
		VERSION=$$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2); \
		SIZE=$$(du -h target/release/ragentop | cut -f1); \
		echo "│  Binary:  target/release/ragentop"; \
		echo "│  Version: $$VERSION"; \
		echo "│  Size:    $$SIZE"; \
		echo "│  Profile: release"; \
	else \
		echo "│  ✗ CLI binary not found. Run 'make build' first."; \
		exit 1; \
	fi
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Adapters ($(words $(ADAPTERS))) ─────────────────────────────────────┐"
	@for adapter in $(ADAPTERS); do \
		if [ -d "crates/adapters/$$adapter" ]; then \
			echo "│  ✓ $$adapter"; \
		fi; \
	done
	@echo "│  Capabilities: tokens, cost, commands, model_info"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Workspace Crates ($(words $(WORKSPACE_MEMBERS))) ─────────────────────────────┐"
	@for member in $(WORKSPACE_MEMBERS); do \
		if [ -d "crates/$$member" ]; then \
			LINES=$$(find "crates/$$member/src" -name '*.rs' -exec cat {} + 2>/dev/null | wc -l); \
			printf "│  ✓ %-22s %5d lines\n" "$$member" "$$LINES"; \
		fi; \
	done
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "Run: ./target/release/ragentop --help"

demo-adapters: build ## Demonstrate adapter capabilities (LIVE, daemon)
	@$(MAKE) --no-print-directory demo-daemon-start
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║          Adapter Capabilities Demo (LIVE)             ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "┌─ Adapter Capability Matrix ──────────────────────────┐"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "Adapter" "Tokens" "Cost" "Cmds" "Model" "Replay"
	@echo "│  ────────────────── ──────── ────── ────── ────── ────────"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "adapter-claude"   "✓" "✓" "✓" "✓" "✓"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "adapter-codex"    "✓" "✓" "✓" "✓" "—"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "adapter-gemini"   "✓" "✓" "✓" "✓" "—"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "adapter-copilot"  "✓" "—" "✓" "✓" "—"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "adapter-qwen"     "✓" "✓" "✓" "✓" "—"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Live Detection ─────────────────────────────────────┐"
	@./target/release/ragentop detect --verbose 2>/dev/null || ./target/release/ragentop detect 2>/dev/null || echo "│  (no sessions detected)"
	@echo "└───────────────────────────────────────────────────────┘"
	@$(MAKE) --no-print-directory demo-cleanup

demo-architecture: ## Show architecture overview
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║            ragentop Architecture                      ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "FUNCTIONAL CORE (ragentop-core)"
	@echo "  ├── Pure functions only — NO I/O, NO side effects"
	@echo "  ├── Types, traits (ports), DAG operations"
	@echo "  └── Test target: 90% coverage"
	@echo ""
	@echo "HEXAGONAL BOUNDARY"
	@echo "  ├── Ports = traits defined in core (AgentAdapter, DagStore)"
	@echo "  └── Adapters = implementations in adapter-* crates"
	@echo ""
	@echo "IMPERATIVE SHELL"
	@echo "  ├── daemon: Background collector, socket API"
	@echo "  ├── tui: Terminal UI (ratatui)"
	@echo "  ├── web: Web UI (leptos)"
	@echo "  └── cli: Command-line interface (clap)"
	@echo ""
	@echo "┌─ Crate Dependency Graph ─────────────────────────────┐"
	@$(CARGO) tree --workspace --depth 1 2>/dev/null | head -30 || echo "│  (cargo tree unavailable)"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Lines of Code per Crate ────────────────────────────┐"
	@for member in $(WORKSPACE_MEMBERS); do \
		if [ -d "crates/$$member/src" ]; then \
			LINES=$$(find "crates/$$member/src" -name '*.rs' -exec cat {} + 2>/dev/null | wc -l); \
			printf "│  %-22s %5d lines\n" "$$member" "$$LINES"; \
		fi; \
	done
	@for adapter in $(ADAPTERS); do \
		if [ -d "crates/adapters/$$adapter/src" ]; then \
			LINES=$$(find "crates/adapters/$$adapter/src" -name '*.rs' -exec cat {} + 2>/dev/null | wc -l); \
			printf "│  %-22s %5d lines\n" "$$adapter" "$$LINES"; \
		fi; \
	done
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Core Traits (Ports) ────────────────────────────────┐"
	@grep -rn 'pub trait' crates/ragentop-core/src/ 2>/dev/null | sed 's/.*src\//│  /' || echo "│  (none found)"
	@echo "└───────────────────────────────────────────────────────┘"

demo-all: demo-types demo-detection demo-metrics demo-history demo-dag demo-multiplexer demo-protocol demo-tracking demo-cleanup ## Run all functionality demos

demo-types: ## Demonstrate core type system
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║            Core Type System Demo                      ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "┌─ AgentType Enum ─────────────────────────────────────┐"
	@printf "│  %-10s %-30s %-20s\n" "Variant" "Description" "Config Dir"
	@echo "│  ────────── ────────────────────────────── ────────────────────"
	@printf "│  %-10s %-30s %-20s\n" "Claude"  "Anthropic's Claude Code"     "~/.claude/"
	@printf "│  %-10s %-30s %-20s\n" "Codex"   "OpenAI's Codex CLI"          "~/.codex/"
	@printf "│  %-10s %-30s %-20s\n" "Gemini"  "Google's Gemini CLI"         "~/.gemini/"
	@printf "│  %-10s %-30s %-20s\n" "Copilot" "GitHub's Copilot CLI"        "~/.config/github-copilot/"
	@printf "│  %-10s %-30s %-20s\n" "Qwen"    "Alibaba's Qwen CLI"          "~/.qwen/"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ SessionStatus Enum ─────────────────────────────────┐"
	@printf "│  %-10s %-45s\n" "Variant" "Description"
	@echo "│  ────────── ─────────────────────────────────────────────"
	@printf "│  %-10s %-45s\n" "Active"  "Running process or modified within 5 minutes"
	@printf "│  %-10s %-45s\n" "Idle"    "Session exists but no recent activity"
	@printf "│  %-10s %-45s\n" "Paused"  "Session explicitly paused by user"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ HistoryDepth Enum ──────────────────────────────────┐"
	@printf "│  %-20s %-8s %-25s\n" "Variant" "Level" "Data Volume Estimate"
	@echo "│  ──────────────────── ──────── ─────────────────────────"
	@printf "│  %-20s %-8s %-25s\n" "ToolCallsOnly"    "1" "~1-5 KB per session"
	@printf "│  %-20s %-8s %-25s\n" "WithResponses"    "2" "~10-50 KB per session"
	@printf "│  %-20s %-8s %-25s\n" "FullConversation" "3" "~100 KB-1 MB per session"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ CommandStatus Enum ─────────────────────────────────┐"
	@printf "│  %-10s %-45s\n" "Success" "Command completed successfully"
	@printf "│  %-10s %-45s\n" "Failed"  "Command exited with error"
	@printf "│  %-10s %-45s\n" "Running" "Command currently executing"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Data Model Summary ─────────────────────────────────┐"
	@TYPES=$$(grep -rn 'pub struct' crates/ragentop-core/src/ 2>/dev/null | wc -l); \
	TRAITS=$$(grep -rn 'pub trait' crates/ragentop-core/src/ 2>/dev/null | wc -l); \
	ENUMS=$$(grep -rn 'pub enum' crates/ragentop-core/src/ 2>/dev/null | wc -l); \
	echo "│  Structs: $$TYPES   Traits: $$TRAITS   Enums: $$ENUMS"; \
	echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "Verify with: make test-core"

demo-detection: build ## Demonstrate agent session detection (LIVE, daemon)
	@$(MAKE) --no-print-directory demo-daemon-start
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║       Agent Session Detection Demo (LIVE)             ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "┌─ Per-Adapter Capability Matrix ──────────────────────┐"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "Adapter" "Tokens" "Cost" "Cmds" "Model" "Replay"
	@echo "│  ────────────────── ──────── ────── ────── ────── ────────"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "adapter-claude"   "✓" "✓" "✓" "✓" "✓"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "adapter-codex"    "✓" "✓" "✓" "✓" "—"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "adapter-gemini"   "✓" "✓" "✓" "✓" "—"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "adapter-copilot"  "✓" "—" "✓" "✓" "—"
	@printf "│  %-18s %-8s %-6s %-6s %-6s %-8s\n" "adapter-qwen"     "✓" "✓" "✓" "✓" "—"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Detected Sessions ──────────────────────────────────┐"
	@echo "│  Active = running process OR modified within 5 min"
	@echo "│"
	@./target/release/ragentop detect --verbose 2>/dev/null | sed 's/^/│  /' || echo "│  (no sessions detected)"
	@echo "└───────────────────────────────────────────────────────┘"
	@$(MAKE) --no-print-directory demo-cleanup

demo-metrics: build ## Demonstrate metrics collection (LIVE, daemon)
	@$(MAKE) --no-print-directory demo-daemon-start
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║          Metrics Collection Demo (LIVE)               ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "┌─ Metrics Fields ─────────────────────────────────────┐"
	@printf "│  %-20s %-10s %-25s\n" "Field" "Unit" "Valid Range"
	@echo "│  ──────────────────── ────────── ─────────────────────────"
	@printf "│  %-20s %-10s %-25s\n" "tokens_in"      "tokens"  "0 .. 2^32"
	@printf "│  %-20s %-10s %-25s\n" "tokens_out"     "tokens"  "0 .. 2^32"
	@printf "│  %-20s %-10s %-25s\n" "cost_usd"       "USD"     "0.00 .. 999.99"
	@printf "│  %-20s %-10s %-25s\n" "cpu_percent"    "%"       "0.0 .. 100.0"
	@printf "│  %-20s %-10s %-25s\n" "memory_mb"      "MB"      "0 .. system max"
	@printf "│  %-20s %-10s %-25s\n" "active_commands" "count"  "0 .. 100"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Live Session Metrics (sampled from history) ────────┐"
	@if [ -d "$$HOME/.claude/projects" ]; then \
		printf "│  %-30s %8s %10s\n" "Project" "Lines" "Size"; \
		echo "│  ────────────────────────────── ──────── ──────────"; \
		DIRS=$$(find "$$HOME/.claude/projects" -name "*.jsonl" -type f 2>/dev/null | xargs -I{} dirname {} | sort -u | head -5 || true); \
		for d in $$DIRS; do \
			NAME=$$(basename "$$d" | cut -c1-28); \
			LINES=$$(cat "$$d"/*.jsonl 2>/dev/null | wc -l); \
			SIZE=$$(du -sh "$$d" 2>/dev/null | cut -f1); \
			printf "│  %-30s %8s %10s\n" "$$NAME" "$$LINES" "$$SIZE"; \
		done; \
		TOTAL_FILES=$$(find "$$HOME/.claude/projects" -name "*.jsonl" -type f 2>/dev/null | wc -l); \
		echo "│"; \
		echo "│  Total session files: $$TOTAL_FILES"; \
	else \
		echo "│  (no Claude projects directory found)"; \
	fi
	@echo "└───────────────────────────────────────────────────────┘"
	@$(MAKE) --no-print-directory demo-cleanup

demo-history: ## Demonstrate command history retrieval (LIVE)
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║          Command History Demo (LIVE)                  ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "┌─ JSONL File Statistics ──────────────────────────────┐"
	@if [ -d "$$HOME/.claude/projects" ]; then \
		COUNT=$$(find "$$HOME/.claude/projects" -name "*.jsonl" -type f 2>/dev/null | wc -l); \
		TOTAL=$$(du -sh "$$HOME/.claude/projects" 2>/dev/null | cut -f1); \
		NEWEST=$$(find "$$HOME/.claude/projects" -name "*.jsonl" -type f -exec stat -c '%Y %n' {} + 2>/dev/null | sort -rn | head -1 | cut -d' ' -f2- || true); \
		OLDEST=$$(find "$$HOME/.claude/projects" -name "*.jsonl" -type f -exec stat -c '%Y %n' {} + 2>/dev/null | sort -n | head -1 | cut -d' ' -f2- || true); \
		echo "│  Location: $$HOME/.claude/projects/"; \
		echo "│  Files:    $$COUNT JSONL files"; \
		echo "│  Size:     $$TOTAL total"; \
		echo "│  Newest:   $$(basename "$$NEWEST" 2>/dev/null || echo '—')"; \
		echo "│  Oldest:   $$(basename "$$OLDEST" 2>/dev/null || echo '—')"; \
	else \
		echo "│  (no Claude projects directory found)"; \
	fi
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ History Depth Comparison ───────────────────────────┐"
	@printf "│  %-20s %-8s %-30s\n" "Level" "Depth" "Data Included"
	@echo "│  ──────────────────── ──────── ──────────────────────────────"
	@printf "│  %-20s %-8s %-30s\n" "ToolCallsOnly"    "1" "Tool names + args only"
	@printf "│  %-20s %-8s %-30s\n" "WithResponses"    "2" "Above + abbreviated responses"
	@printf "│  %-20s %-8s %-30s\n" "FullConversation" "3" "Complete conversation turns"
	@echo "└───────────────────────────────────────────────────────┘"

demo-dag: ## Demonstrate Merkle DAG storage (LIVE)
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║          Merkle DAG Storage Demo (LIVE)               ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "┌─ Storage Engine ─────────────────────────────────────┐"
	@echo "│  Backend:  sled (embedded key-value store)"
	@echo "│  Hash:     BLAKE3 (content-addressable)"
	@echo "│  Structure: Merkle DAG (directed acyclic graph)"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Storage Statistics ─────────────────────────────────┐"
	@DAG_DIR="$$HOME/.local/share/ragentop/dag"; \
	if [ -d "$$DAG_DIR" ]; then \
		SIZE=$$(du -sh "$$DAG_DIR" 2>/dev/null | cut -f1); \
		FILES=$$(find "$$DAG_DIR" -type f 2>/dev/null | wc -l); \
		echo "│  Path:  $$DAG_DIR"; \
		echo "│  Size:  $$SIZE"; \
		echo "│  Files: $$FILES"; \
	else \
		echo "│  Path:   $$DAG_DIR (not created yet)"; \
		echo "│  Status: Will be created when daemon starts"; \
	fi
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Sample StateNode Structure ─────────────────────────┐"
	@echo '│  {'
	@echo '│    "hash": "blake3:a1b2c3d4...",'
	@echo '│    "parent": "blake3:e5f6a7b8...",'
	@echo '│    "timestamp": "2026-01-27T12:00:00Z",'
	@echo '│    "agent_type": "Claude",'
	@echo '│    "session_id": "abc-123",'
	@echo '│    "metrics": { "tokens_in": 1500, "cost_usd": 0.04 }'
	@echo '│  }'
	@echo "└───────────────────────────────────────────────────────┘"

demo-multiplexer: ## Demonstrate terminal multiplexer integration (LIVE)
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║       Terminal Multiplexer Demo (LIVE)                ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "┌─ Capability Matrix ──────────────────────────────────┐"
	@printf "│  %-10s %-10s %-10s %-20s\n" "Mux" "Panes" "Sessions" "Status"
	@echo "│  ────────── ────────── ────────── ────────────────────"
	@TMUX_STATUS="not found"; TMUX_PANES="-"; TMUX_SESSIONS="-"; \
	if command -v tmux >/dev/null 2>&1; then \
		if tmux info >/dev/null 2>&1; then \
			TMUX_PANES=$$(tmux list-panes -a 2>/dev/null | wc -l); \
			TMUX_SESSIONS=$$(tmux list-sessions 2>/dev/null | wc -l); \
			TMUX_STATUS="running"; \
		else \
			TMUX_STATUS="installed"; \
		fi; \
	fi; \
	printf "│  %-10s %-10s %-10s %-20s\n" "tmux" "$$TMUX_PANES" "$$TMUX_SESSIONS" "$$TMUX_STATUS"
	@ZELLIJ_STATUS="not found"; \
	if command -v zellij >/dev/null 2>&1; then \
		ZELLIJ_STATUS="installed"; \
	fi; \
	printf "│  %-10s %-10s %-10s %-20s\n" "zellij" "-" "-" "$$ZELLIJ_STATUS"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@if command -v tmux >/dev/null 2>&1 && tmux info >/dev/null 2>&1; then \
		echo "┌─ tmux Pane Details ──────────────────────────────────┐"; \
		tmux list-panes -a -F '│  #{pane_id}: #{pane_title} (#{pane_current_command})' 2>/dev/null || echo "│  (no panes)"; \
		echo "└───────────────────────────────────────────────────────┘"; \
	fi

demo-protocol: build ## Demonstrate daemon protocol (LIVE, daemon)
	@$(MAKE) --no-print-directory demo-daemon-start
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║          Daemon Protocol Demo (LIVE)                  ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "┌─ Socket Info ────────────────────────────────────────┐"
	@SOCKET="/tmp/ragentop.sock"; \
	if [ -S "$$SOCKET" ]; then \
		echo "│  Path:   $$SOCKET"; \
		echo "│  Status: ACTIVE"; \
		PERMS=$$(stat -c '%a' "$$SOCKET" 2>/dev/null || stat -f '%Lp' "$$SOCKET" 2>/dev/null); \
		echo "│  Perms:  $$PERMS"; \
		echo "│  Type:   Unix domain socket (SOCK_STREAM)"; \
	else \
		echo "│  Path:   $$SOCKET"; \
		echo "│  Status: NOT RUNNING"; \
	fi
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Protocol Messages ──────────────────────────────────┐"
	@printf "│  %-20s %-35s\n" "Message" "Response Structure"
	@echo "│  ──────────────────── ───────────────────────────────────"
	@printf "│  %-20s %-35s\n" "ListSessions"  "{ sessions: [Session...] }"
	@printf "│  %-20s %-35s\n" "GetMetrics(id)" "{ metrics: Metrics }"
	@printf "│  %-20s %-35s\n" "GetHistory(id)" "{ entries: [HistoryEntry...] }"
	@printf "│  %-20s %-35s\n" "Subscribe(id)"  "Stream<MetricsUpdate>"
	@echo "│"
	@echo "│  Protocol: JSON over Unix socket"
	@echo "└───────────────────────────────────────────────────────┘"
	@$(MAKE) --no-print-directory demo-cleanup

demo-tracking: build ## Demonstrate session tracking (LIVE, daemon)
	@$(MAKE) --no-print-directory demo-daemon-start
	@echo "╔═══════════════════════════════════════════════════════╗"
	@echo "║          Session Tracking Demo (LIVE)                 ║"
	@echo "╚═══════════════════════════════════════════════════════╝"
	@echo ""
	@echo "┌─ Detected Sessions ──────────────────────────────────┐"
	@./target/release/ragentop detect 2>/dev/null | sed 's/^/│  /' || echo "│  (no sessions detected)"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Session State Transitions ──────────────────────────┐"
	@echo "│  Active ──► Idle ──► (removed after timeout)"
	@echo "│    │          ▲"
	@echo "│    ▼          │"
	@echo "│  Paused ──────┘"
	@echo "└───────────────────────────────────────────────────────┘"
	@echo ""
	@echo "┌─ Daemon Status ──────────────────────────────────────┐"
	@SOCKET="/tmp/ragentop.sock"; \
	if [ -S "$$SOCKET" ]; then \
		echo "│  Daemon: RUNNING (socket active)"; \
		echo "│  Poll interval: 5s (configurable)"; \
	else \
		echo "│  Daemon: NOT RUNNING"; \
	fi
	@echo "└───────────────────────────────────────────────────────┘"
	@$(MAKE) --no-print-directory demo-cleanup

dogfood: build ## Run ragentop monitoring this project's agents (LIVE)
	@echo "Starting ragentop to monitor current agent sessions..."
	@./target/release/ragentop tui 2>/dev/null || ./target/release/ragentop detect

##@ CI/CD

githooks: ## Install git pre-commit hooks
	@git config core.hooksPath githooks
	@echo "Git hooks installed (githooks/pre-commit)"

ci: fmt-check lint test ## Run CI pipeline locally

ci-full: fmt-check lint test-verbose build ## Run full CI with verbose tests

pre-commit: fmt lint test-quiet ## Quick pre-commit checks
	@echo "Pre-commit checks passed!"

pre-push: ci-full ## Full pre-push validation

##@ Utilities

tree: ## Show workspace tree structure
	@echo "ragentop workspace tree:"
	@tree -L 2 -d crates 2>/dev/null || find crates -maxdepth 2 -type d | sort

deps: ## Show dependency tree
	$(CARGO) tree --workspace

size: ## Show binary sizes
	@echo "Binary sizes:"
	@find target/release -executable -type f -exec ls -lh {} \; 2>/dev/null || echo "No release binaries found. Run 'make build' first."

version: ## Show version information
	@echo "ragentop version:"
	@grep "^version" Cargo.toml | head -1 || echo "Version not found in Cargo.toml"
	@echo "Rust version:"
	@rustc --version

update-deps: ## Update Cargo.lock to latest compatible versions
	$(CARGO) update

##@ Dry Run

dry-run-build: ## Show what would be built (dry-run)
	@echo "Dry run: cargo build --workspace"
	$(CARGO) build --workspace --dry-run 2>&1 | head -20

dry-run-test: ## Show what tests would run (dry-run)
	@echo "Dry run: cargo test --workspace"
	@echo "Would run tests for:"
	@for member in $(WORKSPACE_MEMBERS); do \
		echo "  - $$member"; \
	done
	@for adapter in $(ADAPTERS); do \
		echo "  - $$adapter"; \
	done
