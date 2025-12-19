use std::path::Path;

use anyhow::{Context, Result};

use crate::{config, discovery, git};

pub fn list_worktrees(json: bool, all: bool) -> Result<()> {
    if all {
        list_all_worktrees(json)
    } else {
        list_single_repo_worktrees(json)
    }
}

fn list_single_repo_worktrees(json: bool) -> Result<()> {
    let repo_root = git::repo_root(None).context("not inside a git repository")?;
    let worktrees = git::worktrees_porcelain(&repo_root).context("failed to parse worktrees")?;

    if json {
        // Minimal JSON array of objects; we can refine schema later.
        let value = serde_json::to_value(
            worktrees
                .iter()
                .map(|wt| {
                    serde_json::json!({
                        "path": wt.path,
                        "head": wt.head,
                        "branch": wt.branch,
                        "locked": wt.locked,
                        "prunable": wt.prunable,
                        "bare": wt.bare,
                    })
                })
                .collect::<Vec<_>>(),
        )?;
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    let rendered: Vec<(String, String, String)> = worktrees
        .iter()
        .map(|wt| {
            (
                pretty_ref(wt.branch.as_deref()),
                display_path(&repo_root, &wt.path),
                flags(wt),
            )
        })
        .collect();

    let max_branch = rendered
        .iter()
        .map(|(branch, _, _)| branch.len())
        .max()
        .unwrap_or(0);

    for (branch, path, flags) in rendered {
        if flags.is_empty() {
            println!("{branch:<width$}  {path}", width = max_branch);
        } else {
            println!("{branch:<width$}  {path}  [{flags}]", width = max_branch);
        }
    }

    Ok(())
}

fn list_all_worktrees(json: bool) -> Result<()> {
    let config = config::load()?;
    if config.auto_discovery.paths.is_empty() {
        anyhow::bail!(
            "No auto-discovery paths configured. Run: wt config set-discovery-paths <paths...>"
        );
    }

    let repos = discovery::discover_repos(&config.auto_discovery.paths)?;
    if repos.is_empty() {
        eprintln!("No git repositories found in configured discovery paths.");
        return Ok(());
    }

    // Collect all worktrees from all repos
    let mut all_worktrees: Vec<(String, crate::worktree::Worktree)> = Vec::new();

    for repo_root in repos {
        let repo_name = repo_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("(unknown)")
            .to_string();

        match git::worktrees_porcelain(&repo_root) {
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

    if json {
        let value = serde_json::to_value(
            all_worktrees
                .iter()
                .map(|(repo, wt)| {
                    serde_json::json!({
                        "repo": repo,
                        "path": wt.path,
                        "head": wt.head,
                        "branch": wt.branch,
                        "locked": wt.locked,
                        "prunable": wt.prunable,
                        "bare": wt.bare,
                    })
                })
                .collect::<Vec<_>>(),
        )?;
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    // Render in table format with repo name
    let rendered: Vec<(String, String, String, String)> = all_worktrees
        .iter()
        .map(|(repo, wt)| {
            (
                repo.clone(),
                pretty_ref(wt.branch.as_deref()),
                wt.path.to_string_lossy().to_string(),
                flags(wt),
            )
        })
        .collect();

    let max_repo = rendered
        .iter()
        .map(|(repo, _, _, _)| repo.len())
        .max()
        .unwrap_or(0);

    let max_branch = rendered
        .iter()
        .map(|(_, branch, _, _)| branch.len())
        .max()
        .unwrap_or(0);

    for (repo, branch, path, flags) in rendered {
        if flags.is_empty() {
            println!(
                "{repo:<repo_width$}  {branch:<branch_width$}  {path}",
                repo_width = max_repo,
                branch_width = max_branch
            );
        } else {
            println!(
                "{repo:<repo_width$}  {branch:<branch_width$}  {path}  [{flags}]",
                repo_width = max_repo,
                branch_width = max_branch
            );
        }
    }

    Ok(())
}

fn pretty_ref(r: Option<&str>) -> String {
    r.map(|r| {
        r.strip_prefix("refs/heads/")
            .or_else(|| r.strip_prefix("refs/remotes/"))
            .unwrap_or(r)
            .to_string()
    })
    .unwrap_or_else(|| "(detached)".to_string())
}

fn display_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn flags(wt: &crate::worktree::Worktree) -> String {
    let mut parts = Vec::new();
    if wt.locked {
        parts.push("locked".to_string());
    }
    if let Some(reason) = &wt.prunable {
        if reason.is_empty() {
            parts.push("prunable".to_string());
        } else {
            parts.push(format!("prunable: {reason}"));
        }
    }
    if wt.bare {
        parts.push("bare".to_string());
    }
    parts.join(", ")
}
