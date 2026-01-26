# ragentop Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust TUI/Web monitoring tool for AI coding agents (Claude, Codex, Gemini, Copilot, Qwen, GLM) with Merkle DAG session storage and zellij/tmux integration.

**Architecture:** Daemon-based architecture with Unix socket API. Background daemon collects metrics continuously; TUI/Web/CLI clients connect to daemon. Cargo workspace with separate crates for core types, daemon, TUI (ratatui), web (leptos), CLI, and per-agent adapters.

**Tech Stack:** Rust 2021, ratatui, leptos, sled (DAG storage), tokio, serde, clap

---

## Phase 1: Foundation

### Task 1.1: Initialize Cargo Workspace

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/ragentop-core/Cargo.toml`
- Create: `crates/ragentop-core/src/lib.rs`
- Create: `crates/ragentop-daemon/Cargo.toml`
- Create: `crates/ragentop-daemon/src/lib.rs`
- Create: `crates/ragentop-tui/Cargo.toml`
- Create: `crates/ragentop-tui/src/lib.rs`
- Create: `crates/ragentop-web/Cargo.toml`
- Create: `crates/ragentop-web/src/lib.rs`
- Create: `crates/ragentop-cli/Cargo.toml`
- Create: `crates/ragentop-cli/src/main.rs`
- Create: `crates/adapters/adapter-claude/Cargo.toml`
- Create: `crates/adapters/adapter-claude/src/lib.rs`
- Create: `crates/adapters/adapter-codex/Cargo.toml`
- Create: `crates/adapters/adapter-codex/src/lib.rs`
- Create: `crates/adapters/adapter-gemini/Cargo.toml`
- Create: `crates/adapters/adapter-gemini/src/lib.rs`
- Create: `crates/adapters/adapter-copilot/Cargo.toml`
- Create: `crates/adapters/adapter-copilot/src/lib.rs`
- Create: `crates/adapters/adapter-qwen/Cargo.toml`
- Create: `crates/adapters/adapter-qwen/src/lib.rs`
- Create: `crates/adapters/adapter-glm/Cargo.toml`
- Create: `crates/adapters/adapter-glm/src/lib.rs`

**Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/ragentop-core",
    "crates/ragentop-daemon",
    "crates/ragentop-tui",
    "crates/ragentop-web",
    "crates/ragentop-cli",
    "crates/adapters/adapter-claude",
    "crates/adapters/adapter-codex",
    "crates/adapters/adapter-gemini",
    "crates/adapters/adapter-copilot",
    "crates/adapters/adapter-qwen",
    "crates/adapters/adapter-glm",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/alext/ragentop"

[workspace.dependencies]
# Core
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Config
toml = "0.8"
directories = "5.0"

# Storage
sled = "0.34"
blake3 = "1.5"

# TUI
ratatui = "0.29"
crossterm = "0.28"

# Web
leptos = { version = "0.7", features = ["ssr"] }

# CLI
clap = { version = "4.5", features = ["derive"] }

# Testing
rstest = "0.23"
tempfile = "3.14"
```

**Step 2: Create directory structure**

Run:
```bash
mkdir -p crates/ragentop-core/src
mkdir -p crates/ragentop-daemon/src
mkdir -p crates/ragentop-tui/src
mkdir -p crates/ragentop-web/src
mkdir -p crates/ragentop-cli/src
mkdir -p crates/adapters/adapter-claude/src
mkdir -p crates/adapters/adapter-codex/src
mkdir -p crates/adapters/adapter-gemini/src
mkdir -p crates/adapters/adapter-copilot/src
mkdir -p crates/adapters/adapter-qwen/src
mkdir -p crates/adapters/adapter-glm/src
```

**Step 3: Create ragentop-core Cargo.toml**

```toml
[package]
name = "ragentop-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
toml.workspace = true
directories.workspace = true
sled.workspace = true
blake3.workspace = true

[dev-dependencies]
rstest.workspace = true
tempfile.workspace = true
```

**Step 4: Create ragentop-core/src/lib.rs**

```rust
//! Core types, traits, and utilities for ragentop.

pub mod config;
pub mod dag;
pub mod error;
pub mod types;

pub use config::Config;
pub use error::{Error, Result};
pub use types::*;
```

**Step 5: Create ragentop-daemon Cargo.toml**

```toml
[package]
name = "ragentop-daemon"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
ragentop-core = { path = "../ragentop-core" }
adapter-claude = { path = "../adapters/adapter-claude" }
adapter-codex = { path = "../adapters/adapter-codex" }
adapter-gemini = { path = "../adapters/adapter-gemini" }
adapter-copilot = { path = "../adapters/adapter-copilot" }
adapter-qwen = { path = "../adapters/adapter-qwen" }
adapter-glm = { path = "../adapters/adapter-glm" }
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
tracing.workspace = true
anyhow.workspace = true

[dev-dependencies]
rstest.workspace = true
tempfile.workspace = true
```

**Step 6: Create ragentop-daemon/src/lib.rs**

```rust
//! Background daemon for collecting agent metrics.

pub mod server;
pub mod session;
pub mod registry;
```

**Step 7: Create ragentop-tui Cargo.toml**

```toml
[package]
name = "ragentop-tui"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
ragentop-core = { path = "../ragentop-core" }
ratatui.workspace = true
crossterm.workspace = true
serde.workspace = true
tokio.workspace = true
anyhow.workspace = true

[dev-dependencies]
rstest.workspace = true
```

**Step 8: Create ragentop-tui/src/lib.rs**

```rust
//! Terminal user interface for ragentop.

pub mod app;
pub mod ui;
pub mod input;
```

**Step 9: Create ragentop-web Cargo.toml**

```toml
[package]
name = "ragentop-web"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
ragentop-core = { path = "../ragentop-core" }
leptos.workspace = true
serde.workspace = true
tokio.workspace = true

[dev-dependencies]
rstest.workspace = true
```

**Step 10: Create ragentop-web/src/lib.rs**

```rust
//! Web dashboard for ragentop using Leptos.

pub mod components;
pub mod pages;
```

**Step 11: Create ragentop-cli Cargo.toml**

```toml
[package]
name = "ragentop-cli"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "ragentop"
path = "src/main.rs"

[dependencies]
ragentop-core = { path = "../ragentop-core" }
ragentop-daemon = { path = "../ragentop-daemon" }
ragentop-tui = { path = "../ragentop-tui" }
ragentop-web = { path = "../ragentop-web" }
clap.workspace = true
tokio.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
anyhow.workspace = true

[dev-dependencies]
rstest.workspace = true
```

**Step 12: Create ragentop-cli/src/main.rs**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ragentop")]
#[command(about = "Monitor AI coding agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the background daemon
    Daemon,
    /// Open the TUI dashboard
    Tui,
    /// Show quick status
    Status,
    /// Start the web dashboard
    Web,
}

fn main() {
    let cli = Cli::parse();
    println!("ragentop v{}", env!("CARGO_PKG_VERSION"));
}
```

**Step 13: Create adapter crate templates**

For each adapter (claude, codex, gemini, copilot, qwen, glm), create:

`crates/adapters/adapter-claude/Cargo.toml`:
```toml
[package]
name = "adapter-claude"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
ragentop-core = { path = "../../ragentop-core" }
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
tracing.workspace = true

[dev-dependencies]
rstest.workspace = true
tempfile.workspace = true
```

`crates/adapters/adapter-claude/src/lib.rs`:
```rust
//! Claude Code adapter for ragentop.

use ragentop_core::{AgentAdapter, AgentType};

pub struct ClaudeAdapter;
```

Repeat similar structure for other adapters (codex, gemini, copilot, qwen, glm).

**Step 14: Verify workspace builds**

Run: `cargo check --workspace`
Expected: Compiles with no errors

**Step 15: Commit**

```bash
git add -A
git commit -m "feat: initialize Cargo workspace with all crate skeletons"
```

---

### Task 1.2: Core Types

**Files:**
- Create: `crates/ragentop-core/src/types.rs`
- Create: `crates/ragentop-core/src/error.rs`
- Test: `crates/ragentop-core/src/types.rs` (inline tests)

**Step 1: Write the failing test for AgentType**

Create `crates/ragentop-core/src/types.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Supported AI coding agent types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Claude,
    Codex,
    Gemini,
    Copilot,
    Qwen,
    Glm,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Claude => write!(f, "claude"),
            AgentType::Codex => write!(f, "codex"),
            AgentType::Gemini => write!(f, "gemini"),
            AgentType::Copilot => write!(f, "copilot"),
            AgentType::Qwen => write!(f, "qwen"),
            AgentType::Glm => write!(f, "glm"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_display() {
        assert_eq!(AgentType::Claude.to_string(), "claude");
        assert_eq!(AgentType::Codex.to_string(), "codex");
    }

    #[test]
    fn test_agent_type_serde() {
        let json = serde_json::to_string(&AgentType::Claude).unwrap();
        assert_eq!(json, "\"claude\"");

        let parsed: AgentType = serde_json::from_str("\"gemini\"").unwrap();
        assert_eq!(parsed, AgentType::Gemini);
    }
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test -p ragentop-core`
Expected: PASS

**Step 3: Add SessionId and SessionStatus types**

Append to `crates/ragentop-core/src/types.rs`:

```rust
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Unique identifier for an agent session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Current status of an agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Idle,
    Paused,
}

/// Information about a detected agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: SessionId,
    pub agent_type: AgentType,
    pub model: Option<String>,
    pub session_name: Option<String>,
    pub working_dir: Option<PathBuf>,
    pub pane_id: Option<String>,
    pub pid: Option<u32>,
    pub started_at: Option<SystemTime>,
    pub status: SessionStatus,
}

/// Metrics for a session at a point in time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub token_count: u64,
    pub cost_usd: Option<f64>,
    pub cpu_percent: Option<f32>,
    pub duration: Option<Duration>,
    pub command_count: u64,
}

/// A command executed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub timestamp: SystemTime,
    pub tool: String,
    pub args: String,
    pub status: CommandStatus,
    pub result_summary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandStatus {
    Success,
    Failed,
    Running,
}

/// History depth for command queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryDepth {
    /// Tool calls only (level 1)
    ToolCallsOnly,
    /// Tool calls with abbreviated responses (level 2)
    WithResponses,
    /// Full conversation turns (level 3)
    FullConversation,
}

impl Default for HistoryDepth {
    fn default() -> Self {
        Self::WithResponses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_display() {
        assert_eq!(AgentType::Claude.to_string(), "claude");
        assert_eq!(AgentType::Codex.to_string(), "codex");
    }

    #[test]
    fn test_agent_type_serde() {
        let json = serde_json::to_string(&AgentType::Claude).unwrap();
        assert_eq!(json, "\"claude\"");

        let parsed: AgentType = serde_json::from_str("\"gemini\"").unwrap();
        assert_eq!(parsed, AgentType::Gemini);
    }

    #[test]
    fn test_session_id() {
        let id = SessionId::new("session-123");
        assert_eq!(id.0, "session-123");
    }

    #[test]
    fn test_session_status_serde() {
        let json = serde_json::to_string(&SessionStatus::Active).unwrap();
        assert_eq!(json, "\"active\"");
    }

    #[test]
    fn test_history_depth_default() {
        assert_eq!(HistoryDepth::default(), HistoryDepth::WithResponses);
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p ragentop-core`
Expected: All tests PASS

**Step 5: Create error types**

Create `crates/ragentop-core/src/error.rs`:

```rust
use thiserror::Error;

/// Result type for ragentop operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in ragentop.
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Adapter error: {0}")]
    Adapter(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Config error: {0}")]
    Config(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::SessionNotFound("abc123".to_string());
        assert_eq!(err.to_string(), "Session not found: abc123");
    }
}
```

**Step 6: Run tests**

Run: `cargo test -p ragentop-core`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/ragentop-core/src/
git commit -m "feat(core): add core types - AgentType, Session, Metrics, Command"
```

---

### Task 1.3: Adapter Trait

**Files:**
- Create: `crates/ragentop-core/src/adapter.rs`
- Modify: `crates/ragentop-core/src/lib.rs`
- Test: inline tests

**Step 1: Create adapter trait**

Create `crates/ragentop-core/src/adapter.rs`:

```rust
use crate::{AgentSession, AgentType, Command, HistoryDepth, Result, SessionId, SessionMetrics};
use std::path::PathBuf;

/// Capabilities that an adapter supports.
#[derive(Debug, Clone, Default)]
pub struct AdapterCapabilities {
    /// Can read token counts
    pub tokens: bool,
    /// Can estimate costs
    pub cost: bool,
    /// Can read command history
    pub commands: bool,
    /// Can read model information
    pub model_info: bool,
    /// Can replay session history
    pub session_replay: bool,
}

/// Trait for agent-specific data extraction.
pub trait AgentAdapter: Send + Sync {
    /// Returns the agent type this adapter handles.
    fn agent_type(&self) -> AgentType;

    /// Returns the config directory for this agent.
    fn config_dir(&self) -> PathBuf;

    /// Discovers active sessions for this agent.
    fn detect_sessions(&self) -> Result<Vec<AgentSession>>;

    /// Polls current metrics for a session.
    fn poll_metrics(&self, session_id: &SessionId) -> Result<SessionMetrics>;

    /// Gets command history at the specified depth.
    fn get_command_history(
        &self,
        session_id: &SessionId,
        depth: HistoryDepth,
        limit: usize,
    ) -> Result<Vec<Command>>;

    /// Returns the capabilities of this adapter.
    fn capabilities(&self) -> AdapterCapabilities;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_capabilities_default() {
        let caps = AdapterCapabilities::default();
        assert!(!caps.tokens);
        assert!(!caps.cost);
        assert!(!caps.commands);
    }
}
```

**Step 2: Update lib.rs to export adapter module**

Modify `crates/ragentop-core/src/lib.rs`:

```rust
//! Core types, traits, and utilities for ragentop.

pub mod adapter;
pub mod config;
pub mod dag;
pub mod error;
pub mod types;

pub use adapter::{AdapterCapabilities, AgentAdapter};
pub use config::Config;
pub use error::{Error, Result};
pub use types::*;
```

**Step 3: Run tests**

Run: `cargo test -p ragentop-core`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/ragentop-core/src/
git commit -m "feat(core): add AgentAdapter trait and AdapterCapabilities"
```

---

### Task 1.4: Configuration System

**Files:**
- Create: `crates/ragentop-core/src/config.rs`
- Test: inline tests

**Step 1: Create config module**

Create `crates/ragentop-core/src/config.rs`:

```rust
use crate::{Error, HistoryDepth, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub tui: TuiConfig,
    pub web: WebConfig,
    pub input: InputConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daemon: DaemonConfig::default(),
            tui: TuiConfig::default(),
            web: WebConfig::default(),
            input: InputConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    /// Socket path for daemon communication
    pub socket_path: PathBuf,
    /// Polling interval in milliseconds
    pub poll_interval_ms: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let socket_path = Self::default_socket_path();
        Self {
            socket_path,
            poll_interval_ms: 2000,
        }
    }
}

impl DaemonConfig {
    fn default_socket_path() -> PathBuf {
        if let Some(runtime_dir) = dirs::runtime_dir() {
            runtime_dir.join("ragentop.sock")
        } else {
            PathBuf::from("/tmp/ragentop.sock")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TuiConfig {
    /// Default history depth
    pub default_depth: u8,
    /// Enable mouse support
    pub mouse_enabled: bool,
    /// Use ASCII-only characters
    pub ascii_mode: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            default_depth: 2,
            mouse_enabled: true,
            ascii_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebConfig {
    /// Bind address for web server
    pub bind_address: String,
    /// Port for web server
    pub port: u16,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    /// Lines to scroll per mouse wheel tick
    pub scroll_lines: u8,
    /// Keybinding preset
    pub keybinding_preset: String,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            scroll_lines: 3,
            keybinding_preset: "full".to_string(),
        }
    }
}

impl Config {
    /// Returns the default config directory path.
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("com", "ragentop", "ragentop").map(|p| p.config_dir().to_path_buf())
    }

    /// Returns the default config file path.
    pub fn config_file_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("config.toml"))
    }

    /// Loads config from the default path, or returns default if not found.
    pub fn load() -> Result<Self> {
        if let Some(path) = Self::config_file_path() {
            if path.exists() {
                let contents = std::fs::read_to_string(&path)?;
                let config: Config = toml::from_str(&contents)?;
                return Ok(config);
            }
        }
        Ok(Self::default())
    }

    /// Loads config from a specific path.
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.daemon.poll_interval_ms, 2000);
        assert_eq!(config.tui.default_depth, 2);
        assert!(config.tui.mouse_enabled);
        assert_eq!(config.web.port, 8080);
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.daemon.poll_interval_ms, config.daemon.poll_interval_ms);
    }

    #[test]
    fn test_config_partial_toml() {
        let toml_str = r#"
[daemon]
poll_interval_ms = 5000
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.daemon.poll_interval_ms, 5000);
        // Defaults should still apply
        assert!(config.tui.mouse_enabled);
    }
}
```

**Step 2: Add dirs dependency to workspace**

Update `Cargo.toml` workspace dependencies:

```toml
dirs = "5.0"
```

Update `crates/ragentop-core/Cargo.toml`:

```toml
dirs = "5.0"
```

**Step 3: Run tests**

Run: `cargo test -p ragentop-core`
Expected: PASS

**Step 4: Commit**

```bash
git add Cargo.toml crates/ragentop-core/
git commit -m "feat(core): add configuration system with TOML support"
```

---

### Task 1.5: Merkle DAG Storage

**Files:**
- Create: `crates/ragentop-core/src/dag.rs`
- Create: `crates/ragentop-core/src/dag/store.rs`
- Create: `crates/ragentop-core/src/dag/node.rs`
- Test: `crates/ragentop-core/tests/dag_test.rs`

**Step 1: Write the failing test**

Create `crates/ragentop-core/tests/dag_test.rs`:

```rust
use ragentop_core::dag::{DagStore, StateNode};
use tempfile::tempdir;

#[test]
fn test_dag_store_and_retrieve() {
    let dir = tempdir().unwrap();
    let store = DagStore::open(dir.path()).unwrap();

    let node = StateNode::new(vec![], None);
    let hash = store.store(&node).unwrap();

    let retrieved = store.load(&hash).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().parent, None);
}

#[test]
fn test_dag_parent_chain() {
    let dir = tempdir().unwrap();
    let store = DagStore::open(dir.path()).unwrap();

    // Create chain: node1 <- node2 <- node3
    let node1 = StateNode::new(vec![], None);
    let hash1 = store.store(&node1).unwrap();

    let node2 = StateNode::new(vec![], Some(hash1.clone()));
    let hash2 = store.store(&node2).unwrap();

    let node3 = StateNode::new(vec![], Some(hash2.clone()));
    let hash3 = store.store(&node3).unwrap();

    // Walk history from node3
    let history: Vec<_> = store.walk_history(&hash3).collect();
    assert_eq!(history.len(), 3);
}

#[test]
fn test_dag_content_addressing() {
    let dir = tempdir().unwrap();
    let store = DagStore::open(dir.path()).unwrap();

    let node1 = StateNode::new(vec![], None);
    let node2 = StateNode::new(vec![], None);

    let hash1 = store.store(&node1).unwrap();
    let hash2 = store.store(&node2).unwrap();

    // Same content = same hash
    assert_eq!(hash1, hash2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ragentop-core --test dag_test`
Expected: FAIL (dag module doesn't exist)

**Step 3: Create dag module structure**

Create `crates/ragentop-core/src/dag/mod.rs`:

```rust
//! Merkle DAG storage for versioned session state.

mod node;
mod store;

pub use node::StateNode;
pub use store::{DagStore, Hash};
```

Create `crates/ragentop-core/src/dag/node.rs`:

```rust
use crate::Command;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A node in the session state DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateNode {
    /// Commands executed at this state
    pub commands: Vec<Command>,
    /// Hash of parent node (None for root)
    pub parent: Option<super::Hash>,
    /// Timestamp of this state
    pub timestamp: SystemTime,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl StateNode {
    /// Creates a new state node.
    pub fn new(commands: Vec<Command>, parent: Option<super::Hash>) -> Self {
        Self {
            commands,
            parent,
            timestamp: SystemTime::now(),
            metadata: None,
        }
    }

    /// Creates a new state node with metadata.
    pub fn with_metadata(
        commands: Vec<Command>,
        parent: Option<super::Hash>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            commands,
            parent,
            timestamp: SystemTime::now(),
            metadata: Some(metadata),
        }
    }
}
```

Create `crates/ragentop-core/src/dag/store.rs`:

```rust
use super::StateNode;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Content-addressed hash.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub String);

impl Hash {
    /// Computes hash from bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self(hash.to_hex().to_string())
    }
}

/// Content-addressed DAG storage.
pub struct DagStore {
    db: sled::Db,
}

impl DagStore {
    /// Opens or creates a DAG store at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let db = sled::open(path.join("dag.sled"))
            .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(Self { db })
    }

    /// Stores a node and returns its hash.
    pub fn store(&self, node: &StateNode) -> Result<Hash> {
        let data = serde_json::to_vec(node)?;
        let hash = Hash::from_bytes(&data);

        self.db
            .insert(hash.0.as_bytes(), data)
            .map_err(|e| Error::Storage(e.to_string()))?;

        Ok(hash)
    }

    /// Loads a node by its hash.
    pub fn load(&self, hash: &Hash) -> Result<Option<StateNode>> {
        let data = self
            .db
            .get(hash.0.as_bytes())
            .map_err(|e| Error::Storage(e.to_string()))?;

        match data {
            Some(bytes) => {
                let node: StateNode = serde_json::from_slice(&bytes)?;
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    /// Walks the history from a node back to the root.
    pub fn walk_history(&self, from: &Hash) -> HistoryWalker<'_> {
        HistoryWalker {
            store: self,
            current: Some(from.clone()),
        }
    }
}

/// Iterator over the history chain.
pub struct HistoryWalker<'a> {
    store: &'a DagStore,
    current: Option<Hash>,
}

impl<'a> Iterator for HistoryWalker<'a> {
    type Item = StateNode;

    fn next(&mut self) -> Option<Self::Item> {
        let hash = self.current.take()?;
        let node = self.store.load(&hash).ok()??;
        self.current = node.parent.clone();
        Some(node)
    }
}
```

**Step 4: Update lib.rs to export dag module**

Modify `crates/ragentop-core/src/lib.rs`:

```rust
//! Core types, traits, and utilities for ragentop.

pub mod adapter;
pub mod config;
pub mod dag;
pub mod error;
pub mod types;

pub use adapter::{AdapterCapabilities, AgentAdapter};
pub use config::Config;
pub use error::{Error, Result};
pub use types::*;
```

**Step 5: Run tests**

Run: `cargo test -p ragentop-core --test dag_test`
Expected: PASS

**Step 6: Run all core tests**

Run: `cargo test -p ragentop-core`
Expected: All PASS

**Step 7: Commit**

```bash
git add crates/ragentop-core/
git commit -m "feat(core): add Merkle DAG storage with content-addressing and history walking"
```

---

### Task 1.6: CI Pipeline

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `rustfmt.toml`
- Create: `.clippy.toml`

**Step 1: Create CI workflow**

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, mvp-0.1.0]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -Dwarnings

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --workspace --all-targets

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace --all-targets -- -D warnings

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace

  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-llvm-cov
      - run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: false
```

**Step 2: Create rustfmt.toml**

Create `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Default"
```

**Step 3: Run format check**

Run: `cargo fmt --all -- --check`
Expected: No formatting errors (or run `cargo fmt` to fix)

**Step 4: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: No warnings

**Step 5: Commit**

```bash
git add .github/ rustfmt.toml
git commit -m "ci: add GitHub Actions workflow for fmt, clippy, test, coverage"
```

---

## Phase 2: Daemon & First Adapter

### Task 2.1: Claude Adapter - Stats Cache Parsing

**Files:**
- Modify: `crates/adapters/adapter-claude/src/lib.rs`
- Create: `crates/adapters/adapter-claude/src/parser.rs`
- Create: `crates/adapters/adapter-claude/tests/fixtures/`
- Test: `crates/adapters/adapter-claude/tests/parser_test.rs`

**Step 1: Create test fixture**

Create `crates/adapters/adapter-claude/tests/fixtures/stats_cache.json`:

```json
{
  "totalTokens": 45231,
  "totalCost": 1.82,
  "model": "claude-sonnet-4-20250514",
  "sessionId": "session-abc123"
}
```

**Step 2: Write the failing test**

Create `crates/adapters/adapter-claude/tests/parser_test.rs`:

```rust
use adapter_claude::parser::{parse_stats_cache, StatsCache};
use std::path::PathBuf;

#[test]
fn test_parse_stats_cache() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/stats_cache.json");
    let contents = std::fs::read_to_string(&fixture).unwrap();

    let stats = parse_stats_cache(&contents).unwrap();

    assert_eq!(stats.total_tokens, 45231);
    assert_eq!(stats.total_cost, Some(1.82));
    assert_eq!(stats.model, Some("claude-sonnet-4-20250514".to_string()));
}

#[test]
fn test_parse_stats_cache_minimal() {
    let json = r#"{"totalTokens": 100}"#;
    let stats = parse_stats_cache(json).unwrap();

    assert_eq!(stats.total_tokens, 100);
    assert!(stats.total_cost.is_none());
}
```

**Step 3: Run test to verify it fails**

Run: `cargo test -p adapter-claude --test parser_test`
Expected: FAIL (parser module doesn't exist)

**Step 4: Implement parser**

Create `crates/adapters/adapter-claude/src/parser.rs`:

```rust
use ragentop_core::Result;
use serde::Deserialize;

/// Parsed stats cache data.
#[derive(Debug, Clone)]
pub struct StatsCache {
    pub total_tokens: u64,
    pub total_cost: Option<f64>,
    pub model: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawStatsCache {
    total_tokens: Option<u64>,
    total_cost: Option<f64>,
    model: Option<String>,
    session_id: Option<String>,
}

/// Parses a stats-cache.json file.
pub fn parse_stats_cache(contents: &str) -> Result<StatsCache> {
    let raw: RawStatsCache = serde_json::from_str(contents)?;

    Ok(StatsCache {
        total_tokens: raw.total_tokens.unwrap_or(0),
        total_cost: raw.total_cost,
        model: raw.model,
        session_id: raw.session_id,
    })
}
```

**Step 5: Update adapter lib.rs**

Modify `crates/adapters/adapter-claude/src/lib.rs`:

```rust
//! Claude Code adapter for ragentop.

pub mod parser;

use ragentop_core::{
    AdapterCapabilities, AgentAdapter, AgentSession, AgentType, Command, HistoryDepth, Result,
    SessionId, SessionMetrics,
};
use std::path::PathBuf;

pub struct ClaudeAdapter {
    config_dir: PathBuf,
}

impl ClaudeAdapter {
    pub fn new() -> Self {
        let config_dir = dirs::home_dir()
            .map(|h| h.join(".claude"))
            .unwrap_or_else(|| PathBuf::from("~/.claude"));
        Self { config_dir }
    }

    pub fn with_config_dir(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for ClaudeAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Claude
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    fn detect_sessions(&self) -> Result<Vec<AgentSession>> {
        // TODO: Implement session detection
        Ok(vec![])
    }

    fn poll_metrics(&self, _session_id: &SessionId) -> Result<SessionMetrics> {
        // TODO: Implement metrics polling
        Ok(SessionMetrics::default())
    }

    fn get_command_history(
        &self,
        _session_id: &SessionId,
        _depth: HistoryDepth,
        _limit: usize,
    ) -> Result<Vec<Command>> {
        // TODO: Implement command history
        Ok(vec![])
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            tokens: true,
            cost: true,
            commands: true,
            model_info: true,
            session_replay: true,
        }
    }
}
```

**Step 6: Add dirs dependency**

Update `crates/adapters/adapter-claude/Cargo.toml`:

```toml
[dependencies]
ragentop-core = { path = "../../ragentop-core" }
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
tracing.workspace = true
dirs = "5.0"
```

**Step 7: Run tests**

Run: `cargo test -p adapter-claude`
Expected: PASS

**Step 8: Commit**

```bash
git add crates/adapters/adapter-claude/
git commit -m "feat(adapter-claude): add stats-cache.json parser"
```

---

### Task 2.2: Claude Adapter - Session Detection

**Files:**
- Create: `crates/adapters/adapter-claude/src/detector.rs`
- Modify: `crates/adapters/adapter-claude/src/lib.rs`
- Test: `crates/adapters/adapter-claude/tests/detector_test.rs`

**Step 1: Write the failing test**

Create `crates/adapters/adapter-claude/tests/detector_test.rs`:

```rust
use adapter_claude::ClaudeAdapter;
use ragentop_core::AgentAdapter;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_detect_sessions_finds_projects() {
    let dir = tempdir().unwrap();
    let claude_dir = dir.path().join(".claude");
    let project_dir = claude_dir.join("projects").join("test-project");
    fs::create_dir_all(&project_dir).unwrap();

    // Create a minimal session marker
    fs::write(
        project_dir.join("session.json"),
        r#"{"id": "session-123", "model": "claude-sonnet-4-20250514"}"#,
    )
    .unwrap();

    let adapter = ClaudeAdapter::with_config_dir(claude_dir);
    let sessions = adapter.detect_sessions().unwrap();

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id.0, "session-123");
}

#[test]
fn test_detect_sessions_empty_when_no_projects() {
    let dir = tempdir().unwrap();
    let claude_dir = dir.path().join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    let adapter = ClaudeAdapter::with_config_dir(claude_dir);
    let sessions = adapter.detect_sessions().unwrap();

    assert!(sessions.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p adapter-claude --test detector_test`
Expected: FAIL (detect_sessions returns empty vec)

**Step 3: Implement detector**

Create `crates/adapters/adapter-claude/src/detector.rs`:

```rust
use ragentop_core::{AgentSession, AgentType, Result, SessionId, SessionStatus};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct SessionFile {
    id: Option<String>,
    model: Option<String>,
    #[serde(rename = "sessionName")]
    session_name: Option<String>,
}

/// Detects Claude sessions in the given config directory.
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let projects_dir = config_dir.join("projects");

    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions = Vec::new();

    for entry in std::fs::read_dir(&projects_dir)? {
        let entry = entry?;
        let project_path = entry.path();

        if !project_path.is_dir() {
            continue;
        }

        let session_file = project_path.join("session.json");
        if session_file.exists() {
            if let Ok(contents) = std::fs::read_to_string(&session_file) {
                if let Ok(session_data) = serde_json::from_str::<SessionFile>(&contents) {
                    let id = session_data
                        .id
                        .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());

                    sessions.push(AgentSession {
                        id: SessionId::new(id),
                        agent_type: AgentType::Claude,
                        model: session_data.model,
                        session_name: session_data.session_name,
                        working_dir: Some(project_path.clone()),
                        pane_id: None,
                        pid: None,
                        started_at: None,
                        status: SessionStatus::Idle,
                    });
                }
            }
        }
    }

    Ok(sessions)
}
```

**Step 4: Update lib.rs to use detector**

Modify the `detect_sessions` method in `crates/adapters/adapter-claude/src/lib.rs`:

```rust
//! Claude Code adapter for ragentop.

pub mod detector;
pub mod parser;

use ragentop_core::{
    AdapterCapabilities, AgentAdapter, AgentSession, AgentType, Command, HistoryDepth, Result,
    SessionId, SessionMetrics,
};
use std::path::PathBuf;

pub struct ClaudeAdapter {
    config_dir: PathBuf,
}

impl ClaudeAdapter {
    pub fn new() -> Self {
        let config_dir = dirs::home_dir()
            .map(|h| h.join(".claude"))
            .unwrap_or_else(|| PathBuf::from("~/.claude"));
        Self { config_dir }
    }

    pub fn with_config_dir(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for ClaudeAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Claude
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    fn detect_sessions(&self) -> Result<Vec<AgentSession>> {
        detector::detect_sessions(&self.config_dir)
    }

    fn poll_metrics(&self, _session_id: &SessionId) -> Result<SessionMetrics> {
        // TODO: Implement metrics polling
        Ok(SessionMetrics::default())
    }

    fn get_command_history(
        &self,
        _session_id: &SessionId,
        _depth: HistoryDepth,
        _limit: usize,
    ) -> Result<Vec<Command>> {
        // TODO: Implement command history
        Ok(vec![])
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            tokens: true,
            cost: true,
            commands: true,
            model_info: true,
            session_replay: true,
        }
    }
}
```

**Step 5: Run tests**

Run: `cargo test -p adapter-claude`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/adapters/adapter-claude/
git commit -m "feat(adapter-claude): add session detection from projects directory"
```

---

### Task 2.3: Daemon Socket Server

**Files:**
- Create: `crates/ragentop-daemon/src/server.rs`
- Create: `crates/ragentop-daemon/src/protocol.rs`
- Test: `crates/ragentop-daemon/tests/server_test.rs`

**Step 1: Write the failing test**

Create `crates/ragentop-daemon/tests/server_test.rs`:

```rust
use ragentop_daemon::protocol::{Request, Response};

#[test]
fn test_request_serde() {
    let req = Request::ListSessions;
    let json = serde_json::to_string(&req).unwrap();
    let parsed: Request = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, Request::ListSessions));
}

#[test]
fn test_response_serde() {
    let resp = Response::Sessions(vec![]);
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("sessions"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ragentop-daemon --test server_test`
Expected: FAIL (protocol module doesn't exist)

**Step 3: Create protocol module**

Create `crates/ragentop-daemon/src/protocol.rs`:

```rust
use ragentop_core::{AgentSession, Command, SessionId, SessionMetrics};
use serde::{Deserialize, Serialize};

/// Request from client to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// List all active sessions
    ListSessions,
    /// Get metrics for a session
    GetMetrics { session_id: String },
    /// Get command history for a session
    GetHistory {
        session_id: String,
        depth: u8,
        limit: usize,
    },
    /// Subscribe to updates
    Subscribe,
    /// Unsubscribe from updates
    Unsubscribe,
}

/// Response from daemon to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// List of sessions
    Sessions { sessions: Vec<AgentSession> },
    /// Metrics for a session
    Metrics {
        session_id: String,
        metrics: SessionMetrics,
    },
    /// Command history
    History {
        session_id: String,
        commands: Vec<Command>,
    },
    /// Subscription confirmed
    Subscribed,
    /// Error response
    Error { message: String },
}

/// Event pushed to subscribed clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// Session added
    SessionAdded { session: AgentSession },
    /// Session removed
    SessionRemoved { session_id: String },
    /// Metrics updated
    MetricsUpdated {
        session_id: String,
        metrics: SessionMetrics,
    },
    /// New command executed
    CommandExecuted {
        session_id: String,
        command: Command,
    },
}
```

**Step 4: Update daemon lib.rs**

Modify `crates/ragentop-daemon/src/lib.rs`:

```rust
//! Background daemon for collecting agent metrics.

pub mod protocol;
pub mod registry;
pub mod server;
pub mod session;
```

**Step 5: Create placeholder modules**

Create `crates/ragentop-daemon/src/server.rs`:

```rust
//! Unix socket server for daemon communication.

use crate::protocol::{Request, Response};
use ragentop_core::Result;

pub struct DaemonServer {
    // TODO: Add socket handle
}

impl DaemonServer {
    pub fn new() -> Self {
        Self {}
    }
}
```

Create `crates/ragentop-daemon/src/registry.rs`:

```rust
//! Adapter registry for managing agent adapters.

use ragentop_core::AgentAdapter;
use std::sync::Arc;

pub struct AdapterRegistry {
    adapters: Vec<Arc<dyn AgentAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self { adapters: vec![] }
    }

    pub fn register(&mut self, adapter: Arc<dyn AgentAdapter>) {
        self.adapters.push(adapter);
    }

    pub fn adapters(&self) -> &[Arc<dyn AgentAdapter>] {
        &self.adapters
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

Create `crates/ragentop-daemon/src/session.rs`:

```rust
//! Session tracking and state management.

use ragentop_core::{AgentSession, SessionId};
use std::collections::HashMap;

pub struct SessionTracker {
    sessions: HashMap<SessionId, AgentSession>,
}

impl SessionTracker {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn update(&mut self, session: AgentSession) {
        self.sessions.insert(session.id.clone(), session);
    }

    pub fn remove(&mut self, id: &SessionId) {
        self.sessions.remove(id);
    }

    pub fn get(&self, id: &SessionId) -> Option<&AgentSession> {
        self.sessions.get(id)
    }

    pub fn all(&self) -> Vec<&AgentSession> {
        self.sessions.values().collect()
    }
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 6: Run tests**

Run: `cargo test -p ragentop-daemon`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/ragentop-daemon/
git commit -m "feat(daemon): add protocol types and session tracking"
```

---

## Phase 3: TUI

### Task 3.1: TUI App State

**Files:**
- Create: `crates/ragentop-tui/src/app.rs`
- Create: `crates/ragentop-tui/src/state.rs`
- Test: `crates/ragentop-tui/tests/state_test.rs`

**Step 1: Write the failing test**

Create `crates/ragentop-tui/tests/state_test.rs`:

```rust
use ragentop_tui::state::{AppState, Panel};

#[test]
fn test_initial_state() {
    let state = AppState::new();
    assert_eq!(state.selected_index, 0);
    assert_eq!(state.active_panel, Panel::SessionList);
}

#[test]
fn test_navigate_down() {
    let mut state = AppState::new();
    state.session_count = 5;

    state.navigate_down();
    assert_eq!(state.selected_index, 1);

    state.navigate_down();
    assert_eq!(state.selected_index, 2);
}

#[test]
fn test_navigate_wraps() {
    let mut state = AppState::new();
    state.session_count = 3;
    state.selected_index = 2;

    state.navigate_down();
    assert_eq!(state.selected_index, 0);
}

#[test]
fn test_toggle_expanded() {
    let mut state = AppState::new();
    assert!(!state.detail_expanded);

    state.toggle_expand();
    assert!(state.detail_expanded);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ragentop-tui --test state_test`
Expected: FAIL (state module doesn't exist)

**Step 3: Implement state module**

Create `crates/ragentop-tui/src/state.rs`:

```rust
//! Application state management.

use ragentop_core::{AgentSession, HistoryDepth, SessionMetrics};

/// Active UI panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    SessionList,
    Detail,
}

/// Application state.
#[derive(Debug)]
pub struct AppState {
    /// Currently selected session index
    pub selected_index: usize,
    /// Total number of sessions
    pub session_count: usize,
    /// Currently active panel
    pub active_panel: Panel,
    /// Whether detail panel is expanded
    pub detail_expanded: bool,
    /// Current history depth level (1-3)
    pub history_depth: HistoryDepth,
    /// Sessions data
    pub sessions: Vec<AgentSession>,
    /// Metrics for selected session
    pub selected_metrics: Option<SessionMetrics>,
    /// Whether to quit
    pub should_quit: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            session_count: 0,
            active_panel: Panel::SessionList,
            detail_expanded: false,
            history_depth: HistoryDepth::WithResponses,
            sessions: vec![],
            selected_metrics: None,
            should_quit: false,
        }
    }

    pub fn navigate_up(&mut self) {
        if self.session_count == 0 {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.session_count - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if self.session_count == 0 {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.session_count;
    }

    pub fn toggle_expand(&mut self) {
        self.detail_expanded = !self.detail_expanded;
    }

    pub fn cycle_depth(&mut self) {
        self.history_depth = match self.history_depth {
            HistoryDepth::ToolCallsOnly => HistoryDepth::WithResponses,
            HistoryDepth::WithResponses => HistoryDepth::FullConversation,
            HistoryDepth::FullConversation => HistoryDepth::ToolCallsOnly,
        };
    }

    pub fn selected_session(&self) -> Option<&AgentSession> {
        self.sessions.get(self.selected_index)
    }

    pub fn update_sessions(&mut self, sessions: Vec<AgentSession>) {
        self.session_count = sessions.len();
        self.sessions = sessions;
        if self.selected_index >= self.session_count && self.session_count > 0 {
            self.selected_index = self.session_count - 1;
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Update lib.rs**

Modify `crates/ragentop-tui/src/lib.rs`:

```rust
//! Terminal user interface for ragentop.

pub mod app;
pub mod input;
pub mod state;
pub mod ui;

pub use state::AppState;
```

**Step 5: Create placeholder app module**

Create `crates/ragentop-tui/src/app.rs`:

```rust
//! Main application loop.

use crate::state::AppState;

pub struct App {
    pub state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
```

Create `crates/ragentop-tui/src/input.rs`:

```rust
//! Input handling.
```

Create `crates/ragentop-tui/src/ui.rs`:

```rust
//! UI rendering.
```

**Step 6: Run tests**

Run: `cargo test -p ragentop-tui`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/ragentop-tui/
git commit -m "feat(tui): add application state management"
```

---

### Task 3.2: TUI Layout Rendering

**Files:**
- Modify: `crates/ragentop-tui/src/ui.rs`
- Create: `crates/ragentop-tui/src/ui/dashboard.rs`
- Create: `crates/ragentop-tui/src/ui/session_list.rs`
- Create: `crates/ragentop-tui/src/ui/detail.rs`
- Create: `crates/ragentop-tui/src/ui/theme.rs`

**Step 1: Create theme module**

Create `crates/ragentop-tui/src/ui/theme.rs`:

```rust
//! Color theme and styling.

use ratatui::style::{Color, Modifier, Style};
use ragentop_core::AgentType;

pub struct Theme;

impl Theme {
    // Background colors
    pub const BG_BASE: Color = Color::Rgb(13, 13, 13);
    pub const BG_SURFACE: Color = Color::Rgb(20, 20, 20);
    pub const BG_SELECTED: Color = Color::Rgb(38, 38, 38);

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::Rgb(229, 229, 229);
    pub const TEXT_SECONDARY: Color = Color::Rgb(163, 163, 163);
    pub const TEXT_MUTED: Color = Color::Rgb(82, 82, 82);

    // Status colors
    pub const STATUS_ACTIVE: Color = Color::Green;
    pub const STATUS_IDLE: Color = Color::Yellow;
    pub const STATUS_PAUSED: Color = Color::DarkGray;
    pub const STATUS_ERROR: Color = Color::Red;

    // Agent accent colors
    pub fn agent_color(agent: AgentType) -> Color {
        match agent {
            AgentType::Claude => Color::Rgb(217, 119, 6),   // amber
            AgentType::Codex => Color::Rgb(34, 197, 94),    // green
            AgentType::Gemini => Color::Rgb(59, 130, 246),  // blue
            AgentType::Copilot => Color::Rgb(139, 92, 246), // purple
            AgentType::Qwen => Color::Rgb(6, 182, 212),     // cyan
            AgentType::Glm => Color::Rgb(236, 72, 153),     // pink
        }
    }

    // Common styles
    pub fn default_style() -> Style {
        Style::default().fg(Self::TEXT_PRIMARY).bg(Self::BG_BASE)
    }

    pub fn selected_style() -> Style {
        Style::default()
            .fg(Self::TEXT_PRIMARY)
            .bg(Self::BG_SELECTED)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_style() -> Style {
        Style::default()
            .fg(Self::TEXT_SECONDARY)
            .add_modifier(Modifier::UNDERLINED)
    }

    pub fn border_style() -> Style {
        Style::default().fg(Self::TEXT_MUTED)
    }
}
```

**Step 2: Create session list component**

Create `crates/ragentop-tui/src/ui/session_list.rs`:

```rust
//! Session list panel.

use crate::state::AppState;
use crate::ui::theme::Theme;
use ragentop_core::SessionStatus;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = state
        .sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let status_symbol = match session.status {
                SessionStatus::Active => "●",
                SessionStatus::Idle => "◐",
                SessionStatus::Paused => "○",
            };

            let status_color = match session.status {
                SessionStatus::Active => Theme::STATUS_ACTIVE,
                SessionStatus::Idle => Theme::STATUS_IDLE,
                SessionStatus::Paused => Theme::STATUS_PAUSED,
            };

            let agent_color = Theme::agent_color(session.agent_type);

            let line = Line::from(vec![
                Span::styled(format!("{} ", status_symbol), Style::default().fg(status_color)),
                Span::styled(
                    format!("{:<8}", session.agent_type),
                    Style::default().fg(agent_color),
                ),
                Span::styled(
                    format!("{:<12}", session.model.as_deref().unwrap_or("—")),
                    Style::default().fg(Theme::TEXT_SECONDARY),
                ),
                Span::styled(
                    format!("{:<16}", session.session_name.as_deref().unwrap_or("—")),
                    Style::default().fg(Theme::TEXT_PRIMARY),
                ),
            ]);

            let style = if i == state.selected_index {
                Theme::selected_style()
            } else {
                Theme::default_style()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Sessions ")
                .borders(Borders::ALL)
                .border_style(Theme::border_style()),
        )
        .highlight_style(Theme::selected_style());

    frame.render_widget(list, area);
}
```

**Step 3: Create detail panel component**

Create `crates/ragentop-tui/src/ui/detail.rs`:

```rust
//! Detail panel for selected session.

use crate::state::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = if let Some(session) = state.selected_session() {
        let title = format!(
            " {}: {} ({}) ",
            session.agent_type,
            session.session_name.as_deref().unwrap_or("unnamed"),
            session.model.as_deref().unwrap_or("unknown")
        );

        let metrics_line = if let Some(metrics) = &state.selected_metrics {
            format!(
                "Tokens: {} │ Cost: ${:.2} │ Commands: {}",
                metrics.token_count,
                metrics.cost_usd.unwrap_or(0.0),
                metrics.command_count
            )
        } else {
            "Loading metrics...".to_string()
        };

        let lines = vec![
            Line::from(Span::styled(metrics_line, Theme::default_style())),
            Line::from(""),
            Line::from(Span::styled("Recent Commands", Theme::header_style())),
            Line::from(Span::styled("─".repeat(area.width as usize - 4), Theme::border_style())),
            Line::from(Span::styled("(no commands yet)", Theme::TEXT_MUTED.into())),
        ];

        Paragraph::new(lines).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Theme::border_style()),
        )
    } else {
        Paragraph::new("No session selected").block(
            Block::default()
                .title(" Detail ")
                .borders(Borders::ALL)
                .border_style(Theme::border_style()),
        )
    };

    frame.render_widget(content, area);
}
```

**Step 4: Create dashboard layout**

Create `crates/ragentop-tui/src/ui/dashboard.rs`:

```rust
//! Main dashboard layout.

use crate::state::AppState;
use crate::ui::{detail, session_list, theme::Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Summary bar
            Constraint::Min(10),    // Session list
            Constraint::Min(8),     // Detail panel
            Constraint::Length(1),  // Keybindings
        ])
        .split(frame.area());

    render_summary(frame, chunks[0], state);
    session_list::render(frame, chunks[1], state);
    detail::render(frame, chunks[2], state);
    render_keybindings(frame, chunks[3]);
}

fn render_summary(frame: &mut Frame, area: Rect, state: &AppState) {
    let active_count = state
        .sessions
        .iter()
        .filter(|s| s.status == ragentop_core::SessionStatus::Active)
        .count();

    let total_tokens: u64 = state
        .selected_metrics
        .as_ref()
        .map(|m| m.token_count)
        .unwrap_or(0);

    let summary = format!(
        " {} active │ {} sessions │ {} tokens ",
        active_count,
        state.session_count,
        total_tokens
    );

    let paragraph = Paragraph::new(Line::from(Span::styled(summary, Theme::default_style())))
        .block(
            Block::default()
                .title(" ragentop ")
                .borders(Borders::ALL)
                .border_style(Theme::border_style()),
        );

    frame.render_widget(paragraph, area);
}

fn render_keybindings(frame: &mut Frame, area: Rect) {
    let hints = "[j/k] navigate  [Space] expand  [d] depth  [q] quit  [?] help";
    let paragraph = Paragraph::new(Span::styled(hints, Theme::TEXT_MUTED.into()));
    frame.render_widget(paragraph, area);
}
```

**Step 5: Update ui mod.rs**

Modify `crates/ragentop-tui/src/ui.rs` to be a module:

Rename to `crates/ragentop-tui/src/ui/mod.rs`:

```rust
//! UI rendering.

pub mod dashboard;
pub mod detail;
pub mod session_list;
pub mod theme;

pub use dashboard::render;
```

**Step 6: Verify it compiles**

Run: `cargo check -p ragentop-tui`
Expected: Compiles

**Step 7: Commit**

```bash
git add crates/ragentop-tui/
git commit -m "feat(tui): add dashboard layout with session list and detail panels"
```

---

### Task 3.3: TUI Input Handling

**Files:**
- Modify: `crates/ragentop-tui/src/input.rs`
- Test: `crates/ragentop-tui/tests/input_test.rs`

**Step 1: Write the failing test**

Create `crates/ragentop-tui/tests/input_test.rs`:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ragentop_tui::input::{Action, handle_key_event};

#[test]
fn test_quit_on_q() {
    let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    let action = handle_key_event(event);
    assert_eq!(action, Some(Action::Quit));
}

#[test]
fn test_navigate_down_on_j() {
    let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    let action = handle_key_event(event);
    assert_eq!(action, Some(Action::NavigateDown));
}

#[test]
fn test_navigate_up_on_k() {
    let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
    let action = handle_key_event(event);
    assert_eq!(action, Some(Action::NavigateUp));
}

#[test]
fn test_toggle_expand_on_space() {
    let event = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    let action = handle_key_event(event);
    assert_eq!(action, Some(Action::ToggleExpand));
}

#[test]
fn test_cycle_depth_on_d() {
    let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    let action = handle_key_event(event);
    assert_eq!(action, Some(Action::CycleDepth));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ragentop-tui --test input_test`
Expected: FAIL (Action and handle_key_event don't exist)

**Step 3: Implement input handling**

Modify `crates/ragentop-tui/src/input.rs`:

```rust
//! Input handling.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// Actions that can be triggered by user input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    NavigateUp,
    NavigateDown,
    ToggleExpand,
    CycleDepth,
    Refresh,
    Help,
    SelectSession(usize),
}

/// Handles a key event and returns the corresponding action.
pub fn handle_key_event(event: KeyEvent) -> Option<Action> {
    match (event.code, event.modifiers) {
        // Quit
        (KeyCode::Char('q'), KeyModifiers::NONE) => Some(Action::Quit),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),

        // Navigation
        (KeyCode::Char('j'), KeyModifiers::NONE) => Some(Action::NavigateDown),
        (KeyCode::Down, _) => Some(Action::NavigateDown),
        (KeyCode::Char('k'), KeyModifiers::NONE) => Some(Action::NavigateUp),
        (KeyCode::Up, _) => Some(Action::NavigateUp),

        // Expand/collapse
        (KeyCode::Char(' '), KeyModifiers::NONE) => Some(Action::ToggleExpand),
        (KeyCode::Enter, _) => Some(Action::ToggleExpand),

        // Depth cycling
        (KeyCode::Char('d'), KeyModifiers::NONE) => Some(Action::CycleDepth),

        // Refresh
        (KeyCode::Char('r'), KeyModifiers::NONE) => Some(Action::Refresh),
        (KeyCode::F(5), _) => Some(Action::Refresh),

        // Help
        (KeyCode::Char('?'), _) => Some(Action::Help),
        (KeyCode::F(1), _) => Some(Action::Help),

        // Direct session selection (1-9)
        (KeyCode::Char(c), KeyModifiers::NONE) if c.is_ascii_digit() && c != '0' => {
            let index = c.to_digit(10).unwrap() as usize - 1;
            Some(Action::SelectSession(index))
        }

        _ => None,
    }
}

/// Handles a mouse event and returns the corresponding action.
pub fn handle_mouse_event(event: MouseEvent) -> Option<Action> {
    match event.kind {
        MouseEventKind::ScrollUp => Some(Action::NavigateUp),
        MouseEventKind::ScrollDown => Some(Action::NavigateDown),
        // TODO: Handle click events with row detection
        _ => None,
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p ragentop-tui`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/ragentop-tui/
git commit -m "feat(tui): add input handling for keyboard and mouse"
```

---

## Remaining Phases Summary

Due to length constraints, the remaining tasks follow the same TDD pattern:

### Phase 4: Remaining Adapters
- **Task 4.1:** Codex adapter (API + file parsing)
- **Task 4.2:** Gemini adapter (OTEL logs, shell_history)
- **Task 4.3:** Copilot adapter (config + /usage)
- **Task 4.4:** Qwen adapter (JSON logs, /stats)
- **Task 4.5:** GLM adapter (Claude session detection)
- **Task 4.6:** Zellij integration (bidirectional sync)
- **Task 4.7:** Tmux integration (bidirectional sync)

### Phase 5: Web & Release
- **Task 5.1:** Leptos project setup
- **Task 5.2:** Dashboard components
- **Task 5.3:** SSH socket mode
- **Task 5.4:** CLI polish (help, completions)
- **Task 5.5:** README and documentation
- **Task 5.6:** Release setup (cargo-dist)

---

## Execution Checklist

Before each task:
- [ ] Read the task requirements
- [ ] Write the failing test FIRST
- [ ] Run test to confirm it fails
- [ ] Implement minimal code to pass
- [ ] Run test to confirm it passes
- [ ] Run `cargo fmt` and `cargo clippy`
- [ ] Commit with conventional commit message

After each phase:
- [ ] Run full test suite: `cargo test --workspace`
- [ ] Run coverage check
- [ ] Update CHANGELOG.md
