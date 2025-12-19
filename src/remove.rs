use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::Result;
use serde::Serialize;

use crate::error::WtError;
use crate::git;
use crate::process;
use crate::worktree::Worktree;

/// Result of removing a worktree (for JSON output)
#[derive(Serialize)]
struct RemoveResult {
    success: bool,
    removed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

/// Remove a worktree identified by branch name or path.
/// - target: branch name or path to the worktree
/// - force: if true, skip confirmation and force remove
/// - json: output result as JSON
/// - quiet: suppress interactive prompts (without force, will not remove)
pub fn remove_worktree(target: &str, force: bool, json: bool, quiet: bool) -> Result<()> {
    // Get repo root and list worktrees
    let repo_root = git::repo_root(None)?;
    let worktrees = git::worktrees_porcelain(&repo_root)?;

    // Find matching worktree
    let matching_worktree = find_worktree(&worktrees, target)?;

    let branch_display = matching_worktree
        .branch
        .as_ref()
        .and_then(|b| b.strip_prefix("refs/heads/"))
        .unwrap_or("<detached>")
        .to_string();
    let path_display = matching_worktree.path.display().to_string();

    // Prevent removal of main/bare worktree
    if matching_worktree.bare {
        if json {
            let result = RemoveResult {
                success: false,
                removed: false,
                branch: Some(branch_display),
                path: Some(path_display),
                reason: Some("cannot remove the main worktree (bare repository location)".into()),
            };
            println!("{}", serde_json::to_string(&result)?);
            return Ok(());
        }
        return Err(WtError::user_error(
            "cannot remove the main worktree (bare repository location)",
        )
        .into());
    }

    // Prevent removal of the main branch worktree
    if let Some(branch) = &matching_worktree.branch
        && git::is_main_branch(&repo_root, branch)
    {
        if json {
            let result = RemoveResult {
                success: false,
                removed: false,
                branch: Some(branch_display),
                path: Some(path_display),
                reason: Some("cannot remove the main branch worktree".into()),
            };
            println!("{}", serde_json::to_string(&result)?);
            return Ok(());
        }
        return Err(WtError::user_error(format!(
            "cannot remove the main branch worktree (branch '{}')",
            branch_display
        ))
        .into());
    }

    // Check for locked worktrees
    if matching_worktree.locked {
        if json {
            let result = RemoveResult {
                success: false,
                removed: false,
                branch: Some(branch_display),
                path: Some(path_display),
                reason: Some("worktree is locked".into()),
            };
            println!("{}", serde_json::to_string(&result)?);
            return Ok(());
        }
        return Err(WtError::user_error(format!(
            "worktree '{}' is locked; use `git worktree unlock` first or `git worktree remove --force`",
            matching_worktree.path.display()
        )).into());
    }

    // Confirmation prompt (unless force or quiet)
    if !force {
        if quiet {
            // In quiet mode without force, don't remove (non-interactive)
            if json {
                let result = RemoveResult {
                    success: true,
                    removed: false,
                    branch: Some(branch_display),
                    path: Some(path_display),
                    reason: Some("skipped: --quiet without --force".into()),
                };
                println!("{}", serde_json::to_string(&result)?);
            }
            return Ok(());
        }

        eprint!(
            "Remove worktree '{}' at {}? (y/N): ",
            branch_display, path_display
        );
        io::stderr().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        let response = response.trim();
        if response != "y" && response != "Y" {
            if json {
                let result = RemoveResult {
                    success: true,
                    removed: false,
                    branch: Some(branch_display),
                    path: Some(path_display),
                    reason: Some("cancelled by user".into()),
                };
                println!("{}", serde_json::to_string(&result)?);
            } else {
                eprintln!("Cancelled.");
            }
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
            if json {
                let result = RemoveResult {
                    success: true,
                    removed: true,
                    branch: Some(branch_display),
                    path: Some(path_display),
                    reason: None,
                };
                println!("{}", serde_json::to_string(&result)?);
            } else if !quiet {
                eprintln!("Worktree removed.");
            }
            Ok(())
        }
        Err(e) => {
            // Check if the error is due to uncommitted changes
            let error_msg = format!("{:#}", e);
            if error_msg.contains("uncommitted changes")
                || error_msg.contains("modified files")
                || error_msg.contains("changes would be lost")
            {
                if json {
                    let result = RemoveResult {
                        success: false,
                        removed: false,
                        branch: Some(branch_display),
                        path: Some(path_display),
                        reason: Some("worktree has uncommitted changes".into()),
                    };
                    println!("{}", serde_json::to_string(&result)?);
                    return Ok(());
                }
                return Err(WtError::user_error(format!(
                    "worktree has uncommitted changes; use --force to remove anyway\nOriginal error: {}",
                    error_msg
                )).into());
            }

            // Re-throw the original error as GitError
            Err(WtError::git_error_with_source("failed to remove worktree", e).into())
        }
    }
}

/// Interactive remove: show fzf picker with existing worktrees, then remove selected one.
pub fn interactive_remove(force: bool, json: bool, quiet: bool) -> Result<()> {
    let repo_root = git::repo_root(None)?;
    let worktrees = git::worktrees_porcelain(&repo_root)?;

    // Filter out the main/bare worktree and main branch worktree - can't remove those
    let removable: Vec<_> = worktrees
        .iter()
        .filter(|wt| {
            // Can't remove bare worktree
            if wt.bare {
                return false;
            }
            // Can't remove main branch worktree
            if let Some(branch) = &wt.branch
                && git::is_main_branch(&repo_root, branch)
            {
                return false;
            }
            true
        })
        .collect();

    if removable.is_empty() {
        return Err(WtError::not_found(
            "no removable worktrees found (only the main worktree exists)",
        )
        .into());
    }

    // Prepare candidates for fzf display
    let candidates = prepare_worktree_candidates(&removable);

    // Run fzf to select a worktree
    let selected = run_fzf_worktree_picker(&candidates)?;

    match selected {
        Some(line) => {
            // Extract the branch name from the selected line (first column)
            let branch = line.split("  ").next().unwrap_or(&line).trim();
            remove_worktree(branch, force, json, quiet)
        }
        None => {
            // User cancelled
            Ok(())
        }
    }
}

/// Prepare worktree candidates for fzf display (branch + path).
fn prepare_worktree_candidates(worktrees: &[&Worktree]) -> Vec<String> {
    let max_branch_len = worktrees
        .iter()
        .map(|wt| format_branch_name(wt).len())
        .max()
        .unwrap_or(0);

    worktrees
        .iter()
        .map(|wt| {
            let branch = format_branch_name(wt);
            let path = wt.path.display();
            let locked = if wt.locked { " [locked]" } else { "" };
            format!(
                "{:width$}  {}{}",
                branch,
                path,
                locked,
                width = max_branch_len
            )
        })
        .collect()
}

/// Format branch name for display.
fn format_branch_name(wt: &Worktree) -> String {
    match &wt.branch {
        Some(branch_ref) => branch_ref
            .strip_prefix("refs/heads/")
            .or_else(|| branch_ref.strip_prefix("refs/remotes/"))
            .unwrap_or(branch_ref)
            .to_string(),
        None => "(detached)".to_string(),
    }
}

/// Run fzf to let user pick a worktree to remove.
fn run_fzf_worktree_picker(candidates: &[String]) -> Result<Option<String>> {
    let mut child = Command::new("fzf")
        .args([
            "--height=40%",
            "--layout=reverse",
            "--prompt=Remove> ",
            "--header=Select worktree to remove (Esc to cancel)",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            WtError::user_error_with_source("failed to spawn fzf (is it installed?)", e)
        })?;

    // Write candidates to stdin
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| WtError::io_error("failed to open fzf stdin"))?;

        for candidate in candidates {
            writeln!(stdin, "{}", candidate).map_err(|e| {
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
        0 => Err(WtError::not_found(format!("no worktree found matching '{}'", target)).into()),
        1 => Ok(matches[0]),
        _ => {
            let paths: Vec<_> = matches
                .iter()
                .map(|wt| wt.path.display().to_string())
                .collect();
            Err(WtError::user_error(format!(
                "target '{}' matches multiple worktrees:\n  {}",
                target,
                paths.join("\n  ")
            ))
            .into())
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
