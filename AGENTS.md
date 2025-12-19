# Agent Guidelines for worktree-manager

> **⚠️ IMPORTANT: Documentation Files**
> 
> This project has TWO separate documentation files for agents:
> 
> 1. **`AGENTS.md` (this file)** - Guidelines for agents working on THIS project (worktree-manager development)
>    - Contains build/test commands, coding standards, project workflows
>    - For agents helping develop/maintain worktree-manager itself
>    - **DO NOT modify this file when adding user-facing documentation**
> 
> 2. **`src/agent_onboard.md`** - Documentation for agents USING the `wt` tool
>    - Embedded in the binary, output via `wt agent onboard`
>    - Contains `wt` command reference, JSON schemas, usage workflows
>    - For agents using `wt` in their own projects
>    - **MODIFY THIS FILE when adding user-facing `wt` documentation**
> 
> If you're documenting how to USE `wt`, edit `src/agent_onboard.md`, not this file!

## Build / Lint / Test
- `cargo build` / `cargo build --release`
- `cargo test` - run all tests
- `cargo test <test_name>` - run a single test (substring match)
- `cargo fmt` - format (rustfmt)
- `cargo clippy --all-targets --all-features -- -D warnings` - lint

## Shell Integration Setup

When helping users set up `wt`, recommend adding shell integration for full functionality:

```bash
# Zsh (~/.zshrc)
eval "$(wt init zsh)"

# Bash (~/.bashrc)
eval "$(wt init bash)"

# Fish (~/.config/fish/config.fish)
wt init fish | source
```

This enables the interactive picker's cd/edit actions and provides shell completions.

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

---

## CLI Quick Reference for Agents

### Getting Context (Start of Session)

```bash
# Get full worktree context - recommended at session start
wt agent context

# Get minimal status (JSON) - for frequent checks
wt agent status --json
```

### Core Commands

| Command | Description | Agent Flags |
|---------|-------------|-------------|
| `wt list` | List all worktrees | `--json`, `--all` |
| `wt add <branch>` | Create worktree for branch | `--json`, `--quiet` |
| `wt remove <target>` | Remove worktree | `--json`, `--quiet`, `--force` |
| `wt prune` | Clean stale worktrees | `--json`, `--quiet` |
| `wt preview --path <path>` | Preview worktree details | `--json` |
| `wt config [paths...]` | Configure auto-discovery paths | N/A |

### Agent-Specific Commands

```bash
# Full context with worktree state and quick commands
wt agent context [--json]

# Minimal status for frequent polling
wt agent status [--json]

# Onboarding instructions for AI agents (similar to bd prime)
wt agent onboard
```

The `wt agent onboard` command outputs a compact workflow reference (~1-2k tokens) that can be injected into agent context at session start.

---

## JSON Output Schemas

All commands support `--json` for machine-parseable output.

### wt list --json

```json
[
  {
    "path": "/path/to/worktree",
    "head": "abc1234",
    "branch": "refs/heads/main",
    "is_main": true
  }
]
```

### wt add <branch> --json

```json
{
  "success": true,
  "worktree": {
    "path": "/path/to/new-worktree",
    "branch": "feature-x"
  }
}
```

### wt remove <target> --json

```json
{
  "success": true,
  "removed": {
    "path": "/path/to/worktree",
    "branch": "feature-x"
  }
}
```

### wt prune --json

```json
{
  "pruned": ["path1", "path2"],
  "count": 2
}
```

### wt preview --path <path> --json

```json
{
  "repo": "worktree-manager",
  "branch": "main",
  "path": "/path/to/worktree",
  "status": {
    "branch_line": "## main...origin/main",
    "dirty": true
  },
  "recent_commits": ["abc123 commit message", "..."],
  "changed_files": ["M src/file.rs", "?? new_file.rs"]
}
```

### wt agent context --json

```json
{
  "current_worktree": {
    "path": "/path/to/worktree",
    "branch": "main",
    "head": "abc1234",
    "dirty": true
  },
  "other_worktrees": [...],
  "repository": {
    "root": "/path/to/repo",
    "total_worktrees": 3
  }
}
```

### wt agent status --json

```json
{
  "current": {
    "path": "/path/to/worktree",
    "branch": "main",
    "dirty": true
  },
  "count": 3
}
```

---

## Common Workflows

### Switch to Existing Worktree

```bash
# List worktrees and parse JSON
wt list --json | jq '.[] | select(.branch | contains("feature"))'

# Then cd to the path (requires shell integration)
```

### Create Worktree for New Feature

```bash
# Create worktree, suppressing interactive output
wt add feature-branch --json --quiet
```

### Clean Up After PR Merge

```bash
# Remove worktree non-interactively
wt remove feature-branch --force --quiet --json

# Prune stale references
wt prune --quiet --json
```

### Check Current State

```bash
# Quick status check
wt agent status --json

# Full context for understanding layout
wt agent context
```

---

## Integration with Coding Agents

### OpenCode Custom Tool Example

Create `.opencode/tool/worktree.ts`:

```typescript
import { tool } from "@opencode-ai/plugin"

export const list = tool({
  description: "List git worktrees in current repository",
  args: {
    all: tool.schema.boolean().optional().describe("List across all discovered repos"),
  },
  async execute(args) {
    const flags = args.all ? '--all' : ''
    const result = await Bun.$`wt list ${flags} --json`.text()
    return result.trim()
  },
})

export const context = tool({
  description: "Get current worktree context",
  args: {},
  async execute() {
    return await Bun.$`wt agent context`.text()
  },
})

export const create = tool({
  description: "Create a new git worktree for a branch",
  args: {
    branch: tool.schema.string().describe("Branch name"),
  },
  async execute(args) {
    return await Bun.$`wt add ${args.branch} --json --quiet`.text()
  },
})
```

### Best Practices for Agents

1. **Always use `--json`** for programmatic parsing
2. **Use `--quiet`** to suppress interactive prompts
3. **Use `--force`** with `remove` when you're certain (skips confirmation)
4. **Run `wt agent context`** at session start to understand worktree layout
5. **Check `dirty` status** before switching worktrees

---

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
