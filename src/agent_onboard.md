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
