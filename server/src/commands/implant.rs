use clap::Subcommand;
use log::{error, info, warn};

use crate::database;

#[derive(Subcommand)]
pub enum ImplantCommands {
    /// Shows list of available commands
    #[clap(alias = "cmds")]
    Commands,
    /// Lists jobs numbered by when added
    Jobs,
    /// Runs a command in a terminal (run cmds for list of commands)
    Run { cmd: String, args: Vec<String> },
    /// Cancels a job
    Cancel { job: u32 },
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
                return;
            }
        },
        Err(e) => {
            error!("failed to fetch implant: {}", e);
            return;
        }
    };

    crate::SELECTED_AGENT
        .lock()
        .unwrap()
        .replace((id, agent_info));
    info!("Agent set to {}", id);
}

pub async fn list_jobs() {
    let id = match crate::SELECTED_AGENT.lock().unwrap().clone() {
        Some(v) => v.0,
        None => return,
    };

    let agent = match database::IMPLANT_DB.fetch_implant(id).await {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let info = match agent {
        Some(v) => v,
        None => {
            info!("Agnet not found");
            return;
        }
    };

    let mut queue = String::new();
    for (id, job) in info.queue.into_iter().enumerate() {
        let str = format!(
            "{} {} - {}\n",
            nu_ansi_term::Color::DarkGray.paint(format!("[{}]", id)),
            job.issue_time.format("%d/%m/%Y %H:%M UTC"),
            job.job
        );
        
        queue.push_str(&str);
    }

    if queue.is_empty() {
        queue.push_str("None")
    }

    println!("AGENT {}\n{}\n{}", id, "-".repeat(20), queue);
}
