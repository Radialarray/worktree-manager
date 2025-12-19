use serde::Serialize;

use crate::error::WtError;
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
pub fn show_context(json: bool) -> Result<(), WtError> {
    let repo_root = git::repo_root(None)?;
    let worktrees = git::worktrees_porcelain(&repo_root)
        .map_err(|e| WtError::git_error_with_source("failed to list worktrees", e))?;

    // Get current directory to determine which worktree we're in
    let current_dir = std::env::current_dir()
        .map_err(|e| WtError::io_error_with_source("failed to get current directory", e.into()))?;

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
        let json_str = serde_json::to_string_pretty(&context)
            .map_err(|e| WtError::io_error_with_source("failed to serialize JSON", e.into()))?;
        println!("{}", json_str);
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
) -> Result<(), WtError> {
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
fn is_worktree_dirty(path: &std::path::Path) -> Result<bool, WtError> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .map_err(|e| WtError::git_error_with_source("failed to check git status", e.into()))?;

    Ok(!output.stdout.is_empty())
}

/// Display minimal status suitable for frequent injection.
pub fn show_status(json: bool) -> Result<(), WtError> {
    let repo_root = git::repo_root(None)?;
    let worktrees = git::worktrees_porcelain(&repo_root)
        .map_err(|e| WtError::git_error_with_source("failed to list worktrees", e))?;

    let current_dir = std::env::current_dir()
        .map_err(|e| WtError::io_error_with_source("failed to get current directory", e.into()))?;
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

        println!(
            "{}",
            serde_json::to_string(&status)
                .map_err(|e| WtError::io_error_with_source("failed to serialize JSON", e.into()))?
        );
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
pub fn show_onboard() -> Result<(), WtError> {
    print!("{}", include_str!("agent_onboard.md"));
    Ok(())
}
