# Architecture Overview

> This is a living document. It links to ADRs for decisions and rationale. Update diagrams when the architecture changes; add a new ADR when a decision changes.

## System at a Glance

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Client Layer                                 │
│                                                                     │
│   ┌───────────────────────────┐   ┌───────────────────────────┐    │
│   │       apps/web            │   │      apps/desktop          │    │
│   │  Vite shell               │   │  Tauri 2 shell             │    │
│   │  platform-web adapters    │   │  platform-desktop adapters │    │
│   │  (HTTP/WS)                │   │  (Tauri commands/channels) │    │
│   └─────────────┬─────────────┘   └─────────────┬─────────────┘    │
│                 │                               │                   │
│   ┌─────────────▼───────────────────────────────▼─────────────┐    │
│   │                   Frontend Shared Core                     │    │
│   │                                                            │    │
│   │  packages/ui          Lit base components, iX wrappers    │    │
│   │  packages/app-core    Ports, shared types, app services   │    │
│   │  packages/features/*  tasks, boards, planning, agents...  │    │
│   └────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                 │ REST (web)                 │ Tauri commands (desktop)
                 │                           │ (in-process, same handlers)
┌────────────────▼───────────────────────────▼─────────────────────────┐
│                    Backend — Rust Modular Monolith                    │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  backend/crates/api        HTTP server (REST endpoints)         │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  ┌───────────────┐ ┌──────────┐ ┌────────────────┐ ┌─────────────┐  │
│  │work-management│ │ planning │ │agent-execution │ │identity-    │  │
│  │               │ │          │ │                │ │access       │  │
│  │ domain/       │ │ domain/  │ │ domain/        │ │             │  │
│  │ application/  │ │ applic./ │ │ application/   │ │ domain/     │  │
│  │ ports/        │ │ ports/   │ │ ports/         │ │ application/│  │
│  │ adapters/     │ │ adapters/│ │ adapters/      │ │ ports/      │  │
│  │ projections/  │ │ projec./ │ │ projections/   │ │ adapters/   │  │
│  └───────────────┘ └──────────┘ └────────────────┘ └─────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  backend/crates/integrations   git sync, Jira, webhooks         │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  backend/crates/shared-kernel   IDs, base event types, errors   │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────┬───────────────────────────┘
                                            │
                      ┌─────────────────────▼──────────────────────┐
                      │               SurrealDB                    │
                      │                                            │
                      │  Embedded (Tauri desktop, in-process)      │
                      │  Server mode (cloud, single-node → cluster)│
                      │                                            │
                      │  Multi-model: document, graph, relational, │
                      │  vector (HNSW), full-text (BM25)           │
                      └────────────────────────────────────────────┘
```

## Module Map

| Module | Path | ADR |
|---|---|---|
| General Architecture | — | [ADR-001](../adr/ADR-001-general-architecture.md) |
| Bounded Contexts | `backend/crates/` | [ADR-002](../adr/ADR-002-bounded-contexts.md) |
| Shared Kernel | `backend/crates/shared-kernel` | [ADR-002](../adr/ADR-002-bounded-contexts.md) |
| Technology Stack | — | [ADR-003](../adr/ADR-003-technology-stack.md) |
| Data Technology | SurrealDB | [ADR-004](../adr/ADR-004-data-technology.md) |
| Offline & Sync | Outbox pattern | [ADR-005](../adr/ADR-005-offline-sync-strategy.md) |
| Frontend Architecture | `packages/` | [ADR-006](../adr/ADR-006-frontend-architecture.md) |
| API Contract | `backend/crates/api` | [ADR-007](../adr/ADR-007-frontend-backend-api-contract.md) |
| Monorepo & Dev Env | root config files | [ADR-008](../adr/ADR-008-monorepo-dev-environment.md) |

## Backend Architecture

Each bounded context crate follows the same internal layering (target layout — the current scaffold may have flat files; this is the intended structure):

```
backend/crates/<context>/
  src/
    domain/
      mod.rs              # Aggregates, entities, value objects
      events.rs           # Domain events (shared-kernel base types)
    application/
      commands/           # Command structs + CommandHandler impls
      queries/            # Query structs + QueryHandler impls
    ports/
      mod.rs              # Repository traits, bus traits, external service traits
    adapters/
      surreal.rs          # SurrealDB repository implementations
      http.rs             # HTTP adapter (if context exposes its own endpoints)
    projections/
      mod.rs              # Read models, materialized views
    lib.rs
```

Cross-context communication: direct function calls via shared-kernel types for synchronous queries; in-process event bus (`tokio::sync::broadcast`) for fire-and-forget domain event notifications. No external broker. See [ADR-002](../adr/ADR-002-bounded-contexts.md).

## Frontend Architecture

```
packages/app-core/src/
  ports/                  # TypeScript interfaces (contracts)
  types/                  # Shared domain value types (from Typeshare codegen)
  services/               # Application service implementations (runtime-agnostic)
  generated/              # Auto-generated by Typeshare — do not edit manually

packages/platform-web/src/
  adapters/               # HTTP, WebSocket, browser auth, browser storage

packages/platform-desktop/src/
  adapters/               # Tauri commands, channels, filesystem, keyring, local SurrealDB

apps/web/src/
  main.ts                 # Composition root: instantiate platform-web adapters, inject into features

apps/desktop/src/
  main.ts                 # Composition root: instantiate platform-desktop adapters, inject into features
```

Dependency rule: features → app-core ports only. Never feature → platform-*. See [ADR-006](../adr/ADR-006-frontend-architecture.md).

## Data Flow: Write (Mutation)

```
User action in Lit component
  → Feature application service (app-core)
  → Port: CommandBus.dispatch(command)
     ┌── Web runtime: HTTP POST /v1/<resource> → backend/crates/api
     └── Desktop runtime: Tauri command v1_<resource> → same Rust handler
          → CommandHandler (application layer, context crate)
            → Aggregate.apply(command) → domain event emitted
            → Repository.save(aggregate) → SurrealDB write (ACID)
            → In-process event bus publishes domain event
              → Projection updaters subscribe and update read models
```

## Data Flow: Read (Query)

```
Component requests data
  → Feature application service (app-core)
  → Port: QueryBus.dispatch(query)
     ┌── Web runtime: HTTP GET /v1/<resource> → backend/crates/api
     │    OR SurrealDB SDK direct read (with row-level security)
     └── Desktop runtime: Tauri command v1_<resource>
          OR SurrealDB embedded SDK direct read
          → QueryHandler → Projection/read model → SurrealQL SELECT
```

## Desktop-Specific: Agent Execution

```
User triggers agent run
  → agents feature → AgentRuntimeGateway port
     → TauriAgentExecutor adapter
       → Tauri externalBin sidecar (e.g. claude-code)
         → stdout/stderr streamed via Tauri channel (not events)
           → agent-execution context receives chunks
             → persists to SurrealDB embedded
               → live query subscription updates UI in real time
```

See [ADR-003](../adr/ADR-003-technology-stack.md) for Tauri capabilities constraints. The agent execution flow uses Tauri commands — same transport contract as all other desktop operations; see [ADR-007](../adr/ADR-007-frontend-backend-api-contract.md).

## Offline Sync Flow (Desktop)

```
Write while offline:
  CommandHandler → local outbox table (SurrealDB embedded)
    → sync worker (background Tokio task)
      → on connectivity: POST to server API with idempotency key
        → server applies, returns aggregate_version + events
          → desktop applies events to local replica idempotently

Server → Desktop push:
  Server writes domain events to outbox table
    → sync worker polls outbox
      → pushes to desktop client (WebSocket or live query on single-node)
        → desktop applies events to local replica
```

See [ADR-005](../adr/ADR-005-offline-sync-strategy.md).
