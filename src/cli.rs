use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "wt", about = "Git worktree manager", version)]
#[command(after_help = "SHELL INTEGRATION:
  Run 'wt init' to set up shell integration (auto-detects your shell).")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Supported shells for shell integration
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Set up shell integration for wt
    ///
    /// Without arguments: auto-detects shell and offers to add integration to config file.
    /// With shell argument: prints the integration code to stdout (for manual setup).
    ///
    /// Examples:
    ///   wt init           # Interactive setup (recommended)
    ///   wt init zsh       # Print zsh integration code
    ///   wt init bash      # Print bash integration code
    ///   wt init fish      # Print fish integration code
    Init {
        /// Shell to generate integration for (optional - auto-detects if not provided)
        shell: Option<Shell>,
    },

    /// Interactive picker (fzf)
    Interactive {
        /// Pick from all discovered repositories
        #[arg(long)]
        all: bool,
    },

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

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Suppress non-essential output
        #[arg(short, long)]
        quiet: bool,
    },

    /// Remove a worktree (by branch name or path)
    Remove {
        target: String,

        /// Skip confirmation
        #[arg(long)]
        force: bool,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Suppress interactive prompts (without --force, will not remove)
        #[arg(short, long)]
        quiet: bool,
    },

    /// Prune stale worktrees
    Prune {
        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Suppress non-essential output
        #[arg(short, long)]
        quiet: bool,
    },

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
    Show {
        /// Output as JSON instead of YAML
        #[arg(long)]
        json: bool,
    },

    /// Set default editor (e.g. nvim, code)
    SetEditor { editor: String },

    /// Set auto-discovery search roots (repeatable)
    SetDiscoveryPaths { paths: Vec<String> },
}
