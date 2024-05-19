#![feature(lazy_cell)]
use std::{fmt, path::PathBuf, str::FromStr, sync::LazyLock};

use chrono::Utc;
use directories::ProjectDirs;
use redb::TableDefinition;
use serde::{Deserialize, Serialize};
use strum::EnumString;
use thiserror::Error;

pub static DATA_FOLDER: LazyLock<ProjectDirs> =
    LazyLock::new(|| directories::ProjectDirs::from("com", "codedmasonry", "how_far").unwrap());

/// Key: u32 and Value: Byte array (postcard serialized) of ImplantInfo
pub const DB_TABLE: TableDefinition<u32, &[u8]> = TableDefinition::new("implants");
pub static DB_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    crate::DATA_FOLDER
        .data_local_dir()
        .to_path_buf()
        .join("db.redb")
});

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Job Command Doesn't exist")]
    InvalidJobCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplantJobType {
    Sleep,
    Run { cmd: String },
    Cleanup,
    Interactive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImplantInfo {
    // Expect: OS/ARCH
    pub platform: String,

    pub last_check: Option<chrono::DateTime<Utc>>,
    pub queue: Vec<ImplantJob>,
}

/// Helpful wrapper for providing details only needed by the server
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImplantJob {
    pub issue_time: chrono::DateTime<Utc>,
    pub job: ImplantJobInner,
}

/// Inner value that is serialized and should be sent over the internet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImplantJobInner {
    pub request_type: ImplantJobType,
    pub args: Vec<String>,
}

/// Type for sending array of ImplantJobInner between server and client
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetJobList {
    pub jobs: Vec<ImplantJobInner>,
}

/// List of possible commands that can be ran on an agent / implant
#[derive(Debug, strum::Display, Clone, Serialize, Deserialize, EnumString, strum::EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum JobCommand {
    Ls,
    Pwd,
    Whoami,
    Cd,
    Mkdir,
    Touch,
    // Alias for ls
    Dir,
    Rm,
    Mv,
    Cp,
    Sysinfo,

    #[strum(disabled)]
    WindowsCommand(WindowsJobCommand),
}

#[derive(Debug, strum::Display, Clone, Serialize, Deserialize, EnumString, strum::EnumIter)]
pub enum WindowsJobCommand {
    Migrate { id: u32 },
}

impl ImplantInfo {
    pub fn to_bytes(&mut self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }
}

impl fmt::Display for ImplantJobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            ImplantJobType::Sleep => "sleep",
            ImplantJobType::Run { cmd } => &format!("run {}", cmd),
            ImplantJobType::Cleanup => "cleanup",
            ImplantJobType::Interactive => "interactive",
        };
        write!(f, "{str}")
    }
}

impl fmt::Display for ImplantJob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]: {}", self.issue_time, self.job)
    }
}

impl fmt::Display for ImplantJobInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.request_type.to_string().to_lowercase(),
            self.args.join(" ")
        )
    }
}

impl JobCommand {
    pub fn parse_cmd(str: String) -> Result<JobCommand, TypeError> {
        match JobCommand::from_str(&str) {
            Ok(v) => Ok(v),
            Err(_) => {
                if let Ok(v) = Self::parse_windows_cmd(str) {
                    return Ok(JobCommand::WindowsCommand(v));
                }

                Err(TypeError::InvalidJobCommand)
            }
        }
    }

    fn parse_windows_cmd(str: String) -> Result<WindowsJobCommand, TypeError> {
        match WindowsJobCommand::from_str(&str) {
            Ok(v) => Ok(v),
            Err(_) => Err(TypeError::InvalidJobCommand),
        }
    }
}

pub fn match_cmd_description(cmd: JobCommand) -> String {
    let str = match cmd {
        JobCommand::Ls => "lists files in current directory",
        JobCommand::Pwd => "prints current directory",
        JobCommand::Whoami => "prints the active controlled account",
        JobCommand::Cd => "changes directory",
        JobCommand::Mkdir => "creates new directory",
        JobCommand::Touch => "creates new folder",
        JobCommand::Dir => "alias for ls",
        JobCommand::Rm => "removes file / directory",
        JobCommand::Mv => "moves file / directory",
        JobCommand::Cp => "copies file / directory",
        JobCommand::Sysinfo => "lists general information about system",

        JobCommand::WindowsCommand(cmd) => match cmd {
            WindowsJobCommand::Migrate { id: _ } => "attempts to migrate to specified application",
        },
    };

    return str.to_string()
}