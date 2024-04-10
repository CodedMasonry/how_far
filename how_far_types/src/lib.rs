use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentJobType {
    Sleep,
    Run,
    Cleanup,
    Terminal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    _id: u32,
    _queue: Vec<AgentJob>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentJob {
    request_type: AgentJobType,
    args: Vec<String>,
}
