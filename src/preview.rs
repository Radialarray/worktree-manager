use std::path::Path;

use anyhow::{Context, Result};

use crate::git;
use crate::process;

pub fn print_preview(path: &Path) -> Result<()> {
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

    println!("Repo:   {repo_name}");
    println!("Branch: {branch}");
    println!("Path:   {}", abs_path.to_string_lossy());
    println!();

    // Status summary.
    // Use -sb rather than porcelain=v1 -b because it's short and readable.
    let status = process::run_stdout(
        "git",
        &["-C", &abs_path.to_string_lossy(), "status", "-sb"],
        None,
    )
    .unwrap_or_else(|_| "(failed to read status)".to_string());
    print_section("Status", status.trim_end());

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
    print_section("Recent commits", commits.trim_end());

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

    if !changed.trim().is_empty() {
        print_section("Changed files", changed.trim_end());
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
