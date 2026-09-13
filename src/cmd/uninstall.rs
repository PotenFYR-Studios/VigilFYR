//! `vigil uninstall` CLI: detect-then-teardown with a keep-config prompt.

use anyhow::{bail, Result};
use std::io::IsTerminal;

use vigil::uninstall::{uninstall, ConfigPolicy, UninstallOptions};

/// Prompt until the user answers yes/no; `defaults[0]` is returned on EOF
/// (non-interactive shells, CI).
fn ask_yes_no(prompt: &str, default_yes: bool) -> bool {
    if !std::io::stdin().is_terminal() {
        return default_yes;
    }
    let hint = if default_yes { "Y/n" } else { "y/N" };
    println!("{prompt} [{hint}] ");
    let mut answer = String::new();
    match std::io::stdin().read_line(&mut answer) {
        Ok(0) | Err(_) => default_yes,
        Ok(_) => match answer.trim().to_ascii_lowercase().as_str() {
            "" => default_yes,
            "y" | "yes" => true,
            "n" | "no" => false,
            _ => default_yes,
        },
    }
}

/// `vigil uninstall [--keep-config] [--wipe-config] [--dry-run] [-y]`
pub fn run(keep_config: Option<bool>, dry_run: bool, assume_yes: bool) -> Result<()> {
    if !dry_run && !assume_yes {
        let proceed = ask_yes_no(
            "Uninstall Vigil? This removes agent hooks, autostart, and state.",
            false,
        );
        if !proceed {
            println!("aborted; nothing changed");
            return Ok(());
        }
    }

    let detected = vigil::uninstall::detect_state();
    println!("detected installation state:");
    for row in &detected {
        let mark = if row.present { "[x]" } else { "[ ]" };
        println!("  {mark} {:<14} {}", row.label, row.detail);
    }

    if dry_run {
        println!("dry run: no changes made");
        return Ok(());
    }

    let keep = match keep_config {
        Some(explicit) => explicit,
        None => ask_yes_no(
            "Keep ~/.vigil (config, rules, events) for reuse after reinstall?",
            true,
        ),
    };

    let opts = UninstallOptions {
        config: if keep {
            ConfigPolicy::Keep
        } else {
            ConfigPolicy::Wipe
        },
        remove_hooks: true,
        remove_autostart: true,
        remove_binary: ask_yes_no(
            "Also delete the vigil binary from PATH? (installers can be re-run)",
            false,
        ),
    };

    let report = uninstall(&opts)?;
    for line in &report {
        println!("{line}");
    }

    match opts.config {
        ConfigPolicy::Keep => {
            println!("uninstalled. config kept — `vigil setup` after reinstalling reuses it")
        }
        ConfigPolicy::Wipe => println!("uninstalled. all state removed"),
    }
    Ok(())
}

/// Re-exported for main.rs arg struct docs.
pub fn ensure_not_both(a: Option<bool>, b: Option<bool>) -> Result<()> {
    if a.is_some() && b.is_some() {
        bail!("--keep-config and --wipe-config are mutually exclusive");
    }
    Ok(())
}
