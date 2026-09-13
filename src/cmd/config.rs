//! `vigil config get|set|list`.

use anyhow::{bail, Result};

use vigil::config::Config;

const KEYS: &[&str] = &[
    "general.enabled",
    "general.mode",
    "general.check_updates",
    "daemon.autostart",
    "daemon.tray",
    "masking.enabled",
    "rules.remote_update",
];

pub fn config_get(key: Option<&str>) -> Result<()> {
    let key = key.ok_or_else(|| anyhow::anyhow!("usage: vigil config get KEY"))?;
    if !KEYS.contains(&key) && !key.starts_with("agents.") {
        bail!("unknown config key: {key}");
    }
    println!("{}", Config::load().get(key)?);
    Ok(())
}

pub fn config_set(key: Option<&str>, value: Option<&str>) -> Result<()> {
    let (key, value) = match (key, value) {
        (Some(key), Some(value)) => (key, value),
        _ => bail!("usage: vigil config set KEY VALUE"),
    };
    if !KEYS.contains(&key) && !key.starts_with("agents.") {
        bail!("unknown config key: {key}");
    }
    let mut cfg = Config::load();
    cfg.set(key, value)?;
    cfg.save()?;
    println!("{key} = {value}");
    Ok(())
}

pub fn config_list() -> Result<()> {
    let cfg = Config::load();
    for key in KEYS {
        println!("{key}={}", cfg.get(key)?);
    }
    for (agent, mode) in &cfg.agents {
        println!("agents.{agent}={mode}");
    }
    Ok(())
}
