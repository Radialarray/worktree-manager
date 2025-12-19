Add a new worktree for a branch.

Without arguments: interactive branch picker to select which branch to create worktree for.
With branch argument: creates worktree for the specified branch.

Examples:
  wt add feature-x              # Create worktree for branch
  wt add feature-x -p ~/custom  # Custom path
  wt add feature-x --json       # JSON output
  wt add feature-x --quiet      # Non-interactive (for scripts)

JSON Output Format:
  {
    "success": true,
    "worktree": {
      "path": "/path/to/worktree",
      "branch": "feature-x"
    }
  }
