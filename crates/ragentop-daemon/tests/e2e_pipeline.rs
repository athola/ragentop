//! End-to-end integration test: adapter → session tracker → DAG store → protocol.
//!
//! Verifies the full pipeline without real sockets or processes.

use ragentop_core::{
    dag::{DagStore, StateNode},
    Adapter, AgentSession, AgentType, Capabilities, Command, CommandStatus, HistoryDepth, Request,
    Response, SessionId, SessionMetrics, SessionStatus,
};
use ragentop_daemon::{session::SessionTracker, SledDagStore};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tempfile::TempDir;

/// Mock adapter that returns a fixed session.
struct MockClaudeAdapter {
    sessions: Vec<AgentSession>,
}

impl MockClaudeAdapter {
    const fn with_sessions(sessions: Vec<AgentSession>) -> Self {
        Self { sessions }
    }
}

impl Adapter for MockClaudeAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Claude
    }

    fn config_dir(&self) -> PathBuf {
        PathBuf::from("/tmp/mock-claude")
    }

    fn detect_sessions(&self) -> ragentop_core::Result<Vec<AgentSession>> {
        Ok(self.sessions.clone())
    }

    fn poll_metrics(&self, _session_id: &SessionId) -> ragentop_core::Result<SessionMetrics> {
        Ok(SessionMetrics::new(
            1500,
            Some(0.05),
            Some(12.0),
            Some(std::time::Duration::from_secs(90)),
            7,
        )
        .0)
    }

    fn get_command_history(
        &self,
        _session_id: &SessionId,
        _depth: HistoryDepth,
        _limit: usize,
    ) -> ragentop_core::Result<Vec<Command>> {
        Ok(vec![
            Command {
                timestamp: SystemTime::now(),
                tool: "Read".to_string(),
                args: "src/main.rs".to_string(),
                status: CommandStatus::Success,
                result_summary: Some("ok".to_string()),
            },
            Command {
                timestamp: SystemTime::now(),
                tool: "Edit".to_string(),
                args: "src/lib.rs".to_string(),
                status: CommandStatus::Success,
                result_summary: None,
            },
        ])
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::new()
            .with_tokens(true)
            .with_cost(true)
            .with_commands(true)
            .with_model_info(true)
    }
}

fn make_session(id: &str) -> AgentSession {
    let mut session = AgentSession::new(
        SessionId::new_unchecked(id),
        AgentType::Claude,
        SessionStatus::Active,
    );
    session.model = Some("opus-4".to_string());
    session.session_name = Some("test-project".to_string());
    session.working_dir = Some(PathBuf::from("/home/user/project"));
    session.pane_id = Some("%1".to_string());
    session.pid = Some(12345);
    session.started_at = Some(SystemTime::now());
    session
}

/// Full pipeline: adapter detects → tracker collects → store persists → protocol queries.
#[test]
fn test_full_pipeline_adapter_to_protocol() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Adapter detects sessions
    let adapter =
        MockClaudeAdapter::with_sessions(vec![make_session("sess-001"), make_session("sess-002")]);
    let detected = adapter.detect_sessions()?;
    assert_eq!(detected.len(), 2);

    // 2. SessionTracker collects them
    let mut tracker = SessionTracker::new();
    for session in &detected {
        tracker.update(session.clone());
    }
    assert_eq!(tracker.all().len(), 2);
    assert!(tracker.get(&SessionId::new_unchecked("sess-001")).is_some());

    // 3. DAG store persists command history as state nodes
    let tmp = TempDir::new()?;
    let store = SledDagStore::open(tmp.path())?;

    let commands = adapter.get_command_history(
        &SessionId::new_unchecked("sess-001"),
        HistoryDepth::default(),
        10,
    )?;
    assert_eq!(commands.len(), 2);

    let node = StateNode::new(commands, None, SystemTime::now());
    let hash = store.store(&node)?;

    let loaded = store.load(&hash)?.ok_or("node should exist")?;
    assert_eq!(loaded.commands.len(), 2);
    assert_eq!(loaded.commands[0].tool, "Read");

    // 4. Protocol round-trip: serialize request, build response from tracker
    let request = Request::ListSessions;
    let json = serde_json::to_string(&request)?;
    let parsed: Request = serde_json::from_str(&json)?;
    assert!(matches!(parsed, Request::ListSessions));

    // Build response from tracker data
    let sessions: Vec<AgentSession> = tracker.all().into_iter().cloned().collect();
    let response = Response::Sessions { sessions };
    let resp_json = serde_json::to_string(&response)?;
    let parsed_resp: Response = serde_json::from_str(&resp_json)?;

    match parsed_resp {
        Response::Sessions { sessions } => {
            assert_eq!(sessions.len(), 2);
            let ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
            assert!(ids.contains(&"sess-001"));
            assert!(ids.contains(&"sess-002"));
        }
        _ => return Err("expected Sessions response".into()),
    }
    Ok(())
}

/// Verify metrics polling through the adapter and protocol serialization.
#[test]
fn test_metrics_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = MockClaudeAdapter::with_sessions(vec![make_session("sess-m1")]);
    let metrics = adapter.poll_metrics(&SessionId::new_unchecked("sess-m1"))?;

    assert_eq!(metrics.token_count, 1500);
    assert_eq!(
        metrics.cost_usd,
        Some(ragentop_core::UsdMicros::from_dollars(0.05))
    );
    assert!(metrics.is_valid());

    // Protocol round-trip for metrics response
    let response = Response::Metrics {
        session_id: SessionId::new_unchecked("sess-m1"),
        metrics,
    };
    let json = serde_json::to_string(&response)?;
    let parsed: Response = serde_json::from_str(&json)?;

    match parsed {
        Response::Metrics {
            session_id,
            metrics,
        } => {
            assert_eq!(session_id.as_str(), "sess-m1");
            assert_eq!(metrics.command_count, 7);
        }
        _ => return Err("expected Metrics response".into()),
    }
    Ok(())
}

/// DAG store persists and retrieves chained session snapshots.
#[test]
fn test_dag_store_history_chain() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let store = SledDagStore::open(tmp.path())?;

    let cmd = |tool: &str| Command {
        timestamp: SystemTime::UNIX_EPOCH,
        tool: tool.to_string(),
        args: String::new(),
        status: CommandStatus::Success,
        result_summary: None,
    };

    let root = StateNode::new(vec![cmd("Bash")], None, SystemTime::now());
    let root_hash = store.store(&root)?;

    let snap2 = StateNode::new(
        vec![cmd("Read"), cmd("Edit")],
        Some(root_hash),
        SystemTime::now(),
    );
    let snap2_hash = store.store(&snap2)?;

    let snap3 = StateNode::new(vec![cmd("Write")], Some(snap2_hash), SystemTime::now());
    let snap3_hash = store.store(&snap3)?;

    // Walk full history from latest
    let history: Vec<_> = store.walk_history(&snap3_hash).collect();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].commands[0].tool, "Write");
    assert_eq!(history[1].commands[0].tool, "Read");
    assert_eq!(history[2].commands[0].tool, "Bash");
    Ok(())
}

/// Adapter registry collects and iterates adapters.
#[test]
fn test_registry_with_mock_adapter() -> Result<(), Box<dyn std::error::Error>> {
    use ragentop_daemon::registry::AdapterRegistry;

    let mut registry = AdapterRegistry::new();
    let adapter = Arc::new(MockClaudeAdapter::with_sessions(vec![make_session("s1")]));
    registry.register(adapter);

    assert_eq!(registry.adapters().len(), 1);
    let detected = registry.adapters()[0].detect_sessions()?;
    assert_eq!(detected.len(), 1);
    assert_eq!(detected[0].id.as_str(), "s1");
    Ok(())
}
