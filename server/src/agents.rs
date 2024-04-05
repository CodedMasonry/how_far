use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AgentInfo {
    _id: u32,
    _queue: Vec<String>
}