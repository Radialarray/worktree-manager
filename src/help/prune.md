Clean up worktrees that have been deleted or are no longer accessible.

Examples:
  wt prune         # Interactive cleanup
  wt prune --json  # JSON output with pruned list
  wt prune --quiet # Suppress non-essential output

JSON Output Format:
  {
    "pruned": ["path1", "path2"],
    "count": 2
  }
