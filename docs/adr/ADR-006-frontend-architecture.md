# ADR-006: Frontend Architecture

## Status
Accepted

## Context
The frontend must serve two runtimes (browser and Tauri webview) without duplicating feature code. Platform-specific capabilities (filesystem, secure secrets, agent execution, local database) must be available on desktop but absent on web, without polluting shared feature code with `if (isTauri)` guards.

## Options Considered

**Option A — Monolithic frontend, runtime checks everywhere**
Single package, `if (isTauri)` branches throughout feature code. Simple to start, rapidly becomes unmaintainable. Rejected.

**Option B — Two separate frontend apps sharing some utilities**
Leads to divergent UIs and duplicated business logic. Rejected.

**Option C — Shared core with platform ports/adapters and two thin shells**
Feature code depends only on ports (TypeScript interfaces). Shell apps provide adapter implementations at startup via dependency injection. Feature code is runtime-agnostic. Selected.

## Decision

Package structure (already scaffolded in the monorepo):

```
packages/
  ui/                   # Siemens iX wrappers, shared Lit base components, design tokens
  app-core/             # Ports (TS interfaces), shared types, application services, routing contracts
  features/
    tasks/              # Task management feature (depends only on app-core ports)
    boards/             # Board/kanban feature
    agents/             # Agent session monitoring feature
    projects/           # Project/space management
    settings/           # User/workspace settings
  platform-web/         # Adapters: HTTP/WS, browser auth, browser storage
  platform-desktop/     # Adapters: Tauri commands/channels, filesystem, secure secrets, local DB

apps/
  web/                  # Web shell: composes platform-web adapters, injects into features at startup
  desktop/              # Desktop shell: composes platform-desktop adapters, injects into features at startup
```

> Note: `packages/features/` packages reflect the current scaffold. A `planning` feature package will be added when the Planning bounded context is implemented.

**Dependency rule (enforced):** Feature packages (`packages/features/*`) may only import from `packages/app-core` and `packages/ui`. They must never import from `packages/platform-web` or `packages/platform-desktop`. Violations should be caught by TypeScript path aliases and linter rules.

**Desktop-specific adapters** (defined as ports in `app-core`, implemented in `platform-desktop`):

| Port | Desktop Adapter | Capability |
|---|---|---|
| `AgentRuntimeGateway` | `TauriAgentExecutor` | Runs CLI sidecars (Claude Code, Codex) via Tauri `externalBin` |
| `LocalWorkspaceAdapter` | `TauriFilesystemAdapter` | Reads/writes local filesystem, Git repo artefacts |
| `LocalStorageAdapter` | `TauriStoreAdapter` | Persistent local cache, offline command queue |
| `SecureSecretsAdapter` | `TauriKeyringAdapter` | API keys, credentials via OS keychain |

**Dependency injection:** Shell applications (`apps/web`, `apps/desktop`) are the composition root. They instantiate adapter implementations and inject them into feature modules at startup. `app-core` defines ports only — it has no knowledge of any adapter implementation.

**app-core does not re-export platform adapters.** The dependency direction is strictly: features → app-core ports ← platform-* adapters. Shells compose and wire.

See [ADR-007](./ADR-007-frontend-backend-api-contract.md) for the API contract that governs how these adapters communicate with the backend.

## Consequences

**Easier:**
- Features can be developed and tested in isolation with mock adapters
- Adding a new platform (e.g., mobile) requires only a new platform-* package and a new shell
- Desktop adapters can change implementation (e.g., swap keychain library) without touching feature code

**Harder:**
- Every new platform capability requires defining a port first
- Composition root (shell startup) grows as features are added — needs discipline to stay clean
