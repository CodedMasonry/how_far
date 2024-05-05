use log::{error, info};
use tabled::{builder::Builder, settings::Style};

use crate::database;

use super::DatabaseCommands;

pub async fn handle_database_cmds(command: &DatabaseCommands) {
    match command {
        DatabaseCommands::List => list().await,
        DatabaseCommands::View { id } => view(*id).await,
    }
}

async fn list() {
    let data = match database::IMPLANT_DB.list_implants().await {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let mut builder = Builder::default();
    builder.push_record(["agent", "last_connection", "jobs"]);

    for (id, info) in data {
        let last_check = match info.last_check {
            Some(v) => v.format("%d/%m/%Y %H:%M").to_string(),
            None => String::from("Never"),
        };

        let mut queue = info
            .queue
            .into_iter()
            .map(|v| v.job.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        if queue.is_empty() {
            queue.push_str("None")
        }

        builder.push_record([id.to_string(), last_check, queue]);
    }

    let table = builder.build().with(Style::psql()).to_string();
    println!("{}", table);
}

async fn view(id: u32) {
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
    let last_check = match info.last_check {
        Some(v) => v.format("%d/%m/%Y %H:%M").to_string(),
        None => String::from("Never"),
    };

    let mut queue = info.queue.into_iter().map(|v| v.to_string()).collect::<Vec<String>>().join("\n");
    if queue.is_empty() {
        queue.push_str("None")
    }

    println!(
        "AGENT {}\n{}\nlast connection: {}\njobs: {}",
        id, "-".repeat(15), last_check, queue
    );
}
