use std::{str::SplitWhitespace, thread, time::Duration};
use async_trait::async_trait;
use anyhow::Result;
use crate::Command;

pub struct TestCmd;
pub struct TestArgs;

pub fn add_commands() -> Vec<Box<dyn Command + Send + Sync>> {
    vec![
        Box::new(TestCmd),
        Box::new(TestArgs),
    ]
}

/// Debug CLI
#[async_trait]
impl crate::Command for TestCmd {
    async fn run(&self, mut args: SplitWhitespace<'_>) -> Result<()> {
        let total: u32 = args.next().get_or_insert("100").parse()?;
        let mut result = 1;

        for i in 0..total {
            result += i;
            result = result / 3;

            thread::sleep(Duration::from_millis(1))
        }

        println!("{}", result);
        Ok(())
    }

    fn help(&self) {
        todo!()
    }

    fn name(&self) -> String {
        "load".to_string()
    }
}

#[async_trait]
impl crate::Command for TestArgs {
    async fn run(&self, args: SplitWhitespace<'_>) -> Result<()> {
        println!("{:#?}", crate::parse_flags(args).await);
        Ok(())
    }

    fn help(&self) {
        todo!()
    }

    fn name(&self) -> String {
        "test_args".to_string()
    }
}