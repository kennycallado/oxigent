# ADR-008: Monorepo & Dev Environment

## Status
Accepted

## Context
The project spans Rust (backend + Tauri), TypeScript (frontend packages, apps), and infrastructure (deploy). A single repository reduces cross-cutting changes from multi-repo coordination overhead. The dev environment must be reproducible across macOS and Linux.

## Options Considered

**Option A — Multi-repo (separate repos for backend, frontend, desktop)**
Clean separation, but cross-cutting changes (e.g., shared types via Typeshare) require coordinated PRs across repos. Rejected.

**Option B — Monorepo without tooling (manual coordination)**
Everything in one place, but no workspace-level dependency management. Rejected.

**Option C — Cargo workspace (Rust) + pnpm workspaces (TypeScript) + Nix flake**
Cargo workspace for Rust crates, pnpm workspaces for TypeScript packages/apps, Nix flake for reproducible toolchain pinning. Selected.

## Decision

**Repository structure:**

```
/                           # Cargo workspace root (Cargo.toml, Cargo.lock)
  backend/crates/           # Rust crates (workspace members)
  apps/
    web/                    # Vite web shell (pnpm workspace member)
    desktop/                # Vite + Tauri desktop shell (pnpm workspace member)
      src-tauri/            # Cargo workspace member (Tauri Rust backend)
  packages/                 # Shared TypeScript packages (pnpm workspace members)
  deploy/                   # Docker, scripts, infra (not a workspace member)
  docs/                     # Architecture docs, ADRs, specs, plans
  flake.nix                 # Nix flake: pins Rust toolchain, Node.js, pnpm, SurrealDB, Tauri deps
  flake.lock
  pnpm-workspace.yaml
```

**Cargo workspace** (`Cargo.toml` at root): all Rust crates share `[workspace.dependencies]` for version pinning (serde, tokio, thiserror, tracing, uuid, chrono). New crates are added as `members = ["backend/crates/<name>"]`.

**pnpm workspaces** (`pnpm-workspace.yaml`): all TypeScript packages and apps. Shared devDependencies (TypeScript, Vite, Lit, Siemens iX) declared once at workspace root.

**Nix flake** (`flake.nix`): pins Rust toolchain (via rust-overlay), Node.js, pnpm, SurrealDB CLI, Tauri system dependencies. `crane` for reproducible Rust builds with caching. `sccache` for compilation caching. Developers enter the environment via `nix develop`.

**Git worktrees strategy:** Feature branches use `git worktree` to allow parallel work without stashing. Each worktree shares the same `.git` but has an independent working tree. Useful for running the current stable branch while developing a new ADR or feature in parallel.

**Sparse registry:** `.cargo/config.toml` configures the sparse registry protocol (`sparse+https://index.crates.io/`) for faster dependency resolution.

## Consequences

**Easier:**
- `cargo build --workspace` and `pnpm -r build` build everything from root
- Typeshare codegen runs once and updates all TypeScript consumers
- Nix flake ensures exact same toolchain versions on every machine and in CI
- Worktrees allow reviewing architecture docs in one terminal while writing feature code in another

**Harder:**
- Nix has a learning curve for contributors unfamiliar with it
- pnpm workspace + Cargo workspace means two separate lock files to maintain
- Initial Nix flake setup is non-trivial (Tauri system deps vary by OS)
