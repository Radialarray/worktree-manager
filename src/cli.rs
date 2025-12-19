use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "wt", about = "Git worktree manager", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Interactive picker (fzf)
    Interactive,

    /// List worktrees
    List {
        /// JSON output
        #[arg(long)]
        json: bool,

        /// Discover repos and list across all repos
        #[arg(long)]
        all: bool,
    },

    /// Add a new worktree
    Add {
        branch: String,

        /// Path to create the worktree in
        #[arg(short, long)]
        path: Option<String>,

        /// Remote to track (e.g. origin)
        #[arg(long)]
        track: Option<String>,
    },

    /// Remove a worktree (by branch name or path)
    Remove {
        target: String,

        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },

    /// Prune stale worktrees
    Prune,

    /// Print preview information for a worktree (used by fzf)
    Preview {
        #[arg(long)]
        path: String,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Create an initial config file (no-op if exists)
    Init,

    /// Show effective config
    Show,

    /// Set default editor (e.g. nvim, code)
    SetEditor { editor: String },

    /// Set auto-discovery search roots (repeatable)
    SetDiscoveryPaths { paths: Vec<String> },
}
