#![allow(dead_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

use crate::git;

/// Discover git repositories under the given search paths.
/// Returns a list of repository root paths (deduplicated).
///
/// # Implementation Details
///
/// - Walks each search path up to 3 levels deep
/// - Looks for `.git` entries (either directory or file)
/// - For worktrees (`.git` file), resolves to the main repo root
/// - Deduplicates results so each main repo appears only once
/// - Skips paths that don't exist or can't be read
///
/// # Arguments
///
/// * `search_paths` - List of directory paths to search for git repositories
///
/// # Returns
///
/// A deduplicated list of repository root paths
///
/// # Examples
///
/// ```no_run
/// use worktree_manager::discovery;
///
/// let paths = vec!["/home/user/projects".to_string()];
/// let repos = discovery::discover_repos(&paths)?;
/// for repo in repos {
///     println!("Found repo: {}", repo.display());
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn discover_repos(search_paths: &[String]) -> Result<Vec<PathBuf>> {
    let mut repo_roots = HashSet::new();

    for search_path in search_paths {
        let path = PathBuf::from(search_path);

        // Skip if path doesn't exist
        if !path.exists() {
            eprintln!("Warning: search path does not exist: {}", path.display());
            continue;
        }

        // Skip if path is not a directory
        if !path.is_dir() {
            eprintln!(
                "Warning: search path is not a directory: {}",
                path.display()
            );
            continue;
        }

        // Walk the directory tree up to 3 levels deep
        for entry in WalkDir::new(&path)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            // Look for .git entries
            if entry_path.file_name().and_then(|s| s.to_str()) == Some(".git") {
                // Parent directory is the potential repo root
                if let Some(parent) = entry_path.parent() {
                    match resolve_repo_root(parent) {
                        Ok(repo_root) => {
                            repo_roots.insert(repo_root);
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: failed to resolve repo root for {}: {}",
                                parent.display(),
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    // Convert HashSet to sorted Vec for consistent output
    let mut repos: Vec<PathBuf> = repo_roots.into_iter().collect();
    repos.sort();

    Ok(repos)
}

/// Resolves the true repository root for a given path.
///
/// For normal repos with `.git` directory, this returns the parent directory.
/// For worktrees with `.git` file, this uses `git rev-parse --show-toplevel`
/// to find the main repository root.
///
/// # Arguments
///
/// * `path` - Path that contains a `.git` entry
///
/// # Returns
///
/// The canonical repository root path
fn resolve_repo_root(path: &Path) -> Result<PathBuf> {
    // Use git to determine the actual repo root
    // This handles both normal repos and worktrees correctly
    let repo_root = git::repo_root(Some(path))?;
    Ok(repo_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn discover_repos_returns_empty_for_empty_paths() {
        let repos = discover_repos(&[]).unwrap();
        assert!(repos.is_empty());
    }

    #[test]
    fn discover_repos_skips_nonexistent_paths() {
        let repos = discover_repos(&["/nonexistent/path/12345".to_string()]).unwrap();
        assert!(repos.is_empty());
    }

    #[test]
    fn discover_repos_finds_current_repo() {
        // Get the worktree-manager repo root (the current project)
        let current_repo = git::repo_root(None).unwrap();
        let parent = current_repo.parent().unwrap();

        // Search in the parent directory
        let repos = discover_repos(&[parent.to_string_lossy().to_string()]).unwrap();

        // Should find at least the current repo
        assert!(
            repos.contains(&current_repo),
            "Should find current repo: {:?} in {:?}",
            current_repo,
            repos
        );
    }

    #[test]
    fn discover_repos_deduplicates_results() {
        // Get the current repo and search it twice
        let current_repo = git::repo_root(None).unwrap();
        let parent = current_repo.parent().unwrap();
        let parent_str = parent.to_string_lossy().to_string();

        // Search the same path twice
        let repos = discover_repos(&[parent_str.clone(), parent_str.clone()]).unwrap();

        // Should not have duplicates
        let unique_repos: HashSet<_> = repos.iter().collect();
        assert_eq!(
            repos.len(),
            unique_repos.len(),
            "Should not have duplicate repos"
        );
    }

    #[test]
    fn discover_repos_respects_depth_limit() {
        // Create a temporary directory structure deeper than 3 levels
        let temp_dir = std::env::temp_dir().join("wt_discovery_test_depth");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up if it exists

        let deep_path = temp_dir
            .join("level1")
            .join("level2")
            .join("level3")
            .join("level4");

        fs::create_dir_all(&deep_path).unwrap();

        // Create a .git directory at level 4 (too deep to find)
        let deep_git = deep_path.join(".git");
        fs::create_dir(&deep_git).unwrap();

        // Search from temp_dir
        let repos = discover_repos(&[temp_dir.to_string_lossy().to_string()]).unwrap();

        // Should not find the repo at level 4 (depth limit is 3)
        // Note: This test assumes the temp dir doesn't contain other repos
        assert!(
            !repos.iter().any(|r| r.starts_with(&deep_path)),
            "Should not find repos deeper than 3 levels"
        );

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
