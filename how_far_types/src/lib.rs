use serde::{Deserialize, Serialize};
use chrono::Utc;

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