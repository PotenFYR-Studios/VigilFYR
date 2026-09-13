use clap::{Parser, Subcommand};

mod cmd;

use cmd::intercept;
use cmd::rules;
use vigil::config::Config;

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
    /// Evaluate an event from stdin and print a verdict (agent hooks call this)
    Intercept(intercept::InterceptArgs),
    /// Interactive setup
    Setup,
    /// Manage rules
    #[command(subcommand)]
    Rules(RulesCommands),
    /// Manage configuration
    Config,
    /// Manage agents
    #[command(subcommand)]
    Agents(AgentsCommands),
    /// Reload daemon state
    Reload,
    /// Self-update
    Update,
    /// Launch the TUI (default)
    Tui,
}

#[derive(Subcommand)]
enum AgentsCommands {
    /// List known agents, detection and install status
    List,
    /// Install/update vigil hooks for an agent (or "all")
    Install {
        /// Agent id, or "all" (default: all)
        agent: Option<String>,
    },
    /// Remove vigil hooks for an agent (or "all"); other settings untouched
    Remove {
        /// Agent id, or "all" (default: all)
        agent: Option<String>,
    },
}

#[derive(Subcommand)]
enum RulesCommands {
    /// List effective rules (builtin → remote → user → project)
    List,
    /// Print rules directories
    Path,
    /// Sync remote rules now
    Update,
}

fn not_yet_implemented() -> anyhow::Result<()> {
    anyhow::bail!("not yet implemented")
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Daemon => not_yet_implemented(),
        Commands::Intercept(args) => intercept::intercept(args),
        Commands::Setup => not_yet_implemented(),
        Commands::Rules(cmd) => {
            let cfg = Config::load();
            match cmd {
                RulesCommands::List => rules::rules_list(&cfg),
                RulesCommands::Path => rules::rules_path(),
                RulesCommands::Update => rules::rules_update(&cfg),
            }
        }
        Commands::Config => not_yet_implemented(),
        Commands::Agents(cmd) => match cmd {
            AgentsCommands::List => cmd::agents::agents_list(),
            AgentsCommands::Install { agent } => cmd::agents::agents_install(agent.as_deref()),
            AgentsCommands::Remove { agent } => cmd::agents::agents_remove(agent.as_deref()),
        },
        Commands::Reload => {
            let cfg = Config::load();
            rules::reload(&cfg)
        }
        Commands::Update => not_yet_implemented(),
        Commands::Tui => not_yet_implemented(),
    }
}
