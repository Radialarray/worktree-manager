use anyhow::Result;

use crate::git;
use crate::process;

/// Prune stale worktrees.
/// First lists any prunable worktrees, then runs git worktree prune.
pub fn prune_worktrees() -> Result<()> {
    let repo_root = git::repo_root(None)?;
    let worktrees = git::worktrees_porcelain(&repo_root)?;

    // Filter for stale (prunable) worktrees
    let stale_worktrees: Vec<_> = worktrees
        .iter()
        .filter(|wt| wt.prunable.is_some())
        .collect();

    // Print stale worktrees if any exist
    if !stale_worktrees.is_empty() {
        eprintln!("Stale worktrees to prune:");
        for wt in &stale_worktrees {
            let reason = wt.prunable.as_ref().unwrap();
            eprintln!("  - {} ({})", wt.path.display(), reason);
        }
    } else {
        eprintln!("No stale worktrees found.");
        return Ok(());
    }

    // Run git worktree prune
    process::run("git", &["worktree", "prune"], Some(&repo_root))?;

    eprintln!("Pruned stale worktrees.");
    Ok(())
}
