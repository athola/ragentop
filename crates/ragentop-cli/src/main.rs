use std::process::ExitCode;

use clap::{Parser, Subcommand};
use ragentop_core::Adapter;

/// Exit code returned by stub subcommands that aren't yet implemented.
///
/// Matches BSD `EX_USAGE` so supervisors and shell scripts can distinguish
/// "not implemented" from a real runtime failure (which uses [`ExitCode::FAILURE`]).
const EX_USAGE: u8 = 64;

#[derive(Parser)]
#[command(name = "ragentop", about = "Monitor AI coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage the background daemon [stub: not yet implemented]
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Detect all agent sessions on this machine
    Detect {
        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show status of running agents
    Status,
    /// Launch the terminal UI [stub: not yet implemented]
    Tui,
    /// Start the web dashboard server
    Web {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the daemon [stub: not yet implemented]
    Start,
    /// Stop the daemon [stub: not yet implemented]
    Stop,
}

fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Daemon { action }) => match action {
            DaemonAction::Start => {
                eprintln!("ragentop daemon start: not yet implemented");
                ExitCode::from(EX_USAGE)
            }
            DaemonAction::Stop => {
                eprintln!("ragentop daemon stop: not yet implemented");
                ExitCode::from(EX_USAGE)
            }
        },
        Some(Commands::Tui) => {
            eprintln!("ragentop tui: not yet implemented");
            let _app = ragentop_tui::App::new();
            ExitCode::from(EX_USAGE)
        }
        Some(Commands::Status) => {
            eprintln!("ragentop status");
            let tracker = ragentop_daemon::session::SessionTracker::new();
            let sessions = tracker.all();
            if sessions.is_empty() {
                eprintln!("No active agent sessions.");
            } else {
                for session in sessions {
                    eprintln!("  {session:?}");
                }
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Web { port }) => {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("Failed to create async runtime: {e}");
                    return ExitCode::FAILURE;
                }
            };
            match rt.block_on(ragentop_web::serve([127, 0, 0, 1], port)) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("Web server error: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Detect { verbose }) => {
            let adapters: Vec<Box<dyn Adapter>> = vec![
                Box::new(adapter_claude::ClaudeAdapter::new()),
                Box::new(adapter_codex::CodexAdapter::new()),
                Box::new(adapter_copilot::CopilotAdapter::new()),
                Box::new(adapter_gemini::GeminiAdapter::new()),
                Box::new(adapter_qwen::QwenAdapter::new()),
            ];
            if detect_sessions(&adapters, verbose) {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        None => {
            eprintln!("ragentop v{}", env!("CARGO_PKG_VERSION"));
            eprintln!("Use --help for available commands.");
            ExitCode::SUCCESS
        }
    }
}

/// Detects sessions across the given adapters and prints a summary.
///
/// Returns `true` if any adapter returned an error, so callers can surface
/// a non-zero exit code instead of pretending "no sessions found" on
/// permission errors, malformed configs, or panicked adapters.
///
/// Takes adapters by reference (rather than building them inline) so the
/// error-propagation contract can be exercised in unit tests with mock
/// adapters — without it, reverting the `Err` arm here would compile and
/// ship silently.
fn detect_sessions(adapters: &[Box<dyn Adapter>], verbose: bool) -> bool {
    use std::collections::HashMap;

    let mut total_sessions = 0;
    let mut total_projects = 0;
    let mut any_error = false;

    for adapter in adapters {
        match adapter.detect_sessions() {
            Ok(sessions) if !sessions.is_empty() => {
                // Group sessions by project (working_dir)
                let mut by_project: HashMap<String, Vec<_>> = HashMap::new();
                for session in sessions {
                    let key = session
                        .working_dir
                        .as_ref()
                        .map_or_else(|| "unknown".to_owned(), |path| path.display().to_string());
                    by_project.entry(key).or_default().push(session);
                }

                let session_count: usize = by_project.values().map(Vec::len).sum();
                total_sessions += session_count;
                total_projects += by_project.len();

                eprintln!(
                    "\n{:?}: {} projects, {} sessions",
                    adapter.agent_type(),
                    by_project.len(),
                    session_count
                );
                eprintln!("  Config: {}", adapter.config_dir().display());

                // Sort projects by most recent session
                let mut projects: Vec<_> = by_project.into_iter().collect();
                projects.sort_by(|lhs, rhs| {
                    let lhs_time = lhs.1.iter().filter_map(|sess| sess.started_at).max();
                    let rhs_time = rhs.1.iter().filter_map(|sess| sess.started_at).max();
                    rhs_time.cmp(&lhs_time)
                });

                // Show top 10 projects (or all if verbose)
                let limit = if verbose { projects.len() } else { 10 };
                for (path, sessions) in projects.iter().take(limit) {
                    let active = sessions
                        .iter()
                        .filter(|sess| sess.status == ragentop_core::SessionStatus::Active)
                        .count();
                    if active > 0 {
                        eprintln!(
                            "  {} ({} sessions, {} active)",
                            path,
                            sessions.len(),
                            active
                        );
                    } else {
                        eprintln!("  {} ({} sessions)", path, sessions.len());
                    }
                }
                if !verbose && projects.len() > limit {
                    eprintln!("  ... and {} more projects", projects.len() - limit);
                }
            }
            Err(err) => {
                any_error = true;
                if verbose {
                    eprintln!("\n{:?}: error - {err}", adapter.agent_type());
                } else {
                    eprintln!("{:?}: error ({err})", adapter.agent_type());
                }
            }
            Ok(_) => {}
        }
    }
    eprintln!("\nTotal: {total_projects} projects, {total_sessions} sessions");
    any_error
}

#[cfg(test)]
mod tests {
    use super::*;
    use ragentop_core::{
        AgentSession, AgentType, Capabilities, Command, Error, HistoryDepth, Result, SessionId,
        SessionMetrics, SessionStatus,
    };
    use std::path::PathBuf;

    /// Mock outcome for an adapter's `detect_sessions` call.
    #[derive(Clone, Copy)]
    enum MockOutcome {
        OkEmpty,
        OkOne,
        ErrIo,
    }

    /// Test double that yields a pre-configured `detect_sessions` outcome.
    /// Other trait methods return a sentinel error — `detect_sessions` (the
    /// CLI function under test) must not call them. A failure here surfaces
    /// as a test error, not a panic, satisfying the workspace `panic = deny`
    /// lint.
    struct MockAdapter {
        agent_type: AgentType,
        outcome: MockOutcome,
    }

    impl Adapter for MockAdapter {
        fn agent_type(&self) -> AgentType {
            self.agent_type
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities::default()
        }
        fn config_dir(&self) -> PathBuf {
            PathBuf::from("/tmp/mock")
        }
        fn detect_sessions(&self) -> Result<Vec<AgentSession>> {
            match self.outcome {
                MockOutcome::OkEmpty => Ok(vec![]),
                MockOutcome::OkOne => Ok(vec![AgentSession::new(
                    SessionId::new_unchecked("sess-1"),
                    AgentType::Claude,
                    SessionStatus::Active,
                )]),
                MockOutcome::ErrIo => Err(Error::Adapter("permission denied".to_owned())),
            }
        }
        fn get_command_history(
            &self,
            _: &SessionId,
            _: HistoryDepth,
            _: usize,
        ) -> Result<Vec<Command>> {
            Err(Error::Adapter(
                "mock: get_command_history must not be called from detect_sessions".to_owned(),
            ))
        }
        fn poll_metrics(&self, _: &SessionId) -> Result<SessionMetrics> {
            Err(Error::Adapter(
                "mock: poll_metrics must not be called from detect_sessions".to_owned(),
            ))
        }
    }

    fn mock(agent_type: AgentType, outcome: MockOutcome) -> Box<dyn Adapter> {
        Box::new(MockAdapter {
            agent_type,
            outcome,
        })
    }

    #[test]
    fn detect_sessions_returns_false_when_all_adapters_succeed() {
        let adapters = vec![
            mock(AgentType::Claude, MockOutcome::OkOne),
            mock(AgentType::Codex, MockOutcome::OkEmpty),
        ];
        assert!(
            !detect_sessions(&adapters, false),
            "all adapters Ok → no error → ExitCode::SUCCESS"
        );
    }

    #[test]
    fn detect_sessions_returns_true_when_any_adapter_errors() {
        let adapters = vec![
            mock(AgentType::Claude, MockOutcome::OkOne),
            mock(AgentType::Codex, MockOutcome::ErrIo),
        ];
        assert!(
            detect_sessions(&adapters, false),
            "one adapter Err → any_error=true → ExitCode::FAILURE. \
             Reverting the Err arm would silently re-pass this test only \
             if the `any_error = true` line is preserved."
        );
    }

    #[test]
    fn detect_sessions_returns_true_when_all_adapters_error() {
        let adapters = vec![
            mock(AgentType::Claude, MockOutcome::ErrIo),
            mock(AgentType::Codex, MockOutcome::ErrIo),
        ];
        assert!(detect_sessions(&adapters, false));
    }

    #[test]
    fn detect_sessions_verbose_does_not_change_error_signal() {
        // Verbose only affects output formatting, not the exit-code decision.
        let adapters = vec![mock(AgentType::Claude, MockOutcome::ErrIo)];
        assert!(detect_sessions(&adapters, true));
        assert!(detect_sessions(&adapters, false));
    }
}
