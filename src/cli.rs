use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "wt", about = "Git worktree manager", version)]
#[command(after_help = "SHELL INTEGRATION:
  Run 'wt init' to set up shell integration (auto-detects your shell).")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    /// Check if the command has a --json flag set
    pub fn has_json_flag(&self) -> bool {
        match &self.command {
            Some(Command::List { json, .. }) => *json,
            Some(Command::Add { json, .. }) => *json,
            Some(Command::Remove { json, .. }) => *json,
            Some(Command::Prune { json, .. }) => *json,
            Some(Command::Preview { json, .. }) => *json,
            Some(Command::Config {
                command: ConfigCommand::Show { json },
            }) => *json,
            Some(Command::Agent {
                command: AgentCommand::Context { json } | AgentCommand::Status { json },
            }) => *json,
            _ => false,
        }
    }
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
    #[command(long_about = include_str!("help/list.md"))]
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Discover repos and list across all repos
        #[arg(long)]
        all: bool,
    },

    /// Add a new worktree
    #[command(long_about = include_str!("help/add.md"))]
    Add {
        /// Branch to create worktree for (optional - interactive picker if not provided)
        branch: Option<String>,

        /// Path to create the worktree in
        #[arg(short, long)]
        path: Option<String>,

        /// Remote to track (e.g. origin)
        #[arg(long)]
        track: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Suppress non-essential output and skip prompts
        #[arg(short, long)]
        quiet: bool,
    },

    /// Remove a worktree (by branch name or path)
    #[command(long_about = include_str!("help/remove.md"))]
    Remove {
        /// Worktree to remove (branch name or path) - optional, interactive picker if not provided
        target: Option<String>,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Suppress interactive prompts and non-essential output
        #[arg(short, long)]
        quiet: bool,
    },

    /// Prune stale worktrees
    #[command(long_about = include_str!("help/prune.md"))]
    Prune {
        /// Output as JSON
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

        /// Output as JSON for programmatic use
        #[arg(long)]
        json: bool,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    /// Agent-friendly context and status commands
    #[command(long_about = include_str!("help/agent.md"))]
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum AgentCommand {
    /// Display compact context about current worktree state
    Context {
        /// Output as JSON instead of human-readable format
        #[arg(long)]
        json: bool,
    },

    /// Display minimal status suitable for frequent injection
    Status {
        /// Output as JSON instead of human-readable format
        #[arg(long)]
        json: bool,
    },

    /// Output onboarding instructions for AI agents (similar to bd prime)
    ///
    /// Prints a compact workflow reference that can be injected into agent context.
    /// Includes CLI quick reference, JSON schemas, and common workflows.
    Onboard,
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
