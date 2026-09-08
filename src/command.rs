use std::{fs, path::{Path, PathBuf}};

use crate::parser;

pub fn get_path() -> String {
    resolve_config_path(home::home_dir())
}

/// Resolves the `~/.ssh/config` path from a given home directory, creating an
/// empty config file if it does not exist yet. Split out from `get_path` so the
/// home-resolution branches can be unit tested without touching `$HOME`.
fn resolve_config_path(home: Option<PathBuf>) -> String {
    let mut path = match home {
        Some(p) => p.to_str().unwrap().to_string(),
        None => {
            println!("Impossible to get your home dir!");
            String::new()
        }
    };
    path.push_str("/.ssh/config");
    if !Path::new(&path).exists() {
        reset_config_file(&path);
    }
    path
}

fn reset_config_file(path: &str) {
    match fs::write(path, "") {
        Ok(_) => println!("Recreated config file!"),
        Err(_) => ()
    }
}

pub fn delete_all_command() {
    let path = get_path();
    reset_config_file(&path);
}

fn extract_host_name(host_item: &mut parser::HostItem, host_string: String) {
    let mut user = String::from("root");
    let mut hostname = String::from("");
    let mut port = String::from("22");

    let splitted: Vec<String> = host_string.split("@")
        .map(|s| s.to_string())
        .collect();

    if splitted.len() == 1 {
        hostname = String::from(&splitted[0]);
    } else if splitted.len() == 2 {
        user = String::from(&splitted[0]);
        hostname = String::from(&splitted[1]);
    }

    let hostname_splitted: Vec<String> = hostname.split(":")
        .map(|s| s.to_string())
        .collect();

    if hostname_splitted.len() == 2 {
        hostname = String::from(&hostname_splitted[0]);
        port = String::from(&hostname_splitted[1]);
    }

    host_item.user = user;
    host_item.host = hostname;
    host_item.port = port;
}

pub fn add_command(name: &String, host: &String, private_key: &Option<String>) {
    let path = get_path();

    let mut new_host = parser::HostItem::new();
    new_host.name = name.clone();
    extract_host_name(&mut new_host, host.to_string());
    if let Some(key) = private_key {
        new_host.identity_file = key.clone();
    }
    let mut config = parser::parse(&path);
    config.hosts.push(new_host);
    config.write(&path).unwrap();
}

pub fn edit_command(name: &String, host: &String, private_key: &Option<String>) {
    let path = get_path();
    let mut config = parser::parse(&path);
    let mut host_item = parser::HostItem::new();

    extract_host_name(&mut host_item, host.to_string());
    if let Some(key) = private_key {
        host_item.identity_file = key.clone();
    }

    match config.edit(name, &host_item) {
        Ok(_) => config.write(&path).unwrap(),
        Err(err) => eprintln!("Can not edit: {}", err)
    }
}

pub fn rename_command(name: &String, host: &String) {
    let path = get_path();
    let mut config = parser::parse(&path);
    match config.rename(name, host) {
        Ok(_) => config.write(&path).unwrap(),
        Err(err) => eprintln!("Can not rename: {}", err)
    }
}

pub fn delete_command(name: &String) {
    let path = get_path();
    let mut config = parser::parse(&path);
    match config.delete(name) {
        Ok(_) => config.write(&path).unwrap(),
        Err(err) => eprintln!("Can not delete: {}", err)
    }
}

pub fn list_command() {
    let path = get_path();
    let config = parser::parse(&path);

    if config.hosts.len() == 0 {
        println!("No records");
    } else {
        config.log()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::HostItem;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn unique_tmp(tag: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut dir = std::env::temp_dir();
        dir.push(format!("zoi_cmd_{}_{}_{}", tag, std::process::id(), n));
        dir
    }

    #[test]
    fn extract_hostname_only_uses_defaults() {
        let mut host = HostItem::new();
        extract_host_name(&mut host, "example.com".to_string());
        assert_eq!(host.user, "root");
        assert_eq!(host.host, "example.com");
        assert_eq!(host.port, "22");
    }

    #[test]
    fn extract_user_host_and_port() {
        let mut host = HostItem::new();
        extract_host_name(&mut host, "deploy@10.0.0.5:2222".to_string());
        assert_eq!(host.user, "deploy");
        assert_eq!(host.host, "10.0.0.5");
        assert_eq!(host.port, "2222");
    }

    #[test]
    fn extract_user_host_without_port_defaults_22() {
        let mut host = HostItem::new();
        extract_host_name(&mut host, "admin@myhost".to_string());
        assert_eq!(host.user, "admin");
        assert_eq!(host.host, "myhost");
        assert_eq!(host.port, "22");
    }

    #[test]
    fn extract_malformed_multiple_at_leaves_host_empty() {
        // "a@b@c" splits into 3 parts: neither the len==1 nor len==2 arm runs,
        // so hostname stays empty and the defaults are kept.
        let mut host = HostItem::new();
        extract_host_name(&mut host, "a@b@c".to_string());
        assert_eq!(host.user, "root");
        assert_eq!(host.host, "");
        assert_eq!(host.port, "22");
    }

    #[test]
    fn resolve_path_creates_config_when_missing() {
        let dir = unique_tmp("missing");
        std::fs::create_dir_all(dir.join(".ssh")).unwrap();

        let path = resolve_config_path(Some(dir.clone()));

        assert_eq!(path, format!("{}/.ssh/config", dir.to_str().unwrap()));
        assert!(Path::new(&path).exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_path_keeps_existing_config() {
        let dir = unique_tmp("existing");
        let ssh = dir.join(".ssh");
        std::fs::create_dir_all(&ssh).unwrap();
        std::fs::write(ssh.join("config"), "Host x\nhostname y\nuser z\nport 1\n").unwrap();

        let path = resolve_config_path(Some(dir.clone()));

        // Existing content must be preserved (reset must NOT run).
        assert!(std::fs::read_to_string(&path).unwrap().contains("Host x"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_path_without_home_does_not_panic() {
        // Exercises the `None` arm; writing to /.ssh/config fails silently so
        // this must return the fallback path without panicking.
        let path = resolve_config_path(None);
        assert_eq!(path, "/.ssh/config");
    }

    #[test]
    fn reset_config_file_truncates_and_ignores_bad_path() {
        let dir = unique_tmp("reset");
        std::fs::create_dir_all(&dir).unwrap();
        let good = dir.join("cfg");
        std::fs::write(&good, "some content").unwrap();

        reset_config_file(good.to_str().unwrap());
        assert_eq!(std::fs::read_to_string(&good).unwrap(), "");

        // Err branch: parent directory does not exist -> silently ignored.
        reset_config_file("/zoi_nonexistent_dir_xyz/cfg");

        std::fs::remove_dir_all(&dir).ok();
    }
}
