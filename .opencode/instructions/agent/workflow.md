# Project Workflow

All work MUST follow the project's documented workflows. Read them before acting.

## Required Reading

Before starting any task, read these files in order:

1. `docs/workflows/development-flow.md` — full lifecycle: issue → branch → PR → merge
2. `docs/workflows/conventions.md` — branch names, commit format, scopes, PR titles
3. `docs/workflows/onboarding.md` — setup commands, project structure, dev commands
4. `docs/architecture/overview.md` — system architecture, module map, data flows

For architectural decisions, read the relevant ADR in `docs/adr/`.

## Hard Rules

- **No branch without a GitHub issue.** No merge without a PR.
- **Branch from `main` only.** Format: `type/issue-number-description`.
- **Squash and merge** all PRs. Delete branch after merge.
- **`Closes #N`** goes in the PR body, never in commit messages.
- **Only work on the current milestone** unless explicitly told otherwise.
- **Commits**: `type(scope): description` — see conventions.md for valid scopes.

## Pre-Push Checks

Always run before pushing:

```
cargo clippy && cargo test    # backend
pnpm lint && pnpm test        # frontend
```

Do not push if any check fails.

## Agent-Specific

- When using superpowers: worktree from `main`, use ORCHESTRATE, delegate implementation to subagents.
- Spec before code. Code review after implementation.
- Do not skip tasks if critical or important issues exist.
- When in doubt about architecture, read the ADRs before proposing changes.
