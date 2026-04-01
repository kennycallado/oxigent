# ADR-001: General Architecture

## Status
Accepted

## Context
The project requires a kanban/task-management application (similar to Jira) with two distinct access points: a web browser client and a native desktop client. The desktop client adds agent CLI integration (Claude Code, Codex, etc.) not available in the browser context. Both surfaces must share the same domain logic and UI components to avoid divergence and duplication.

## Options Considered

**Option A — Two independent apps (web SPA + Tauri app)**
Simpler to start, but leads to duplicated domain logic, divergent UIs, and doubled maintenance cost. Rejected.

**Option B — Modular monolith backend + shared frontend core with two shells**
Single backend with bounded contexts as Rust crates. Single frontend core (shared UI components, application services, ports) with two thin shells (web via Vite, desktop via Tauri 2). Each shell provides only platform-specific adapters. Selected.

**Option C — Microservices**
Premature for current scale and team size. Adds operational complexity with no benefit at this stage. Rejected.

## Decision

Modular monolith Rust backend + shared frontend core (Lit + Siemens iX) + two shells (web via Vite, desktop via Tauri 2).

```
┌──────────────────────────────────────────────────────┐
│               Frontend Shared Core                   │
│  packages/ui  |  packages/app-core  |  packages/features/*  │
│  Lit + Siemens iX + TypeScript + ports               │
└──────────────────┬───────────────────────────────────┘
                   │
      ┌────────────┴────────────┐
      │                         │
┌─────▼──────────┐     ┌────────▼────────────┐
│  apps/web       │     │  apps/desktop        │
│  Vite shell     │     │  Tauri 2 shell       │
│  platform-web   │     │  platform-desktop    │
│  adapters       │     │  adapters            │
│  HTTP/WS        │     │  Tauri commands +    │
└─────┬───────────┘     │  channels + sidecars │
      │                 └────────┬────────────┘
      │                          │
      └──────────┬───────────────┘
                 │
┌────────────────▼─────────────────────────────────────┐
│            Backend — Rust Modular Monolith            │
│                                                      │
│  backend/crates/shared-kernel   (shared vocabulary)  │
│  backend/crates/work-management                      │
│  backend/crates/planning                             │
│  backend/crates/agent-execution                      │
│  backend/crates/identity-access                      │
│  backend/crates/integrations    (git, Jira, webhooks)│
│  backend/crates/api             (delivery crate)     │
└────────────────┬─────────────────────────────────────┘
                 │
┌────────────────▼─────────────────────────────────────┐
│                    SurrealDB                         │
│  Embedded (desktop) | Server mode (cloud)            │
└──────────────────────────────────────────────────────┘
```

Each backend crate follows DDD + CQRS + hexagonal layering:
- `domain/` — entities, aggregates, value objects, domain events
- `application/` — command handlers, query handlers
- `ports/` — repository traits, bus traits, external service traits
- `adapters/` — SurrealDB, HTTP, Tauri bridge implementations
- `projections/` — read models optimized for queries

ADR immutability convention: once an ADR is marked Accepted, it is immutable. Changes require a new ADR that supersedes this one. An ADR may be Accepted before its decision is fully implemented.

See [ADR-002](./ADR-002-bounded-contexts.md) for bounded context definitions, [ADR-003](./ADR-003-technology-stack.md) for the technology choices, [ADR-004](./ADR-004-data-technology.md) for the data technology decision, and [ADR-006](./ADR-006-frontend-architecture.md) for frontend package structure.

## Consequences

**Easier:**
- Single source of truth for domain logic
- UI components work identically in both shells
- New features ship to both surfaces simultaneously
- Each bounded context can be extracted to a separate service if needed later

**Harder:**
- Port/adapter discipline must be enforced — no shortcuts importing adapters directly from feature code
- Shell setup is more involved than a simple Vite SPA
