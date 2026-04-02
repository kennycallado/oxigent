# ADR-005: Offline & Sync Strategy

## Status

Accepted

## Context

The desktop application must work offline and sync with the server when connectivity is restored. SurrealDB live queries (as of v3.0) are limited to single-node deployments, so a sync mechanism that does not depend on live queries is required for distributed server setups.

## Options Considered

**Option A — Full local replica, CRDT-based merge**
Maximum offline capability, automatic conflict resolution. High complexity, requires CRDT types for every domain aggregate. Premature for current scope. Rejected for now; noted as future path for collaborative editing.

**Option B — No offline support**
Simplest, but desktop loses its main differentiator over web. Rejected.

**Option C — Server-authoritative partial replica with outbox pattern**
Desktop maintains a read replica of data relevant to the current user/workspace. Writes go to a local outbox and are reconciled with the server when online. Server is always the authority. Selected.

## Decision

**Firm decisions:**

- **Server-authoritative:** The server's state is always correct. The desktop replica is a cache + pending write queue.
- **Idempotency keys:** Every command sent from desktop to server carries a client-generated idempotency key (UUID v7). The server ignores duplicate commands with the same key.
- **Conflict resolution:** Last-write-wins based on server timestamp. The server assigns a monotonic `updated_at` timestamp to every aggregate write; this is the authoritative ordering.
- **Vector clocks:** Maintained on the desktop for causal ordering tracking and debugging/audit. Not used for conflict resolution — that is LWW only.

**Operational hypothesis:**

- **Outbox pattern for server → desktop push:** Server writes domain events to an outbox table. A sync worker (a background Tokio task inside `backend/crates/api`, started at server boot) polls the outbox and pushes changes to connected desktop clients (via WebSocket or SurrealDB live queries on single-node). Desktop applies events to its local replica idempotently.
- **Outbox pattern for desktop → server reconciliation:** Desktop stores pending commands in a local outbox table (SurrealDB embedded). A sync worker (a background Tokio task inside `apps/desktop/src-tauri`) retries them against the server API with exponential backoff and idempotency key protection.

**Watch items:**

- Migrate server → desktop push to SurrealDB live queries once multi-node live query support lands in SurrealDB.
- Evaluate CRDTs for collaborative editing scenarios (multiple users editing the same board/task simultaneously).

See [ADR-004](./ADR-004-data-technology.md) for SurrealDB live query limitations that motivate this strategy.

## Consequences

**Easier:**

- Desktop works fully offline for reads and captures writes to retry later
- Server is never wrong — no merge conflicts in the strong sense
- Idempotency keys make retry logic safe and simple

**Harder:**

- Stale reads on desktop until next sync cycle
- Outbox table requires polling or event notification to drive the sync worker
- Agent execution sessions complicate sync (large streaming outputs, partial state)
