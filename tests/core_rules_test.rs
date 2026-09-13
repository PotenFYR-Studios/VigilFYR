//! Integration tests for the built-in core ruleset.

use std::path::PathBuf;

use vigil::engine::Mode;
use vigil::event::{Action, Event, VerdictAction};
use vigil::rules::{builtin_rules, Ruleset};

fn ev(action: Action, paths: &[&str]) -> Event {
    Event {
        schema: 1,
        agent: "claude-code".to_string(),
        tool: "test".to_string(),
        action,
        paths: paths.iter().map(PathBuf::from).collect(),
        command: None,
        cwd: PathBuf::from("/home/u/proj"),
        session: "s1".to_string(),
    }
}

fn ev_cmd(action: Action, command: &str) -> Event {
    Event {
        command: Some(command.to_string()),
        ..ev(action, &[])
    }
}

#[test]
fn builtin_rules_block_canonical_secrets() {
    let rs = Ruleset::compile(builtin_rules());
    for p in [
        "/home/u/proj/.env",
        "/home/u/.ssh/id_rsa",
        "/home/u/proj/.env.local",
        "/etc/passwd",
    ] {
        let v = rs.evaluate(&ev(Action::Read, &[p]), Mode::Enforce);
        assert_eq!(v.action, VerdictAction::Deny, "must deny read of {p}");
    }
}

#[test]
fn builtin_rules_block_enterprise_secret_stores() {
    let rs = Ruleset::compile(builtin_rules());
    for p in [
        "/home/u/proj/.github/workflows/deploy.yml",
        "/home/u/proj/.vault-token",
        "/home/u/proj/terraform.tfstate",
        "/home/u/.cargo/credentials.toml",
        "/home/u/proj/appsettings.Production.json",
        "/home/u/Library/Application Support/1Password/Backup/1Password.sqlite",
        "/home/u/proj/.claude/settings.local.json",
        "/home/u/.zsh_history",
    ] {
        let v = rs.evaluate(&ev(Action::Read, &[p]), Mode::Enforce);
        assert_eq!(
            v.action,
            VerdictAction::Deny,
            "must deny read of {p}: {}",
            v.reason
        );
    }
}

#[test]
fn builtin_rules_warn_on_repository_metadata() {
    let rs = Ruleset::compile(builtin_rules());
    let v = rs.evaluate(
        &ev(Action::Read, &["/home/u/proj/.git/objects/ab/cdef"]),
        Mode::Enforce,
    );
    assert_eq!(v.action, VerdictAction::Warn);
    assert_eq!(v.rule, "warn-source-control-metadata");
}

#[test]
fn builtin_rules_deny_private_key_write() {
    let rs = Ruleset::compile(builtin_rules());
    let v = rs.evaluate(
        &ev(Action::Write, &["/home/u/proj/server.pem"]),
        Mode::Enforce,
    );
    assert_eq!(v.action, VerdictAction::Deny, "must deny write of .pem");
}

#[test]
fn builtin_rules_deny_secrets_dir_and_demote_in_audit() {
    let rs = Ruleset::compile(builtin_rules());
    let enforce = rs.evaluate(
        &ev(Action::Read, &["/home/u/proj/secrets/token.txt"]),
        Mode::Enforce,
    );
    assert_eq!(enforce.action, VerdictAction::Deny);
    let audit = rs.evaluate(
        &ev(Action::Read, &["/home/u/proj/secrets/token.txt"]),
        Mode::Audit,
    );
    assert_eq!(
        audit.action,
        VerdictAction::Warn,
        "audit mode must demote deny to warn"
    );
}

#[test]
fn builtin_rules_deny_destructive_exec() {
    let rs = Ruleset::compile(builtin_rules());
    for cmd in ["sudo rm -rf /", "rm -rf /"] {
        let v = rs.evaluate(&ev_cmd(Action::Exec, cmd), Mode::Enforce);
        assert_eq!(v.action, VerdictAction::Deny, "must deny exec of {cmd}");
    }
    let home_delete = rs.evaluate(&ev_cmd(Action::Exec, "rm -rf ~"), Mode::Enforce);
    assert_eq!(
        home_delete.action,
        VerdictAction::Warn,
        "home recursive delete must warn, not deny"
    );
    let deep_delete = rs.evaluate(&ev_cmd(Action::Exec, "rm -rf /home/u/tmp"), Mode::Enforce);
    assert_eq!(
        deep_delete.action,
        VerdictAction::Warn,
        "delete below home must warn, not be denied as root delete"
    );
}

#[test]
fn builtin_rules_deny_root_delete_variants() {
    let rs = Ruleset::compile(builtin_rules());
    for cmd in [
        "rm -rf /*",
        "rm --recursive /",
        "rm -rf --no-preserve-root /",
        "sudo rm -rf /*",
    ] {
        let v = rs.evaluate(&ev_cmd(Action::Exec, cmd), Mode::Enforce);
        assert_eq!(v.action, VerdictAction::Deny, "must deny exec of {cmd}");
    }
}

#[test]
fn builtin_rules_deny_sudo_copy_into_system_dir() {
    let rs = Ruleset::compile(builtin_rules());
    for cmd in [
        "sudo cp payload /etc/hosts",
        "sudo mv evil /usr/bin/vigil-backdoor",
        "sudo install -m 755 x /boot/vmlinuz.bak",
    ] {
        let v = rs.evaluate(&ev_cmd(Action::Exec, cmd), Mode::Enforce);
        assert_eq!(v.action, VerdictAction::Deny, "must deny exec of {cmd}");
    }
}

#[test]
fn builtin_rules_block_exfiltration_and_recon_commands() {
    let rs = Ruleset::compile(builtin_rules());
    for (command, expected) in [
        (
            "tar -czf backup.tgz /home/u/.ssh/id_rsa",
            VerdictAction::Deny,
        ),
        ("grep -R password /home/u", VerdictAction::Deny),
        ("nmap -sV 192.168.1.1", VerdictAction::Deny),
        ("cat /etc/shadow", VerdictAction::Deny),
        ("sudo useradd backdoor", VerdictAction::Deny),
        ("systemctl enable evil.service", VerdictAction::Deny),
        ("docker exec -it web sh", VerdictAction::Deny),
        ("ps aux", VerdictAction::Deny),
        ("npm install left-pad", VerdictAction::Warn),
        ("cargo build --release", VerdictAction::Allow),
    ] {
        let v = rs.evaluate(&ev_cmd(Action::Exec, command), Mode::Enforce);
        assert_eq!(
            v.action, expected,
            "{command} => {} ({})",
            v.action, v.reason
        );
    }
}

#[test]
fn builtin_rules_block_internal_and_exfil_networks() {
    let rs = Ruleset::compile(builtin_rules());
    for command in [
        "curl http://kubernetes.default.svc.cluster.local",
        "curl https://webhook.site/example",
        "curl https://transfer.sh/secret.txt",
    ] {
        let v = rs.evaluate(&ev_cmd(Action::Net, command), Mode::Enforce);
        assert_eq!(v.action, VerdictAction::Deny, "{command}: {}", v.reason);
    }
    let registry = rs.evaluate(
        &ev_cmd(Action::Net, "curl https://registry.npmjs.org/vigil"),
        Mode::Enforce,
    );
    assert_eq!(registry.action, VerdictAction::Warn);
}
