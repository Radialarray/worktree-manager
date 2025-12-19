List worktrees in the current repository or across all discovered repositories.

Examples:
  wt list                    # List worktrees in current repo
  wt list --all              # List across all discovered repos
  wt list --json             # JSON output for scripting
  wt list --json | jq '.'    # Parse with jq

JSON Output Format:
  [
    {
      "path": "/path/to/worktree",
      "branch": "refs/heads/main",
      "head": "abc1234",
      "is_main": true
    }
  ]
