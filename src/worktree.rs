#![allow(dead_code)]

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Worktree {
    pub path: PathBuf,
    pub head: Option<String>,
    pub branch: Option<String>, // refs/heads/foo or refs/remotes/origin/foo
    pub locked: bool,
    pub prunable: Option<String>, // reason from `prunable <reason>`
    pub bare: bool,
}

/// Parse `git worktree list --porcelain` output.
///
/// Format (repeated blocks separated by blank lines):
/// - `worktree <path>`
/// - `HEAD <sha>` or `HEAD detached`
/// - `branch <ref>` OR may be missing on detached HEAD
/// - `locked` or `locked <reason>`
/// - `prunable <reason>`
/// - `bare`
pub fn parse_porcelain(input: &str) -> Result<Vec<Worktree>> {
    let mut worktrees = Vec::new();
    let mut current: Option<Worktree> = None;

    for raw_line in input.lines() {
        let line = raw_line.trim_end();

        if line.is_empty() {
            if let Some(wt) = current.take() {
                worktrees.push(wt);
            }
            continue;
        }

        let (key, rest) = line
            .split_once(' ')
            .map(|(k, r)| (k, Some(r)))
            .unwrap_or((line, None));

        match key {
            "worktree" => {
                if let Some(wt) = current.take() {
                    worktrees.push(wt);
                }

                let path = rest.context("missing worktree path")?;
                current = Some(Worktree {
                    path: PathBuf::from(path),
                    head: None,
                    branch: None,
                    locked: false,
                    prunable: None,
                    bare: false,
                });
            }
            "HEAD" => {
                let wt = current.as_mut().context("HEAD before worktree")?;
                let value = rest.context("missing HEAD value")?;
                // Usually a sha, but on detached may appear as "detached".
                if value == "detached" {
                    wt.head = None;
                } else {
                    wt.head = Some(value.to_string());
                }
            }
            "branch" => {
                let wt = current.as_mut().context("branch before worktree")?;
                wt.branch = rest.map(|s| s.to_string());
            }
            "locked" => {
                let wt = current.as_mut().context("locked before worktree")?;
                wt.locked = true;
                // ignore optional reason for now
            }
            "prunable" => {
                let wt = current.as_mut().context("prunable before worktree")?;
                wt.prunable = rest.map(|s| s.to_string());
            }
            "bare" => {
                let wt = current.as_mut().context("bare before worktree")?;
                wt.bare = true;
            }
            _ => {
                // Ignore unknown keys for forwards compatibility.
            }
        }
    }

    if let Some(wt) = current.take() {
        worktrees.push(wt);
    }

    Ok(worktrees)
}

#[cfg(test)]
mod tests {
    use super::{Worktree, parse_porcelain};
    use std::path::PathBuf;

    #[test]
    fn parses_single_worktree() {
        let input = "worktree /tmp/repo\nHEAD abcdef\nbranch refs/heads/main\n\n";
        let got = parse_porcelain(input).unwrap();
        assert_eq!(
            got,
            vec![Worktree {
                path: PathBuf::from("/tmp/repo"),
                head: Some("abcdef".to_string()),
                branch: Some("refs/heads/main".to_string()),
                locked: false,
                prunable: None,
                bare: false,
            }]
        );
    }

    #[test]
    fn parses_detached_and_flags() {
        let input = "worktree /tmp/repo-wt\nHEAD detached\nlocked\nprunable stale\nbare\n";
        let got = parse_porcelain(input).unwrap();
        assert_eq!(got.len(), 1);
        let wt = &got[0];
        assert_eq!(wt.path, PathBuf::from("/tmp/repo-wt"));
        assert_eq!(wt.head, None);
        assert_eq!(wt.branch, None);
        assert!(wt.locked);
        assert_eq!(wt.prunable.as_deref(), Some("stale"));
        assert!(wt.bare);
    }
}
