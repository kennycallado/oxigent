# Oxigent — Agent Instructions

## Project

- Monorepo: Rust backend (DDD) + TypeScript frontend (Lit/Tauri)
- DB: SurrealDB (embedded desktop, server cloud)
- Package manager: pnpm (frontend), cargo (backend)
- Dev env: Nix flakes + direnv

## Architecture

- Backend: bounded contexts in `backend/crates/` (work-management, planning, agent-execution, identity-access, integrations, shared-kernel, api)
- Frontend: Hexagonal — `packages/app-core` (ports) → `packages/platform-*` (adapters)
- Shared kernel: IDs, events, AppError in `backend/crates/shared-kernel`
- Codegen: Typeshare (Rust → TypeScript types)
- See `docs/architecture/overview.md` for full diagrams and data flows

## Mandatory Workflow

- Read `docs/workflows/development-flow.md` and `docs/workflows/conventions.md` before any task
- Issue → branch from main → commits → PR → **notify human** → wait for review → postmortem → merge → cleanup
- Commit format: `type(scope): description` (see conventions.md for valid scopes)
- `Closes #N` in PR body, never in commits
- Pre-push checks: `cargo clippy && cargo test` | `pnpm lint && pnpm test`
- Only work on the current milestone unless explicitly told otherwise
- **After PR is created:** stop, write postmortem, notify human, wait for instructions. Do NOT merge or start new work autonomously.
- **Post-merge:** pull main, cleanup worktree/branch. See `development-flow.md` sections 5a–5c for full protocol.

## Board Protocol (agents)

Before starting any issue, update the GitHub Projects board:

1. Set `Agent` field to your identifier (e.g. `@agent_gpt`, `@agent_glm`)
2. Move issue to `In Progress`
3. Set `Priority` if not already set
4. If blocked by a dependency, set `Blocked: Yes` and add a note to the issue body explaining the blocker

On completion (PR merged):

- The issue closes automatically via `Closes #N` in the PR body
- No manual board cleanup needed

Never start work on an issue with `Blocked: Yes` unless you are actively resolving the blocker.
Never work on issues outside the current milestone.

## Superpowers & Agents

- Always use superpowers skills when available
- Worktree from `main`, activate ORCHESTRATE
- Delegate implementation to `@agent_gpt`
- Spec + code review with `@agent_glm`
- Do not proceed past a task if critical or important issues exist and are not addressed in a later task
- If a subagent needs ULTRATHINK, instruct it to read the proper documentation.
- **Postmortem required:** after every superpowers-driven implementation, write a postmortem in `docs/postmortems/YYYY-MM-DD-topic.md` covering: what was done, what went well, what went wrong, and lessons learned. Do not mark the issue as complete until the postmortem is written. Write it **before merge**, not after.

## Context-Mode Rules

You have context-mode MCP tools available. These rules are NOT optional — they protect your context window from flooding.

### BLOCKED commands

- **curl / wget** — use `ctx_fetch_and_index` or `ctx_execute` instead
- **Inline HTTP** (`fetch('http`, `requests.get(`, etc.) — use `ctx_execute` sandbox instead
- **Direct web fetching** — use `ctx_fetch_and_index` then `ctx_search`

### Redirected tools

- **Shell (>20 lines)** — use `ctx_batch_execute` or `ctx_execute` sandbox
- **File reading (for analysis)** — use `ctx_execute_file` instead of Read
- **grep / search (large results)** — use `ctx_execute` sandbox

### Tool hierarchy

1. `ctx_batch_execute` — run commands + search in ONE call
2. `ctx_search` — query indexed content
3. `ctx_execute` / `ctx_execute_file` — sandbox execution
4. `ctx_fetch_and_index` → `ctx_search` — web content
5. `ctx_index` — store for later search

### Output constraints

- Keep responses under 500 words
- Write artifacts to FILES — never inline
- Use descriptive source labels when indexing
