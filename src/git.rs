#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::process;
use crate::worktree::{self, Worktree};

pub fn repo_root(cwd: Option<&Path>) -> Result<PathBuf> {
    let out = process::run_stdout("git", &["rev-parse", "--show-toplevel"], cwd)?;
    Ok(PathBuf::from(out.trim()))
}

pub fn worktrees_porcelain(repo_root: &Path) -> Result<Vec<Worktree>> {
    let out = process::run_stdout("git", &["worktree", "list", "--porcelain"], Some(repo_root))
        .context("failed to list worktrees")?;
    worktree::parse_porcelain(&out)
}
