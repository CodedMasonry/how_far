use nu_ansi_term::{Color, Style};
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;
use std::{collections::HashMap, thread};
use std::str::SplitWhitespace;

use reedline::{DefaultHinter, DefaultPrompt, ExternalPrinter, Reedline, Signal};

pub async fn test_tui() -> Result<(), anyhow::Error> {
    Ok(())
}

pub async fn tui() -> Result<(), anyhow::Error> {
    let printer = ExternalPrinter::default();
    // make a clone to use it in a different thread
    let p_clone = printer.clone();
    // get the Sender<String> to have full sending control
    let p_sender = printer.sender();

    // external printer that prints a message every second
    thread::spawn(move || {
        let mut i = 1;
        loop {
            sleep(Duration::from_secs(1));
            assert!(p_clone
                .print(format!("Message {i} delivered.\nWith two lines!"))
                .is_ok());
            i += 1;
            stdout().flush().unwrap_or_default();
        }
    });

    // external printer that prints a bunch of messages after 3 seconds
    thread::spawn(move || {
        sleep(Duration::from_secs(3));
        for _ in 0..10 {
            sleep(Duration::from_millis(1));
            assert!(p_sender.send("Fast Hello !".to_string()).is_ok());
        }
    });

    let mut line_editor = Reedline::create().with_external_printer(printer);
    let prompt = DefaultPrompt::default();

    loop {
        if let Ok(sig) = line_editor.read_line(&prompt) {
            match sig {
                Signal::Success(buffer) => {
                    println!("We processed: {buffer}");
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

/// Handles parsing flags in a SplitWhitespace item
/// default_args refers to args passed with no flags
/// I know it isn't clean but it works
async fn _parse_flags(input: SplitWhitespace<'_>) -> (Vec<String>, HashMap<String, String>) {
    let mut flags_with_args = HashMap::new();
    let mut current_flag = String::new();
    let mut is_long_string = false;
    let mut long_string = Vec::new(); // In case someone has a long input ("my home/repos")
    let mut args = Vec::new();

    for word in input {
        if word.starts_with('-') {
            if !current_flag.is_empty() {
                flags_with_args.insert(current_flag.clone(), String::new());
            }
            current_flag = word.trim_start_matches('-').to_owned();
        } else if !current_flag.is_empty() {
            if word.starts_with("\"") {
                long_string.push(word.trim_start_matches('\"'));
                is_long_string = true
            } else if word.ends_with("\"") {
                long_string.push(word.trim_end_matches('\"'));

                flags_with_args.insert(current_flag.clone(), long_string.join(" "));
                long_string.clear();
                current_flag.clear();

                is_long_string = false;
            } else if is_long_string == true {
                long_string.push(word);
            } else {
                flags_with_args.insert(current_flag.clone(), word.to_owned());
                current_flag.clear();
            }
        } else {
            // Default argument handling
            // Ex: test_args SOME_ARGUMENT
            args.push(word.to_string());
        }
    }

    if !current_flag.is_empty() {
        flags_with_args.insert(current_flag.clone(), String::new());
    }

    (args, flags_with_args)
}
