use std::path::Path;

use anyhow::{Context, Result};

use crate::git;

pub fn list_worktrees(json: bool) -> Result<()> {
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
