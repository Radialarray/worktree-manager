Remove a worktree by branch name or path.

Without arguments: interactive picker to select which worktree to remove.
With target argument: removes the specified worktree.

Examples:
  wt remove feature-x                  # Remove with confirmation
  wt remove feature-x --force          # Skip confirmation
  wt remove feature-x --json           # JSON output
  wt remove old-branch --force --quiet # Non-interactive removal

JSON Output Format:
  {
    "success": true,
    "removed": true,
    "branch": "feature-x",
    "path": "/path/to/worktree"
  }
