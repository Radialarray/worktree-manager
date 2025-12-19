# worktree-manager (wt)

A fast, intuitive CLI tool for managing Git worktrees with fzf-powered interactive selection. Quickly switch between worktrees, create new ones, and clean up stale branches—all from the command line.

## Features

- **Interactive Worktree Picker**: Use fzf to quickly navigate and select worktrees
- **Easy Worktree Management**: Create, remove, and list worktrees with simple commands
- **Shell Integration**: Seamlessly change directories or open worktrees in your editor
- **Auto-Discovery**: Automatically discover and manage worktrees across multiple repositories
- **JSON Output**: Machine-readable output for scripting and automation
- **Configurable**: Customize editor, fzf layout, discovery paths, and more
- **Fast & Lightweight**: Written in Rust for performance and minimal dependencies

## Installation

### Prerequisites

- **Git** (2.7.0+)
- **fzf** (for interactive selection) - [Install fzf](https://github.com/junegunn/fzf#installation)
- **Rust** (1.70+) - [Install Rust](https://rustup.rs/)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/worktree-manager.git
cd worktree-manager

# Install the binary
cargo install --path .
```

The binary will be installed as `wt` in your Cargo bin directory (usually `~/.cargo/bin`).

## Shell Integration

For full functionality, especially to use the `cd` action in interactive mode, you need to add a shell wrapper function to your shell configuration.

### For Zsh/Bash

Add this to your `~/.zshrc` or `~/.bashrc`:

```bash
wt() {
  if [[ $# -eq 0 ]] || [[ "$1" == "interactive" ]]; then
    local result
    result=$(command wt interactive "$@")
    if [[ -n $result ]]; then
      local action=$(echo "$result" | cut -d'|' -f1)
      local path=$(echo "$result" | cut -d'|' -f2)
      
      case $action in
        "cd")
          cd "$path" || return
          ;;
        "edit")
          ${EDITOR:-nvim} "$path"
          ;;
      esac
    fi
  else
    command wt "$@"
  fi
}
```

Then reload your shell:

```bash
source ~/.zshrc   # for zsh
# or
source ~/.bashrc  # for bash
```

This wrapper enables:
- **Enter**: Change directory to the selected worktree
- **Ctrl-E**: Open the worktree in your configured editor

## Usage

### Interactive Mode (Default)

Simply type `wt` to launch the interactive picker:

```bash
# Pick from current repository's worktrees
wt

# Pick from all discovered repositories
wt interactive --all
```

The fzf interface shows:
- **Branch name** on the left
- **Worktree path** on the right
- **Preview pane** showing commit info and status

**Keyboard shortcuts:**
- **Enter**: Change directory to the selected worktree
- **Ctrl-E**: Open the worktree in your editor
- **Esc/Ctrl-C**: Cancel

### List Worktrees

```bash
# List worktrees in the current repository
wt list

# List worktrees across all discovered repositories
wt list --all

# Get JSON output (useful for scripting)
wt list --json
wt list --all --json
```

Example output:
```
main         /path/to/repo/main
feature-x    /path/to/repo/feature-x
bugfix       /path/to/repo/bugfix
```

### Add Worktree

```bash
# Create a new worktree for a branch (auto-detects path)
wt add feature-new

# Create worktree at a specific path
wt add feature-new -p ~/work/custom-path

# Create worktree tracking a remote branch
wt add feature-new --track origin
```

### Remove Worktree

```bash
# Remove a worktree (by branch name or path) - prompts for confirmation
wt remove feature-old

# Force remove without confirmation
wt remove feature-old --force
```

### Prune Stale Worktrees

Clean up worktrees that have been deleted or are no longer accessible:

```bash
wt prune
```

### Configuration

#### Initialize Config

Create a default configuration file:

```bash
wt config init
```

This creates `~/.config/worktree-manager/config.yaml` if it doesn't exist.

#### View Current Config

```bash
wt config show
```

Shows the effective configuration with all current settings.

#### Set Default Editor

```bash
wt config set-editor nvim
wt config set-editor code
wt config set-editor emacs
```

#### Set Auto-Discovery Paths

Configure directories where wt should search for git repositories:

```bash
wt config set-discovery-paths ~/projects ~/work ~/.config
```

## Configuration File

The configuration file is located at `~/.config/worktree-manager/config.yaml`.

### Example Configuration

```yaml
version: "1.0.0"
editor: nvim
fzf:
  height: "40%"
  layout: reverse
  preview_window: "right:60%"
auto_discovery:
  enabled: true
  paths:
    - ~/projects
    - ~/work
    - ~/.config
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `version` | string | `1.0.0` | Config file version |
| `editor` | string | `nvim` | Default editor for opening worktrees (e.g., `code`, `vim`, `emacs`) |
| `fzf.height` | string | `40%` | Height of fzf window (percentage or fixed lines) |
| `fzf.layout` | string | `reverse` | fzf layout mode (`default`, `reverse`, `reverse-list`) |
| `fzf.preview_window` | string | `right:60%` | Preview pane position and size |
| `auto_discovery.enabled` | boolean | `true` | Enable automatic repository discovery |
| `auto_discovery.paths` | array | `[]` | Directories to search for repositories (empty = current dir only) |

### FZF Layout Options

- `default`: Normal top-to-bottom layout
- `reverse`: Bottom-to-top layout with preview on right
- `reverse-list`: Reverse list layout

## Examples

### Workflow: Create and Switch to a New Worktree

```bash
# Create a new feature branch worktree
wt add my-feature

# Interactive picker will show it
wt
# Select it and press Enter to cd into it
```

### Workflow: List and Remove Old Worktrees

```bash
# See all current worktrees
wt list --all

# Remove a worktree (with confirmation)
wt remove old-feature

# Or force remove
wt remove old-feature --force

# Clean up stale entries
wt prune
```

### Workflow: Use with Multiple Repositories

```bash
# Configure discovery paths to include multiple projects
wt config set-discovery-paths ~/work ~/personal ~/.config

# Now list all worktrees across all repos
wt list --all

# Pick from any discovered repository
wt interactive --all
```

### Workflow: Scripting with JSON

```bash
# Get all worktrees as JSON for processing
wt list --json | jq '.[] | select(.branch | contains("feature"))'

# Count worktrees
wt list --json | jq 'length'
```

## Troubleshooting

### `fzf: command not found`

Make sure fzf is installed and in your PATH:

```bash
# macOS with Homebrew
brew install fzf

# Ubuntu/Debian
apt-get install fzf

# Arch
pacman -S fzf

# Manual installation
git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf
~/.fzf/install
```

### Interactive Mode Doesn't Change Directory

Make sure you've installed the shell wrapper function (see [Shell Integration](#shell-integration)).

```bash
# Test the wrapper is loaded
type wt

# Should show it's a function, not a binary
```

### "No Worktrees Found"

Ensure your current directory or configured discovery paths contain valid git repositories with worktrees:

```bash
# Check current repo
git worktree list

# Check if wt can find repos
wt list
```

### Config File Issues

Check that your config file is valid YAML:

```bash
wt config show

# Should display the current configuration without errors
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Linting & Formatting

```bash
# Format code
cargo fmt

# Check with clippy
cargo clippy --all-targets --all-features -- -D warnings
```

## Architecture

The tool is organized into modular components:

- **cli.rs**: Command-line interface definition
- **interactive.rs**: Interactive fzf-based picker
- **git.rs**: Git operations and worktree listing
- **config.rs**: Configuration file management
- **add.rs**: Create new worktrees
- **remove.rs**: Delete worktrees
- **prune.rs**: Clean up stale worktrees
- **list.rs**: List worktrees with optional JSON output
- **discovery.rs**: Auto-discovery of git repositories
- **fzf.rs**: fzf integration and process handling
- **process.rs**: External process utilities

## Contributing

Contributions are welcome! Please ensure:

1. Code is formatted with `cargo fmt`
2. All tests pass: `cargo test`
3. No clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
4. Commits follow conventional commit style
5. Add tests for new functionality

## License

[Add your license here]

## See Also

- [git-worktree Documentation](https://git-scm.com/docs/git-worktree)
- [fzf](https://github.com/junegunn/fzf)
- [Other worktree tools](https://github.com/topics/git-worktree)
