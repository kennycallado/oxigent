# ADR-007: Frontend–Backend API Contract

## Status
Accepted

## Context
The frontend has two runtimes (web, desktop) and two access paths to the backend (HTTP REST and Tauri commands). SurrealDB can also be accessed directly from clients via its SDK. We need clear rules about which path is used for which operations, and how types are shared.

## Options Considered

**Option A — Direct SurrealDB SDK access from frontend for all operations**
Minimal API surface, fast prototyping. But bypasses all business rules, domain validation, and auth logic. Rejected for writes; conditionally accepted for reads with row-level security.

**Option B — REST API only, no direct DB access**
Maximum control. Adds latency for read-heavy views where direct DB queries with row-level security are sufficient. Rejected as the only path.

**Option C — Two access paths with strict rules per operation type**
REST/Tauri commands for all mutations and auth. SurrealDB SDK for reads/subscriptions only, with DB-enforced row-level security. Selected.

## Decision

**Two access paths with a strict boundary:**

**REST API** (`backend/crates/api` crate, web transport):
- Mandatory for: all mutations (create, update, delete), operations involving business rules or domain invariants, auth and session orchestration, multi-step workflows.
- Web shell calls over HTTP. Desktop shell short-circuits HTTP via Tauri commands calling the same Rust handlers in-process.

**SurrealDB SDK** (direct client connection):

- **Desktop (embedded):** SDK direct access fully permitted for both reads and live query subscriptions. The database is local to the user's machine; no network boundary exists.

- **Web (server mode):** SDK direct access permitted **only for explicitly declared public tables**. Rules:
  1. A table is "public" only if it is listed in a `-- @public-read` comment block at the top of the corresponding SurrealQL schema file (e.g. `backend/schema/<table>.surql`).
  2. SurrealDB session auth must be fully configured before any SDK connection is opened from the web client. The web adapter (in `packages/platform-web`) is responsible for establishing and refreshing the session token.
  3. All writes and all business-logic reads (joins, projections, aggregations that enforce domain invariants) must go through the REST API even for public tables.

- **Forbidden for:** any write operation from the frontend. Frontend must never bypass the REST/Tauri layer for writes, regardless of table visibility.

**Desktop short-circuit:** The desktop shell does not make HTTP calls to itself. Tauri command handlers call the same Rust application service functions that the REST handlers call. Same business logic, two transports.

**Semantic parity:** Every REST endpoint has an equivalent Tauri command with identical request/response types. One schema, two transports.

**Type sharing:** Rust types in the `api` crate are annotated with `#[derive(Typeshare)]`. The Typeshare CLI generates TypeScript types into `packages/app-core/src/generated/`. SurrealDB schema types are defined in SurrealQL schema files and documented separately.

**CI enforcement:** Every CI run executes `typeshare` and then checks that the working tree is clean (`git diff --exit-code packages/app-core/src/generated/`). The build fails if generated types are stale. Developers must re-run `typeshare` locally before pushing any commit that changes `#[derive(Typeshare)]`-annotated types.

**Versioning:**
- REST: URL-based (`/v1/...`)
- Tauri commands: same version by convention (command names prefixed `v1_`)

See [ADR-006](./ADR-006-frontend-architecture.md) for the frontend package structure that consumes this contract.

---

*Amendment 2026-04-01: Tightened web SurrealDB SDK rules (explicit public-table declaration required; session auth precondition; `-- @public-read` annotation convention). Added CI enforcement for Typeshare codegen freshness. See git log for full diff.*

## Consequences

**Easier:**
- All mutations go through domain validation — no way for frontend to write invalid state directly to DB
- Desktop and web share type definitions — one change propagates to both
- Row-level security in SurrealDB provides a second layer of read access control

**Harder:**
- Every new operation needs both a REST endpoint and a Tauri command (same handler, two registrations)
- Typeshare requires running the codegen step when Rust API types change; CI enforces freshness via `git diff --exit-code` on `packages/app-core/src/generated/`
- SurrealDB schema types and Typeshare-generated types must stay in sync manually
- Web SDK direct access requires each public-read table to be explicitly declared with a `-- @public-read` annotation in its SurrealQL schema file; undeclared tables stay private
