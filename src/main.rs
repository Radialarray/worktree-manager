mod cli;
mod process;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Command};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Interactive) {
        Command::Interactive => {
            anyhow::bail!("interactive mode not implemented yet")
        }
        Command::List { .. } => anyhow::bail!("list not implemented yet"),
        Command::Add { .. } => anyhow::bail!("add not implemented yet"),
        Command::Remove { .. } => anyhow::bail!("remove not implemented yet"),
        Command::Prune => anyhow::bail!("prune not implemented yet"),
        Command::Preview { .. } => anyhow::bail!("preview not implemented yet"),
        Command::Config { .. } => anyhow::bail!("config not implemented yet"),
    }
}
