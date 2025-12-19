# Architecture: worktree-manager (wt)

## Goal
Fast CLI for navigating and managing **git worktrees** with a `pj`-like UX:
- default: operate on the **current repository**
- optional: **cross-repo** mode via auto-discovery

## Tech Stack
- Language/runtime: **Rust (stable)**
- CLI: **clap** (derive)
- Config + JSON output: **serde / serde_json**
- Config locations: **directories** (XDG-ish)
- Error handling: **anyhow** (optionally `thiserror` later)
- External tools: **git** + **fzf**

## Core Strategy (Option A)
Shell out to `git` for all Git operations (no libgit2):
- discovery: `git worktree list --porcelain`
- repo root: `git rev-parse --show-toplevel`
- preview/status: `git -C <path> status --porcelain=v1 -b`
- preview/commits: `git -C <path> log -n 5 --oneline --decorate`

## Modes
### Per-repo (default)
`wt` lists worktrees for the repo you are currently inside.

### Cross-repo (auto-discovery)
`wt --all` (or `wt all`) finds repositories from configured search paths and lists worktrees across all of them.

Auto-discovery algorithm (simple + safe):
- configurable search roots (e.g. `~/Dev`)
- scan depth-limited for `.git` directories/files
- for each repo root, run `git -C <repo> worktree list --porcelain`

## Interactive Picker (fzf)
We feed candidates to `fzf` and use a Rust subcommand for preview:
- candidate format: `<repo>\t<branch>\t<path>`
- `fzf` flags:
  - `--delimiter='\t' --with-nth=1,2`
  - `--preview 'wt preview --path {3}'`
  - `--preview-window 'right:60%'`

### Preview Command
`wt preview --path <worktreePath>` prints:
- Repo name + branch + path
- status summary (dirty/ahead/behind)
- recent commits
- (optional) short changed-files list

## Shell Integration (cd)
Rust cannot `cd` the parent shell. Interactive mode prints an action:
- `cd|/abs/path`
- `edit|/abs/path`

A small shell function (zsh) interprets this and performs `cd` / editor launch.

## Commands
- `wt` / `wt interactive` (fzf picker)
- `wt list [--json] [--all]`
- `wt add <branch> [-p <path>] [--track <remote>]`
- `wt remove <worktree|--path <path>> [--force]`
- `wt prune`
- `wt config (init|show|set-editor|set-discovery-paths ...)`

## Non-goals (initially)
- embedding a fuzzy finder (we rely on `fzf`)
- libgit2 integration
- advanced caching (only add if preview performance demands it)
