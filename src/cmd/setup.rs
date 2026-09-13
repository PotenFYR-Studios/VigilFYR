//! `vigil setup` wizard with CI-friendly defaults.

use std::collections::BTreeMap;
use std::io::Write;

use anyhow::Result;

use vigil::agents::detect_agents;
use vigil::config::{AgentMode, Config};
use vigil::engine::Mode;

#[derive(Debug, Clone)]
pub struct SetupChoices {
    pub agents: Vec<(String, AgentMode)>,
    pub masking_enabled: bool,
    pub remote_update: bool,
    pub autostart: bool,
    pub tray: bool,
}

pub fn default_choices() -> SetupChoices {
    SetupChoices {
        agents: detect_agents()
            .into_iter()
            .filter(|agent| agent.supports_hooks)
            .map(|agent| (agent.id.to_string(), AgentMode::Enforce))
            .collect(),
        masking_enabled: false,
        remote_update: true,
        autostart: true,
        tray: true,
    }
}

pub fn apply(choices: SetupChoices) -> Result<Config> {
    let mut cfg = Config::default();
    cfg.masking.enabled = choices.masking_enabled;
    cfg.rules.remote_update = choices.remote_update;
    cfg.daemon.autostart = choices.autostart;
    cfg.daemon.tray = choices.tray;
    cfg.agents = choices
        .agents
        .into_iter()
        .collect::<BTreeMap<String, AgentMode>>()
        .into_iter()
        .collect();
    cfg.save()?;
    install_selected_hooks(&cfg)?;
    Ok(cfg)
}

fn install_selected_hooks(cfg: &Config) -> Result<()> {
    let detected = detect_agents();
    for agent in detected {
        if !agent.supports_hooks || agent.hook_paths.is_empty() {
            continue;
        }
        if matches!(cfg.agents.get(agent.id), Some(AgentMode::Off)) {
            continue;
        }
        for path in &agent.hook_paths {
            (agent.install_hook)(path, Mode::Enforce)?;
        }
    }
    Ok(())
}

pub fn setup(defaults: bool) -> Result<()> {
    let choices = if defaults {
        default_choices()
    } else {
        prompt()?
    };
    let cfg = apply(choices)?;
    println!(
        "Setup complete: {} agents configured; masking={}; remote-update={}.",
        cfg.agents.len(),
        cfg.masking.enabled,
        cfg.rules.remote_update
    );
    Ok(())
}

fn prompt() -> Result<SetupChoices> {
    let mut choices = default_choices();
    println!("Vigil setup - press Enter to accept defaults.");
    print!("Enable masking? [y/N]: ");
    std::io::stdout().flush()?;
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    choices.masking_enabled = matches!(answer.trim().to_lowercase().as_str(), "y" | "yes");

    answer.clear();
    print!("Enable remote rule updates? [Y/n]: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut answer)?;
    choices.remote_update = !matches!(answer.trim().to_lowercase().as_str(), "n" | "no");

    answer.clear();
    print!("Enable daemon autostart? [Y/n]: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut answer)?;
    choices.autostart = !matches!(answer.trim().to_lowercase().as_str(), "n" | "no");
    Ok(choices)
}
