mod add;
mod cli;
mod config;
mod discovery;
mod fzf;
mod git;
mod init;
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

    match cli.command.unwrap_or(Command::Interactive { all: false }) {
        Command::Init { shell } => match shell {
            Some(s) => {
                // Explicit shell - output code to stdout (for manual setup)
                print!("{}", crate::init::shell_init(s));
                Ok(())
            }
            None => {
                // No shell specified - run interactive setup
                crate::init::run_interactive_setup()
            }
        },
        Command::Interactive { all } => crate::interactive::run_interactive(all),
        Command::List { json, all } => crate::list::list_worktrees(json, all),
        Command::Add {
            branch,
            path,
            track,
            json,
            quiet,
        } => match branch {
            Some(b) => crate::add::add_worktree(&b, path.as_deref(), track.as_deref(), json, quiet),
            None => crate::add::interactive_add(path.as_deref(), track.as_deref(), json, quiet),
        },
        Command::Remove {
            target,
            force,
            json,
            quiet,
        } => match target {
            Some(t) => crate::remove::remove_worktree(&t, force, json, quiet),
            None => crate::remove::interactive_remove(force, json, quiet),
        },
        Command::Prune { json, quiet } => crate::prune::prune_worktrees(json, quiet),
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
                ConfigCommand::Show { json } => {
                    let config = crate::config::load()?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&config)?);
                    } else {
                        let path = crate::config::config_path();
                        println!("# Config file: {}", path.display());
                        let yaml = serde_yaml::to_string(&config)?;
                        println!("{}", yaml);
                    }
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
