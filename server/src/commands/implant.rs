use log::{error, info, warn};

use crate::database;

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