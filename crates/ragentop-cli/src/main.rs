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
            let any_error = detect_sessions(verbose);
            if any_error {
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

/// Detects sessions across all bundled adapters and prints a summary.
///
/// Returns `true` if any adapter returned an error, so callers can surface
/// a non-zero exit code instead of pretending "no sessions found" on
/// permission errors, malformed configs, or panicked adapters.
fn detect_sessions(verbose: bool) -> bool {
    use std::collections::HashMap;

    let adapters: Vec<Box<dyn Adapter>> = vec![
        Box::new(adapter_claude::ClaudeAdapter::new()),
        Box::new(adapter_codex::CodexAdapter::new()),
        Box::new(adapter_copilot::CopilotAdapter::new()),
        Box::new(adapter_gemini::GeminiAdapter::new()),
        Box::new(adapter_qwen::QwenAdapter::new()),
    ];

    let mut total_sessions = 0;
    let mut total_projects = 0;
    let mut any_error = false;

    for adapter in &adapters {
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
