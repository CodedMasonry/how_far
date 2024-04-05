#![feature(fs_try_exists)]
pub mod database;
pub mod modules;
pub mod terminal;
pub mod agents;

use anyhow::Result;
use async_trait::async_trait;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use log::debug;
use rcgen::{date_time_ymd, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, ErrorKind},
    path::PathBuf,
    str::SplitWhitespace,
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::Mutex;

const HELP_SPACING: usize = 20;

/// Basic error handling for root module handling
#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Command Doesn't Exist")]
    NonExistant,
}

/// Command is the default template for command modules
/// Sub commands are EXPECTED to be handled by the run fn
/// help fn expects module to print it's own help message (default help message functions will be provided soon)
/// name fn is simply for indexing purposes (should return name of command)
#[async_trait]
pub trait Command: Send + Sync {
    async fn run(&self, args: SplitWhitespace<'_>) -> Result<()>;
    fn description(&self) -> String;
    fn name(&self) -> String;
}

/// A struct for storing help overview and fetching commmand lists
#[async_trait]
pub trait CommandSet: Send + Sync {
    fn add_commands() -> Vec<Box<dyn Command + Send + Sync>>;
    async fn help_overview() -> String;
}

lazy_static! {
    pub static ref DATA_FOLDER: ProjectDirs =
        directories::ProjectDirs::from("com", "codedmasonry", "how_far").unwrap();
    static ref CERTS: PathBuf = DATA_FOLDER.data_local_dir().to_path_buf().join("certs");
    static ref COMMANDS_SET: Arc<Mutex<Vec<Box<dyn Command + Send + Sync>>>> = {
        let mut temp_set: Vec<Box<dyn Command + Send + Sync>> = vec![];

        temp_set.append(&mut modules::debug::DebugSet::add_commands());

        Arc::new(Mutex::new(temp_set))
    };
}

pub fn generate_cert() -> anyhow::Result<()> {
    let mut params: CertificateParams = Default::default();
    params.not_before = date_time_ymd(1975, 1, 1);
    params.not_after = date_time_ymd(4096, 1, 1);
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::OrganizationName, "hardcoded analytics");
    params
        .distinguished_name
        .push(DnType::CommonName, "data server");
    params.subject_alt_names = vec![SanType::DnsName("localhost".try_into()?)];

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    let pem_serialized = cert.pem();
    fs::create_dir_all(CERTS.as_os_str())?;
    fs::write(CERTS.join("cert.pem"), pem_serialized.as_bytes())?;
    fs::write(CERTS.join("key.pem"), key_pair.serialize_pem().as_bytes())?;

    debug!("cert written to {}", CERTS.to_string_lossy());
    Ok(())
}

pub fn get_cert() -> anyhow::Result<(
    Vec<rustls_pki_types::CertificateDer<'static>>,
    rustls_pki_types::PrivateKeyDer<'static>,
)> {
    // Check if certs already generated
    fs::create_dir_all(CERTS.as_os_str())?;
    if !fs::try_exists(CERTS.join("cert.pem"))? {
        debug!("Certs don't exist; generating...");
        generate_cert()?;
    }

    let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(
        CERTS.join("cert.pem"),
    )?))
    .collect::<Result<Vec<_>, _>>()?;
    let private_key =
        rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(CERTS.join("key.pem"))?))?
            .unwrap();

    Ok((certs, private_key))
}

/// Intended for CLI; attempts to run cmd
pub async fn run_command(command: &str, args: SplitWhitespace<'_>) -> anyhow::Result<()> {
    let cmd_guard = COMMANDS_SET.lock();
    if let Some(cmd) = cmd_guard.await.iter().find(|&cmd| cmd.name() == command) {
        return cmd.run(args).await;
    }

    // Hits if no commands are it
    return Err(ModuleError::NonExistant.into());
}

/// Handles parsing flags in a SplitWhitespace item
/// default_args refers to args passed with no flags
/// I know it isn't clean but it works
async fn parse_flags(input: SplitWhitespace<'_>) -> (Vec<String>, HashMap<String, String>) {
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
        Err(e) if e.kind() == ErrorKind::NotFound => println!("Command doesn't exist"),
        Err(e) => eprintln!("{:?}", e),
    };
}

async fn format_help_section(title: &str, commands: Vec<Box<dyn Command + Send + Sync>>) -> String {
    let title = format!("{} {}", title, "Commands");
    let descriptor_headers = average_spacing("Command", "Description", HELP_SPACING).await;
    let descriptor_underlines = average_spacing("-------", "-----------", HELP_SPACING).await;

    let mut result = format!(
        "\n{}\n{}\n\n\t{}\n\t{}\n",
        title,
        "=".repeat(title.len()),
        descriptor_headers,
        descriptor_underlines
    );
    for cmd in commands {
        let spaced_line = average_spacing(&cmd.name(), &cmd.description(), HELP_SPACING).await;
        let line = format!("{}{}{}", "\t", spaced_line, "\n");

        result.push_str(&line);
    }

    result
}

async fn average_spacing(str1: &str, str2: &str, spacing: usize) -> String {
    let mut result = str1.to_string() + " ".repeat(spacing - str1.len()).as_str();

    result.push_str(str2);
    result
}
