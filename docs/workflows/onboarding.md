# Onboarding

Getting the development environment running from scratch.

---

## Prerequisites

The project uses [Nix](https://nixos.org/) with flakes to manage the development environment.
All tools (Rust, Node, pnpm, Tauri dependencies) are provided by the flake — no manual
installation of toolchains required.

**Required:**

- Nix with flakes enabled ([install guide](https://nixos.org/download))
- [direnv](https://direnv.net/) (recommended, to auto-load the environment)

**To enable flakes** if you have Nix but not flakes:

```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

---

## Clone and enter the environment

```bash
git clone git@github.com:kennycallado/oxigent.git
cd oxigent
```

**With direnv (recommended):**

```bash
direnv allow
# .envrc is already committed — just allow it once.
# The environment loads automatically on every cd into the directory.
```

**Without direnv:**

```bash
nix develop
# You are now inside the dev shell with all tools available
```

The first run downloads and compiles dependencies — this takes a few minutes. Subsequent
runs are instant thanks to sccache.

---

## What the flake provides

| Tool                      | Purpose                            |
| ------------------------- | ---------------------------------- |
| Rust (stable, latest)     | Backend compiler                   |
| rust-analyzer             | LSP for editors                    |
| cargo-watch               | Auto-recompile on file change      |
| cargo-edit                | `cargo add` / `cargo rm`           |
| bacon                     | Background cargo check runner      |
| Node.js 22                | Frontend runtime                   |
| pnpm                      | Frontend package manager           |
| bun                       | Fast JS runtime (scripts)          |
| sccache                   | Rust build cache                   |
| webkitgtk + wrapGAppsHook | Tauri desktop dependencies (Linux) |

---

## Install frontend dependencies

```bash
pnpm install
```

---

## Verify everything works

```bash
# Backend: compile + test
cargo build
cargo test

# Backend: lint
cargo clippy

# Frontend: build
pnpm build

# Frontend: lint
pnpm lint
```

All four commands should exit 0 on a clean checkout.

---

## Running in development

**Backend only:**

```bash
cargo watch -x run
```

**Frontend web (hot reload):**

```bash
pnpm --filter apps/web dev
```

**Desktop (Tauri):**

```bash
pnpm --filter apps/desktop tauri dev
```

The desktop command starts the Tauri dev server, which compiles the Rust backend and the
frontend together and opens the app window.

---

## Project structure at a glance

```
oxigent/
  backend/
    crates/
      api/                  # HTTP server (REST endpoints)
      shared-kernel/        # IDs, base events, AppError
      work-management/      # Task, workflow, labels
      planning/             # Board, sprint, backlog
      agent-execution/      # Agent runs, sessions, output
      identity-access/      # User, role, auth
      integrations/         # Git sync, Jira, webhooks
  packages/
    ui/                     # Siemens iX wrappers, Lit base components
    app-core/               # Ports, shared types, application services
    features/               # tasks/, boards/, agents/, projects/, settings/
    platform-web/           # HTTP/WS adapters
    platform-desktop/       # Tauri command/channel adapters
  apps/
    web/                    # Vite shell (web)
    desktop/                # Tauri shell (desktop)
  docs/
    adr/                    # Architecture Decision Records (ADR-001 → ADR-010)
    architecture/           # overview.md — system diagram and module map
    workflows/              # This directory
```

For architecture decisions and rationale, start with
[docs/architecture/overview.md](../architecture/overview.md).

---

## Useful commands reference

| Command                   | What it does                                      |
| ------------------------- | ------------------------------------------------- |
| `cargo build`             | Compile backend                                   |
| `cargo test`              | Run backend tests                                 |
| `cargo clippy`            | Lint backend                                      |
| `cargo watch -x run`      | Run backend with auto-reload                      |
| `pnpm install`            | Install frontend dependencies                     |
| `pnpm build`              | Build all frontend packages                       |
| `pnpm lint`               | Lint all frontend packages                        |
| `pnpm --filter <pkg> dev` | Dev server for a specific package                 |
| `nix build`               | Build the full project via Nix                    |
| `nix flake check`         | Run all Nix checks (build + clippy + fmt + tests) |
