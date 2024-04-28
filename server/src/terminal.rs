use reedline::{DefaultPrompt, ExternalPrinter, Reedline, Signal};

use crate::commands::CommandRan;

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
                    let cmd = CommandRan::parse(buffer).await;

                    println!("We processed: {:#?}", cmd);
                }
                Signal::CtrlD | Signal::CtrlC => {
                    println!("\nAborted!");
                    return Ok(());
                }
            }
            continue;
        }
        return Ok(());
    }
}