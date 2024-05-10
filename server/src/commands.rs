mod data;

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::str::SplitWhitespace;

use crate::color_level;

/// Interactive server for managing implants
#[derive(Parser, Debug)]
#[command(version, about = None, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Commands for manipulating the database; Alias: db
    #[command(alias = "db")]
    Database {
        #[command(subcommand)]
        command: Option<DatabaseCommands>,
    },
}

#[derive(Subcommand, Debug)]
enum DatabaseCommands {
    /// Lists the entries in the database
    List,
    /// View specified entry
    View { id: u32 },

    /// Removes the specified entry
    #[clap(alias = "rm")]
    Remove { id: u32, yes: bool },
}

pub async fn parse_cmd(str: String) -> Result<Cli, clap::Error> {
    let str = format!("{} {}", env!("CARGO_PKG_NAME"), str);
    Cli::try_parse_from(str.split_whitespace())
}

pub async fn handle_cmd(cli: &Cli) {
    match &cli.command {
        Commands::Database { command } => data::handle_database_cmds(command).await,
    }
}

/// Returns boolean denoting whether it successfully ran the command
pub async fn try_external_command(command: &str, args: SplitWhitespace<'_>) -> bool {
    let child = tokio::process::Command::new(command).args(args).spawn();

    match child {
        Ok(mut child) => {
            println!(
                "{} running local: {}\n",
                color_level(log::Level::Info),
                command
            );

            child.wait().await.unwrap();
            return true;
        }
        Err(e) if e.kind() == tokio::io::ErrorKind::NotFound => return false,
        Err(_) => return false,
    }
}

/// Handles parsing flags in a SplitWhitespace item
/// default_args refers to args passed with no flags
/// I know it isn't clean but it works
async fn _parse_flags(input: SplitWhitespace<'_>) -> (Vec<String>, HashMap<String, String>) {
    let mut flags_with_args = HashMap::new();
    let mut current_flag = String::new();
    let mut is_long_string = false;
    let mut long_string = Vec::new(); // In case someone has a long input ("my home/repos")
    let mut args = Vec::new();

    for word in input {
        if word.starts_with('-') {
            if !current_flag.is_empty() {
                flags_with_args.insert(current_flag.clone(), String::new());
            }
            current_flag = word.trim_start_matches('-').to_owned();
        } else if !current_flag.is_empty() {
            if word.starts_with('"') {
                long_string.push(word.trim_start_matches('\"'));
                is_long_string = true
            } else if word.ends_with('"') {
                long_string.push(word.trim_end_matches('\"'));

                flags_with_args.insert(current_flag.clone(), long_string.join(" "));
                long_string.clear();
                current_flag.clear();

                is_long_string = false;
            } else if is_long_string {
                long_string.push(word);
            } else {
                flags_with_args.insert(current_flag.clone(), word.to_owned());
                current_flag.clear();
            }
        } else {
            // Default argument handling
            // Ex: test_args SOME_ARGUMENT
            args.push(word.to_string());
        }
    }

    if !current_flag.is_empty() {
        flags_with_args.insert(current_flag.clone(), String::new());
    }

    (args, flags_with_args)
}
