use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::Result;
use serde::Serialize;

use crate::error::WtError;
use crate::{git, process};

/// Result of adding a worktree (for JSON output)
#[derive(Serialize)]
struct AddResult {
    success: bool,
    branch: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tracking: Option<String>,
}

/// Interactive add: show fzf picker with available branches, then create worktree.
pub fn interactive_add(
    path: Option<&str>,
    track: Option<&str>,
    json: bool,
    quiet: bool,
) -> Result<()> {
    let repo_root = git::repo_root(None)?;

    // Get available branches (local + remote, excluding ones that already have worktrees)
    let mut branches = get_available_branches(&repo_root)?;

    // Add option to create a new branch at the top
    let create_new_option = "[+] Create new branch...";
    branches.insert(0, create_new_option.to_string());

    // Run fzf to select a branch
    let selected = run_fzf_branch_picker(&branches)?;

    match selected {
        Some(branch) if branch == create_new_option => {
            // Prompt for new branch name
            eprint!("Enter new branch name: ");
            std::io::stderr().flush()?;

            let mut new_branch = String::new();
            std::io::stdin().read_line(&mut new_branch)?;
            let new_branch = new_branch.trim();

            if new_branch.is_empty() {
                eprintln!("Cancelled.");
                return Ok(());
            }

            add_worktree(new_branch, path, track, json, quiet)
        }
        Some(branch) => {
            // Strip remote prefix if present (e.g., "origin/feature" -> "feature")
            let branch_name = if let Some(stripped) = branch.strip_prefix("origin/") {
                stripped
            } else if let Some(pos) = branch.find('/') {
                // Handle other remotes like "upstream/feature"
                &branch[pos + 1..]
            } else {
                &branch
            };

            add_worktree(branch_name, path, track, json, quiet)
        }
        None => {
            // User cancelled
            Ok(())
        }
    }
}

/// Add a new worktree for the given branch.
/// - branch: the branch name to create a worktree for
/// - path: optional custom path (defaults to sibling directory named after branch)
/// - track: optional remote to track (e.g., "origin")
/// - json: output result as JSON
/// - quiet: suppress non-essential output
pub fn add_worktree(
    branch: &str,
    path: Option<&str>,
    track: Option<&str>,
    json: bool,
    quiet: bool,
) -> Result<()> {
    // Get the current repository root
    let repo_root = git::repo_root(None)?;

    // Determine the target path
    let target_path = if let Some(custom_path) = path {
        PathBuf::from(custom_path)
    } else {
        calculate_default_path(&repo_root, branch)?
    };

    // Check if the path already exists
    if target_path.exists() {
        return Err(WtError::user_error(format!(
            "path already exists: {}\nChoose a different path with --path",
            target_path.display()
        ))
        .into());
    }

    // Check if a worktree for this branch already exists
    check_existing_worktree(&repo_root, branch)?;

    // Display what we're doing (unless quiet or json)
    if !quiet && !json {
        eprintln!("Creating worktree at: {}", target_path.display());
    }

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
                target_path
                    .to_str()
                    .ok_or_else(|| WtError::io_error("invalid path encoding"))?,
                &remote_branch,
            ],
            Some(&repo_root),
        )
        .map_err(|e| {
            WtError::git_error_with_source(
                format!("failed to add worktree tracking {}", remote_branch),
                e,
            )
        })?;
    } else if branch_exists(&repo_root, branch)? {
        // Branch exists, just add worktree for it
        process::run(
            "git",
            &[
                "worktree",
                "add",
                target_path
                    .to_str()
                    .ok_or_else(|| WtError::io_error("invalid path encoding"))?,
                branch,
            ],
            Some(&repo_root),
        )
        .map_err(|e| WtError::git_error_with_source("failed to add worktree", e))?;
    } else {
        // Branch doesn't exist, create it with -b
        process::run(
            "git",
            &[
                "worktree",
                "add",
                "-b",
                branch,
                target_path
                    .to_str()
                    .ok_or_else(|| WtError::io_error("invalid path encoding"))?,
            ],
            Some(&repo_root),
        )
        .map_err(|e| {
            WtError::git_error_with_source(
                format!("failed to create worktree with new branch '{}'", branch),
                e,
            )
        })?;
    }

    if json {
        let result = AddResult {
            success: true,
            branch: branch.to_string(),
            path: target_path.to_string_lossy().to_string(),
            tracking: track.map(|r| format!("{}/{}", r, branch)),
        };
        println!("{}", serde_json::to_string(&result)?);
    } else if !quiet {
        eprintln!("Worktree created successfully");
    }

    Ok(())
}

/// Calculate the default path for a worktree based on the branch name.
/// Pattern: <repo_root_parent>/<repo_name>-<branch_sanitized>
fn calculate_default_path(repo_root: &Path, branch: &str) -> Result<PathBuf> {
    // Get the parent directory of the repo root
    let repo_parent = repo_root
        .parent()
        .ok_or_else(|| WtError::io_error("repository root has no parent directory"))?;

    // Get the repository name (last component of repo root)
    let repo_name = repo_root
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| WtError::io_error("failed to extract repository name"))?;

    // Sanitize the branch name: replace / with -
    let sanitized_branch = branch.replace('/', "-");

    // Construct the path: <parent>/<repo_name>-<branch_sanitized>
    let worktree_dir_name = format!("{}-{}", repo_name, sanitized_branch);
    Ok(repo_parent.join(worktree_dir_name))
}

/// Check if a branch exists (local or remote).
fn branch_exists(repo_root: &Path, branch: &str) -> Result<bool> {
    // Check local branches
    let local_ref = format!("refs/heads/{}", branch);
    let result = std::process::Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &local_ref])
        .current_dir(repo_root)
        .status()
        .map_err(|e| WtError::git_error_with_source("failed to run git show-ref", e.into()))?;

    if result.success() {
        return Ok(true);
    }

    // Check remote branches (any remote)
    let output = std::process::Command::new("git")
        .args(["branch", "-r", "--list", &format!("*/{}", branch)])
        .current_dir(repo_root)
        .output()
        .map_err(|e| WtError::git_error_with_source("failed to run git branch -r", e.into()))?;

    let remote_branches = String::from_utf8_lossy(&output.stdout);
    Ok(!remote_branches.trim().is_empty())
}

/// Check if a worktree for the given branch already exists.
fn check_existing_worktree(repo_root: &Path, branch: &str) -> Result<()> {
    let worktrees = git::worktrees_porcelain(repo_root)
        .map_err(|e| WtError::git_error_with_source("failed to list existing worktrees", e))?;

    for wt in worktrees {
        // Branch is stored as refs/heads/<branch> or refs/remotes/<remote>/<branch>
        let branch_ref = format!("refs/heads/{}", branch);
        if wt.branch.as_deref() == Some(&branch_ref) {
            return Err(WtError::user_error(format!(
                "worktree for branch '{}' already exists at: {}",
                branch,
                wt.path.display()
            ))
            .into());
        }
    }

    Ok(())
}

/// Get available branches for creating new worktrees.
/// Returns local and remote branches that don't already have worktrees.
fn get_available_branches(repo_root: &Path) -> Result<Vec<String>> {
    // Get existing worktree branches to exclude them
    let worktrees = git::worktrees_porcelain(repo_root)
        .map_err(|e| WtError::git_error_with_source("failed to list existing worktrees", e))?;
    let existing_branches: std::collections::HashSet<String> = worktrees
        .iter()
        .filter_map(|wt| {
            wt.branch.as_ref().and_then(|b| {
                b.strip_prefix("refs/heads/")
                    .or_else(|| b.strip_prefix("refs/remotes/"))
                    .map(|s| s.to_string())
            })
        })
        .collect();

    let mut branches = Vec::new();

    // Get local branches
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| WtError::git_error_with_source("failed to list local branches", e.into()))?;

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let branch = line.trim();
        if !branch.is_empty() && !existing_branches.contains(branch) {
            branches.push(branch.to_string());
        }
    }

    // Get remote branches
    let output = Command::new("git")
        .args(["branch", "-r", "--format=%(refname:short)"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| WtError::git_error_with_source("failed to list remote branches", e.into()))?;

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let branch = line.trim();
        // Skip HEAD pointers and already existing worktrees
        if !branch.is_empty() && !branch.contains("HEAD") {
            // Extract just the branch name part for comparison
            let branch_name = branch.split('/').skip(1).collect::<Vec<_>>().join("/");
            if !existing_branches.contains(&branch_name) && !existing_branches.contains(branch) {
                branches.push(branch.to_string());
            }
        }
    }

    // Sort and deduplicate
    branches.sort();
    branches.dedup();

    Ok(branches)
}

/// Run fzf to let user pick a branch.
fn run_fzf_branch_picker(branches: &[String]) -> Result<Option<String>> {
    let mut child = Command::new("fzf")
        .args([
            "--height=40%",
            "--layout=reverse",
            "--prompt=Branch> ",
            "--header=Select branch to create worktree for (Esc to cancel)",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            WtError::user_error(format!("failed to spawn fzf (is it installed?): {}", e))
        })?;

    // Write branches to stdin
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| WtError::io_error("failed to open fzf stdin"))?;

        for branch in branches {
            writeln!(stdin, "{}", branch).map_err(|e| {
                WtError::io_error_with_source("failed to write to fzf stdin", e.into())
            })?;
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| WtError::io_error_with_source("failed to wait for fzf", e.into()))?;

    match output.status.code() {
        Some(0) => {
            let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if selection.is_empty() {
                Ok(None)
            } else {
                Ok(Some(selection))
            }
        }
        Some(1) | Some(130) => Ok(None), // No match or cancelled
        Some(code) => Err(WtError::user_error(format!("fzf exited with code: {}", code)).into()),
        None => Err(WtError::user_error("fzf terminated by signal").into()),
    }
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
