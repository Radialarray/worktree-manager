use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::git;
use crate::process;
use crate::worktree::Worktree;

/// Remove a worktree identified by branch name or path.
/// - target: branch name or path to the worktree
/// - force: if true, skip confirmation and force remove
pub fn remove_worktree(target: &str, force: bool) -> Result<()> {
    // Get repo root and list worktrees
    let repo_root = git::repo_root(None).context("not in a git repository")?;
    let worktrees = git::worktrees_porcelain(&repo_root)?;

    // Find matching worktree
    let matching_worktree = find_worktree(&worktrees, target)?;

    // Prevent removal of main/bare worktree
    if matching_worktree.bare {
        bail!("cannot remove the main worktree (bare repository location)");
    }

    // Check for locked worktrees
    if matching_worktree.locked {
        bail!(
            "worktree '{}' is locked; use `git worktree unlock` first or `git worktree remove --force`",
            matching_worktree.path.display()
        );
    }

    // Confirmation prompt (unless force)
    if !force {
        let branch_display = matching_worktree
            .branch
            .as_ref()
            .and_then(|b| b.strip_prefix("refs/heads/"))
            .unwrap_or("<detached>");

        eprint!(
            "Remove worktree '{}' at {}? (y/N): ",
            branch_display,
            matching_worktree.path.display()
        );
        io::stderr().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        let response = response.trim();
        if response != "y" && response != "Y" {
            eprintln!("Cancelled.");
            return Ok(());
        }
    }

    // Attempt to remove the worktree
    let path_str = matching_worktree.path.to_string_lossy();
    let result = process::run(
        "git",
        &["worktree", "remove", path_str.as_ref()],
        Some(&repo_root),
    );

    match result {
        Ok(_) => {
            eprintln!("Worktree removed.");
            Ok(())
        }
        Err(e) => {
            // Check if the error is due to uncommitted changes
            let error_msg = format!("{:#}", e);
            if error_msg.contains("uncommitted changes")
                || error_msg.contains("modified files")
                || error_msg.contains("changes would be lost")
            {
                bail!(
                    "worktree has uncommitted changes; use --force to remove anyway\nOriginal error: {}",
                    error_msg
                );
            }

            // Re-throw the original error
            Err(e)
        }
    }
}

/// Find a worktree by target (path or branch name).
/// Returns error if no match or multiple matches found.
fn find_worktree<'a>(worktrees: &'a [Worktree], target: &str) -> Result<&'a Worktree> {
    let target_path = Path::new(target);
    let mut matches = Vec::new();

    for wt in worktrees {
        // Try exact path match
        if wt.path == target_path {
            matches.push(wt);
            continue;
        }

        // Try branch name match
        if let Some(branch) = &wt.branch {
            let branch_name = branch
                .strip_prefix("refs/heads/")
                .or_else(|| branch.strip_prefix("refs/remotes/"))
                .unwrap_or(branch);

            if branch_name == target {
                matches.push(wt);
            }
        }
    }

    match matches.len() {
        0 => bail!("no worktree found matching '{}'", target),
        1 => Ok(matches[0]),
        _ => {
            let paths: Vec<_> = matches
                .iter()
                .map(|wt| wt.path.display().to_string())
                .collect();
            bail!(
                "target '{}' matches multiple worktrees:\n  {}",
                target,
                paths.join("\n  ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_worktree(path: &str, branch: Option<&str>) -> Worktree {
        Worktree {
            path: PathBuf::from(path),
            head: Some("abc123".to_string()),
            branch: branch.map(|b| format!("refs/heads/{}", b)),
            locked: false,
            prunable: None,
            bare: false,
        }
    }

    #[test]
    fn find_by_exact_path() {
        let worktrees = vec![
            make_worktree("/tmp/repo", Some("main")),
            make_worktree("/tmp/repo-feature", Some("feature")),
        ];

        let found = find_worktree(&worktrees, "/tmp/repo-feature").unwrap();
        assert_eq!(found.path, PathBuf::from("/tmp/repo-feature"));
    }

    #[test]
    fn find_by_branch_name() {
        let worktrees = vec![
            make_worktree("/tmp/repo", Some("main")),
            make_worktree("/tmp/repo-feature", Some("feature")),
        ];

        let found = find_worktree(&worktrees, "feature").unwrap();
        assert_eq!(found.path, PathBuf::from("/tmp/repo-feature"));
    }

    #[test]
    fn error_on_no_match() {
        let worktrees = vec![make_worktree("/tmp/repo", Some("main"))];

        let result = find_worktree(&worktrees, "nonexistent");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("no worktree found")
        );
    }

    #[test]
    fn error_on_multiple_matches() {
        // Create two worktrees with same branch name (edge case, but possible)
        let worktrees = vec![
            Worktree {
                path: PathBuf::from("/tmp/repo1"),
                head: Some("abc123".to_string()),
                branch: Some("refs/heads/feature".to_string()),
                locked: false,
                prunable: None,
                bare: false,
            },
            Worktree {
                path: PathBuf::from("/tmp/repo2"),
                head: Some("def456".to_string()),
                branch: Some("refs/heads/feature".to_string()),
                locked: false,
                prunable: None,
                bare: false,
            },
        ];

        let result = find_worktree(&worktrees, "feature");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("multiple worktrees")
        );
    }

    #[test]
    fn strips_refs_heads_prefix() {
        let worktrees = vec![make_worktree("/tmp/repo", Some("main"))];

        // Should find it by branch name without prefix
        let found = find_worktree(&worktrees, "main").unwrap();
        assert_eq!(found.path, PathBuf::from("/tmp/repo"));
    }
}
