use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::{git, process};

/// Add a new worktree for the given branch.
/// - branch: the branch name to create a worktree for
/// - path: optional custom path (defaults to sibling directory named after branch)
/// - track: optional remote to track (e.g., "origin")
pub fn add_worktree(branch: &str, path: Option<&str>, track: Option<&str>) -> Result<()> {
    // Get the current repository root
    let repo_root = git::repo_root(None).context("failed to determine repository root")?;

    // Determine the target path
    let target_path = if let Some(custom_path) = path {
        PathBuf::from(custom_path)
    } else {
        calculate_default_path(&repo_root, branch)?
    };

    // Check if the path already exists
    if target_path.exists() {
        bail!(
            "path already exists: {}\nChoose a different path with --path",
            target_path.display()
        );
    }

    // Check if a worktree for this branch already exists
    check_existing_worktree(&repo_root, branch)?;

    // Display what we're doing
    eprintln!("Creating worktree at: {}", target_path.display());

    // Execute the git worktree add command
    if let Some(remote) = track {
        // Create a new branch tracking the remote
        let remote_branch = format!("{}/{}", remote, branch);
        process::run(
            "git",
            &[
                "worktree",
                "add",
                "--track",
                "-b",
                branch,
                target_path.to_str().context("invalid path encoding")?,
                &remote_branch,
            ],
            Some(&repo_root),
        )
        .with_context(|| format!("failed to add worktree tracking {}", remote_branch))?;
    } else {
        // Add worktree for existing branch or create new branch
        process::run(
            "git",
            &[
                "worktree",
                "add",
                target_path.to_str().context("invalid path encoding")?,
                branch,
            ],
            Some(&repo_root),
        )
        .context("failed to add worktree")?;
    }

    eprintln!("Worktree created successfully");

    Ok(())
}

/// Calculate the default path for a worktree based on the branch name.
/// Pattern: <repo_root_parent>/<repo_name>-<branch_sanitized>
fn calculate_default_path(repo_root: &Path, branch: &str) -> Result<PathBuf> {
    // Get the parent directory of the repo root
    let repo_parent = repo_root
        .parent()
        .context("repository root has no parent directory")?;

    // Get the repository name (last component of repo root)
    let repo_name = repo_root
        .file_name()
        .and_then(|n| n.to_str())
        .context("failed to extract repository name")?;

    // Sanitize the branch name: replace / with -
    let sanitized_branch = branch.replace('/', "-");

    // Construct the path: <parent>/<repo_name>-<branch_sanitized>
    let worktree_dir_name = format!("{}-{}", repo_name, sanitized_branch);
    Ok(repo_parent.join(worktree_dir_name))
}

/// Check if a worktree for the given branch already exists.
fn check_existing_worktree(repo_root: &Path, branch: &str) -> Result<()> {
    let worktrees =
        git::worktrees_porcelain(repo_root).context("failed to list existing worktrees")?;

    for wt in worktrees {
        // Branch is stored as refs/heads/<branch> or refs/remotes/<remote>/<branch>
        let branch_ref = format!("refs/heads/{}", branch);
        if wt.branch.as_deref() == Some(&branch_ref) {
            bail!(
                "worktree for branch '{}' already exists at: {}",
                branch,
                wt.path.display()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_default_path() {
        let repo_root = PathBuf::from("/home/user/repos/my-project");
        let branch = "feature/new-ui";

        let result = calculate_default_path(&repo_root, branch).unwrap();
        let expected = PathBuf::from("/home/user/repos/my-project-feature-new-ui");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_sanitize_branch_name() {
        let repo_root = PathBuf::from("/home/user/repos/project");
        let branch = "bugfix/issue-123/part-2";

        let result = calculate_default_path(&repo_root, branch).unwrap();
        let expected = PathBuf::from("/home/user/repos/project-bugfix-issue-123-part-2");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_simple_branch_name() {
        let repo_root = PathBuf::from("/repos/app");
        let branch = "main";

        let result = calculate_default_path(&repo_root, branch).unwrap();
        let expected = PathBuf::from("/repos/app-main");

        assert_eq!(result, expected);
    }
}
