# ragentop MVP Design Specification

**Date:** 2025-01-25
**Version:** v0.1.0
**Status:** Approved for implementation

---

## 1. Project Overview & Goals

ragentop is a Rust-based monitoring tool for AI coding agents. It runs as a persistent background daemon that continuously collects agent metrics. The `ragentop` command provides CLI access, with `ragtop` as a convenience alias.

### Primary Goals

1. **Unified monitoring** - Single dashboard for Claude, Codex, Gemini, Copilot, Qwen, and GLM-4.7 agents
2. **Remote-friendly** - Designed for SSH access via Termius; works in constrained terminal environments
3. **Non-intrusive** - Runs in a dedicated pane without disrupting other zellij/tmux panes
4. **Versioned state** - Merkle DAG storage enables session replay, deduplication, and audit trails

### Execution Model

```
┌─────────────────────────────────────────────────────────────┐
│ Desktop (always running)                                    │
│  ┌─────────────────┐                                        │
│  │ ragentop-daemon │ ← collects metrics continuously        │
│  │ (systemd/bg)    │                                        │
│  └────────┬────────┘                                        │
│           │ Unix socket                                     │
│           ├──────────────┬──────────────┐                   │
│           ▼              ▼              ▼                   │
│    ┌───────────┐  ┌───────────┐  ┌───────────┐              │
│    │ ragtop    │  │ ragtop    │  │ web UI    │              │
│    │ (local)   │  │ (over SSH)│  │ (browser) │              │
│    └───────────┘  └───────────┘  └───────────┘              │
└─────────────────────────────────────────────────────────────┘
```

### CLI Commands

| Command | Description |
|---------|-------------|
| `ragentop daemon` | Start the background collector |
| `ragentop tui` / `ragtop` | Open TUI dashboard |
| `ragentop status` | Quick one-liner of active agents |
| `ragentop web` | Start web UI (localhost:8080) |

### MVP Scope

| Included (v0.1.0) | Deferred |
|-------------------|----------|
| Dashboard panel TUI | Tab-based view (v0.2.0) |
| All 6 agent adapters (best-effort) | Tree view (v0.2.0) |
| Merkle DAG session storage | Syntax-aware code diffing (v0.1.1) |
| Session replay | Token auth for web (v0.3.0) |
| Zellij/tmux bidirectional sync | mTLS (v0.3.0) |
| Local + SSH socket web access | |
| Selectable command history depth | |

### Non-Goals

- **Not a session manager** - Use agent-of-empires for that; ragentop is read-mostly monitoring
- **Not an agent launcher** - Monitors existing processes, doesn't start them
- **Not a cost optimizer** - Shows token usage but doesn't suggest optimizations

---

## 2. Architecture & Crate Structure

### Workspace Layout

```
ragentop/
├── Cargo.toml                    # Workspace manifest
├── crates/
│   ├── ragentop-core/            # Shared types, traits, config
│   ├── ragentop-daemon/          # Background data collector
│   ├── ragentop-tui/             # Ratatui dashboard
│   ├── ragentop-web/             # Leptos SSR interface
│   ├── ragentop-cli/             # Unified CLI binary
│   └── adapters/
│       ├── adapter-claude/
│       ├── adapter-codex/
│       ├── adapter-gemini/
│       ├── adapter-copilot/
│       ├── adapter-qwen/
│       └── adapter-glm/
├── tests/                        # Integration tests
└── docs/plans/
```

### Crate Responsibilities

| Crate | Responsibility | Dependencies |
|-------|---------------|--------------|
| ragentop-core | Types, traits, Merkle DAG, config parsing | None (pure) |
| ragentop-daemon | File watching, process detection, socket API | core, adapters |
| ragentop-tui | Dashboard rendering, keyboard handling | core, ratatui |
| ragentop-web | Leptos components, SSR, socket client | core, leptos |
| ragentop-cli | Argument parsing, subcommand dispatch | daemon, tui, web |
| adapter-* | Agent-specific data extraction | core |

### Key Traits

```rust
pub trait AgentAdapter: Send + Sync {
    fn agent_type(&self) -> AgentType;
    fn detect_sessions(&self) -> Result<Vec<AgentSession>>;
    fn poll_metrics(&self, session: &AgentSession) -> Result<SessionMetrics>;
    fn get_command_history(&self, session: &AgentSession, depth: HistoryDepth) -> Result<Vec<Command>>;
    fn capabilities(&self) -> AdapterCapabilities;
    fn config_dir(&self) -> PathBuf;
}

pub struct AdapterCapabilities {
    pub tokens: bool,
    pub cost: bool,
    pub commands: bool,
    pub model_info: bool,
    pub session_replay: bool,
}

pub enum HistoryDepth {
    ToolCallsOnly,      // Level 1
    WithResponses,      // Level 2
    FullConversation,   // Level 3
}
```

---

## 3. Merkle DAG Session Storage

### Purpose

Every agent session is stored as a content-addressed DAG, enabling:
- **Session replay** - Navigate to any point in history
- **Deduplication** - Identical content shares storage across sessions
- **Audit trail** - Cryptographic proof of what happened when

### Data Model

```
SessionRoot (hash: abc123)
    │
    ├── StateNode @t1 (hash: def456)
    │   ├── commands: [Command]
    │   ├── metrics: SessionMetrics
    │   └── parent: None
    │
    ├── StateNode @t2 (hash: ghi789)
    │   ├── commands: [Command]
    │   ├── metrics: SessionMetrics
    │   └── parent: def456
    │
    └── StateNode @t3 (hash: jkl012) ← current
        ├── commands: [Command]
        ├── metrics: SessionMetrics
        └── parent: ghi789
```

### Storage Backend

```rust
pub struct DagStore {
    blobs: sled::Db,  // Content-addressed blob storage
    roots: HashMap<SessionId, Hash>,  // Session root pointers
}

impl DagStore {
    fn store(&self, node: &StateNode) -> Hash;
    fn load(&self, hash: &Hash) -> Option<StateNode>;
    fn walk_history(&self, from: Hash) -> impl Iterator<Item = StateNode>;
}
```

### Scope

| v0.1.0 | v0.1.1 |
|--------|--------|
| DAG storage layer | Tree-sitter AST parsing |
| Session state snapshots | Syntax-aware diffing |
| History traversal | Cross-session search |
| Basic deduplication | Advanced compression |

---

## 4. Adapters & Agent Support

### Data Sources

| Agent | Config Dir | Session Data | Token/Usage |
|-------|-----------|--------------|-------------|
| Claude Code | `~/.claude/` | `projects/*/`, `stats-cache.json` | stats-cache.json |
| Codex | `~/.codex/` | Session files | `/usage` API |
| Gemini CLI | `~/.gemini/` | `tmp/<hash>/shell_history`, OTEL logs | OpenTelemetry |
| Copilot CLI | `~/.copilot/` | Session logs, trusted dirs | `/usage` command |
| Qwen Code | `~/.qwen/` | `logs/openai/` (JSON) | `/stats` command |
| GLM-4 | `~/.glm/` | Via underlying agent | Via underlying agent |

### Detailed Source Mapping

```
Gemini CLI (~/.gemini/)
├── settings.json              # Config, telemetry settings
├── tmp/<project_hash>/
│   ├── shell_history          # Command history
│   └── otel/collector-gcp.log # OpenTelemetry metrics
└── telemetry.log              # Optional telemetry output

Copilot CLI (~/.copilot/)
├── config.json                # Settings, debug level
├── config                     # URL allowlists, trusted dirs
└── [session logs via API]     # Token usage, session length

Qwen Code (~/.qwen/)
├── settings.json              # Token limits, model config
└── logs/openai/*.json         # Request/response logs
```

### Capability Matrix

| Feature | Claude | Codex | Gemini | Copilot | Qwen | GLM |
|---------|--------|-------|--------|---------|------|-----|
| Session detection | Full | Full | Full | Full | Full | Full* |
| Token count | Full | Full | Full | Full | Full | Full* |
| Cost estimate | Full | Full | Full | Partial | Full | Full* |
| Command history | Full | Full | Full | Full | Full | Full* |
| Session replay | Full | Full | Full | Partial | Full | Partial |
| Model info | Full | Full | Full | Full | Full | Full |

*Via underlying Claude Code session

---

## 5. Zellij/Tmux Integration

### Three-Tier Detection Strategy

```
Tier 1: Bidirectional Sync (Best)
├── Detect multiplexer type (zellij/tmux)
├── Query pane info via CLI
├── Auto-name panes based on agent session
└── Respect user-set names

Tier 2: Read-Only Detection (Fallback)
├── Query pane names without modifying
├── Match by PID/cwd correlation
└── Display pane context in TUI

Tier 3: Naming Convention (Final Fallback)
├── Parse pane names matching pattern: agent-<type>-<id>
├── Works without zellij/tmux installed
└── Manual user setup required
```

### Multiplexer Detection

```rust
pub enum Multiplexer {
    Zellij { session: String, pane_id: u32 },
    Tmux { session: String, window: u32, pane: u32 },
    None,
}

impl Multiplexer {
    pub fn detect() -> Self {
        if std::env::var("ZELLIJ_SESSION_NAME").is_ok() {
            Multiplexer::Zellij { .. }
        } else if std::env::var("TMUX").is_ok() {
            Multiplexer::Tmux { .. }
        } else {
            Multiplexer::None
        }
    }
}
```

### Pane Commands

| Multiplexer | Read Pane Name | Set Pane Name |
|-------------|----------------|---------------|
| Zellij | `zellij action query-tab-names` | `zellij action rename-pane "name"` |
| Tmux | `tmux display -p '#{pane_title}'` | `tmux select-pane -T "name"` |

### Auto-Naming Format

```
<agent>: <session_summary> (<model>)

Examples:
  claude: fix-auth-bug (opus)
  claude: docs-update (sonnet)
  codex: refactor-api (gpt-4o)
  gemini: test-coverage (2.5-pro)
  qwen: api-client (qwen3-coder)
  glm: backend-work (glm-4.7)
```

---

## 6. Input Architecture

### Three Input Modes

| Mode | Detection | Capabilities |
|------|-----------|--------------|
| Mobile | `$TERM` hints, narrow width | Single-key + command mode |
| Keyboard | Default | Full keybindings |
| Mouse | Terminal capability query | Click, scroll, drag |

### Keybinding Matrix

| Action | Mobile (Termius) | Full Keyboard |
|--------|------------------|---------------|
| Navigate | `j`/`k` | `j`/`k`, `↑`/`↓` |
| Select session | `1`-`9` | `1`-`9`, `Enter` |
| Expand/collapse | `Space` | `Space`, `Enter` |
| Cycle depth | `d` | `d` |
| Filter | `:filter` | `f` |
| Search | `:search` | `/` |
| History/replay | `:jump` | `H`, `[`, `]` |
| Help | `?` | `?`, `F1` |
| Quit | `q` | `q`, `Ctrl+c` |
| Refresh | `r` | `r`, `F5` |

### Mouse Support

| Action | Mouse Input |
|--------|-------------|
| Select session | Click row |
| Expand/collapse | Double-click row |
| Scroll session list | Scroll wheel in list pane |
| Scroll command history | Scroll wheel in detail pane |

### Configuration

```toml
# ~/.config/ragentop/config.toml
[input]
mouse_enabled = true
scroll_lines = 3

[keybindings]
preset = "full"  # "full" | "mobile" | "vim" | "custom"
```

---

## 7. TUI Design

### Design Direction: Precision & Density

Following interface-design principles - tight, technical, monochromatic.

### Color System (Terminal 256)

```
Background layers:
  bg-base:     Color::Rgb(13, 13, 13)
  bg-surface:  Color::Rgb(20, 20, 20)
  bg-selected: Color::Rgb(38, 38, 38)

Text hierarchy:
  text-primary:   Color::Rgb(229, 229, 229)
  text-secondary: Color::Rgb(163, 163, 163)
  text-muted:     Color::Rgb(82, 82, 82)

Status:
  active:  Color::Green
  idle:    Color::Yellow
  paused:  Color::DarkGray
  error:   Color::Red

Agent accents:
  claude:  Color::Rgb(217, 119, 6)   // amber
  codex:   Color::Rgb(34, 197, 94)   // green
  gemini:  Color::Rgb(59, 130, 246)  // blue
  copilot: Color::Rgb(139, 92, 246)  // purple
  qwen:    Color::Rgb(6, 182, 212)   // cyan
  glm:     Color::Rgb(236, 72, 153)  // pink
```

### Layout Structure

```
┌─ ragentop ─────────────────────────────────────────────────────┐
│ Summary Bar (1 line)                                           │
├────────────────────────────────────────────────────────────────┤
│ Session List Panel (scrollable, ~40% height)                   │
├────────────────────────────────────────────────────────────────┤
│ Detail Panel (selected session, ~50% height)                   │
├────────────────────────────────────────────────────────────────┤
│ Keybinding Hints (1 line)                                      │
└────────────────────────────────────────────────────────────────┘
```

### Full Dashboard

```
┌─ ragentop ──────────────────────────────────────────────────────┐
│ ▓▓▓▓▓░░░░░ 3 active │ 67.3k tokens │ $4.21 │ ⏱ 2h 14m uptime   │
├─────────────────────────────────────────────────────────────────┤
│ AGENT     MODEL        SESSION          PANE         TOKENS    │
│ ───────────────────────────────────────────────────────────────│
│▸claude    opus         fix-auth-bug     zj:main:2    45.2k     │
│ claude    sonnet       docs-update      zj:main:4    12.1k     │
│ codex     gpt-4o       refactor-api     zj:main:3     8.4k     │
│ gemini    2.5-pro      test-coverage    tmux:dev     10.0k     │
│ qwen      qwen3-coder  api-client       zj:work:1     6.2k     │
├─────────────────────────────────────────────────────────────────┤
│ ▶ claude: fix-auth-bug (opus) [zj:main:2]                       │
│ ───────────────────────────────────────────────────────────────│
│ Tokens: 45,231 │ Cost: $1.82 │ Duration: 24m │ CPU: 12%        │
│                                                                 │
│ Recent Commands                                           [d:2] │
│ 14:37  Read   src/auth/token.rs              ✓   312 lines     │
│ 14:36  Bash   cargo test auth                ✓   14 passed     │
│ 14:35  Edit   src/auth/middleware.rs         ✓   +12 -3        │
│ 14:33  Read   src/auth/middleware.rs         ✓   245 lines     │
│ 14:32  Bash   git status                     ✓   3 modified    │
├─────────────────────────────────────────────────────────────────┤
│ [j/k] navigate  [Space] expand  [d] depth  [q] quit  [?] help   │
└─────────────────────────────────────────────────────────────────┘
```

### Responsive Breakpoints

| Width | Adaptation |
|-------|------------|
| < 60 cols | Hide PANE, MODEL columns |
| < 80 cols | Hide CPU column, truncate SESSION to 12 |
| < 100 cols | Truncate SESSION to 20 |
| >= 100 cols | Full layout |

### ASCII Fallback Mode (`--ascii`)

| Unicode | ASCII |
|---------|-------|
| `●` | `[*]` |
| `◐` | `[-]` |
| `○` | `[ ]` |
| `─` | `-` |
| `│` | `\|` |
| `┌┐└┘` | `+` |

---

## 8. Web Dashboard (Leptos)

### Design Direction: Precision & Density

Same principles as TUI, adapted for browser.

### Color System

```css
:root {
  /* Background layers */
  --bg-base: #0d0d0d;
  --bg-surface: #141414;
  --bg-elevated: #1a1a1a;

  /* Text hierarchy */
  --text-primary: #e5e5e5;
  --text-secondary: #a3a3a3;
  --text-muted: #525252;

  /* Status */
  --status-active: #22c55e;
  --status-idle: #eab308;
  --status-paused: #6b7280;
  --status-error: #ef4444;

  /* Agent accents */
  --agent-claude: #d97706;
  --agent-codex: #22c55e;
  --agent-gemini: #3b82f6;
  --agent-copilot: #8b5cf6;
  --agent-qwen: #06b6d4;
  --agent-glm: #ec4899;

  /* Typography */
  --font-mono: 'JetBrains Mono', monospace;
  --text-xs: 11px;
  --text-sm: 12px;
  --text-base: 13px;
  --text-lg: 14px;
  --text-xl: 16px;

  /* Spacing */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-6: 24px;
  --space-8: 32px;
}
```

### Access Model

| Mode | Binding | Use Case |
|------|---------|----------|
| Local only | `127.0.0.1:8080` | Default, requires SSH tunnel |
| SSH socket | Unix socket via SSH | No HTTP server, stream over SSH |

---

## 9. Testing Strategy

### Iron Law: TDD/BDD First

Every feature starts with a failing test. No implementation without documented failure.

### Test Pyramid

```
          ╱╲
         ╱  ╲     E2E Tests (5%)
        ╱────╲
       ╱      ╲   Integration Tests (25%)
      ╱────────╲
     ╱          ╲ Unit Tests (70%)
    ╱────────────╲
```

### Test Organization

```
crates/
├── ragentop-core/tests/
│   ├── dag_storage_test.rs
│   ├── config_parsing_test.rs
│   └── types_test.rs
├── ragentop-daemon/tests/
│   ├── session_detection_test.rs
│   ├── socket_api_test.rs
│   └── adapter_integration_test.rs
├── ragentop-tui/tests/
│   ├── layout_test.rs
│   ├── keybinding_test.rs
│   └── state_machine_test.rs
├── adapters/adapter-claude/tests/
│   ├── stats_cache_parsing_test.rs
│   └── fixtures/
└── tests/  # Workspace integration
    └── daemon_tui_integration_test.rs
```

### Coverage Targets

| Crate | Target |
|-------|--------|
| ragentop-core | 90% |
| ragentop-daemon | 80% |
| ragentop-tui | 70% |
| adapters/* | 85% |

### CI Pipeline

```yaml
test:
  - cargo fmt --check
  - cargo clippy -- -D warnings
  - cargo test --workspace
  - cargo test --workspace -- --ignored
  - cargo llvm-cov --workspace --lcov > coverage.lcov
```

---

## 10. Implementation Phases

### Phase 1: Foundation

| Task | Crate | Deliverable |
|------|-------|-------------|
| Workspace init | root | Cargo.toml with all crates |
| CI pipeline | .github/ | fmt, clippy, test, coverage |
| Core types | ragentop-core | AgentType, Session, Metrics |
| Adapter trait | ragentop-core | AgentAdapter, AdapterCapabilities |
| Config parsing | ragentop-core | TOML config, XDG paths |
| Merkle DAG | ragentop-core | DagStore, content-addressed nodes |

### Phase 2: Daemon & First Adapter

| Task | Crate | Deliverable |
|------|-------|-------------|
| Daemon skeleton | ragentop-daemon | Process loop, signal handling |
| Unix socket API | ragentop-daemon | JSON-RPC over socket |
| Adapter registry | ragentop-daemon | Dynamic adapter loading |
| Claude adapter | adapter-claude | Full implementation |
| Session polling | ragentop-daemon | 2-second detection loop |

### Phase 3: TUI

| Task | Crate | Deliverable |
|------|-------|-------------|
| App state machine | ragentop-tui | States, transitions |
| Dashboard layout | ragentop-tui | Summary, list, detail panels |
| Keybinding system | ragentop-tui | Mobile + full keyboard |
| Mouse support | ragentop-tui | Click, scroll handlers |
| Daemon client | ragentop-tui | Socket connection |

### Phase 4: Remaining Adapters

| Task | Crate | Deliverable |
|------|-------|-------------|
| Codex adapter | adapter-codex | API + file parsing |
| Gemini adapter | adapter-gemini | OTEL logs, shell_history |
| Copilot adapter | adapter-copilot | Config + /usage parsing |
| Qwen adapter | adapter-qwen | JSON logs, /stats |
| GLM adapter | adapter-glm | Claude session detection |
| Multiplexer integration | ragentop-daemon | Bidirectional pane sync |

### Phase 5: Web & Release

| Task | Crate | Deliverable |
|------|-------|-------------|
| Leptos setup | ragentop-web | SSR scaffold |
| Dashboard components | ragentop-web | Sessions, detail, charts |
| SSH socket mode | ragentop-web | Stream over SSH |
| CLI polish | ragentop-cli | Help, completions |
| Release | root | cargo dist, binaries |

---

## 11. Future Versions

| Version | Features |
|---------|----------|
| v0.1.1 | Tree-sitter AST, syntax-aware diffing |
| v0.2.0 | Tab-based view, tree view, context menu |
| v0.3.0 | Token auth for web, dynamic adapter plugins |

### GitHub Issues (Deferred)

1. `[v0.1.1]` Tree-sitter AST integration for code diffing
2. `[v0.1.1]` Syntax-aware session replay diffs
3. `[v0.2.0]` Tab-based view by agent type
4. `[v0.2.0]` Tree view hierarchical layout
5. `[v0.2.0]` Right-click context menu (mouse)
6. `[v0.3.0]` Token authentication for web dashboard
7. `[v0.3.0]` mTLS support
8. `[v0.3.0]` Dynamic adapter plugin loading

---

## References

- [Gemini CLI Configuration](https://geminicli.com/docs/get-started/configuration/)
- [Gemini CLI Telemetry](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/telemetry.md)
- [GitHub Copilot CLI Docs](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/use-copilot-cli)
- [Qwen Code GitHub](https://github.com/QwenLM/qwen-code)
- [GLM CLI Tool](https://github.com/xqsit94/glm)
- [interface-design](https://github.com/Dammyjay93/interface-design)
- [agent-of-empires](https://github.com/njbrake/agent-of-empires)
- [rtop](https://github.com/Bored-UI/rtop)
- [agentop](https://github.com/dadwadw233/agentop)
