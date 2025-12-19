use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::error::WtError;
use crate::process;
use crate::worktree::{self, Worktree};

pub fn repo_root(cwd: Option<&Path>) -> Result<PathBuf> {
    let out = process::run_stdout("git", &["rev-parse", "--show-toplevel"], cwd)
        .map_err(|_| anyhow::Error::new(WtError::not_found("not in a git repository")))?;
    Ok(PathBuf::from(out.trim()))
}

pub fn worktrees_porcelain(repo_root: &Path) -> Result<Vec<Worktree>> {
    let out = process::run_stdout("git", &["worktree", "list", "--porcelain"], Some(repo_root))
        .map_err(|e| {
            anyhow::Error::new(WtError::git_error_with_source(
                "failed to list worktrees",
                e,
            ))
        })?;
    worktree::parse_porcelain(&out)
}

/// Detect the main branch for a repository.
///
/// Tries in order:
/// 1. `git symbolic-ref refs/remotes/origin/HEAD` (remote default)
/// 2. Check if `main` branch exists
/// 3. Check if `master` branch exists
///
/// Returns the branch name (e.g., "main") without the refs/heads/ prefix.
pub fn main_branch(repo_root: &Path) -> Option<String> {
    // Try to get the remote default branch
    if let Ok(output) = process::run_stdout(
        "git",
        &["symbolic-ref", "refs/remotes/origin/HEAD"],
        Some(repo_root),
    ) {
        // Output is like "refs/remotes/origin/main"
        let trimmed = output.trim();
        if let Some(branch) = trimmed.strip_prefix("refs/remotes/origin/") {
            return Some(branch.to_string());
        }
    }

    // Fallback: check if common default branches exist
    for candidate in &["main", "master"] {
        let ref_path = format!("refs/heads/{}", candidate);
        if process::run(
            "git",
            &["show-ref", "--verify", "--quiet", &ref_path],
            Some(repo_root),
        )
        .is_ok()
        {
            return Some((*candidate).to_string());
        }
    }

    None
}

/// Check if a branch reference matches the main branch.
///
/// `branch_ref` should be in the form "refs/heads/main" or just "main".
pub fn is_main_branch(repo_root: &Path, branch_ref: &str) -> bool {
    let Some(main) = main_branch(repo_root) else {
        return false;
    };

    // Handle both "refs/heads/main" and "main" formats
    let branch_name = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);

    branch_name == main
}
