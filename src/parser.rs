use std::{fs::{File, OpenOptions}, io::{BufReader, self, BufRead, Write}};
use colored::*;

#[derive(Clone)]
pub struct HostItem {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: String,
}

impl HostItem {
    fn reset(&mut self) {
        self.name = String::from("");
        self.host = String::from("");
        self.user = String::from("");
        self.port = String::from("");
    }

    fn is_valid(&self) -> bool {
        self.port != "" && self.name != "" && self.host != "" && self.user != ""
    }

    pub fn new() -> Self {
        Self {
            name: String::from(""),
            host: String::from(""),
            user: String::from(""),
            port: String::from(""),
        }
    }
}

pub struct Config {
    pub hosts: Vec<HostItem>
}

impl Config {
    pub fn new() -> Self {
        Config{
            hosts: Vec::new()
        }
    }

    pub fn log(self) {
        let mut max_name_len = 0;

        for host in &self.hosts {
            if host.name.len() > max_name_len {
                max_name_len = host.name.len();
            }
        }

        println!("");
        for host in &self.hosts {
            let spaces = " ".repeat(max_name_len - host.name.len());
            let leading_whitespaces = " ".repeat(5);

            println!(
                "{leading_whitespaces}{0}{spaces} -> {1}@{2}:{3}\n",
                host.name.bold().green(),
                host.user.bold(),
                host.host.bold().yellow(),
                host.port
            )
        }
    }

    pub fn get_host_copy(self, name: String) -> Option<HostItem> {
        for host in self.hosts {
            if host.name == name {
                return Some(host.clone());
            }
        }
        None
    }

    pub fn rename(&mut self, name: String, new_name: String) -> Result<(), &'static str> {
        if let Some(index) = self.hosts.iter().position(|x| *x.name == name) {
            self.hosts[index].name = new_name;
            return Ok(())
        } else {
            return Err("No host found!")
        }
    }

    pub fn edit(&mut self, name: String, host_item: HostItem) -> Result<(), &'static str> {
        if let Some(index) = self.hosts.iter().position(|x| *x.name == name) {
            self.hosts[index].host = host_item.host;
            self.hosts[index].user = host_item.user;
            self.hosts[index].port = host_item.port;
            return Ok(())
        } else {
            return Err("No host found!")
        }
    }

    pub fn delete(&mut self, name: String) -> Result<(), &'static str> {
        if let Some(index) = self.hosts.iter().position(|x| *x.name == name) {
            self.hosts.remove(index);
            return Ok(())
        } else {
            return Err("No host found!")
        }
    }

    pub fn write(self, path: String) -> std::io::Result<()> {
        let mut file = File::create(path.clone())?;
        write!(file, "")?;

        let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();

        for host in self.hosts {
            let new_config = format!("Host {0}
                hostname {1}
                user {2}
                port {3}\n", host.name, host.host, host.user, host.port);

            file.write(new_config.as_bytes()).expect("Write failed");
        }
        Ok(())
    }
}

fn read_lines(filename: String) -> io::Lines<BufReader<File>> {
    let file = File::open(filename).unwrap();

    return io::BufReader::new(file).lines()
}

pub fn parse(filename: String) -> Config {
    let lines = read_lines(filename);

    let mut config = Config::new();

    let mut current_host = HostItem::new();
    for line in lines {
        let val = line.unwrap();
        let list: Vec<String> = val
            .trim()
            .split(" ")
            .map(|s| s.to_string())
            .collect();

        if list.len() == 2 && list[0] == "Host" {
            current_host.name = String::from(&list[1]);
        }
        if list.len() == 2 && list[0] == "hostname" {
            current_host.host = String::from(&list[1]);
        }
        if list.len() == 2 && list[0] == "user" {
            current_host.user = String::from(&list[1]);
        }
        if list.len() == 2 && list[0] == "port" {
            current_host.port = String::from(&list[1]);
        }
        if current_host.is_valid() {
            config.hosts.push(current_host.clone());
            current_host.reset();
        }
    }

    config
}
