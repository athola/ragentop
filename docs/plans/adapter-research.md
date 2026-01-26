# Agent Adapter Research

Research findings for ragentop adapter implementations. Each adapter extracts session data, token usage, command history, and metrics from agent CLI tools.

## Claude Code

**Config Directory:** `~/.claude/`

**Data Sources:**
- `~/.claude/stats-cache.json` - Token usage, cost statistics
- `~/.claude/projects/<project>/` - Project-specific session data
- `~/.claude/settings.json` - User configuration

**Capabilities:** Full (tokens, cost, commands, replay, model info)

**Notes:**
- Well-documented data format
- Primary reference implementation for adapter trait

---

## Codex (OpenAI)

**Config Directory:** `~/.codex/`

**Data Sources:**
- Session files in config directory
- `/usage` API endpoint for token stats

**Capabilities:** Full (tokens, cost, commands, replay, model info)

**Notes:**
- API-based usage tracking
- Session files contain command history

---

## Gemini CLI

**Config Directory:** `~/.gemini/`

**Data Sources:**
```
~/.gemini/
├── settings.json              # Config, telemetry settings
├── tmp/<project_hash>/
│   ├── shell_history          # Command history per project
│   └── otel/collector-gcp.log # OpenTelemetry metrics
└── telemetry.log              # Optional telemetry output
```

**Key Fields in settings.json:**
- Telemetry configuration (enabled, logPrompts, useCollector)
- MCP server configurations
- Model settings

**OpenTelemetry Integration:**
- `session.id` - Unique session identifier
- `installation.id` - Installation identifier
- `user.email` - When authenticated with Google account
- Events: `gemini_cli.config`, tool executions, output truncation

**Capabilities:** Full (tokens, cost, commands, replay, model info)

**References:**
- https://geminicli.com/docs/get-started/configuration/
- https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/telemetry.md

---

## GitHub Copilot CLI

**Config Directory:** `~/.copilot/`

**Data Sources:**
```
~/.copilot/
├── config.json    # Settings, debug logging level
├── config         # URL allowlists, trusted directories
└── [session API]  # Token usage, session stats via /usage command
```

**Debug Logging Levels:** `none`, `error`, `warning`, `info`, `debug`, `all`, `default`

**Session Tracking:**
- Token consumption via `/usage` command
- Session count and session length
- Auto-compaction at 95% token limit

**Authentication:**
- `GH_TOKEN` or `GITHUB_TOKEN` environment variables
- `COPILOT_GITHUB_TOKEN` (highest precedence)
- Token storage for session persistence

**Capabilities:** Full (tokens, cost, commands, replay*, model info)
- *Replay may be partial depending on session log retention

**References:**
- https://docs.github.com/en/copilot/how-tos/use-copilot-agents/use-copilot-cli
- https://github.com/github/copilot-cli

---

## Qwen Code

**Config Directory:** `~/.qwen/`

**Data Sources:**
```
~/.qwen/
├── settings.json        # Token limits, model config
└── logs/openai/*.json   # Request/response logs (when enableOpenAILogging: true)
```

**Settings Configuration:**
- Token limit configuration
- Model selection
- Logging enablement (`enableOpenAILogging: true`)

**Commands:**
- `/stats` - Current session usage information
- `/compress` - Shrink history
- `/clear` - Start fresh session

**OAuth Integration:**
- Free tier: 2000 requests/day, 60 requests/minute
- Usage logging for Qwen integration
- Token usage optimizations

**Log Format:** JSON files with request/response data, session ID, timestamps

**Capabilities:** Full (tokens, cost, commands, replay, model info)

**References:**
- https://github.com/QwenLM/qwen-code
- https://qwenlm.github.io/qwen-code-docs/

---

## GLM-4 (via glm CLI tool)

**Config Directory:** `~/.glm/`

**Data Sources:**
```
~/.glm/
└── config.json    # Auth token storage (BigModel API)
```

**Execution Model:**
- GLM CLI launches Claude Code with temporary environment variables
- `ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic`
- No persistent file modifications to `~/.claude/settings.json`
- Session data stored via underlying Claude Code instance

**Model Selection:** `glm-4.7`, `glm-4.6`, `glm-4.5`, `glm-4.5-air`

**Capabilities:** Partial (via underlying Claude Code session)
- Session detection: Via process inspection + Claude session
- Token count: Via Claude session metrics
- Cost estimate: Via Claude session (BigModel pricing)
- Command history: Via Claude session
- Session replay: Partial (depends on Claude session)

**Detection Strategy:**
1. Detect Claude process with BigModel API base URL
2. Link to GLM config for model info
3. Track as GLM session with Claude backend

**References:**
- https://github.com/xqsit94/glm
- https://z.ai/blog/glm-4.7

---

## Capability Summary Matrix

| Feature          | Claude | Codex | Gemini | Copilot | Qwen | GLM   |
|------------------|--------|-------|--------|---------|------|-------|
| Session detect   | Full   | Full  | Full   | Full    | Full | Full* |
| Token count      | Full   | Full  | Full   | Full    | Full | Full* |
| Cost estimate    | Full   | Full  | Full   | Partial | Full | Full* |
| Command history  | Full   | Full  | Full   | Full    | Full | Full* |
| Session replay   | Full   | Full  | Full   | Partial | Full | Partial |
| Model info       | Full   | Full  | Full   | Full    | Full | Full  |

*Via underlying Claude Code session

---

## Implementation Priority

1. **Claude Code** - Primary reference, best documented
2. **Gemini CLI** - Excellent OpenTelemetry integration
3. **Qwen Code** - Good logging when enabled
4. **Copilot CLI** - API-based, well-structured
5. **Codex** - Standard patterns
6. **GLM-4** - Piggybacks on Claude adapter

## Common Adapter Interface

```rust
pub trait AgentAdapter: Send + Sync {
    /// Agent type identifier
    fn agent_type(&self) -> AgentType;

    /// Discover active sessions
    fn detect_sessions(&self) -> Result<Vec<AgentSession>>;

    /// Poll current metrics for a session
    fn poll_metrics(&self, session: &AgentSession) -> Result<SessionMetrics>;

    /// Get command history at specified depth
    fn get_command_history(
        &self,
        session: &AgentSession,
        depth: HistoryDepth
    ) -> Result<Vec<Command>>;

    /// Declare adapter capabilities
    fn capabilities(&self) -> AdapterCapabilities;

    /// Config directory path
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
