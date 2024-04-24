use how_far_types::AgentInfo;
use log::{error, info, warn};
use rustyline::{error::ReadlineError, DefaultEditor};
use tokio::sync::Mutex;
use std::{path::Path, sync::LazyLock};
use std::collections::HashMap;
use std::str::SplitWhitespace;

use crate::{ModuleError, COMMANDS_SET};

static _CURRENT_AGENT: LazyLock<Mutex<Option<AgentInfo>>> = LazyLock::new(|| Mutex::new(None));

pub async fn tui() -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline("how_far > ");
        match readline {
            Ok(input) => {
                rl.add_history_entry(input.as_str())?;
                let mut parts = input.trim().split_whitespace();
                let command = parts.next().unwrap();
                let args = parts;

                match command {
                    "cd" => {
                        let new_dir = args.peekable().peek().map_or("/", |x| *x);
                        let root = Path::new(new_dir);
                        if let Err(e) = std::env::set_current_dir(&root) {
                            error!("{}", e);
                        }
                    }

                    "exit" => return Ok(()),

                    // Attempts to run internal command; if internal command doesn't exist
                    // then attempts to run external command
                    command => match run_command(command, args.clone()).await {
                        Ok(_) => continue,
                        Err(e) => {
                            if e.is::<crate::ModuleError>() {
                                run_external_command(command, args).await;
                            } else {
                                error!("Error running command: {:#?}", e);
                            }
                        }
                    },
                }
            }
            Err(ReadlineError::Interrupted) => {
                warn!("Please type 'exit' to leave");
            }
            Err(ReadlineError::Eof) => {
                warn!("Please type 'exit' to leave");
            }
            Err(err) => {
                error!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

/// Intended for CLI; attempts to run cmd
pub async fn run_command(command: &str, args: SplitWhitespace<'_>) -> anyhow::Result<()> {
    if let Some(cmd) = COMMANDS_SET.iter().find(|&cmd| cmd.name() == command) {
        return cmd.run(args).await;
    }

    // Hits if no commands are it
    return Err(ModuleError::NonExistant.into());
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
            if word.starts_with("\"") {
                long_string.push(word.trim_start_matches('\"'));
                is_long_string = true
            } else if word.ends_with("\"") {
                long_string.push(word.trim_end_matches('\"'));

                flags_with_args.insert(current_flag.clone(), long_string.join(" "));
                long_string.clear();
                current_flag.clear();

                is_long_string = false;
            } else if is_long_string == true {
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

async fn run_external_command(command: &str, args: SplitWhitespace<'_>) {
    let child = tokio::process::Command::new(command).args(args).spawn();

    match child {
        Ok(mut child) => {
            child.wait().await.unwrap();
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => info!("Command doesn't exist"),
        Err(e) => error!("{:?}", e),
    };
}