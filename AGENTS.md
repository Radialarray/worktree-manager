# Agent Guidelines for worktree-manager

## Build & Test Commands
- `npm run build` - Compile TypeScript to dist/
- `npm test` - Run all tests
- `npm test -- <test-file>` - Run single test file
- `npm run lint` - Run ESLint
- `npm run format` - Format code with Prettier

## Code Style
- **Language**: TypeScript with strict mode enabled
- **Imports**: Use ES6 imports, group by external → internal → relative
- **Formatting**: 2-space indent, single quotes, semicolons, trailing commas
- **Types**: Explicit return types on functions, avoid `any`, prefer interfaces over types
- **Naming**: camelCase (variables/functions), PascalCase (classes/types), UPPER_SNAKE_CASE (constants)
- **Error Handling**: Use custom error classes, throw descriptive errors, avoid silent failures
- **Comments**: JSDoc for public APIs, inline comments for complex logic only
- **Testing**: Colocate tests with source files (*.test.ts), use describe/it blocks, mock external dependencies

## Project-Specific Rules
- This is a Git worktree management tool - prioritize CLI UX and Git integration
- Follow conventional commits: feat/fix/docs/refactor/test/chore
- Update README.md when adding new commands or features

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

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
