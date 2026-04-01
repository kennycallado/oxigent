# ADR-004: Data Technology

## Status
Accepted

## Context
The application requires a database that works both embedded (in-process, for the Tauri desktop app) and as a standalone server (for cloud/web deployment). It also needs to support relational queries, graph relationships (task dependencies, agent memory), vector search (semantic agent memory), and full-text search — without adding separate infrastructure components for each capability.

## Options Considered

**Option A — PostgreSQL + pgvector + Elasticsearch**
Proven stack, excellent operational tooling. But requires three separate systems, complex sync between embedded and server modes. Rejected.

**Option B — PostgreSQL server + SurrealDB embedded (desktop only)**
PostgreSQL as system of record, SurrealDB only for the local agent workspace on desktop. Reduces SurrealDB blast radius. Viable, but introduces two data models and a sync translation layer. Considered as fallback.

**Option C — SurrealDB 3.0 as sole database**
Single system for both embedded (desktop) and server (cloud) modes. Multi-model: document, graph, relational, vector (HNSW), full-text search (BM25). One data model, one query language (SurrealQL), no translation layer. Selected.

## Decision

**Firm decision:** SurrealDB 3.0 as the sole database for all contexts and deployment modes.

- **Embedded mode:** SurrealDB Rust crate runs in-process within the Tauri desktop application. Storage backend: SurrealKV (persistent) for production, in-memory for tests.
- **Server mode:** SurrealDB server process for cloud/web deployment. Single-node initially.
- **Multi-model capabilities used:** document storage, graph edges (task dependencies, agent session graphs), vector search via HNSW (agent memory embeddings), full-text search via BM25.
- **ACID transactions** for command-side writes. Row-level security via SurrealDB `DEFINE TABLE ... PERMISSIONS`.
- **Schema:** schemafull for core domain aggregates (enforced via `DEFINE TABLE ... SCHEMAFULL`), schemaless allowed for agent artifact payloads.

**Operational hypothesis:** Live queries (`LIVE SELECT`) available for single-node deployments. Used for real-time projections in web and desktop clients connected to a single SurrealDB node. See [ADR-005](./ADR-005-offline-sync-strategy.md) for the sync strategy when live queries are not available.

**Watch item:** In-database ML inference via SurrealML. Not a near-term requirement. Will be evaluated when agent embedding use cases are implemented.

## Consequences

**Easier:**
- Single data model for embedded and server modes
- No separate vector DB or search engine to operate
- SurrealQL covers graph traversal, full-text, and vector similarity in one query
- Desktop development uses the same database driver as server

**Harder:**
- SurrealDB is less mature than PostgreSQL — fewer operational tools, smaller community
- Live queries limited to single-node: horizontal scaling requires the outbox pattern (see [ADR-005](./ADR-005-offline-sync-strategy.md))
- Multi-node SurrealDB deployments are not yet production-proven for all workloads
