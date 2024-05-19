use std::{borrow::Cow, env, path::Path};

use reedline::{
    ExternalPrinter, Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus,
    Reedline, Signal,
};

use crate::{
    color_level,
    commands::{self, parse_cmd, parse_implant_cmd, try_external_command},
};

/// Purely so code is understandable
const SUCCESS: bool = true;

#[derive(Clone)]
pub struct CustomPrompt(&'static str);
pub static DEFAULT_MULTILINE_INDICATOR: &str = ">>> ";
impl Prompt for CustomPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        {
            match crate::SELECTED_AGENT.lock().unwrap().clone() {
                Some(v) => Cow::Owned(format!(
                    "{} ({}) ",
                    self.0.to_string(),
                    nu_ansi_term::Color::Default
                        .underline()
                        .paint(v.0.to_string())
                )),
                None => Cow::Owned(format!("{} ", self.0.to_string(),)),
            }
        }
    }

    fn render_prompt_right(&self) -> Cow<str> {
        {
            Cow::Borrowed("")
        }
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<str> {
        Cow::Owned("> ".to_string())
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(DEFAULT_MULTILINE_INDICATOR)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }

    fn get_prompt_color(&self) -> reedline::Color {
        reedline::Color::White
    }

    fn get_indicator_color(&self) -> reedline::Color {
        reedline::Color::White
    }
}

pub async fn tui(printer: ExternalPrinter<String>) -> Result<(), anyhow::Error> {
    let mut line_editor = Reedline::create().with_external_printer(printer);
    let prompt = CustomPrompt("how_far");

    loop {
        if let Ok(sig) = line_editor.read_line(&prompt) {
            match sig {
                Signal::Success(buffer) => {
                    if buffer == *"exit" && crate::SELECTED_AGENT.lock().unwrap().is_none() {
                        return Ok(());
                    }

                    if buffer.contains("cd") {
                        let mut split = buffer.split_whitespace();
                        split.next().unwrap();

                        let new_dir = split.peekable().peek().map_or("/", |x| *x);
                        let root = Path::new(new_dir);
                        if let Err(e) = env::set_current_dir(root) {
                            eprintln!("{}", e);
                        }

                        continue;
                    }

                    if crate::SELECTED_AGENT.lock().unwrap().is_some() {
                        let cmd = parse_implant_cmd(buffer.clone()).await;

                        match cmd {
                            Ok(v) => {
                                commands::handle_implant_cmd(&v).await;
                            }

                            Err(e) if e.kind() == clap::error::ErrorKind::InvalidSubcommand => {
                                commands::try_handle_implant_unknown(buffer).await;
                            }
                            Err(e) => println!("{}", e),
                        }
                    } else {
                        let cmd = parse_cmd(buffer.clone()).await;
                        match cmd {
                            Ok(v) => {
                                commands::handle_cmd(&v).await;
                            }
                            Err(e) if e.kind() == clap::error::ErrorKind::InvalidSubcommand => {
                                // Code to check if command exists and if so run it
                                let mut split = buffer.split_whitespace();
                                if try_external_command(split.next().unwrap_or_default(), split)
                                    .await
                                    != SUCCESS
                                {
                                    eprintln!("{} Unknown command: use the 'help' command to view possible commands", color_level(log::Level::Warn))
                                }
                            }
                            Err(e) => println!("{}", e),
                        }
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
