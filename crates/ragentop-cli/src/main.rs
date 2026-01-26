use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ragentop", about = "Monitor AI coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Daemon,
    Tui,
    Status,
    Web,
}

fn main() {
    let _cli = Cli::parse();
    println!("ragentop v{}", env!("CARGO_PKG_VERSION"));
}
