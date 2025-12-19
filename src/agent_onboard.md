# wt - Git Worktree Manager

## Quick Reference

| Command | Description | Agent Flags |
|---------|-------------|-------------|
| `wt list` | List all worktrees | `--json`, `--all` |
| `wt add <branch>` | Create worktree | `--json`, `--quiet` |
| `wt remove <target>` | Remove worktree | `--json`, `--quiet`, `--force` |
| `wt prune` | Clean stale worktrees | `--json`, `--quiet` |
| `wt preview --path <p>` | Preview worktree | `--json` |
| `wt agent context` | Full worktree context | `--json` |
| `wt agent status` | Minimal status | `--json` |

## JSON Output Examples

### wt list --json
```json
[{"path": "/path/to/wt", "head": "abc123", "branch": "refs/heads/main", "is_main": true}]
```

### wt add <branch> --json --quiet
```json
{"success": true, "worktree": {"path": "/path/to/wt", "branch": "feature-x"}}
```

### wt remove <target> --json --force
```json
{"success": true, "removed": true, "branch": "feature-x", "path": "/path/to/wt"}
```

### wt agent status --json
```json
{"current": {"path": "/path/to/wt", "branch": "main", "dirty": true}, "count": 3}
```

## Common Workflows

```bash
# Create worktree for feature branch
wt add feature-x --json --quiet

# List all worktrees
wt list --json

# Check current state
wt agent status --json

# Remove worktree after PR merge
wt remove feature-x --force --json

# Clean up stale worktrees
wt prune --quiet --json
```

## Best Practices

1. Always use `--json` for programmatic parsing
2. Use `--quiet` to suppress interactive prompts
3. Use `--force` with remove when certain (skips confirmation)
4. Run `wt agent context` at session start for full layout
5. Check `dirty` status before switching worktrees

## Multi-Agent Workflows

Worktrees enable parallel work with multiple AI agents in isolated environments.

### Benefits
- **Parallel Work** - Multiple agents work on different features simultaneously
- **Isolation** - Each agent has its own working directory and branch
- **Context Clarity** - Agent knows exactly which branch/feature it's working on
- **Easy Cleanup** - Remove worktree when done without affecting others

### Pattern: One Agent Per Worktree

```bash
# Agent 1 works on feature-auth (Session 1)
wt add feature-auth --json --quiet
cd /path/to/feature-auth-worktree
# Agent 1 does its work...

# Agent 2 works on feature-api (Session 2 - separate process)
wt add feature-api --json --quiet
cd /path/to/feature-api-worktree
# Agent 2 does its work...
```

### Session Start Pattern

```bash
wt agent context      # Understand current worktree state
git status           # Verify working directory is clean
wt list --json       # See all available worktrees
```

### Session End Pattern

```bash
git status                                # Check uncommitted changes
wt list --json                            # Verify worktree state
wt remove <branch> --force --quiet --json # Clean up temporary worktree
wt prune --quiet --json                   # Prune stale references
```

### Avoiding Cross-Worktree Confusion

**Do:**
- ✅ Verify current worktree at session start with `wt agent context`
- ✅ Use absolute paths when referencing files across worktrees
- ✅ Don't `cd` between worktrees within a single agent session
- ✅ Each agent session owns exactly one worktree
- ✅ Clean up temporary worktrees when done

**Don't:**
- ❌ Assume you're in the main worktree without checking
- ❌ Accidentally commit to the wrong branch
- ❌ Leave temporary worktrees around after completion
- ❌ Have multiple agents modify the same worktree simultaneously

### Example: Parallel Feature Development

```bash
# Agent Manager creates two worktrees
wt add feature-auth --json --quiet
wt add feature-api --json --quiet

# Agent 1 (Session 1)
cd /repo/worktrees/feature-auth
wt agent context  # Confirms: working on feature-auth
# ... implement authentication ...
git add . && git commit -m "feat(auth): add JWT validation"
git push

# Agent 2 (Session 2 - different process)
cd /repo/worktrees/feature-api  
wt agent context  # Confirms: working on feature-api
# ... implement API endpoints ...
git add . && git commit -m "feat(api): add user endpoints"
git push

# Both complete independently, then clean up
wt remove feature-auth --force --quiet
wt remove feature-api --force --quiet
```
