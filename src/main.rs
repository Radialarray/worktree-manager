mod add;
mod agent;
mod cli;
mod config;
mod discovery;
mod error;
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
use crate::error::WtError;

fn main() {
    let cli = Cli::parse();

    // Check if --json flag is present in any command for error handling
    let has_json_flag = cli.has_json_flag();

    if let Err(err) = run() {
        handle_error(err, has_json_flag);
    }
}

/// Handle errors with proper exit codes and optional JSON output
fn handle_error(err: anyhow::Error, json: bool) {
    // Try to downcast to WtError for structured error handling
    if let Some(wt_err) = err.downcast_ref::<WtError>() {
        let exit_code = wt_err.exit_code();

        if json {
            // Output JSON error format
            println!("{}", serde_json::to_string(&wt_err.to_json()).unwrap());
        } else {
            // Output human-readable error
            wt_err.print_human();
        }

        std::process::exit(exit_code);
    } else {
        // Fallback for non-WtError errors (shouldn't happen, but handle gracefully)
        if json {
            let json_err = serde_json::json!({
                "error": true,
                "code": "unknown",
                "message": format!("{:#}", err)
            });
            println!("{}", serde_json::to_string(&json_err).unwrap());
        } else {
            eprintln!("error: {:#}", err);
        }
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
        Command::Prune { json, quiet } => {
            crate::prune::prune_worktrees(json, quiet).map_err(|e| anyhow::anyhow!(e))
        }
        Command::Preview { path, json } => {
            crate::preview::print_preview(std::path::Path::new(&path), json)
        }

        Command::Config { paths } => {
            let mut config = crate::config::load()?;
            config.auto_discovery.paths = paths.clone();
            crate::config::save(&config)?;
            eprintln!("Auto-discovery paths configured:");
            for path in &paths {
                eprintln!("  {}", path);
            }
            eprintln!("\nYou can now use:");
            eprintln!("  wt list --all         # List worktrees across all repos");
            eprintln!("  wt interactive --all  # Interactive picker across all repos");
            Ok(())
        }
        Command::Agent { command } => {
            use crate::cli::AgentCommand;
            match command {
                AgentCommand::Context { json } => {
                    crate::agent::show_context(json).map_err(|e| anyhow::anyhow!(e))
                }
                AgentCommand::Status { json } => {
                    crate::agent::show_status(json).map_err(|e| anyhow::anyhow!(e))
                }
                AgentCommand::Onboard => {
                    crate::agent::show_onboard().map_err(|e| anyhow::anyhow!(e))
                }
            }
        }
    }
}
