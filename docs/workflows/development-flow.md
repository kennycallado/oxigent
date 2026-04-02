# Development Flow

## Overview

```
GitHub Issue → branch → commits → PR → review → merge → issue closes
```

All work is tracked via GitHub Issues. No branch without an issue; no merge without a PR.

---

## 1. Pick an issue

- Go to [GitHub Projects board](https://github.com/users/kennycallado/projects/2)
- Pick an issue from `Todo` in the current milestone
- Assign it to yourself (or set the `Agent` field if a subagent is taking it) and move it to `In Progress`
- Set `Priority` if not already set (see table below)
- If the issue is blocked by a dependency, set `Blocked: Yes` and note the reason in the issue body

---

## 2. Create a branch

Branch off `main`:

```bash
git checkout main && git pull
git checkout -b <type>/<issue-number>-<short-description>
```

Examples:

```bash
git checkout -b feat/3-work-management-task-scaffold
git checkout -b fix/11-typeshare-ci-check
git checkout -b docs/7-onboarding-guide
```

See [conventions.md](./conventions.md) for branch naming rules.

---

## 3. Develop

- Commit early and often (see [conventions.md](./conventions.md) for commit format)
- Keep each commit focused on one thing
- Run local checks before pushing:

```bash
cargo clippy && cargo test       # backend
pnpm lint && pnpm test           # frontend
```

---

## 4. Open a PR

When the work is ready for review:

```bash
git push -u origin <branch-name>
gh pr create --fill
```

PR requirements:

- Title follows the same format as commit messages (see [conventions.md](./conventions.md))
- Body contains `Closes #<issue-number>` so the issue closes automatically on merge
- CI must pass before requesting review

Move the issue to `In Review` on the board.

---

## 5. Review and merge

- At least one review approval before merging (self-review acceptable for solo work)
- Merge strategy: **squash and merge** to keep `main` history clean
- Delete the branch after merge

The issue closes automatically when the PR is merged (via `Closes #N` in the PR body).

### 5a. Agent: Post-PR protocol

After opening a PR, the agent **must**:

1. **Notify the human** with a summary:

   ```
   PR #N listo para review: <URL>
   Issue #<N>: <title>
   Rama: <branch-name>
   Resumen: <1-2 líneas de qué se hizo>
   Checks: <tests, clippy status>
   ```

2. **Write the postmortem** in `docs/superpowers/postmortems/YYYY-MM-DD-<topic>.md` (see AGENTS.md)
3. **Stop and wait** for the human to review

The agent must **NOT** merge or take another issue until the human responds.

### 5b. Human: Review and decide

The human reviews the PR (or requests `@agent_glm` review). Two options:

| Option                | Human says           | Agent executes                                           |
| --------------------- | -------------------- | -------------------------------------------------------- |
| **A: Merge manual**   | "listo", "merged"    | Pull main + cleanup worktree                             |
| **B: Delegate merge** | "merge #N y cleanup" | Postmortem → squash merge → pull main → cleanup worktree |

### 5c. Agent: Post-merge cleanup

Regardless of who merges, the agent performs cleanup **after** the postmortem is written:

```bash
# Option B only: merge the PR
gh pr merge <N> --squash --delete-branch \
  --subject "<conventional-commit-title>" \
  --body "<PR-description>"

# Always:
git checkout main && git pull
git worktree remove <worktree-path>   # if using worktrees
```

If using the `finishing-a-development-branch` skill, follow it instead — it handles worktree cleanup automatically.

---

## 6. Board hygiene

### Status

| State       | When to set it                        |
| ----------- | ------------------------------------- |
| Todo        | Issue created, not started            |
| In Progress | Branch created, work started          |
| In Review   | PR opened                             |
| Done        | PR merged (automatic via `Closes #N`) |

### Additional fields

| Field      | Values                                  | When to set                                                             |
| ---------- | --------------------------------------- | ----------------------------------------------------------------------- |
| `Priority` | `Critical` / `High` / `Medium` / `None` | At triage or when picking the issue                                     |
| `Blocked`  | `Yes` / `No`                            | When a dependency prevents progress; note the blocker in the issue body |
| `Agent`    | e.g. `@agent_gpt`                       | When a subagent is assigned to the issue (set before starting work)     |

Rules:

- An issue with `Blocked: Yes` should not be in `In Progress` unless the blocker is being resolved
- `Agent` is cleared when the work is merged (issue closes)
- `Priority: Critical` means the issue must be resolved before any other work in the milestone

---

## Milestones

Issues are grouped into four milestones:

| Milestone                           | Target     | Scope                                |
| ----------------------------------- | ---------- | ------------------------------------ |
| Fase 1 — Esqueleto funcional        | 2026-05-30 | Auth + tasks list on web and desktop |
| Fase 2 — Gestión de tareas completa | 2026-08-15 | Full task lifecycle, boards, offline |
| Fase 3 — Integración de agentes     | 2026-10-31 | Agent execution from desktop         |
| Fase 4 — Integraciones externas     | Open       | Git sync, Jira, webhooks             |

Only work on the current milestone unless there is a specific reason to look ahead.
