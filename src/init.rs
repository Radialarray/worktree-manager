//! Shell integration code generation and setup for wt.
//!
//! This module provides:
//! - `wt init` - Interactive setup that detects shell and modifies config
//! - `wt init <shell>` - Outputs shell code for manual setup

use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::cli::Shell;

/// The marker comment we add to identify our integration line
const MARKER: &str = "# wt shell integration";

/// Returns the shell integration code for the given shell.
pub fn shell_init(shell: Shell) -> String {
    match shell {
        Shell::Zsh => ZSH_INIT.to_string(),
        Shell::Bash => BASH_INIT.to_string(),
        Shell::Fish => FISH_INIT.to_string(),
    }
}

/// Run interactive shell setup - detect shell, find config, ask user, add integration.
pub fn run_interactive_setup() -> Result<()> {
    // Detect shell
    let shell = detect_shell()?;
    eprintln!("Detected shell: {}", shell_name(shell));

    // Find config file
    let config_path = shell_config_path(shell)?;
    eprintln!("Config file: {}", config_path.display());

    // Check if already configured
    if is_already_configured(&config_path)? {
        eprintln!(
            "\n✓ Shell integration is already configured in {}",
            config_path.display()
        );
        eprintln!(
            "  To reconfigure, remove the line containing '{}' and run 'wt init' again.",
            MARKER
        );
        return Ok(());
    }

    // Show what we'll add
    let integration_line = integration_line_for_shell(shell);
    eprintln!("\nwt needs to add shell integration for cd/edit actions to work.");
    eprintln!(
        "The following line will be added to {}:\n",
        config_path.display()
    );
    eprintln!("  {}", integration_line);
    eprintln!();

    // Ask for confirmation
    if !confirm("Add shell integration?")? {
        eprintln!("Aborted. To set up manually, add the line above to your shell config.");
        return Ok(());
    }

    // Append to config file
    append_to_config(&config_path, shell)?;

    eprintln!("\n✓ Added shell integration to {}", config_path.display());
    eprintln!(
        "  Run '{}' or restart your shell to activate.",
        reload_command(shell, &config_path)
    );

    Ok(())
}

/// Detect the user's shell from $SHELL environment variable.
fn detect_shell() -> Result<Shell> {
    let shell_path = env::var("SHELL").context("$SHELL environment variable not set")?;

    if shell_path.contains("zsh") {
        Ok(Shell::Zsh)
    } else if shell_path.contains("bash") {
        Ok(Shell::Bash)
    } else if shell_path.contains("fish") {
        Ok(Shell::Fish)
    } else {
        bail!(
            "Unsupported shell: {}\nSupported shells: zsh, bash, fish\n\nFor manual setup, run: wt init <shell>",
            shell_path
        )
    }
}

/// Get the config file path for the given shell.
fn shell_config_path(shell: Shell) -> Result<PathBuf> {
    let home = env::var("HOME").context("$HOME environment variable not set")?;
    let home = PathBuf::from(home);

    let path = match shell {
        Shell::Zsh => {
            // Check for .zshrc first, common location
            let zshrc = home.join(".zshrc");
            if zshrc.exists() {
                zshrc
            } else {
                // Check ZDOTDIR
                if let Ok(zdotdir) = env::var("ZDOTDIR") {
                    PathBuf::from(zdotdir).join(".zshrc")
                } else {
                    // Default to ~/.zshrc even if it doesn't exist
                    zshrc
                }
            }
        }
        Shell::Bash => {
            // Prefer .bashrc for interactive shells
            let bashrc = home.join(".bashrc");
            if bashrc.exists() {
                bashrc
            } else {
                // Fall back to .bash_profile
                let bash_profile = home.join(".bash_profile");
                if bash_profile.exists() {
                    bash_profile
                } else {
                    bashrc
                }
            }
        }
        Shell::Fish => {
            // Fish config is always in the same place
            home.join(".config/fish/config.fish")
        }
    };

    Ok(path)
}

/// Check if the config file already has wt integration.
fn is_already_configured(config_path: &PathBuf) -> Result<bool> {
    if !config_path.exists() {
        return Ok(false);
    }

    let contents = fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;

    // Check for our marker or the eval line
    Ok(contents.contains(MARKER)
        || contents.contains("eval \"$(wt init")
        || contents.contains("wt init fish | source")
        || contents.contains("wt init zsh)")
        || contents.contains("wt init bash)"))
}

/// Get the integration line for a shell (what we show the user).
fn integration_line_for_shell(shell: Shell) -> &'static str {
    match shell {
        Shell::Zsh => "eval \"$(wt init zsh)\"",
        Shell::Bash => "eval \"$(wt init bash)\"",
        Shell::Fish => "wt init fish | source",
    }
}

/// Append the integration to the config file.
fn append_to_config(config_path: &PathBuf, shell: Shell) -> Result<()> {
    // Ensure parent directory exists (for fish)
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(config_path)
        .with_context(|| format!("failed to open {} for writing", config_path.display()))?;

    // Add a newline before our content if the file doesn't end with one
    let needs_newline = if config_path.exists() {
        let contents = fs::read_to_string(config_path).unwrap_or_default();
        !contents.is_empty() && !contents.ends_with('\n')
    } else {
        false
    };

    if needs_newline {
        writeln!(file)?;
    }

    // Write the integration line with marker
    writeln!(file, "\n{}", MARKER)?;
    writeln!(file, "{}", integration_line_for_shell(shell))?;

    Ok(())
}

/// Get the command to reload the shell config.
fn reload_command(shell: Shell, config_path: &std::path::Path) -> String {
    match shell {
        Shell::Zsh | Shell::Bash => format!("source {}", config_path.display()),
        Shell::Fish => "exec fish".to_string(),
    }
}

/// Get the display name for a shell.
fn shell_name(shell: Shell) -> &'static str {
    match shell {
        Shell::Zsh => "zsh",
        Shell::Bash => "bash",
        Shell::Fish => "fish",
    }
}

/// Ask for yes/no confirmation.
fn confirm(prompt: &str) -> Result<bool> {
    eprint!("{} [y/N]: ", prompt);
    io::stderr().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    Ok(line.trim().eq_ignore_ascii_case("y") || line.trim().eq_ignore_ascii_case("yes"))
}

/// Zsh shell integration
const ZSH_INIT: &str = r#"# wt - git worktree manager shell integration (zsh)

__wt_cd() {
    local dir="$1"
    if [[ -d "$dir" ]]; then
        builtin cd "$dir" || return 1
    else
        echo "wt: directory not found: $dir" >&2
        return 1
    fi
}

__wt_edit() {
    local dir="$1"
    if [[ -d "$dir" ]]; then
        builtin cd "$dir" || return 1
        "${EDITOR:-vim}" .
    else
        echo "wt: directory not found: $dir" >&2
        return 1
    fi
}

wt() {
    if [[ $# -eq 0 ]] || [[ "$1" == "interactive" ]]; then
        local output
        output=$(command wt "$@" 2>&1)
        local exit_code=$?
        
        if [[ $exit_code -ne 0 ]]; then
            echo "$output" >&2
            return $exit_code
        fi
        
        case "$output" in
            cd\|*)
                __wt_cd "${output#cd|}"
                ;;
            edit\|*)
                __wt_edit "${output#edit|}"
                ;;
            *)
                [[ -n "$output" ]] && echo "$output"
                ;;
        esac
    else
        command wt "$@"
    fi
}

# Completions
_wt() {
    local -a commands
    commands=(
        'init:Set up shell integration'
        'interactive:Interactive picker (fzf)'
        'list:List worktrees'
        'add:Add a new worktree'
        'remove:Remove a worktree'
        'prune:Prune stale worktrees'
        'preview:Print preview information'
        'config:Configuration management'
        'help:Print help'
    )

    local -a config_commands
    config_commands=(
        'init:Create an initial config file'
        'show:Show effective config'
        'set-editor:Set default editor'
        'set-discovery-paths:Set auto-discovery search roots'
    )

    local -a shells
    shells=('bash' 'zsh' 'fish')

    _arguments -C \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe -t commands 'wt command' commands
            ;;
        args)
            case $words[1] in
                init)
                    _describe -t shells 'shell' shells
                    ;;
                config)
                    _describe -t config_commands 'config command' config_commands
                    ;;
                add)
                    local -a branches
                    branches=($(git branch --format='%(refname:short)' 2>/dev/null))
                    _describe -t branches 'branch' branches
                    ;;
                remove)
                    local -a worktrees
                    worktrees=($(git worktree list --porcelain 2>/dev/null | grep '^branch' | sed 's/branch refs\/heads\///'))
                    _describe -t worktrees 'worktree' worktrees
                    ;;
                list)
                    _arguments \
                        '--json[JSON output]' \
                        '--all[List across all discovered repositories]'
                    ;;
                interactive)
                    _arguments \
                        '--all[Pick from all discovered repositories]'
                    ;;
            esac
            ;;
    esac
}

compdef _wt wt
"#;

/// Bash shell integration
const BASH_INIT: &str = r#"# wt - git worktree manager shell integration (bash)

__wt_cd() {
    local dir="$1"
    if [[ -d "$dir" ]]; then
        builtin cd "$dir" || return 1
    else
        echo "wt: directory not found: $dir" >&2
        return 1
    fi
}

__wt_edit() {
    local dir="$1"
    if [[ -d "$dir" ]]; then
        builtin cd "$dir" || return 1
        "${EDITOR:-vim}" .
    else
        echo "wt: directory not found: $dir" >&2
        return 1
    fi
}

wt() {
    if [[ $# -eq 0 ]] || [[ "$1" == "interactive" ]]; then
        local output
        output=$(command wt "$@" 2>&1)
        local exit_code=$?
        
        if [[ $exit_code -ne 0 ]]; then
            echo "$output" >&2
            return $exit_code
        fi
        
        case "$output" in
            cd\|*)
                __wt_cd "${output#cd|}"
                ;;
            edit\|*)
                __wt_edit "${output#edit|}"
                ;;
            *)
                [[ -n "$output" ]] && echo "$output"
                ;;
        esac
    else
        command wt "$@"
    fi
}

# Completions
_wt_completions() {
    local cur prev commands config_commands shells
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    commands="init interactive list add remove prune preview config help"
    config_commands="init show set-editor set-discovery-paths"
    shells="bash zsh fish"

    case "${COMP_CWORD}" in
        1)
            COMPREPLY=( $(compgen -W "${commands}" -- "${cur}") )
            ;;
        2)
            case "${prev}" in
                init)
                    COMPREPLY=( $(compgen -W "${shells}" -- "${cur}") )
                    ;;
                config)
                    COMPREPLY=( $(compgen -W "${config_commands}" -- "${cur}") )
                    ;;
                add)
                    local branches
                    branches=$(git branch --format='%(refname:short)' 2>/dev/null)
                    COMPREPLY=( $(compgen -W "${branches}" -- "${cur}") )
                    ;;
                remove)
                    local worktrees
                    worktrees=$(git worktree list --porcelain 2>/dev/null | grep '^branch' | sed 's/branch refs\/heads\///')
                    COMPREPLY=( $(compgen -W "${worktrees}" -- "${cur}") )
                    ;;
                list)
                    COMPREPLY=( $(compgen -W "--json --all" -- "${cur}") )
                    ;;
                interactive)
                    COMPREPLY=( $(compgen -W "--all" -- "${cur}") )
                    ;;
            esac
            ;;
    esac
}

complete -F _wt_completions wt
"#;

/// Fish shell integration
const FISH_INIT: &str = r#"# wt - git worktree manager shell integration (fish)

function __wt_cd
    set -l dir $argv[1]
    if test -d "$dir"
        builtin cd "$dir"
    else
        echo "wt: directory not found: $dir" >&2
        return 1
    end
end

function __wt_edit
    set -l dir $argv[1]
    if test -d "$dir"
        builtin cd "$dir"
        if set -q EDITOR
            $EDITOR .
        else
            vim .
        end
    else
        echo "wt: directory not found: $dir" >&2
        return 1
    end
end

function wt
    if test (count $argv) -eq 0; or test "$argv[1]" = "interactive"
        set -l output (command wt $argv 2>&1)
        set -l exit_code $status
        
        if test $exit_code -ne 0
            echo "$output" >&2
            return $exit_code
        end
        
        switch "$output"
            case 'cd|*'
                set -l path (string replace 'cd|' '' "$output")
                __wt_cd "$path"
            case 'edit|*'
                set -l path (string replace 'edit|' '' "$output")
                __wt_edit "$path"
            case '*'
                if test -n "$output"
                    echo "$output"
                end
        end
    else
        command wt $argv
    end
end

# Completions
complete -c wt -e
complete -c wt -n "__fish_use_subcommand" -a "init" -d "Set up shell integration"
complete -c wt -n "__fish_use_subcommand" -a "interactive" -d "Interactive picker (fzf)"
complete -c wt -n "__fish_use_subcommand" -a "list" -d "List worktrees"
complete -c wt -n "__fish_use_subcommand" -a "add" -d "Add a new worktree"
complete -c wt -n "__fish_use_subcommand" -a "remove" -d "Remove a worktree"
complete -c wt -n "__fish_use_subcommand" -a "prune" -d "Prune stale worktrees"
complete -c wt -n "__fish_use_subcommand" -a "preview" -d "Print preview information"
complete -c wt -n "__fish_use_subcommand" -a "config" -d "Configuration management"
complete -c wt -n "__fish_use_subcommand" -a "help" -d "Print help"

complete -c wt -n "__fish_seen_subcommand_from init" -a "bash zsh fish" -d "Shell"

complete -c wt -n "__fish_seen_subcommand_from config" -a "init" -d "Create initial config file"
complete -c wt -n "__fish_seen_subcommand_from config" -a "show" -d "Show effective config"
complete -c wt -n "__fish_seen_subcommand_from config" -a "set-editor" -d "Set default editor"
complete -c wt -n "__fish_seen_subcommand_from config" -a "set-discovery-paths" -d "Set discovery search roots"

complete -c wt -n "__fish_seen_subcommand_from list" -l json -d "JSON output"
complete -c wt -n "__fish_seen_subcommand_from list" -l all -d "List across all repos"

complete -c wt -n "__fish_seen_subcommand_from interactive" -l all -d "Pick from all repos"

complete -c wt -n "__fish_seen_subcommand_from add" -a "(git branch --format='%(refname:short)' 2>/dev/null)"

complete -c wt -n "__fish_seen_subcommand_from remove" -a "(git worktree list --porcelain 2>/dev/null | string match 'branch *' | string replace 'branch refs/heads/' '')"
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zsh_init_contains_wt_function() {
        let output = shell_init(Shell::Zsh);
        assert!(output.contains("wt()"));
        assert!(output.contains("__wt_cd"));
        assert!(output.contains("__wt_edit"));
    }

    #[test]
    fn test_bash_init_contains_wt_function() {
        let output = shell_init(Shell::Bash);
        assert!(output.contains("wt()"));
        assert!(output.contains("__wt_cd"));
        assert!(output.contains("__wt_edit"));
    }

    #[test]
    fn test_fish_init_contains_wt_function() {
        let output = shell_init(Shell::Fish);
        assert!(output.contains("function wt"));
        assert!(output.contains("function __wt_cd"));
        assert!(output.contains("function __wt_edit"));
    }

    #[test]
    fn test_integration_line_for_shell() {
        assert_eq!(
            integration_line_for_shell(Shell::Zsh),
            "eval \"$(wt init zsh)\""
        );
        assert_eq!(
            integration_line_for_shell(Shell::Bash),
            "eval \"$(wt init bash)\""
        );
        assert_eq!(
            integration_line_for_shell(Shell::Fish),
            "wt init fish | source"
        );
    }

    #[test]
    fn test_shell_name() {
        assert_eq!(shell_name(Shell::Zsh), "zsh");
        assert_eq!(shell_name(Shell::Bash), "bash");
        assert_eq!(shell_name(Shell::Fish), "fish");
    }
}
