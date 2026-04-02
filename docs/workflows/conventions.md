# Conventions

## Branch names

```
<type>/<issue-number>-<short-description>
```

| Type       | When to use                           |
| ---------- | ------------------------------------- |
| `feat`     | New feature or capability             |
| `fix`      | Bug fix                               |
| `docs`     | Documentation only                    |
| `refactor` | Code change with no behavior change   |
| `test`     | Adding or fixing tests                |
| `infra`    | CI, tooling, build system             |
| `chore`    | Dependency updates, minor maintenance |

Examples:

```
feat/3-work-management-task-scaffold
fix/11-typeshare-ci-stale-types
docs/7-onboarding-guide
infra/13-github-actions-pipeline
```

Rules:

- Always include the issue number
- Use kebab-case for the description
- Keep it short (3-5 words)

---

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <short description>

[optional body]

[optional footer: Closes #N]
```

### Type

Same values as branch types: `feat`, `fix`, `docs`, `refactor`, `test`, `infra`, `chore`.

### Scope

The crate or package being changed:

| Scope              | What it covers                   |
| ------------------ | -------------------------------- |
| `work-management`  | `backend/crates/work-management` |
| `identity-access`  | `backend/crates/identity-access` |
| `planning`         | `backend/crates/planning`        |
| `agent-execution`  | `backend/crates/agent-execution` |
| `integrations`     | `backend/crates/integrations`    |
| `shared-kernel`    | `backend/crates/shared-kernel`   |
| `api`              | `backend/crates/api`             |
| `app-core`         | `packages/app-core`              |
| `features`         | `packages/features/*`            |
| `platform-web`     | `packages/platform-web`          |
| `platform-desktop` | `packages/platform-desktop`      |
| `ui`               | `packages/ui`                    |
| `web`              | `apps/web`                       |
| `desktop`          | `apps/desktop`                   |
| `ci`               | GitHub Actions workflows         |
| `adr`              | `docs/adr/`                      |

### Examples

```
feat(work-management): add CreateTask command and SurrealDB adapter

fix(api): return 422 instead of 500 for domain validation errors

docs(adr): add ADR-009 error handling strategy

infra(ci): add Typeshare freshness check

refactor(app-core): extract TaskService port into separate file
```

### Breaking changes

Add `!` after the type/scope and a `BREAKING CHANGE:` footer:

```
feat(api)!: rename /v1/tasks to /v1/work-items

BREAKING CHANGE: all clients must update their task endpoint URLs.
Closes #42
```

---

## PR titles

Same format as commit messages. The squash-merge commit will use the PR title.

```
feat(work-management): scaffold task/ module with CreateTask and ListTasks
```

---

## Issue references

Always close issues from the PR body, not from commit messages:

```markdown
## What

Short description of what this PR does.

## Why

Why this change was needed (link to issue context if useful).

Closes #3
```

Using `Closes #N` (not `Fixes` or `Resolves`) is the convention in this project — it works
for both features and bugs.

---

## Project board fields

When working on an issue, keep these fields up to date on the board:

| Field      | Values                                  | Rule                                                                                   |
| ---------- | --------------------------------------- | -------------------------------------------------------------------------------------- |
| `Priority` | `Critical` / `High` / `Medium` / `None` | Set at triage; `Critical` blocks all other work in the milestone                       |
| `Blocked`  | `Yes` / `No`                            | Set to `Yes` if a dependency prevents progress; document the blocker in the issue body |
| `Agent`    | e.g. `@agent_gpt`                       | Set to the subagent name before starting work; clear on merge                          |
