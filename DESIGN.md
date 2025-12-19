# worktree-manager Design

## Inspiration
Based on the `pj` script pattern from .zshrc:
```zsh
pj() {
  local dir
  dir=$(find "$PROJECTS_DIR" -mindepth 1 -maxdepth 1 -type d | \
    fzf --height=40% --reverse --prompt="Projects> " \
        --preview 'ls -1 {} | head -n 20')

  if [[ -n $dir ]]; then
    echo "Open in Neovim? (y/N): \c"
    read -r answer
    if [[ $answer =~ ^[Yy]$ ]]; then
      nvim "$dir"
    else
      cd "$dir" || return
    fi
  fi
}
```

## Core Concept
A fuzzy-finder based git worktree manager that operates on the **current repository**. When you run `wt` from any worktree, it finds all worktrees for that repo and lets you quickly switch between them.

## Key Features

### 1. Interactive Worktree Picker (`wt`)
**Primary use case**: Run `wt` from any worktree in a repository
- Lists all worktrees for the **current repository only**
- Fuzzy finding with `fzf`
- Rich preview pane showing:
  - Repository name
  - Branch name
  - Git status (clean/dirty, ahead/behind)
  - Recent commits (last 3-5)
  - Modified files count
  - Worktree path
- Actions after selection:
  - **Navigate**: Change directory to selected worktree
  - **Edit**: Open worktree in configured editor (Neovim, VSCode, etc.)
  - **Info**: Display detailed worktree information
  - **Cancel**: Exit without action

### 2. Worktree Management

#### List worktrees
```bash
wt list          # List all worktrees for current repo
wt list --json   # JSON output for scripting
```

#### Create worktree
```bash
wt add <branch>                    # Create worktree for branch
wt add <branch> -p <path>          # Custom path
wt add <branch> --track <remote>   # Track remote branch
```

#### Remove worktree
```bash
wt remove <worktree>    # Remove worktree (interactive confirmation)
wt remove --force       # Skip confirmation
wt prune                # Clean up stale worktrees
```

#### Configuration
```bash
wt config init                 # Initialize config
wt config set-editor <editor>  # Set default editor (nvim, code, etc.)
wt config show                 # Show current config
```

## Technical Architecture

### Language & Tools
- **Language**: TypeScript with Node.js
- **CLI Framework**: Commander.js or yargs
- **Fuzzy Finder**: fzf (via child_process)
- **Git Integration**: simple-git or direct git commands

### Project Structure
```
src/
├── cli/
│   ├── commands/
│   │   ├── list.ts
│   │   ├── add.ts
│   │   ├── remove.ts
│   │   ├── config.ts
│   │   └── interactive.ts
│   └── index.ts
├── core/
│   ├── worktree.ts       # Worktree management
│   ├── git.ts            # Git operations
│   ├── fzf.ts            # fzf integration
│   └── config.ts         # Config management
├── types/
│   └── index.ts
└── utils/
    ├── paths.ts
    └── shell.ts
```

### Configuration File
Location: `~/.config/worktree-manager/config.json`

```json
{
  "version": "1.0.0",
  "editor": "nvim",
  "fzf": {
    "height": "40%",
    "layout": "reverse",
    "preview_window": "right:60%"
  },
  "autoDiscovery": {
    "enabled": true,
    "paths": ["/Users/user/dev"]
  }
}
```

### Shell Integration
For `cd` functionality, need shell wrapper (like pj):

```zsh
# ~/.zshrc
wt() {
  if [[ $# -eq 0 ]]; then
    # Interactive mode
    local result
    result=$(worktree-manager interactive)
    if [[ -n $result ]]; then
      local action=$(echo "$result" | cut -d'|' -f1)
      local path=$(echo "$result" | cut -d'|' -f2)
      
      case $action in
        "cd")
          cd "$path" || return
          ;;
        "edit")
          $EDITOR "$path"
          ;;
      esac
    fi
  else
    # Pass through to CLI
    worktree-manager "$@"
  fi
}
```

## Implementation Phases

### Phase 1: Core Functionality
- [ ] Basic project setup (TypeScript, dependencies)
- [ ] Git worktree discovery (`git worktree list --porcelain`)
- [ ] Config file management
- [ ] Simple list command

### Phase 2: Interactive Mode
- [ ] fzf integration
- [ ] Preview pane with git info
- [ ] Action selection (cd/edit/info)
- [ ] Shell integration script

### Phase 3: Management Commands
- [ ] Add worktree command
- [ ] Remove worktree command
- [ ] Prune stale worktrees
- [ ] Config commands

### Phase 4: Advanced Features
- [ ] Auto-discovery of repositories
- [ ] Custom preview formatting
- [ ] Tab completion for shells
- [ ] Documentation and examples

## Example Usage

```bash
# You're in any worktree of a repository
$ cd ~/dev/myproject  # or ~/dev/myproject-feature-x, etc.

# Interactive picker (main use case)
$ wt
# Shows fzf with all worktrees for myproject, select one → choose action

# List all worktrees for current repo
$ wt list
main       /Users/user/dev/myproject          [main] ✓ clean
feature-x  /Users/user/dev/myproject-feature  [feature-x] ✗ dirty
develop    /Users/user/dev/myproject-develop  [develop] ✓ clean

# Create new worktree
$ wt add feature-auth
Created worktree at: /Users/user/dev/myproject-feature-auth

# Remove worktree
$ wt remove feature-auth
Remove worktree 'feature-auth'? (y/N): y
Worktree removed.

# Configure editor
$ wt config set-editor code
Editor set to: code
```

## Preview Pane Format
```
┌─ Worktree Info ────────────────────────┐
│ Repo:     myproject                     │
│ Branch:   feature-auth                  │
│ Path:     ~/dev/myproject-feature-auth  │
│ Status:   2 modified, 1 untracked       │
│                                         │
│ Recent Commits:                         │
│ • abc1234 Add authentication routes     │
│ • def5678 Update user model             │
│ • ghi9012 Fix validation bug            │
│                                         │
│ Modified Files:                         │
│ • src/auth/routes.ts                    │
│ • src/models/user.ts                    │
└─────────────────────────────────────────┘
```

## Dependencies
- `commander` - CLI framework
- `fzf` (external) - Fuzzy finder
- `simple-git` - Git operations
- `chalk` - Terminal colors
- `inquirer` or `prompts` - Interactive prompts
- `cosmiconfig` - Config management

## Distribution
- npm package: `worktree-manager` or `wtm`
- Binary name: `wt` (via npm bin)
- Shell function: `wt` (wrapper for cd integration)
