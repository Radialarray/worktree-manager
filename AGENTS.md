# Agent Guidelines for worktree-manager

## Build / Lint / Test
- `cargo build` / `cargo build --release`
- `cargo test` - run all tests
- `cargo test <test_name>` - run a single test (substring match)
- `cargo fmt` - format (rustfmt)
- `cargo clippy --all-targets --all-features -- -D warnings` - lint

## Code Style
- Run `cargo fmt` before committing; keep code clippy-clean.
- Imports: use rustfmt ordering; avoid unused imports.
- Naming: snake_case (fns/vars/modules), CamelCase (types/traits), SCREAMING_SNAKE_CASE (consts).
- Types: prefer explicit structs/enums; avoid overly generic lifetimes; no `unsafe` unless justified.
- Errors: return `Result<T>`; use `anyhow` at boundaries, `thiserror` for domain errors.
- CLI UX: clear stderr errors, helpful exit codes, `--json` for machine output.

## Issue Tracking

This project uses **bd (beads)** for issue tracking.
Run `bd prime` for workflow context, or install hooks (`bd hooks install`) for auto-injection.

**Quick reference:**
- `bd ready` - Find unblocked work
- `bd create "Title" --type task --priority 2` - Create issue
- `bd close <id>` - Complete work
- `bd sync` - Sync with git (run at session end)

For full workflow details: `bd prime`

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**
- If code changed: run `cargo test` + `cargo fmt` + `cargo clippy`.
1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
