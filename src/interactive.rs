use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::Result;

use crate::error::WtError;
use crate::{config, git};

/// Run the interactive worktree picker.
/// Outputs action in format "cd|PATH" or "edit|PATH" for shell wrapper to parse.
///
/// # Arguments
///
/// * `all` - If true, show worktrees from all discovered repositories
pub fn run_interactive(all: bool) -> Result<()> {
    // Load config for fzf settings
    let config = config::load()
        .map_err(|e| WtError::config_error_with_source("failed to load config", e))?;

    if all {
        run_interactive_all(&config)
    } else {
        run_interactive_single(&config)
    }
}

/// Run interactive picker for a single repository (current directory).
fn run_interactive_single(config: &crate::config::Config) -> Result<()> {
    // Get repository root and worktrees
    let repo_root = git::repo_root(None)?;
    let worktrees = git::worktrees_porcelain(&repo_root)?;

    if worktrees.is_empty() {
        return Err(WtError::not_found("no worktrees found in repository").into());
    }

    // Prepare candidates for fzf
    // Format: "<branch>  <path>" with aligned columns
    let candidates = prepare_candidates(&worktrees);

    // Run fzf with --expect to capture which key was pressed
    let selection = run_fzf_with_expect(&candidates, &config.fzf, false)?;

    // Handle the selection
    match selection {
        Some((key, line)) => {
            // Extract path from the selected line (second column)
            let path = extract_path(&line)?;

            // Output action based on which key was pressed
            if key == "ctrl-e" {
                println!("edit|{}", path);
            } else {
                // Enter key or empty means cd action
                println!("cd|{}", path);
            }
            Ok(())
        }
        None => {
            // User cancelled - exit cleanly without output
            Ok(())
        }
    }
}

/// Run interactive picker across all discovered repositories.
fn run_interactive_all(config: &crate::config::Config) -> Result<()> {
    // Check that discovery paths are configured
    if config.auto_discovery.paths.is_empty() {
        return Err(WtError::user_error(
            "No auto-discovery paths configured. Run: wt config set-discovery-paths <paths...>",
        )
        .into());
    }

    // Discover all repos
    let repos = crate::discovery::discover_repos(&config.auto_discovery.paths)?;
    if repos.is_empty() {
        return Err(
            WtError::not_found("No git repositories found in configured discovery paths.").into(),
        );
    }

    // Collect worktrees from all repos
    let candidates = prepare_all_candidates(&repos)?;

    if candidates.is_empty() {
        return Err(WtError::not_found("No worktrees found in any discovered repository").into());
    }

    // Run fzf with --expect to capture which key was pressed
    let selection = run_fzf_with_expect(&candidates, &config.fzf, true)?;

    // Handle the selection
    match selection {
        Some((key, line)) => {
            // Extract path from the selected line (third column for --all mode)
            let path = extract_path_from_all(&line)?;

            // Output action based on which key was pressed
            if key == "ctrl-e" {
                println!("edit|{}", path);
            } else {
                // Enter key or empty means cd action
                println!("cd|{}", path);
            }
            Ok(())
        }
        None => {
            // User cancelled - exit cleanly without output
            Ok(())
        }
    }
}

/// Prepare candidate lines for fzf display.
/// Format: "<branch>  <path>" with aligned columns.
fn prepare_candidates(worktrees: &[crate::worktree::Worktree]) -> Vec<String> {
    // First pass: find the maximum branch name length for alignment
    let max_branch_len = worktrees
        .iter()
        .map(|wt| format_branch_name(wt).len())
        .max()
        .unwrap_or(0);

    // Second pass: format each worktree with aligned columns
    worktrees
        .iter()
        .map(|wt| {
            let branch = format_branch_name(wt);
            let path = wt.path.display();
            // Use two spaces as separator between columns
            format!("{:width$}  {}", branch, path, width = max_branch_len)
        })
        .collect()
}

/// Format the branch name for display, stripping common prefixes.
fn format_branch_name(wt: &crate::worktree::Worktree) -> String {
    match &wt.branch {
        Some(branch_ref) => {
            // Strip refs/heads/ or refs/remotes/ prefix
            if let Some(name) = branch_ref.strip_prefix("refs/heads/") {
                name.to_string()
            } else if let Some(name) = branch_ref.strip_prefix("refs/remotes/") {
                name.to_string()
            } else {
                branch_ref.clone()
            }
        }
        None => "(detached)".to_string(),
    }
}

/// Extract the path from a formatted candidate line (second column).
fn extract_path(line: &str) -> Result<String> {
    // Split on two spaces (the separator we used)
    let parts: Vec<&str> = line.split("  ").collect();

    if parts.len() >= 2 {
        Ok(parts[1].to_string())
    } else {
        Err(WtError::user_error(format!("failed to extract path from fzf output: {}", line)).into())
    }
}

/// Prepare candidates for cross-repo display (3 columns: repo, branch, path).
fn prepare_all_candidates(repos: &[std::path::PathBuf]) -> Result<Vec<String>> {
    let mut all_worktrees: Vec<(String, crate::worktree::Worktree)> = Vec::new();

    // Collect all worktrees from all repos
    for repo_root in repos {
        let repo_name = repo_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("(unknown)")
            .to_string();

        match git::worktrees_porcelain(repo_root) {
            Ok(worktrees) => {
                for wt in worktrees {
                    all_worktrees.push((repo_name.clone(), wt));
                }
            }
            Err(e) => {
                eprintln!("Warning: failed to list worktrees for {}: {}", repo_name, e);
            }
        }
    }

    // Find max widths for alignment
    let max_repo_len = all_worktrees
        .iter()
        .map(|(repo, _)| repo.len())
        .max()
        .unwrap_or(0);

    let max_branch_len = all_worktrees
        .iter()
        .map(|(_, wt)| format_branch_name(wt).len())
        .max()
        .unwrap_or(0);

    // Format each worktree with aligned columns: <repo>  <branch>  <path>
    let candidates: Vec<String> = all_worktrees
        .iter()
        .map(|(repo, wt)| {
            let branch = format_branch_name(wt);
            let path = wt.path.display();
            format!(
                "{:repo_width$}  {:branch_width$}  {}",
                repo,
                branch,
                path,
                repo_width = max_repo_len,
                branch_width = max_branch_len
            )
        })
        .collect();

    Ok(candidates)
}

/// Extract path from 3-column format (repo, branch, path).
fn extract_path_from_all(line: &str) -> Result<String> {
    // Split on two spaces and filter out empty parts, then trim each part
    // (padding creates multiple spaces, so split on "  " can create empty strings and parts with spaces)
    let parts: Vec<&str> = line
        .split("  ")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() >= 3 {
        Ok(parts[2].to_string())
    } else {
        Err(WtError::user_error(format!("failed to extract path from fzf output: {}", line)).into())
    }
}

/// Run fzf with --expect flag to capture which key was pressed.
/// Returns (key, selected_line) tuple, where key is empty string for Enter.
///
/// # Arguments
///
/// * `candidates` - List of formatted candidate strings
/// * `fzf_config` - Fzf configuration
/// * `all_mode` - If true, use 3-column format (repo, branch, path); otherwise 2-column (branch, path)
fn run_fzf_with_expect(
    candidates: &[String],
    fzf_config: &config::FzfConfig,
    all_mode: bool,
) -> Result<Option<(String, String)>> {
    // Preview column depends on mode: {2} for single repo, {3} for all repos
    let preview_column = if all_mode { "{3}" } else { "{2}" };
    let preview_cmd = format!("wt preview --path {}", preview_column);

    // Build fzf command arguments
    let args = vec![
        "--height".to_string(),
        fzf_config.height.clone(),
        "--layout".to_string(),
        fzf_config.layout.clone(),
        "--preview-window".to_string(),
        fzf_config.preview_window.clone(),
        "--preview".to_string(),
        preview_cmd,
        "--prompt".to_string(),
        "Worktree> ".to_string(),
        "--header".to_string(),
        "Enter: cd | Ctrl-E: edit".to_string(),
        "--expect".to_string(),
        "ctrl-e".to_string(), // Capture ctrl-e presses
    ];

    // Spawn fzf process
    let mut child = Command::new("fzf")
        .args(&args)
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
                WtError::io_error_with_source(
                    "failed to write to fzf stdin",
                    anyhow::Error::from(e),
                )
            })?;
        }
        // stdin is dropped here, closing the pipe
    }

    // Wait for fzf to complete and capture output
    let output = child.wait_with_output().map_err(|e| {
        WtError::io_error_with_source("failed to wait for fzf to complete", anyhow::Error::from(e))
    })?;

    // Handle exit codes
    match output.status.code() {
        Some(0) => {
            // User made a selection
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();

            // When using --expect, fzf outputs:
            // Line 1: The key pressed (empty for Enter, "ctrl-e" for Ctrl-E)
            // Line 2: The selected item
            match lines.len() {
                0 => Ok(None), // No selection
                1 => {
                    // Only one line means empty key (Enter) and no selection on second line
                    // This shouldn't happen with valid selection, treat as no selection
                    Ok(None)
                }
                _ => {
                    // Normal case: key on first line, selection on second
                    let key = lines[0].to_string();
                    let selection = lines[1].to_string();

                    if selection.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some((key, selection)))
                    }
                }
            }
        }
        Some(1) => {
            // No match found
            Ok(None)
        }
        Some(130) => {
            // User cancelled (Ctrl-C or Esc)
            Ok(None)
        }
        Some(code) => {
            Err(WtError::user_error(format!("fzf exited with unexpected code: {}", code)).into())
        }
        None => Err(WtError::user_error("fzf was terminated by a signal").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worktree::Worktree;
    use std::path::PathBuf;

    #[test]
    fn test_format_branch_name_strips_refs_heads() {
        let wt = Worktree {
            path: PathBuf::from("/tmp/repo"),
            head: Some("abc123".to_string()),
            branch: Some("refs/heads/main".to_string()),
            locked: false,
            prunable: None,
            bare: false,
        };
        assert_eq!(format_branch_name(&wt), "main");
    }

    #[test]
    fn test_format_branch_name_strips_refs_remotes() {
        let wt = Worktree {
            path: PathBuf::from("/tmp/repo"),
            head: Some("abc123".to_string()),
            branch: Some("refs/remotes/origin/feature".to_string()),
            locked: false,
            prunable: None,
            bare: false,
        };
        assert_eq!(format_branch_name(&wt), "origin/feature");
    }

    #[test]
    fn test_format_branch_name_detached() {
        let wt = Worktree {
            path: PathBuf::from("/tmp/repo"),
            head: None,
            branch: None,
            locked: false,
            prunable: None,
            bare: false,
        };
        assert_eq!(format_branch_name(&wt), "(detached)");
    }

    #[test]
    fn test_prepare_candidates_alignment() {
        let worktrees = vec![
            Worktree {
                path: PathBuf::from("/tmp/repo1"),
                head: Some("abc".to_string()),
                branch: Some("refs/heads/main".to_string()),
                locked: false,
                prunable: None,
                bare: false,
            },
            Worktree {
                path: PathBuf::from("/tmp/repo2"),
                head: Some("def".to_string()),
                branch: Some("refs/heads/feature-branch".to_string()),
                locked: false,
                prunable: None,
                bare: false,
            },
        ];

        let candidates = prepare_candidates(&worktrees);
        assert_eq!(candidates.len(), 2);

        // Check that shorter branch name is padded to match longer one
        assert!(candidates[0].starts_with("main           "));
        assert!(candidates[1].starts_with("feature-branch"));
    }

    #[test]
    fn test_extract_path_success() {
        let line = "main  /tmp/repo/main";
        let path = extract_path(line).unwrap();
        assert_eq!(path, "/tmp/repo/main");
    }

    #[test]
    fn test_extract_path_with_spaces_in_path() {
        let line = "feature  /tmp/my repo/feature";
        let path = extract_path(line).unwrap();
        assert_eq!(path, "/tmp/my repo/feature");
    }

    #[test]
    fn test_extract_path_failure() {
        let line = "invalid-line-format";
        assert!(extract_path(line).is_err());
    }

    #[test]
    fn test_extract_path_from_all_success() {
        let line = "worktree-manager  main       /Users/user/dev/worktree-manager";
        let path = extract_path_from_all(line).unwrap();
        assert_eq!(path, "/Users/user/dev/worktree-manager");
    }

    #[test]
    fn test_extract_path_from_all_with_spaces() {
        let line = "my-repo  feature-x  /Users/user/my projects/my-repo";
        let path = extract_path_from_all(line).unwrap();
        assert_eq!(path, "/Users/user/my projects/my-repo");
    }

    #[test]
    fn test_extract_path_from_all_failure() {
        let line = "only-two  columns";
        assert!(extract_path_from_all(line).is_err());
    }
}
