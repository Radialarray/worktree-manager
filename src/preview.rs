use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::git;
use crate::process;

#[derive(Serialize)]
struct PreviewOutput {
    repo: String,
    branch: String,
    path: String,
    status: StatusInfo,
    recent_commits: Vec<String>,
    changed_files: Vec<String>,
}

#[derive(Serialize)]
struct StatusInfo {
    branch_line: String,
    dirty: bool,
}

pub fn print_preview(path: &Path, json: bool) -> Result<()> {
    let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    // Repo name derived from repo root directory name.
    let repo_root = git::repo_root(Some(&abs_path)).context("not inside a git repository")?;
    let repo_name = repo_root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| repo_root.to_string_lossy().to_string());

    // Best-effort: branch from worktree porcelain.
    let worktrees = git::worktrees_porcelain(&repo_root).unwrap_or_default();
    let branch = worktrees
        .iter()
        .find(|wt| wt.path == abs_path)
        .and_then(|wt| wt.branch.as_deref())
        .map(pretty_ref)
        .unwrap_or_else(|| "(unknown)".to_string());

    // Status summary.
    let status = process::run_stdout(
        "git",
        &["-C", &abs_path.to_string_lossy(), "status", "-sb"],
        None,
    )
    .unwrap_or_else(|_| "(failed to read status)".to_string());

    // Recent commits.
    let commits = process::run_stdout(
        "git",
        &[
            "-C",
            &abs_path.to_string_lossy(),
            "log",
            "-n",
            "5",
            "--oneline",
            "--decorate",
        ],
        None,
    )
    .unwrap_or_else(|_| "(failed to read log)".to_string());

    // Changed files summary.
    let changed = process::run_stdout(
        "git",
        &[
            "-C",
            &abs_path.to_string_lossy(),
            "status",
            "--porcelain=v1",
        ],
        None,
    )
    .unwrap_or_else(|_| "".to_string());

    if json {
        let status_trimmed = status.trim();
        let branch_line = status_trimmed.lines().next().unwrap_or("").to_string();
        let dirty = !changed.trim().is_empty();

        let output = PreviewOutput {
            repo: repo_name,
            branch: branch.clone(),
            path: abs_path.to_string_lossy().to_string(),
            status: StatusInfo { branch_line, dirty },
            recent_commits: commits.trim().lines().map(|s| s.to_string()).collect(),
            changed_files: changed.trim().lines().map(|s| s.to_string()).collect(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Repo:   {repo_name}");
        println!("Branch: {branch}");
        println!("Path:   {}", abs_path.to_string_lossy());
        println!();

        print_section("Status", status.trim_end());
        print_section("Recent commits", commits.trim_end());

        if !changed.trim().is_empty() {
            print_section("Changed files", changed.trim_end());
        }
    }

    Ok(())
}

fn pretty_ref(r: &str) -> String {
    r.strip_prefix("refs/heads/")
        .or_else(|| r.strip_prefix("refs/remotes/"))
        .unwrap_or(r)
        .to_string()
}

fn print_section(title: &str, body: &str) {
    println!("{title}:");
    if body.is_empty() {
        println!("  (none)");
    } else {
        // Indent for readability.
        for line in body.lines() {
            println!("  {line}");
        }
    }
    println!();
}
