use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use colored::*;

#[derive(Clone)]
pub struct HostItem {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: String,
    pub identity_file: String,
    /// Any other lines belonging to this host block that zoi does not manage
    /// directly (e.g. ProxyJump, ForwardAgent, LocalForward, comments...).
    /// Preserved verbatim, in order, so rewriting the config never drops them.
    pub extras: Vec<String>,
}

impl HostItem {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            host: String::new(),
            user: String::new(),
            port: String::new(),
            identity_file: String::new(),
            extras: Vec::new(),
        }
    }
}

pub struct Config {
    pub hosts: Vec<HostItem>,
    /// Lines that appear before the first `Host` block (global options,
    /// leading comments...). Preserved verbatim.
    pub preamble: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            hosts: Vec::new(),
            preamble: Vec::new(),
        }
    }

    pub fn log(&self) {
        let max_name_len = self.hosts.iter().map(|host| host.name.len()).max().unwrap_or(0);

        println!();
        for host in &self.hosts {
            let spaces = " ".repeat(max_name_len - host.name.len());
            let leading_whitespaces = " ".repeat(5);

            let mut target = String::new();
            if !host.user.is_empty() {
                target.push_str(&format!("{}@", host.user.bold()));
            }
            target.push_str(&format!("{}", host.host.bold().yellow()));
            if !host.port.is_empty() {
                target.push_str(&format!(":{}", host.port));
            }

            let identity = if host.identity_file.is_empty() {
                String::new()
            } else {
                format!(" ({})", host.identity_file.dimmed())
            };

            println!(
                "{leading_whitespaces}{0}{spaces} -> {1}{2}\n",
                host.name.bold().green(),
                target,
                identity
            );
        }
    }

    pub fn get_host_copy(&self, name: &str) -> Option<HostItem> {
        self.hosts.iter().find(|host| host.name == name).cloned()
    }

    pub fn rename(&mut self, name: &str, new_name: &str) -> Result<(), &'static str> {
        if let Some(host) = self.hosts.iter_mut().find(|x| x.name == name) {
            host.name = new_name.to_string();
            Ok(())
        } else {
            Err("No host found!")
        }
    }

    pub fn edit(&mut self, name: &str, host_item: &HostItem) -> Result<(), &'static str> {
        if let Some(host) = self.hosts.iter_mut().find(|x| x.name == name) {
            host.host = host_item.host.clone();
            host.user = host_item.user.clone();
            host.port = host_item.port.clone();
            // Only overwrite the private key when a new one is provided,
            // otherwise keep the existing IdentityFile untouched.
            if !host_item.identity_file.is_empty() {
                host.identity_file = host_item.identity_file.clone();
            }
            // `extras` are intentionally left untouched so every other ssh
            // config directive on this host survives the edit.
            Ok(())
        } else {
            Err("No host found!")
        }
    }

    pub fn delete(&mut self, name: &str) -> Result<(), &'static str> {
        if let Some(pos) = self.hosts.iter().position(|x| x.name == name) {
            self.hosts.remove(pos);
            Ok(())
        } else {
            Err("No host found!")
        }
    }

    pub fn write(&self, path: &str) -> io::Result<()> {
        // File::create truncates the file to empty, then we write the whole
        // config back out.
        let mut file = File::create(path)?;

        for line in &self.preamble {
            file.write_all(line.as_bytes())?;
            file.write_all(b"\n")?;
        }

        for host in &self.hosts {
            let mut block = format!("Host {}\n", host.name);

            if !host.host.is_empty() {
                block.push_str(&format!("hostname {}\n", host.host));
            }
            if !host.user.is_empty() {
                block.push_str(&format!("user {}\n", host.user));
            }
            if !host.port.is_empty() {
                block.push_str(&format!("port {}\n", host.port));
            }
            if !host.identity_file.is_empty() {
                block.push_str(&format!("identityfile {}\n", host.identity_file));
            }
            for extra in &host.extras {
                block.push_str(extra);
                block.push('\n');
            }

            file.write_all(block.as_bytes())?;
        }
        Ok(())
    }
}

fn read_lines(filename: &str) -> io::Lines<BufReader<File>> {
    let file = File::open(filename).expect("Failed to open file");
    BufReader::new(file).lines()
}

pub fn parse(filename: &str) -> Config {
    let lines = read_lines(filename);

    let mut config = Config::new();
    let mut current_host: Option<HostItem> = None;

    for line in lines {
        let raw = line.expect("Failed to read line");
        let trimmed = raw.trim();
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        // ssh config keywords are case-insensitive.
        let keyword = tokens.first().map(|k| k.to_lowercase());

        match keyword.as_deref() {
            Some("host") if tokens.len() >= 2 => {
                if let Some(host) = current_host.take() {
                    config.hosts.push(host);
                }
                let mut host = HostItem::new();
                host.name = tokens[1].to_string();
                current_host = Some(host);
            }
            _ => match current_host.as_mut() {
                Some(host) => match (keyword.as_deref(), tokens.len()) {
                    (Some("hostname"), 2) => host.host = tokens[1].to_string(),
                    (Some("user"), 2) => host.user = tokens[1].to_string(),
                    (Some("port"), 2) => host.port = tokens[1].to_string(),
                    (Some("identityfile"), 2) => host.identity_file = tokens[1].to_string(),
                    // Anything else on this host is kept verbatim (trimmed to
                    // match zoi's left-aligned output style).
                    _ => host.extras.push(trimmed.to_string()),
                },
                // Lines before the first Host block are preserved as preamble.
                None => config.preamble.push(trimmed.to_string()),
            },
        }
    }

    if let Some(host) = current_host.take() {
        config.hosts.push(host);
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn tmp_file(tag: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut dir = std::env::temp_dir();
        dir.push(format!("zoi_parser_{}_{}_{}", tag, std::process::id(), n));
        dir
    }

    fn parse_str(content: &str, tag: &str) -> (Config, PathBuf) {
        let path = tmp_file(tag);
        std::fs::write(&path, content).unwrap();
        (parse(path.to_str().unwrap()), path)
    }

    // ---- HostItem / Config constructors ----

    #[test]
    fn new_host_item_is_empty() {
        let h = HostItem::new();
        assert!(h.name.is_empty() && h.host.is_empty() && h.user.is_empty());
        assert!(h.port.is_empty() && h.identity_file.is_empty());
        assert!(h.extras.is_empty());
    }

    #[test]
    fn new_config_is_empty() {
        let c = Config::new();
        assert!(c.hosts.is_empty());
        assert!(c.preamble.is_empty());
    }

    // ---- parse ----

    #[test]
    fn parse_full_block_all_managed_fields() {
        let (c, p) = parse_str(
            "Host a\nhostname h\nuser u\nport 2222\nidentityfile ~/.ssh/id\n",
            "full",
        );
        assert_eq!(c.hosts.len(), 1);
        let h = &c.hosts[0];
        assert_eq!(h.name, "a");
        assert_eq!(h.host, "h");
        assert_eq!(h.user, "u");
        assert_eq!(h.port, "2222");
        assert_eq!(h.identity_file, "~/.ssh/id");
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn parse_preserves_preamble_extras_and_multiple_hosts() {
        let content = "\
# Global options
ServerAliveInterval 60

Host one
hostname h1
user u1
port 22
ForwardAgent yes
LocalForward 5432 localhost:5432
# a comment

Host two
hostname h2
";
        let (c, p) = parse_str(content, "extras");
        // preamble captured (comment, directive, blank line)
        assert_eq!(c.preamble, vec![
            "# Global options".to_string(),
            "ServerAliveInterval 60".to_string(),
            "".to_string(),
        ]);
        assert_eq!(c.hosts.len(), 2);
        let one = &c.hosts[0];
        assert_eq!(one.name, "one");
        // unknown keyword, multi-token line, comment and a blank line all kept
        assert_eq!(one.extras, vec![
            "ForwardAgent yes".to_string(),
            "LocalForward 5432 localhost:5432".to_string(),
            "# a comment".to_string(),
            "".to_string(),
        ]);
        assert_eq!(c.hosts[1].name, "two");
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn parse_is_case_insensitive_for_keywords() {
        let (c, p) = parse_str(
            "HOST a\nHostName h\nUser u\nPort 22\nIdentityFile k\n",
            "case",
        );
        let h = &c.hosts[0];
        assert_eq!(h.host, "h");
        assert_eq!(h.user, "u");
        assert_eq!(h.port, "22");
        assert_eq!(h.identity_file, "k");
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn parse_bare_host_keyword_goes_to_preamble_or_extras() {
        // Leading bare "Host" (no name) -> preamble; a bare "Host" inside a
        // block -> that host's extras.
        let (c, p) = parse_str("Host\nHost real\nhostname h\nHost\n", "bare");
        assert_eq!(c.preamble, vec!["Host".to_string()]);
        assert_eq!(c.hosts.len(), 1);
        assert_eq!(c.hosts[0].name, "real");
        assert_eq!(c.hosts[0].extras, vec!["Host".to_string()]);
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn parse_empty_file_yields_empty_config() {
        let (c, p) = parse_str("", "empty");
        assert!(c.hosts.is_empty());
        assert!(c.preamble.is_empty());
        std::fs::remove_file(p).ok();
    }

    // ---- write (round-trips through parse) ----

    #[test]
    fn write_round_trips_all_fields_extras_and_preamble() {
        let mut c = Config::new();
        c.preamble.push("# top".to_string());
        let mut h = HostItem::new();
        h.name = "a".to_string();
        h.host = "hh".to_string();
        h.user = "uu".to_string();
        h.port = "2200".to_string();
        h.identity_file = "~/.ssh/id".to_string();
        h.extras.push("ForwardAgent yes".to_string());
        c.hosts.push(h);

        let path = tmp_file("write_full");
        c.write(path.to_str().unwrap()).unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert_eq!(written, "\
# top
Host a
hostname hh
user uu
port 2200
identityfile ~/.ssh/id
ForwardAgent yes
");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn write_returns_err_when_path_is_unwritable() {
        // Covers the error-propagation (`?`) arm of `File::create` in `write`.
        let res = Config::new().write("/zoi_nonexistent_dir_for_write/config");
        assert!(res.is_err());
    }

    #[test]
    fn write_skips_empty_managed_fields() {
        // A host with only a name must emit just the `Host` line, never blank
        // `hostname`/`user`/`port`/`identityfile` directives.
        let mut c = Config::new();
        let mut h = HostItem::new();
        h.name = "bare".to_string();
        c.hosts.push(h);

        let path = tmp_file("write_bare");
        c.write(path.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "Host bare\n");
        std::fs::remove_file(path).ok();
    }

    // ---- get_host_copy ----

    #[test]
    fn get_host_copy_found_and_not_found() {
        let (c, p) = parse_str("Host a\nhostname h\nuser u\nport 22\n", "copy");
        assert!(c.get_host_copy("a").is_some());
        assert!(c.get_host_copy("missing").is_none());
        std::fs::remove_file(p).ok();
    }

    // ---- rename ----

    #[test]
    fn rename_ok_and_err() {
        let (mut c, p) = parse_str("Host a\nhostname h\nuser u\nport 22\n", "rename");
        assert!(c.rename("a", "b").is_ok());
        assert_eq!(c.hosts[0].name, "b");
        assert_eq!(c.rename("nope", "x"), Err("No host found!"));
        std::fs::remove_file(p).ok();
    }

    // ---- edit ----

    #[test]
    fn edit_updates_fields_and_overwrites_key_when_provided() {
        let (mut c, p) = parse_str(
            "Host a\nhostname h\nuser u\nport 22\nidentityfile old\n",
            "edit_key",
        );
        let mut item = HostItem::new();
        item.host = "h2".to_string();
        item.user = "u2".to_string();
        item.port = "2200".to_string();
        item.identity_file = "new".to_string();
        assert!(c.edit("a", &item).is_ok());
        let h = &c.hosts[0];
        assert_eq!(h.host, "h2");
        assert_eq!(h.user, "u2");
        assert_eq!(h.port, "2200");
        assert_eq!(h.identity_file, "new");
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn edit_preserves_key_when_not_provided() {
        let (mut c, p) = parse_str(
            "Host a\nhostname h\nuser u\nport 22\nidentityfile keepme\n",
            "edit_keep",
        );
        let mut item = HostItem::new();
        item.host = "h2".to_string();
        item.user = "u2".to_string();
        item.port = "2200".to_string();
        // identity_file left empty -> must not clobber the existing key
        assert!(c.edit("a", &item).is_ok());
        assert_eq!(c.hosts[0].identity_file, "keepme");
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn edit_missing_host_errors() {
        let (mut c, p) = parse_str("Host a\nhostname h\nuser u\nport 22\n", "edit_err");
        let item = HostItem::new();
        assert_eq!(c.edit("nope", &item), Err("No host found!"));
        std::fs::remove_file(p).ok();
    }

    // ---- delete ----

    #[test]
    fn delete_ok_and_err() {
        let (mut c, p) = parse_str("Host a\nhostname h\nuser u\nport 22\n", "delete");
        assert!(c.delete("a").is_ok());
        assert!(c.hosts.is_empty());
        assert_eq!(c.delete("a"), Err("No host found!"));
        std::fs::remove_file(p).ok();
    }

    // ---- log ----

    #[test]
    fn log_handles_empty_and_populated_configs() {
        // empty -> unwrap_or(0) path, loop body skipped
        Config::new().log();

        // one host with all fields (identity shown, user & port present),
        // one host missing user/port/identity (the else/false branches)
        let mut c = Config::new();
        let mut a = HostItem::new();
        a.name = "a".to_string();
        a.host = "h".to_string();
        a.user = "u".to_string();
        a.port = "22".to_string();
        a.identity_file = "~/.ssh/id".to_string();
        let mut b = HostItem::new();
        b.name = "bb".to_string();
        b.host = "h2".to_string();
        c.hosts.push(a);
        c.hosts.push(b);
        c.log();
    }
}
