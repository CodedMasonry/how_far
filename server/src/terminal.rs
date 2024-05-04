use std::{env, path::Path};

use reedline::{DefaultPrompt, ExternalPrinter, Reedline, Signal};

use crate::{
    color_level,
    commands::{self, parse_cmd, try_external_command},
};

/// Purely so code is understandable
const SUCCESS: bool = true;

pub async fn test_tui() -> Result<(), anyhow::Error> {
    Ok(())
}

pub async fn tui(printer: ExternalPrinter<String>) -> Result<(), anyhow::Error> {
    let mut line_editor = Reedline::create().with_external_printer(printer);
    let prompt = DefaultPrompt::default();

    loop {
        if let Ok(sig) = line_editor.read_line(&prompt) {
            match sig {
                Signal::Success(buffer) => {
                    if buffer == String::from("exit") {
                        return Ok(());
                    }

                    if buffer.contains("cd") {
                        let mut split = buffer.split_whitespace();
                        split.next().unwrap();

                        let new_dir = split.peekable().peek().map_or("/", |x| *x);
                        let root = Path::new(new_dir);
                        if let Err(e) = env::set_current_dir(&root) {
                            eprintln!("{}", e);
                        }

                        continue;
                    }

                    let cmd = parse_cmd(buffer.clone()).await;
                    match cmd {
                        Ok(v) => {
                            commands::handle_cmd(&v).await;
                        }
                        Err(e) if e.kind() == clap::error::ErrorKind::InvalidSubcommand => {
                            // Code to check if command exists and if so run it
                            let mut split = buffer.split_whitespace();
                            if try_external_command(split.next().unwrap_or_default(), split).await
                                != SUCCESS
                            {
                                eprintln!("{} Unknown command: use the 'help' command to view possible commands", color_level(log::Level::Warn))
                            }
                        }
                        Err(e) => println!("{}", e),
                    }
                }
                Signal::CtrlD | Signal::CtrlC => {
                    // quicker exit for debug
                    if cfg!(debug_assertions) {
                    return Ok(());
                    }

                    println!(
                        "{} use the 'exit' command to quit",
                        color_level(log::Level::Error)
                    );
                }
            }
            continue;
        }
        return Ok(());
    }
}
