use clap::{Parser, Subcommand};
use zoi::command;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    pub commands: Option<Commands>,
}


#[derive(Subcommand)]
enum Commands {
    /// List all hosts
    List {},
    /// Add a host
    Add {
        /// alias name of the host
        name: String,
        /// format: user@hostname:port
        value: String,
    },
    /// Delete a host
    Delete {
        name: String,
    },
    /// Delete all hosts
    DeleteAll {},
    /// Edit a host
    Edit {
        name: String,
        /// format: user@hostname:port
        value: String,
    },
    /// rename a host
    Rename {
        name: String,
        new_name: String,
    }
}

fn main() {
    let args = Cli::parse();

    match &args.commands {
        Some(Commands::List {}) => {
            command::list_command()
        },
        Some(Commands::Add { name, value }) => {
            command::add_command(name, value)
        },
        Some(Commands::DeleteAll {  }) => {
            command::delete_all_command()
        },
        Some(Commands::Edit { name, value }) => {
            command::edit_command(name, value)
        },
        Some(Commands::Rename { name, new_name }) => {
            command::rename_command(name, new_name)
        },
        Some(Commands::Delete { name }) => {
            command::delete_command(name)
        },
        None => {}
    }
}
