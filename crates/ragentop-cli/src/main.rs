use clap::{Parser, Subcommand};
use ragentop_core::AgentAdapter;

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
    /// Launch the terminal UI [stub: not yet implemented]
    Tui,
    /// Show status of running agents
    Status,
    /// Start the web dashboard server [stub: not yet implemented]
    Web {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Detect all agent sessions on this machine
    Detect {
        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,
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
                println!("Starting ragentop daemon...");
                // TODO: Implement daemon start with ragentop_daemon
            }
            DaemonAction::Stop => {
                println!("Stopping ragentop daemon...");
                // TODO: Implement daemon stop
            }
        },
        Some(Commands::Tui) => {
            println!("Launching TUI...");
            let _app = ragentop_tui::App::new();
            // TODO: Run TUI event loop
        }
        Some(Commands::Status) => {
            println!("ragentop status");
            let tracker = ragentop_daemon::session::SessionTracker::new();
            let sessions = tracker.all();
            if sessions.is_empty() {
                println!("No active agent sessions.");
            } else {
                for session in sessions {
                    println!("  {session:?}");
                }
            }
        }
        Some(Commands::Web { port }) => {
            println!("Starting web server on port {port}...");
            // TODO: Start actual web server with ragentop_web
        }
        Some(Commands::Detect { verbose }) => {
            detect_sessions(verbose);
        }
        None => {
            println!("ragentop v{}", env!("CARGO_PKG_VERSION"));
            println!("Use --help for available commands.");
        }
    }
}

fn detect_sessions(verbose: bool) {
    use std::collections::HashMap;

    let adapters: Vec<Box<dyn AgentAdapter>> = vec![
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
                        .map_or_else(|| "unknown".to_string(), |p| p.display().to_string());
                    by_project.entry(key).or_default().push(session);
                }

                let session_count: usize = by_project.values().map(std::vec::Vec::len).sum();
                total_sessions += session_count;
                total_projects += by_project.len();

                println!(
                    "\n{:?}: {} projects, {} sessions",
                    adapter.agent_type(),
                    by_project.len(),
                    session_count
                );
                println!("  Config: {}", adapter.config_dir().display());

                // Sort projects by most recent session
                let mut projects: Vec<_> = by_project.into_iter().collect();
                projects.sort_by(|a, b| {
                    let a_time = a.1.iter().filter_map(|s| s.started_at).max();
                    let b_time = b.1.iter().filter_map(|s| s.started_at).max();
                    b_time.cmp(&a_time)
                });

                // Show top 10 projects (or all if verbose)
                let limit = if verbose { projects.len() } else { 10 };
                for (path, sessions) in projects.iter().take(limit) {
                    let active = sessions
                        .iter()
                        .filter(|s| s.status == ragentop_core::SessionStatus::Active)
                        .count();
                    if active > 0 {
                        println!(
                            "  {} ({} sessions, {} active)",
                            path,
                            sessions.len(),
                            active
                        );
                    } else {
                        println!("  {} ({} sessions)", path, sessions.len());
                    }
                }
                if !verbose && projects.len() > limit {
                    println!("  ... and {} more projects", projects.len() - limit);
                }
            }
            Err(e) if verbose => {
                println!("\n{:?}: error - {e}", adapter.agent_type());
            }
            Ok(_) | Err(_) => {}
        }
    }
    println!("\nTotal: {total_projects} projects, {total_sessions} sessions");
}
