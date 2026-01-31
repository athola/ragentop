use clap::{Parser, Subcommand};
use ragentop_core::Adapter;

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
    /// Start the web dashboard server [stub: not yet implemented]
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

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Daemon { action }) => match action {
            DaemonAction::Start => {
                eprintln!("Starting ragentop daemon...");
                // TODO: Implement daemon start with ragentop_daemon
            }
            DaemonAction::Stop => {
                eprintln!("Stopping ragentop daemon...");
                // TODO: Implement daemon stop
            }
        },
        Some(Commands::Tui) => {
            eprintln!("Launching TUI...");
            let _app = ragentop_tui::App::new();
            // TODO: Run TUI event loop
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
        }
        Some(Commands::Web { port }) => {
            eprintln!("Starting web server on port {port}...");
            // TODO: Start actual web server with ragentop_web
        }
        Some(Commands::Detect { verbose }) => {
            detect_sessions(verbose);
        }
        None => {
            eprintln!("ragentop v{}", env!("CARGO_PKG_VERSION"));
            eprintln!("Use --help for available commands.");
        }
    }
}

fn detect_sessions(verbose: bool) {
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
            Err(err) if verbose => {
                eprintln!("\n{:?}: error - {err}", adapter.agent_type());
            }
            Ok(_) | Err(_) => {}
        }
    }
    eprintln!("\nTotal: {total_projects} projects, {total_sessions} sessions");
}
