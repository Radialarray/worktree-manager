use serde::Serialize;

use crate::error::WtError;
use crate::git;
use crate::process;

/// Result of pruning worktrees (for JSON output)
#[derive(Serialize)]
struct PruneResult {
    success: bool,
    pruned: Vec<PrunedWorktree>,
}

/// A single pruned worktree entry
#[derive(Serialize)]
struct PrunedWorktree {
    path: String,
    reason: String,
}

/// Prune stale worktrees.
/// First lists any prunable worktrees, then runs git worktree prune.
/// - json: output result as JSON
/// - quiet: suppress non-essential output
pub fn prune_worktrees(json: bool, quiet: bool) -> Result<(), WtError> {
    let repo_root = git::repo_root(None)?;
    let worktrees = git::worktrees_porcelain(&repo_root)
        .map_err(|e| WtError::git_error_with_source("failed to list worktrees", e))?;

    // Filter for stale (prunable) worktrees
    let stale_worktrees: Vec<_> = worktrees
        .iter()
        .filter(|wt| wt.prunable.is_some())
        .collect();

    // Handle case with no stale worktrees
    if stale_worktrees.is_empty() {
        if json {
            let result = PruneResult {
                success: true,
                pruned: vec![],
            };
            println!(
                "{}",
                serde_json::to_string(&result).map_err(|e| WtError::io_error_with_source(
                    "failed to serialize JSON",
                    e.into()
                ))?
            );
        } else if !quiet {
            eprintln!("No stale worktrees found.");
        }
        return Ok(());
    }

    // Print stale worktrees if not quiet and not json
    if !quiet && !json {
        eprintln!("Stale worktrees to prune:");
        for wt in &stale_worktrees {
            let reason = wt.prunable.as_ref().unwrap();
            eprintln!("  - {} ({})", wt.path.display(), reason);
        }
    }

    // Collect info for JSON output before pruning
    let pruned_info: Vec<PrunedWorktree> = stale_worktrees
        .iter()
        .map(|wt| PrunedWorktree {
            path: wt.path.display().to_string(),
            reason: wt.prunable.clone().unwrap_or_default(),
        })
        .collect();

    // Run git worktree prune
    process::run("git", &["worktree", "prune"], Some(&repo_root))
        .map_err(|e| WtError::git_error_with_source("failed to prune worktrees", e))?;

    if json {
        let result = PruneResult {
            success: true,
            pruned: pruned_info,
        };
        println!(
            "{}",
            serde_json::to_string(&result)
                .map_err(|e| WtError::io_error_with_source("failed to serialize JSON", e.into()))?
        );
    } else if !quiet {
        eprintln!("Pruned stale worktrees.");
    }

    Ok(())
}
