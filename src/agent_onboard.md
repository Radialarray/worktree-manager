# wt - Git Worktree Manager

## Quick Reference

| Command | Description | Flags |
|---------|-------------|-------|
| `wt list [--all]` | List worktrees | `--json` |
| `wt add <branch>` | Create worktree | `--json`, `--quiet` |
| `wt remove <target>` | Remove worktree | `--json`, `--quiet`, `--force` |
| `wt prune` | Clean stale worktrees | `--json`, `--quiet` |
| `wt agent context` | Full worktree state | `--json` |
| `wt agent status` | Minimal status | `--json` |
| `wt config <paths>` | Set auto-discovery paths | - |

**Key flags:** `--json` (machine-readable), `--quiet` (non-interactive), `--force` (skip confirmations)

## JSON Schemas

```bash
# List worktrees
wt list --json
# [{"path": "/path", "head": "abc123", "branch": "refs/heads/main", "is_main": true}]

# Add worktree
wt add feature-x --json --quiet
# {"success": true, "worktree": {"path": "/path", "branch": "feature-x"}}

# Current status
wt agent status --json
# {"current": {"path": "/path", "branch": "main", "dirty": true}, "count": 3}
```

## Basic Workflows

```bash
# Session start
wt agent context              # Get worktree state
git status                   # Check for uncommitted changes

# Create and work
wt add feature-x --json --quiet
cd /path/to/feature-x
# ... do work ...
git add . && git commit && git push

# Session end
wt remove feature-x --force --quiet --json
wt prune --quiet --json
```

## Multi-Agent Patterns

**One Agent Per Worktree:** Each agent creates and owns a separate worktree for parallel work.

```bash
# Agent 1
wt add feature-auth --json --quiet && cd /path/to/feature-auth
wt agent context  # Verify location
# ... work ...
wt remove feature-auth --force --quiet

# Agent 2 (parallel, different process)
wt add feature-api --json --quiet && cd /path/to/feature-api
wt agent context  # Verify location
# ... work ...
wt remove feature-api --force --quiet
```

**Best Practices:**
- ✅ Run `wt agent context` at session start
- ✅ Each agent owns exactly one worktree
- ✅ Clean up temporary worktrees when done
- ❌ Don't `cd` between worktrees in one session
- ❌ Don't modify the same worktree from multiple agents

## Configuration

For `wt list --all` or `wt interactive --all` (cross-repo discovery):

```bash
wt config ~/projects ~/work
wt list --all
```

Config file: `~/.config/worktree-manager/config.yaml` (optional, for FZF customization)
