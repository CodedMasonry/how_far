#![feature(lazy_cell)]
use std::{path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;
use redb::TableDefinition;
use serde::{Deserialize, Serialize};
use chrono::Utc;

pub static DATA_FOLDER: LazyLock<ProjectDirs> =
    LazyLock::new(|| directories::ProjectDirs::from("com", "codedmasonry", "how_far").unwrap());

/// Key: u32 and Value: Byte array (postcard serialized) of AgentInfo
pub const DB_TABLE: TableDefinition<u32, &[u8]> = TableDefinition::new("agents");
pub static DB_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    crate::DATA_FOLDER
        .data_local_dir()
        .to_path_buf()
        .join("db.redb")
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentJobType {
    Sleep,
    Run,
    Cleanup,
    Terminal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub last_check: Option<chrono::DateTime<Utc>>,
    pub queue: Vec<AgentJob>,
}

/// Helpful wrapper for providing details only needed by the server
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentJob {
    pub issue_time: chrono::DateTime<Utc>,
    pub job: AgentJobInner,
}

/// Inner value that is serialized and should be sent over the internet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentJobInner {
    pub request_type: AgentJobType,
    pub args: Vec<String>,
}

/// Type for sending array of AgentJobInner between server and client
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetJobList {
    pub jobs: Vec<AgentJobInner>,
}