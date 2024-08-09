use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use colored::*;

#[derive(Clone)]
pub struct HostItem {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: String,
}

impl HostItem {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.host.is_empty() && !self.user.is_empty() && !self.port.is_empty()
    }

    pub fn new() -> Self {
        Self {
            name: String::new(),
            host: String::new(),
            user: String::new(),
            port: String::new(),
        }
    }
}

pub struct Config {
    pub hosts: Vec<HostItem>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            hosts: Vec::new(),
        }
    }

    pub fn log(&self) {
        let max_name_len = self.hosts.iter().map(|host| host.name.len()).max().unwrap_or(0);

        println!();
        for host in &self.hosts {
            let spaces = " ".repeat(max_name_len - host.name.len());
            let leading_whitespaces = " ".repeat(5);

            println!(
                "{leading_whitespaces}{0}{spaces} -> {1}@{2}:{3}\n",
                host.name.bold().green(),
                host.user.bold(),
                host.host.bold().yellow(),
                host.port
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
        let mut file = File::create(path)?;
        file.write_all(b"")?; // Create the file with empty content

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(path)?;

        for host in &self.hosts {
            let new_config = format!(
                "Host {}\nhostname {}\nuser {}\nport {}\n",
                host.name,
                host.host,
                host.user,
                host.port
            );

            file.write_all(new_config.as_bytes())?;
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
    let mut current_host = HostItem::new();

    for line in lines {
        let val = line.expect("Failed to read line");
        let list: Vec<&str> = val.trim().split_whitespace().collect();

        if list.len() == 2 {
            match list[0] {
                "Host" => {
                    if current_host.is_valid() {
                        config.hosts.push(current_host);
                    }
                    current_host = HostItem::new();
                    current_host.name = list[1].to_string();
                }
                "hostname" => current_host.host = list[1].to_string(),
                "user" => current_host.user = list[1].to_string(),
                "port" => current_host.port = list[1].to_string(),
                _ => {}
            }
        }
    }

    if current_host.is_valid() {
        config.hosts.push(current_host);
    }

    config
}
