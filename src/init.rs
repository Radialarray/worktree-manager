//! Shell integration code generation for wt.
//!
//! This module provides `wt init <shell>` functionality, outputting shell code
//! that users can eval in their shell configuration files.

use crate::cli::Shell;

/// Returns the shell integration code for the given shell.
pub fn shell_init(shell: Shell) -> String {
    match shell {
        Shell::Zsh => ZSH_INIT.to_string(),
        Shell::Bash => BASH_INIT.to_string(),
        Shell::Fish => FISH_INIT.to_string(),
    }
}

/// Zsh shell integration
///
/// Creates a `wt` function that wraps the binary and handles cd/edit actions.
/// Also includes shell completions.
const ZSH_INIT: &str = r#"# wt - git worktree manager shell integration (zsh)
# Add to ~/.zshrc: eval "$(wt init zsh)"

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
        # Use $EDITOR, fall back to vim
        "${EDITOR:-vim}" .
    else
        echo "wt: directory not found: $dir" >&2
        return 1
    fi
}

wt() {
    # Check if this is an interactive command (no args, or 'interactive')
    if [[ $# -eq 0 ]] || [[ "$1" == "interactive" ]]; then
        local output
        output=$(command wt "$@" 2>&1)
        local exit_code=$?
        
        if [[ $exit_code -ne 0 ]]; then
            echo "$output" >&2
            return $exit_code
        fi
        
        # Parse the action|path output
        case "$output" in
            cd\|*)
                __wt_cd "${output#cd|}"
                ;;
            edit\|*)
                __wt_edit "${output#edit|}"
                ;;
            *)
                # Not an action output, just print it
                [[ -n "$output" ]] && echo "$output"
                ;;
        esac
    else
        # Pass through to the binary for other commands
        command wt "$@"
    fi
}

# Completions
_wt() {
    local -a commands
    commands=(
        'init:Print shell integration code for the given shell'
        'interactive:Interactive picker (fzf)'
        'list:List worktrees'
        'add:Add a new worktree'
        'remove:Remove a worktree (by branch name or path)'
        'prune:Prune stale worktrees'
        'preview:Print preview information for a worktree'
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
                    # Complete branch names
                    local -a branches
                    branches=($(git branch --format='%(refname:short)' 2>/dev/null))
                    _describe -t branches 'branch' branches
                    ;;
                remove)
                    # Complete worktree branches
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
///
/// Creates a `wt` function that wraps the binary and handles cd/edit actions.
/// Also includes shell completions.
const BASH_INIT: &str = r#"# wt - git worktree manager shell integration (bash)
# Add to ~/.bashrc: eval "$(wt init bash)"

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
        # Use $EDITOR, fall back to vim
        "${EDITOR:-vim}" .
    else
        echo "wt: directory not found: $dir" >&2
        return 1
    fi
}

wt() {
    # Check if this is an interactive command (no args, or 'interactive')
    if [[ $# -eq 0 ]] || [[ "$1" == "interactive" ]]; then
        local output
        output=$(command wt "$@" 2>&1)
        local exit_code=$?
        
        if [[ $exit_code -ne 0 ]]; then
            echo "$output" >&2
            return $exit_code
        fi
        
        # Parse the action|path output
        case "$output" in
            cd\|*)
                __wt_cd "${output#cd|}"
                ;;
            edit\|*)
                __wt_edit "${output#edit|}"
                ;;
            *)
                # Not an action output, just print it
                [[ -n "$output" ]] && echo "$output"
                ;;
        esac
    else
        # Pass through to the binary for other commands
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
                    # Complete branch names
                    local branches
                    branches=$(git branch --format='%(refname:short)' 2>/dev/null)
                    COMPREPLY=( $(compgen -W "${branches}" -- "${cur}") )
                    ;;
                remove)
                    # Complete worktree branches
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
///
/// Creates a `wt` function that wraps the binary and handles cd/edit actions.
/// Also includes shell completions.
const FISH_INIT: &str = r#"# wt - git worktree manager shell integration (fish)
# Add to ~/.config/fish/config.fish: wt init fish | source

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
        # Use $EDITOR, fall back to vim
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
    # Check if this is an interactive command (no args, or 'interactive')
    if test (count $argv) -eq 0; or test "$argv[1]" = "interactive"
        set -l output (command wt $argv 2>&1)
        set -l exit_code $status
        
        if test $exit_code -ne 0
            echo "$output" >&2
            return $exit_code
        end
        
        # Parse the action|path output
        switch "$output"
            case 'cd|*'
                set -l path (string replace 'cd|' '' "$output")
                __wt_cd "$path"
            case 'edit|*'
                set -l path (string replace 'edit|' '' "$output")
                __wt_edit "$path"
            case '*'
                # Not an action output, just print it
                if test -n "$output"
                    echo "$output"
                end
        end
    else
        # Pass through to the binary for other commands
        command wt $argv
    end
end

# Completions
complete -c wt -e  # Clear existing completions
complete -c wt -n "__fish_use_subcommand" -a "init" -d "Print shell integration code"
complete -c wt -n "__fish_use_subcommand" -a "interactive" -d "Interactive picker (fzf)"
complete -c wt -n "__fish_use_subcommand" -a "list" -d "List worktrees"
complete -c wt -n "__fish_use_subcommand" -a "add" -d "Add a new worktree"
complete -c wt -n "__fish_use_subcommand" -a "remove" -d "Remove a worktree"
complete -c wt -n "__fish_use_subcommand" -a "prune" -d "Prune stale worktrees"
complete -c wt -n "__fish_use_subcommand" -a "preview" -d "Print preview information"
complete -c wt -n "__fish_use_subcommand" -a "config" -d "Configuration management"
complete -c wt -n "__fish_use_subcommand" -a "help" -d "Print help"

# init subcommand completions
complete -c wt -n "__fish_seen_subcommand_from init" -a "bash zsh fish" -d "Shell"

# config subcommand completions
complete -c wt -n "__fish_seen_subcommand_from config" -a "init" -d "Create initial config file"
complete -c wt -n "__fish_seen_subcommand_from config" -a "show" -d "Show effective config"
complete -c wt -n "__fish_seen_subcommand_from config" -a "set-editor" -d "Set default editor"
complete -c wt -n "__fish_seen_subcommand_from config" -a "set-discovery-paths" -d "Set discovery search roots"

# list subcommand options
complete -c wt -n "__fish_seen_subcommand_from list" -l json -d "JSON output"
complete -c wt -n "__fish_seen_subcommand_from list" -l all -d "List across all repos"

# interactive subcommand options
complete -c wt -n "__fish_seen_subcommand_from interactive" -l all -d "Pick from all repos"

# add subcommand - complete branch names
complete -c wt -n "__fish_seen_subcommand_from add" -a "(git branch --format='%(refname:short)' 2>/dev/null)"

# remove subcommand - complete worktree branches
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
        assert!(output.contains(r"cd\|*)")); // escaped pipe in shell case
        assert!(output.contains(r"edit\|*)")); // escaped pipe in shell case
    }

    #[test]
    fn test_zsh_init_contains_completions() {
        let output = shell_init(Shell::Zsh);
        assert!(output.contains("_wt()"));
        assert!(output.contains("compdef _wt wt"));
        assert!(output.contains("'init:"));
        assert!(output.contains("'list:"));
    }

    #[test]
    fn test_bash_init_contains_wt_function() {
        let output = shell_init(Shell::Bash);
        assert!(output.contains("wt()"));
        assert!(output.contains("__wt_cd"));
        assert!(output.contains("__wt_edit"));
        assert!(output.contains(r"cd\|*)")); // escaped pipe in shell case
        assert!(output.contains(r"edit\|*)")); // escaped pipe in shell case
    }

    #[test]
    fn test_bash_init_contains_completions() {
        let output = shell_init(Shell::Bash);
        assert!(output.contains("_wt_completions()"));
        assert!(output.contains("complete -F _wt_completions wt"));
        assert!(output.contains("commands="));
    }

    #[test]
    fn test_fish_init_contains_wt_function() {
        let output = shell_init(Shell::Fish);
        assert!(output.contains("function wt"));
        assert!(output.contains("function __wt_cd"));
        assert!(output.contains("function __wt_edit"));
        assert!(output.contains("'cd|*'"));
        assert!(output.contains("'edit|*'"));
    }

    #[test]
    fn test_fish_init_contains_completions() {
        let output = shell_init(Shell::Fish);
        assert!(output.contains("complete -c wt"));
        assert!(output.contains("-a \"init\""));
        assert!(output.contains("-a \"list\""));
    }
}
