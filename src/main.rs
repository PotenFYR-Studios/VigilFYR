use clap::{Parser, Subcommand};

mod cmd;

use cmd::config;
use cmd::daemon;
use cmd::ext;
use cmd::intercept;
use cmd::rules;
use cmd::setup;
use cmd::shim;
use cmd::update;
use vigil::config::Config;

#[derive(Parser)]
#[command(
    name = "vigil",
    version,
    about = "VigilFYR: local-first guard for AI coding agents"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the background daemon
    Daemon {
        /// Disable tray integration even when available
        #[arg(long)]
        no_tray: bool,
    },
    /// Evaluate an event from stdin and print a verdict (agent hooks call this)
    Intercept(intercept::InterceptArgs),
    /// Run a command under Vigil's filesystem monitoring wrapper
    /// (monitoring only - hooks enforce)
    Shim(shim::ShimArgs),
    /// Interactive setup
    Setup {
        /// Accept every default without prompts
        #[arg(long)]
        defaults: bool,
    },
    /// Manage rules
    #[command(subcommand)]
    Rules(RulesCommands),
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Manage extensions
    Extension(ext::ExtensionArgs),
    /// Manage agents
    #[command(subcommand)]
    Agents(AgentsCommands),
    /// Reload daemon state
    Reload,
    /// Self-update
    Update(update::UpdateArgs),
    /// Launch the TUI (default)
    Tui {
        /// Print one snapshot and exit
        #[arg(long)]
        once: bool,
    },
    /// Run read-only setup and policy health checks
    Doctor,
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
    /// Evaluate a synthetic event and print the verdict
    Test {
        /// Event action: read, write, search, exec, net, delete, rename, connect
        action: String,
        /// Paths to evaluate
        #[arg(default_values_t = Vec::<String>::new())]
        paths: Vec<String>,
        /// Command line to evaluate
        #[arg(long)]
        command: Option<String>,
        /// Simulated agent id
        #[arg(long)]
        agent: Option<String>,
        /// Simulated tool name
        #[arg(long)]
        tool: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    Get {
        /// Dotted configuration key
        key: Option<String>,
    },
    Set {
        /// Dotted configuration key
        key: Option<String>,
        /// New value
        value: Option<String>,
    },
    /// Print every known key
    List,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Tui { once: false }) {
        Commands::Daemon { no_tray } => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            runtime.block_on(daemon::daemon(no_tray))
        }
        Commands::Intercept(args) => intercept::intercept(args),
        Commands::Shim(args) => {
            let cmd = args.command.clone();
            shim::shim(args, &cmd)
        }
        Commands::Setup { defaults } => setup::setup(defaults),
        Commands::Rules(cmd) => {
            let cfg = Config::load();
            match cmd {
                RulesCommands::List => rules::rules_list(&cfg),
                RulesCommands::Path => rules::rules_path(),
                RulesCommands::Update => rules::rules_update(&cfg),
                RulesCommands::Test {
                    action,
                    paths,
                    command,
                    agent,
                    tool,
                } => {
                    let args = rules::RuleTestArgs {
                        action: action
                            .parse()
                            .map_err(|error: String| anyhow::anyhow!(error))?,
                        paths,
                        command,
                        agent,
                        tool,
                    };
                    rules::rules_test(args)
                }
            }
        }
        Commands::Config { command } => match command {
            ConfigCommands::Get { key } => config::config_get(key.as_deref()),
            ConfigCommands::Set { key, value } => {
                config::config_set(key.as_deref(), value.as_deref())
            }
            ConfigCommands::List => config::config_list(),
        },
        Commands::Agents(cmd) => match cmd {
            AgentsCommands::List => cmd::agents::agents_list(),
            AgentsCommands::Install { agent } => cmd::agents::agents_install(agent.as_deref()),
            AgentsCommands::Remove { agent } => cmd::agents::agents_remove(agent.as_deref()),
        },
        Commands::Extension(args) => ext::run(args),
        Commands::Reload => {
            let cfg = Config::load();
            rules::reload(&cfg)
        }
        Commands::Update(args) => update::run(args),
        Commands::Tui { once } => vigil::tui::run_once(once),
        Commands::Doctor => rules::doctor(),
    }
}
