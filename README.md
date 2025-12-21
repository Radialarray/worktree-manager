# wt - Git Worktree Manager

A fast, intuitive CLI for managing Git worktrees. Built for both humans (interactive fzf picker) and AI agents (JSON output, non-interactive mode).

## Features

- **Interactive Picker** - fzf-powered navigation with live preview
- **Shell Integration** - Press Enter to cd, Ctrl-E to open in editor
- **Agent-Friendly** - JSON output, `--quiet` mode, specialized agent commands
- **Multi-Repo Support** - Discover and manage worktrees across projects
- **Simple & Fast** - Written in Rust, minimal configuration

## Installation

### Prerequisites

- Git 2.7.0+
- [fzf](https://github.com/junegunn/fzf#installation) (for interactive mode)

### Homebrew (macOS/Linux) - Recommended

```bash
brew install Radialarray/worktree-manager/worktree-manager
```

Or tap first, then install:

```bash
brew tap Radialarray/worktree-manager
brew install worktree-manager
```

After installation, set up shell integration:

```bash
wt init  # Auto-detects your shell and configures it
source ~/.zshrc  # or ~/.bashrc, or open new terminal
```

### From Source

```bash
git clone https://github.com/Radialarray/worktree-manager.git
cd worktree-manager
cargo install --path .
wt init  # Set up shell integration
```

### Shell Integration

Required for interactive mode (`cd` and editor actions):

```bash
# Auto-setup (recommended)
wt init

# Manual setup
eval "$(wt init zsh)"   # Add to ~/.zshrc
eval "$(wt init bash)"  # Add to ~/.bashrc
wt init fish | source   # Add to ~/.config/fish/config.fish
```

Reload your shell after installation:
```bash
source ~/.zshrc  # or ~/.bashrc, or open new terminal
```

## Quick Start

```bash
# Interactive picker (requires shell integration)
wt

# List all worktrees
wt list

# Create new worktree
wt add feature-branch

# Remove worktree
wt remove feature-branch

# Clean up stale worktrees
wt prune
```

## Usage

### Interactive Mode

```bash
wt                    # Pick from current repo
wt interactive --all  # Pick from all configured repos
```

**Keyboard shortcuts:**
- **Enter** - Change to selected worktree
- **Ctrl-E** - Open worktree in `$EDITOR`
- **Esc** - Cancel

### CLI Commands

```bash
# List worktrees
wt list              # Current repo
wt list --all        # All discovered repos
wt list --json       # Machine-readable output

# Add worktree
wt add feature-x                # Auto-detect path
wt add feature-x -p ~/custom    # Custom path
wt add feature-x --track origin # Track remote

# Remove worktree
wt remove feature-x         # With confirmation
wt remove feature-x --force # Skip confirmation

# Prune stale worktrees
wt prune
```

### Multi-Repo Discovery

Configure paths to search for repositories:

```bash
wt config ~/projects ~/work
wt list --all           # List worktrees across all repos
wt interactive --all    # Interactive picker across all repos
```

## AI Agent Integration

`wt` is designed for AI coding agents with JSON output and non-interactive modes.

### Agent Commands

```bash
# Get onboarding documentation (~500 tokens)
wt agent onboard

# Get full worktree context
wt agent context [--json]

# Get minimal status (for frequent checks)
wt agent status [--json]
```

### Agent Best Practices

- Always use `--json` for machine-readable output
- Use `--quiet` to suppress interactive prompts
- Use `--force` with `remove` to skip confirmations
- Run `wt agent context` at session start

See [AGENTS.md](AGENTS.md) for comprehensive agent integration documentation.

### Example: OpenCode Custom Tool

Create `.opencode/tool/worktree.ts`:

```typescript
import { tool } from "@opencode-ai/plugin"

export const list = tool({
  description: "List git worktrees",
  args: {},
  async execute() {
    return await Bun.$`wt list --json`.text()
  },
})

export const create = tool({
  description: "Create worktree for branch",
  args: { branch: tool.schema.string() },
  async execute(args) {
    return await Bun.$`wt add ${args.branch} --json --quiet`.text()
  },
})
```

## Configuration

Configuration file: `~/.config/worktree-manager/config.yaml`

### Default Configuration

```yaml
version: "1.0.0"
fzf:
  height: "40%"
  layout: reverse
  preview_window: "right:60%"
auto_discovery:
  enabled: true
  paths: []
```

### Customization

- **FZF appearance**: Edit config.yaml to customize height, layout, preview window
- **Auto-discovery**: Use `wt config <paths...>` or edit `auto_discovery.paths`
- **Editor**: Ctrl-E uses `$EDITOR` environment variable (e.g., `export EDITOR=nvim`)

## Troubleshooting

### Interactive mode doesn't change directory

Ensure shell integration is installed and loaded:

```bash
type wt
# Should show "wt is a shell function", not "wt is /path/to/binary"
```

If not, run `wt init` and reload your shell.

### fzf not found

Install fzf for your system:

```bash
# macOS
brew install fzf

# Ubuntu/Debian  
apt-get install fzf

# Arch
pacman -S fzf
```

### Editor doesn't open with Ctrl-E

Set your `$EDITOR` environment variable:

```bash
export EDITOR=nvim  # Add to ~/.zshrc or ~/.bashrc
```

## Development

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test <test_name>

# Format & lint
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings

# Reinstall after changes
./reinstall.sh
```

## Project Structure

- `src/cli.rs` - Command-line interface
- `src/interactive.rs` - fzf-based picker
- `src/git.rs` - Git worktree operations
- `src/add.rs`, `src/remove.rs`, `src/prune.rs`, `src/list.rs` - Core commands
- `src/agent.rs` - Agent-specific commands
- `src/init.rs` - Shell integration generation
- `src/config.rs` - Configuration management
- `src/discovery.rs` - Multi-repo discovery

## Contributing

Contributions welcome! Please:

1. Format with `cargo fmt`
2. Pass tests: `cargo test`
3. Fix clippy warnings: `cargo clippy`
4. Follow conventional commit style
5. Add tests for new features

## Uninstall

To completely remove `wt` from your Mac:

```bash
# Download and run the uninstall script
curl -sSL https://raw.githubusercontent.com/yourusername/worktree-manager/main/uninstall.sh | bash

# Or if you have the repository cloned
./uninstall.sh
```

The uninstall script will:
- Remove the `wt` binary from your PATH
- Remove Homebrew installation (if applicable)
- Prompt before removing configuration directory (`~/.config/worktree-manager`)
- Remove shell integration from config files (creates backups)
- Clean up cargo installation artifacts

**Manual uninstall:**

```bash
# Remove binary
rm $(which wt)

# Remove configuration
rm -rf ~/.config/worktree-manager

# Remove shell integration (edit these files manually)
# - ~/.zshrc or ~/.bashrc: Remove lines containing "wt init"
# - ~/.config/fish/config.fish: Remove lines containing "wt init"

# If installed via Homebrew
brew uninstall worktree-manager

# If installed via cargo
cargo uninstall worktree-manager
```

## See Also

- [Git Worktree Documentation](https://git-scm.com/docs/git-worktree)
- [fzf](https://github.com/junegunn/fzf)
- [AGENTS.md](AGENTS.md) - AI agent integration guide
