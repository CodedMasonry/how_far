use crate::{Command, CommandSet};
use anyhow::Result;
use async_trait::async_trait;
use std::str::SplitWhitespace;

/// Debug Command Set
pub struct DebugSet;
/// Example math method
pub struct TestCmd;
/// Args & Flags Example
pub struct TestArgs;
/// Prints help messages
pub struct Help;

#[async_trait]
impl crate::CommandSet for DebugSet {
    fn add_commands() -> Vec<Box<dyn Command + Send + Sync>> {
        vec![Box::new(TestCmd), Box::new(TestArgs), Box::new(Help)]
    }

    async fn help_overview() -> String {
        crate::format_help_section("Debug", Self::add_commands()).await
    }
}

#[async_trait]
impl crate::Command for TestCmd {
    async fn run(&self, mut args: SplitWhitespace<'_>) -> Result<()> {
        let total: u32 = args.next().get_or_insert("100").parse()?;

        println!("{}", total);
        Ok(())
    }

    fn description(&self) -> String {
        "Example Command".to_string()
    }

    fn name(&self) -> String {
        "load".to_string()
    }
}

#[async_trait]
impl crate::Command for TestArgs {
    async fn run(&self, args: SplitWhitespace<'_>) -> Result<()> {
        let (args, flags) = crate::parse_flags(args).await;
        println!("args: {:#?}\nFlags: {:#?}", args, flags);
        Ok(())
    }

    fn description(&self) -> String {
        "Test Arg Parsing".to_string()
    }

    fn name(&self) -> String {
        "test_args".to_string()
    }
}

#[async_trait]
impl crate::Command for Help {
    async fn run(&self, _args: SplitWhitespace<'_>) -> Result<()> {
        println!("{}", DebugSet::help_overview().await);
        Ok(())
    }

    fn description(&self) -> String {
        "Prints the help message".to_string()
    }

    fn name(&self) -> String {
        "help".to_string()
    }
}
