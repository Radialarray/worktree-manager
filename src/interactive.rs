use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow};

use crate::{config, git};

/// Run the interactive worktree picker.
/// Outputs action in format "cd|PATH" or "edit|PATH" for shell wrapper to parse.
pub fn run_interactive() -> Result<()> {
    // Load config for fzf settings
    let config = config::load()?;

    // Get repository root and worktrees
    let repo_root = git::repo_root(None)?;
    let worktrees = git::worktrees_porcelain(&repo_root)?;

    if worktrees.is_empty() {
        anyhow::bail!("no worktrees found in repository");
    }

    // Prepare candidates for fzf
    // Format: "<branch>  <path>" with aligned columns
    let candidates = prepare_candidates(&worktrees);

    // Run fzf with --expect to capture which key was pressed
    let selection = run_fzf_with_expect(&candidates, &config.fzf)?;

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
        Err(anyhow!("failed to extract path from fzf output: {}", line))
    }
}

/// Run fzf with --expect flag to capture which key was pressed.
/// Returns (key, selected_line) tuple, where key is empty string for Enter.
fn run_fzf_with_expect(
    candidates: &[String],
    fzf_config: &config::FzfConfig,
) -> Result<Option<(String, String)>> {
    // Build fzf command arguments
    let args = vec![
        "--height".to_string(),
        fzf_config.height.clone(),
        "--layout".to_string(),
        fzf_config.layout.clone(),
        "--preview-window".to_string(),
        fzf_config.preview_window.clone(),
        "--preview".to_string(),
        "wt preview --path {2}".to_string(), // {2} refers to second column (path)
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
        .context("failed to spawn fzf (is it installed?)")?;

    // Write candidates to stdin
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("failed to open fzf stdin"))?;

        for candidate in candidates {
            writeln!(stdin, "{}", candidate).context("failed to write to fzf stdin")?;
        }
        // stdin is dropped here, closing the pipe
    }

    // Wait for fzf to complete and capture output
    let output = child
        .wait_with_output()
        .context("failed to wait for fzf to complete")?;

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
        Some(code) => Err(anyhow!("fzf exited with unexpected code: {}", code)),
        None => Err(anyhow!("fzf was terminated by a signal")),
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
}
