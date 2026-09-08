//! End-to-end tests that drive the real `zoi` binary against a throwaway
//! `$HOME`, so `main.rs` and every `*_command` wrapper are exercised for real.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn unique_home() -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut dir = std::env::temp_dir();
    dir.push(format!("zoi_cli_{}_{}", std::process::id(), n));
    std::fs::create_dir_all(dir.join(".ssh")).unwrap();
    dir
}

fn run(home: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zoi"))
        .env("HOME", home)
        .args(args)
        .output()
        .expect("failed to run zoi")
}

fn config(home: &PathBuf) -> String {
    std::fs::read_to_string(home.join(".ssh/config")).unwrap_or_default()
}

#[test]
fn full_lifecycle_of_every_command() {
    let home = unique_home();

    // list on an empty config (also triggers first-run config creation).
    let out = run(&home, &["list"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("No records"));

    // add with --private-key, and a plain add + edit -i short flag.
    assert!(run(&home, &["add", "zino-pg", "deploy@10.0.0.5:2222", "--private-key", "~/.ssh/id_me"]).status.success());
    assert!(run(&home, &["add", "web", "root@example.com"]).status.success());
    assert!(run(&home, &["edit", "web", "root@example.com", "-i", "~/.ssh/id_web"]).status.success());

    let cfg = config(&home);
    assert!(cfg.contains("Host zino-pg"));
    assert!(cfg.contains("identityfile ~/.ssh/id_me"));
    assert!(cfg.contains("identityfile ~/.ssh/id_web"));

    // list with records goes through Config::log().
    let out = run(&home, &["list"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("zino-pg"));
    assert!(stdout.contains("web"));

    // edit without --private-key must keep the existing key.
    assert!(run(&home, &["edit", "zino-pg", "deploy@10.0.0.9:2200"]).status.success());
    let cfg = config(&home);
    assert!(cfg.contains("port 2200"));
    assert!(cfg.contains("identityfile ~/.ssh/id_me"));

    // rename.
    assert!(run(&home, &["rename", "zino-pg", "pg"]).status.success());
    let cfg = config(&home);
    assert!(cfg.contains("Host pg"));
    assert!(!cfg.contains("Host zino-pg"));

    // delete a single host.
    assert!(run(&home, &["delete", "web"]).status.success());
    assert!(!config(&home).contains("Host web"));

    // delete-all empties the file.
    assert!(run(&home, &["delete-all"]).status.success());
    assert_eq!(config(&home), "");

    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn error_paths_report_missing_host() {
    let home = unique_home();
    run(&home, &["add", "only", "user@host"]);

    let out = run(&home, &["edit", "nope", "user@host"]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("Can not edit"));

    let out = run(&home, &["rename", "nope", "x"]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("Can not rename"));

    let out = run(&home, &["delete", "nope"]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("Can not delete"));

    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn no_subcommand_and_meta_flags() {
    let home = unique_home();

    // No subcommand hits the `None => {}` arm and exits cleanly.
    assert!(run(&home, &[]).status.success());
    // Built-in help and version.
    assert!(run(&home, &["--help"]).status.success());
    assert!(run(&home, &["--version"]).status.success());

    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn preserves_unmanaged_ssh_fields_across_edit() {
    let home = unique_home();
    let cfg_path = home.join(".ssh/config");
    std::fs::write(
        &cfg_path,
        "# Global options\nServerAliveInterval 60\n\nHost box\nhostname 10.0.0.5\nuser deploy\nport 2222\nForwardAgent yes\nProxyJump bastion\nLocalForward 5432 localhost:5432\n# keep this\n",
    )
    .unwrap();

    assert!(run(&home, &["edit", "box", "deploy@10.0.0.9:2200"]).status.success());

    let cfg = config(&home);
    for needle in [
        "# Global options",
        "ServerAliveInterval 60",
        "ForwardAgent yes",
        "ProxyJump bastion",
        "LocalForward 5432 localhost:5432",
        "# keep this",
        "port 2200",
    ] {
        assert!(cfg.contains(needle), "lost `{needle}` after edit:\n{cfg}");
    }

    std::fs::remove_dir_all(&home).ok();
}
