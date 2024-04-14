use log::{error, warn};
use rustyline::{error::ReadlineError, DefaultEditor};
use std::path::Path;

use crate::{run_command, run_external_command};

pub async fn tui() -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline("how_far > ");
        match readline {
            Ok(input) => {
                rl.add_history_entry(input.as_str())?;
                let mut parts = input.trim().split_whitespace();
                let command = parts.next().unwrap();
                let args = parts;

                match command {
                    "cd" => {
                        let new_dir = args.peekable().peek().map_or("/", |x| *x);
                        let root = Path::new(new_dir);
                        if let Err(e) = std::env::set_current_dir(&root) {
                            error!("{}", e);
                        }
                    }

                    "exit" => return Ok(()),

                    // Attempts to run internal command; if internal command doesn't exist
                    // then attempts to run external command
                    command => match run_command(command, args.clone()).await {
                        Ok(_) => continue,
                        Err(e) => {
                            if e.is::<crate::ModuleError>() {
                                run_external_command(command, args).await;
                            } else {
                                error!("Error running command: {:#?}", e);
                            }
                        }
                    },
                }
            }
            Err(ReadlineError::Interrupted) => {
                warn!("Please type 'exit' to leave");
            }
            Err(ReadlineError::Eof) => {
                warn!("Please type 'exit' to leave");
            }
            Err(err) => {
                error!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}
