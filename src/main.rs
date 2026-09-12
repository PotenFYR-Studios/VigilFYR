use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vigil", version, about = "VigilFYR: failover-ready CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the background daemon
    Daemon,
    /// Intercept commands
    Intercept,
    /// Interactive setup
    Setup,
    /// Manage rules
    Rules,
    /// Manage configuration
    Config,
    /// Manage agents
    Agents,
    /// Reload daemon state
    Reload,
    /// Self-update
    Update,
    /// Launch the TUI (default)
    Tui,
}

fn not_yet_implemented() -> anyhow::Result<()> {
    anyhow::bail!("not yet implemented")
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Daemon => not_yet_implemented(),
        Commands::Intercept => not_yet_implemented(),
        Commands::Setup => not_yet_implemented(),
        Commands::Rules => not_yet_implemented(),
        Commands::Config => not_yet_implemented(),
        Commands::Agents => not_yet_implemented(),
        Commands::Reload => not_yet_implemented(),
        Commands::Update => not_yet_implemented(),
        Commands::Tui => not_yet_implemented(),
    }
}
