use std::{fs, path::Path};

use crate::parser;

pub fn get_path() -> String {
    let mut path = String::from("");
    match home::home_dir() {
        Some(p) => path = p.to_str().unwrap().to_string(),
        None => println!("Impossible to get your home dir!"),
    }
    path.push_str("/.ssh/config");
    if !Path::new(&path).exists() {
        reset_config_file(path.to_owned());
    }
    path
}

fn reset_config_file(path: String) {
    match fs::write(path, "") {
        Ok(_) => println!("Recreated config file!"),
        Err(_) => ()
    }
}

pub fn delete_all_command() {
    let path = get_path();
    reset_config_file(path);
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

pub fn add_command(name: &String, host: &String) {
    let path = get_path();

    let mut new_host = parser::HostItem::new();
    new_host.name = name.clone();
    extract_host_name(&mut new_host, host.to_owned());
    let mut config = parser::parse(path.clone());
    config.hosts.push(new_host);
    config.write(path).unwrap();
}

pub fn edit_command(name: &String, host: &String) {
    let path = get_path();
    let mut config = parser::parse(path.to_owned());
    let mut host_item = parser::HostItem::new();

    extract_host_name(&mut host_item, host.to_owned());

    match config.edit(name.to_owned(), host_item) {
        Ok(_) => config.write(path).unwrap(),
        Err(err) => eprintln!("Can not edit: {}", err)
    }
}

pub fn rename_command(name: &String, host: &String) {
    let path = get_path();
    let mut config = parser::parse(path.clone());
    match config.rename(name.to_owned(), host.to_owned()) {
        Ok(_) => config.write(path).unwrap(),
        Err(err) => eprintln!("Can not rename: {}", err)
    }
}

pub fn delete_command(name: &String) {
    let path = get_path();
    let mut config = parser::parse(path.clone());
    match config.delete(name.to_owned()) {
        Ok(_) => config.write(path).unwrap(),
        Err(err) => eprintln!("Can not delete: {}", err)
    }
}

pub fn list_command() {
    let path = get_path();
    let config = parser::parse(path);

    if config.hosts.len() == 0 {
        println!("No records");
    } else {
        config.log()
    }
}
