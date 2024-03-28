use std::{io, process};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// any HARMFUL functions won't be executed
    #[arg(short, long)]
    safe: bool,

    /// Skips confirmation (Not Recommended)
    #[arg(short, long)]
    yes: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!(include_str!("../disclaimer.md"));
    println!("{}", "_".repeat(10));
    // confirms consent
    if !args.yes {
        println!("\nAre you sure [y/N]? ");

        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut buffer)?;

        if buffer.trim().to_lowercase() == "n" {
            println!("quiting...");
            process::exit(0);
        }
    }

    Ok(())
}
