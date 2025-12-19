mod add;
mod cli;
mod config;
mod fzf;
mod git;
mod interactive;
mod list;
mod preview;
mod process;
mod prune;
mod remove;
mod worktree;

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
        Command::Interactive => crate::interactive::run_interactive(),
        Command::List { json, all } => {
            if all {
                anyhow::bail!("--all list not implemented yet")
            }
            crate::list::list_worktrees(json)?;
            Ok(())
        }
        Command::Add {
            branch,
            path,
            track,
        } => {
            crate::add::add_worktree(&branch, path.as_deref(), track.as_deref())?;
            Ok(())
        }
        Command::Remove { target, force } => {
            crate::remove::remove_worktree(&target, force)?;
            Ok(())
        }
        Command::Prune => crate::prune::prune_worktrees(),
        Command::Preview { path } => crate::preview::print_preview(std::path::Path::new(&path)),
        Command::Config { command } => {
            use crate::cli::ConfigCommand;
            match command {
                ConfigCommand::Init => {
                    let path = crate::config::config_path();
                    if path.exists() {
                        println!("Config file already exists at {}", path.display());
                    } else {
                        let config = crate::config::Config::default();
                        crate::config::save(&config)?;
                        eprintln!("Created config file at {}", path.display());
                    }
                    Ok(())
                }
                ConfigCommand::Show => {
                    let config = crate::config::load()?;
                    let path = crate::config::config_path();
                    println!("# Config file: {}", path.display());
                    let yaml = serde_yaml::to_string(&config)?;
                    println!("{}", yaml);
                    Ok(())
                }
                ConfigCommand::SetEditor { editor } => {
                    let mut config = crate::config::load()?;
                    config.editor = editor.clone();
                    crate::config::save(&config)?;
                    eprintln!("Editor set to: {}", editor);
                    Ok(())
                }
                ConfigCommand::SetDiscoveryPaths { paths } => {
                    let mut config = crate::config::load()?;
                    config.auto_discovery.paths = paths.clone();
                    crate::config::save(&config)?;
                    eprintln!("Auto-discovery paths set to:");
                    for path in &paths {
                        eprintln!("  {}", path);
                    }
                    Ok(())
                }
            }
        }
    }
}
