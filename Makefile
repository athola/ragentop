# ragentop Makefile
# Rust monitoring tool for AI coding agents

.PHONY: help all check check-verbose build build-dev watch \
        test test-quiet test-verbose test-core test-adapters coverage \
        lint fmt fmt-check audit outdated \
        clean clean-docs deep-clean \
        docs docs-open readme-deps \
        install install-release \
        demo demo-adapters demo-architecture demo-all demo-types \
        demo-detection demo-metrics demo-history demo-dag \
        demo-multiplexer demo-protocol demo-tracking \
        ci ci-full pre-commit pre-push githooks \
        tree deps size version dry-run-build dry-run-test

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
ADAPTERS := adapter-claude adapter-codex adapter-gemini adapter-copilot adapter-qwen adapter-glm

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

demo: build ## Run demonstration of core functionality
	@echo "=== ragentop Demonstration ==="
	@echo ""
	@echo "1. Checking binary availability..."
	@if [ -f target/release/ragentop ]; then \
		echo "   ✓ CLI binary available"; \
	else \
		echo "   ✗ CLI binary not found. Run 'make build' first."; \
		exit 1; \
	fi
	@echo ""
	@echo "2. Testing adapter detection..."
	@for adapter in $(ADAPTERS); do \
		if [ -d "crates/adapters/$$adapter" ]; then \
			echo "   ✓ $$adapter"; \
		fi; \
	done
	@echo ""
	@echo "3. Workspace members:"
	@for member in $(WORKSPACE_MEMBERS); do \
		if [ -d "crates/$$member" ]; then \
			echo "   ✓ $$member"; \
		fi; \
	done
	@echo ""
	@echo "Demonstration complete!"
	@echo "To run ragentop:"
	@echo "  ./target/release/ragentop --help"

demo-adapters: build ## Demonstrate adapter capabilities (LIVE)
	@echo "=== Adapter Detection Demo (LIVE) ==="
	@echo ""
	@./target/release/ragentop detect --verbose 2>/dev/null || ./target/release/ragentop detect
	@echo ""
	@echo "Supported adapters: $(ADAPTERS)"

demo-architecture: ## Show architecture overview
	@echo "=== ragentop Architecture ==="
	@echo ""
	@echo "FUNCTIONAL CORE (ragentop-core)"
	@echo "  ├── Pure functions only"
	@echo "  ├── Types, traits, DAG operations"
	@echo "  └── Test target: 90% coverage"
	@echo ""
	@echo "HEXAGONAL BOUNDARY"
	@echo "  ├── Ports = traits in core"
	@echo "  └── Adapters = implementations"
	@echo ""
	@echo "IMPERATIVE SHELL"
	@echo "  ├── daemon: Background collector"
	@echo "  ├── tui: Terminal UI (ratatui)"
	@echo "  ├── web: Web UI (leptos)"
	@echo "  └── cli: Command-line interface"
	@echo ""
	@echo "AGENT ADAPTERS"
	@for adapter in $(ADAPTERS); do \
		echo "  ├── $$adapter"; \
	done
	@echo "  └── (each implements AgentAdapter trait)"

demo-all: demo-types demo-detection demo-metrics demo-history demo-dag demo-multiplexer demo-protocol demo-tracking ## Run all functionality demos

demo-types: ## Demonstrate core type system
	@echo "=== Core Type System Demo ==="
	@echo ""
	@echo "Agent Types Supported:"
	@echo "  - Claude: Anthropic's Claude Code"
	@echo "  - Codex: OpenAI's Codex CLI"
	@echo "  - Gemini: Google's Gemini CLI"
	@echo "  - Copilot: GitHub's Copilot CLI"
	@echo "  - Qwen: Alibaba's Qwen CLI"
	@echo "  - GLM: 智谱 GLM (Claude proxy)"
	@echo ""
	@echo "Session Status Types:"
	@echo "  - Active: Currently running session"
	@echo "  - Idle: Session exists but inactive"
	@echo "  - Paused: Session paused by user"
	@echo ""
	@echo "History Depth Levels:"
	@echo "  - ToolCallsOnly: Level 1 - tool calls only"
	@echo "  - WithResponses: Level 2 - with abbreviated responses (default)"
	@echo "  - FullConversation: Level 3 - full conversation turns"
	@echo ""
	@echo "Command Status Types:"
	@echo "  - Success: Command completed successfully"
	@echo "  - Failed: Command failed"
	@echo "  - Running: Command currently executing"
	@echo ""
	@echo "Run tests to verify type system:"
	@echo "  make test-core"

demo-detection: build ## Demonstrate agent session detection (LIVE)
	@echo "=== Agent Session Detection Demo (LIVE) ==="
	@echo ""
	@echo "Active = running claude process OR modified within 5 minutes"
	@echo ""
	@./target/release/ragentop detect --verbose

demo-metrics: build ## Demonstrate metrics collection (LIVE)
	@echo "=== Metrics Collection Demo (LIVE) ==="
	@echo ""
	@echo "Detecting sessions on this machine..."
	@./target/release/ragentop detect
	@echo ""
	@echo "Note: Full metrics polling requires running daemon."
	@echo "Start daemon with: ./target/release/ragentop daemon start"

demo-history: ## Demonstrate command history retrieval (LIVE)
	@echo "=== Command History Demo (LIVE) ==="
	@echo ""
	@echo "Claude Code session history locations:"
	@if [ -d "$$HOME/.claude/projects" ]; then \
		echo "  $$HOME/.claude/projects/"; \
		find "$$HOME/.claude/projects" -name "*.jsonl" -type f 2>/dev/null | head -5 | while IFS= read -r f; do \
			echo "    - $$f ($$(wc -l < "$$f") entries)"; \
		done || echo "    (no JSONL history files found)"; \
	else \
		echo "  (no Claude projects directory found)"; \
	fi
	@echo ""
	@echo "History depth levels: ToolCallsOnly, WithResponses, FullConversation"

demo-dag: ## Demonstrate Merkle DAG storage (LIVE)
	@echo "=== Merkle DAG Storage Demo (LIVE) ==="
	@echo ""
	@DAG_DIR="$$HOME/.local/share/ragentop/dag"; \
	if [ -d "$$DAG_DIR" ]; then \
		echo "DAG storage: $$DAG_DIR"; \
		echo "  Size: $$(du -sh "$$DAG_DIR" 2>/dev/null | cut -f1)"; \
		echo "  Files: $$(find "$$DAG_DIR" -type f 2>/dev/null | wc -l)"; \
	else \
		echo "DAG storage: $$DAG_DIR (not created yet)"; \
		echo "  Will be created when daemon starts"; \
	fi
	@echo ""
	@echo "Backend: sled (embedded), Hash: BLAKE3"

demo-multiplexer: ## Demonstrate terminal multiplexer integration (LIVE)
	@echo "=== Terminal Multiplexer Demo (LIVE) ==="
	@echo ""
	@if command -v tmux >/dev/null 2>&1 && tmux info >/dev/null 2>&1; then \
		echo "tmux panes:"; \
		tmux list-panes -a -F '  #{pane_id}: #{pane_title} (#{pane_current_command})' 2>/dev/null || echo "  (no panes)"; \
	else \
		echo "tmux: not running or not installed"; \
	fi
	@echo ""
	@if command -v zellij >/dev/null 2>&1; then \
		echo "zellij: installed"; \
		zellij list-sessions 2>/dev/null || echo "  (no sessions)"; \
	else \
		echo "zellij: not installed"; \
	fi

demo-protocol: ## Demonstrate daemon protocol (LIVE)
	@echo "=== Daemon Protocol Demo (LIVE) ==="
	@echo ""
	@SOCKET="/tmp/ragentop.sock"; \
	if [ -S "$$SOCKET" ]; then \
		echo "Daemon socket: $$SOCKET (ACTIVE)"; \
		ls -la "$$SOCKET"; \
	else \
		echo "Daemon socket: $$SOCKET (not running)"; \
		echo "  Start with: ./target/release/ragentop daemon start"; \
	fi
	@echo ""
	@echo "Protocol: JSON over Unix socket"

demo-tracking: build ## Demonstrate session tracking (LIVE)
	@echo "=== Session Tracking Demo (LIVE) ==="
	@echo ""
	@echo "Currently detected sessions:"
	@./target/release/ragentop detect 2>/dev/null || echo "  (run 'make build' first)"
	@echo ""
	@SOCKET="/tmp/ragentop.sock"; \
	if [ -S "$$SOCKET" ]; then \
		echo "Daemon: running (query via socket for live tracking)"; \
	else \
		echo "Daemon: not running"; \
		echo "  Start with: ./target/release/ragentop daemon start"; \
	fi

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
