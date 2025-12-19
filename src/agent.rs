use anyhow::{Context, Result};
use serde::Serialize;

use crate::git;

#[derive(Serialize)]
struct AgentContext {
    current_worktree: Option<WorktreeInfo>,
    other_worktrees: Vec<WorktreeInfo>,
    repository: RepositoryInfo,
}

#[derive(Serialize)]
struct WorktreeInfo {
    path: String,
    branch: Option<String>,
    head: Option<String>,
    dirty: bool,
}

#[derive(Serialize)]
struct RepositoryInfo {
    root: String,
    total_worktrees: usize,
}

/// Display compact context about current worktree state for agents.
pub fn show_context(json: bool) -> Result<()> {
    let repo_root = git::repo_root(None).context("not in a git repository")?;
    let worktrees = git::worktrees_porcelain(&repo_root)?;

    // Get current directory to determine which worktree we're in
    let current_dir = std::env::current_dir()?;

    // Find current worktree
    let current_wt = worktrees
        .iter()
        .find(|wt| current_dir.starts_with(&wt.path));

    // Separate current from others
    let mut other_wts = Vec::new();
    let mut current_info = None;

    for wt in &worktrees {
        let is_current = current_wt.is_some_and(|c| c.path == wt.path);
        let info = WorktreeInfo {
            path: wt.path.display().to_string(),
            branch: wt.branch.as_ref().map(|b| {
                b.strip_prefix("refs/heads/")
                    .or_else(|| b.strip_prefix("refs/remotes/"))
                    .unwrap_or(b)
                    .to_string()
            }),
            head: wt.head.clone(),
            dirty: is_worktree_dirty(&wt.path).unwrap_or(false),
        };

        if is_current {
            current_info = Some(info);
        } else {
            other_wts.push(info);
        }
    }

    if json {
        let context = AgentContext {
            current_worktree: current_info,
            other_worktrees: other_wts,
            repository: RepositoryInfo {
                root: repo_root.display().to_string(),
                total_worktrees: worktrees.len(),
            },
        };
        println!("{}", serde_json::to_string_pretty(&context)?);
    } else {
        print_human_readable_context(current_info, other_wts, &repo_root, worktrees.len())?;
    }

    Ok(())
}

/// Print human-readable context output.
fn print_human_readable_context(
    current: Option<WorktreeInfo>,
    others: Vec<WorktreeInfo>,
    repo_root: &std::path::Path,
    total: usize,
) -> Result<()> {
    println!("## Worktree Context");
    println!();

    if let Some(current) = current {
        let branch = current.branch.as_deref().unwrap_or("<detached>");
        let status = if current.dirty { "dirty" } else { "clean" };
        println!("Current: {} @ {}", branch, current.path);
        println!("Status: {}", status);
        println!();
    } else {
        println!("Not currently in a worktree");
        println!();
    }

    if !others.is_empty() {
        println!("Other worktrees:");
        for wt in others {
            let branch = wt.branch.as_deref().unwrap_or("<detached>");
            let status = if wt.dirty { " (dirty)" } else { "" };
            println!("  - {} @ {}{}", branch, wt.path, status);
        }
        println!();
    }

    println!("Repository: {}", repo_root.display());
    println!("Total worktrees: {}", total);
    println!();

    println!("## Quick Commands");
    println!();
    println!("  wt                  # Interactive picker");
    println!("  wt list --json      # List all worktrees");
    println!("  wt add <branch>     # Create new worktree");
    println!("  wt remove <target>  # Remove worktree");
    println!("  wt prune            # Clean stale worktrees");

    Ok(())
}

/// Check if a worktree has uncommitted changes.
fn is_worktree_dirty(path: &std::path::Path) -> Result<bool> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .context("failed to check git status")?;

    Ok(!output.stdout.is_empty())
}

/// Display minimal status suitable for frequent injection.
pub fn show_status(json: bool) -> Result<()> {
    let repo_root = git::repo_root(None).context("not in a git repository")?;
    let worktrees = git::worktrees_porcelain(&repo_root)?;

    let current_dir = std::env::current_dir()?;
    let current_wt = worktrees
        .iter()
        .find(|wt| current_dir.starts_with(&wt.path));

    if json {
        #[derive(Serialize)]
        struct MinimalStatus {
            current: Option<CurrentWorktree>,
            count: usize,
        }

        #[derive(Serialize)]
        struct CurrentWorktree {
            path: String,
            branch: Option<String>,
            dirty: bool,
        }

        let current = current_wt.map(|wt| CurrentWorktree {
            path: wt.path.display().to_string(),
            branch: wt
                .branch
                .as_ref()
                .map(|b| b.strip_prefix("refs/heads/").unwrap_or(b).to_string()),
            dirty: is_worktree_dirty(&wt.path).unwrap_or(false),
        });

        let status = MinimalStatus {
            current,
            count: worktrees.len(),
        };

        println!("{}", serde_json::to_string(&status)?);
    } else {
        if let Some(wt) = current_wt {
            let branch = wt
                .branch
                .as_ref()
                .and_then(|b| b.strip_prefix("refs/heads/"))
                .unwrap_or("<detached>");
            let dirty = if is_worktree_dirty(&wt.path).unwrap_or(false) {
                " (dirty)"
            } else {
                ""
            };
            println!("{} @ {}{}", branch, wt.path.display(), dirty);
        } else {
            println!("Not in a worktree");
        }
        println!("Total: {} worktrees", worktrees.len());
    }

    Ok(())
}

/// Output onboarding instructions for AI agents.
/// Similar to `bd prime` - outputs a compact workflow reference for context injection.
pub fn show_onboard() -> Result<()> {
    print!(
        r#"# wt - Git Worktree Manager

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
[{{"path": "/path/to/wt", "head": "abc123", "branch": "refs/heads/main", "is_main": true}}]
```

### wt add <branch> --json --quiet
```json
{{"success": true, "worktree": {{"path": "/path/to/wt", "branch": "feature-x"}}}}
```

### wt remove <target> --json --force
```json
{{"success": true, "removed": true, "branch": "feature-x", "path": "/path/to/wt"}}
```

### wt agent status --json
```json
{{"current": {{"path": "/path/to/wt", "branch": "main", "dirty": true}}, "count": 3}}
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
"#
    );
    Ok(())
}
