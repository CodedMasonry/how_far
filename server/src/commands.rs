use std::str::SplitWhitespace;
use std::collections::HashMap;
use clap::Parser;

/// Commands for interacting with the database
#[derive(Parser, Debug)]
pub struct Cli {
    /// Testing
    #[arg(short, long)]
    some_commands: String,

    /// Test Flag
    #[arg(short, long)]
    flag_1: Option<String>
}

pub async fn parse_cmd(str: String) -> Result<Cli, clap::Error> {
    let str = format!("{} {}", env!("CARGO_PKG_NAME"), str);
    Cli::try_parse_from(str.split_whitespace())
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