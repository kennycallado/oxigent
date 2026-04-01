# ADR-002: Bounded Contexts

## Status
Accepted

## Context
A task-management platform with agent integration spans several distinct areas of responsibility. Without explicit boundaries, domain concepts leak across contexts, making the codebase harder to maintain and reason about. We need to define which contexts exist, what they own, and how they communicate.

## Options Considered

**Option A — Single context, all domain logic in one crate**
Simple initially, but collapses distinctions between task management, planning, agent execution, and identity. Leads to a big ball of mud. Rejected.

**Option B — Bounded contexts as Rust crates, synchronous direct calls**
Each context is a crate. Inter-context communication uses direct function calls through shared-kernel types. No runtime coupling, no serialization overhead. Selected.

**Option C — Bounded contexts as separate services with async messaging**
Adds operational complexity (service discovery, network partitions, distributed tracing) before any of those benefits are needed. Rejected for now; architecture allows future extraction.

## Decision

Five bounded contexts, each as a Rust crate under `backend/crates/`:

| Context | Crate | Responsibilities |
|---|---|---|
| Work Management | `work-management` | Task, subtask, workflow state, labels, priority, assignee, SLA |
| Planning | `planning` | Board, sprint/cycle, backlog ordering, capacity |
| Agent Execution | `agent-execution` | Run, session, tool invocation, prompt, output chunk, artifact, approval |
| Identity & Access | `identity-access` | User, role, permission, tenancy |
| Integrations | `integrations` | Git sync, Jira import/export, webhooks, notifications |

Shared vocabulary lives in `backend/crates/shared-kernel`: common value objects (UserId, TaskId, Timestamp, etc.), domain event base types, error types.

**Inter-context communication rules:**

1. **Synchronous coupling:** direct function calls using shared-kernel types only. A context may call another context's public application service (query handler) directly. No context accesses another context's domain internals.

2. **Asynchronous notification:** in-process domain events for fire-and-forget cross-context notification. Published via an in-process event bus (e.g. `tokio::sync::broadcast`). Consumers register handlers; no response is expected.

3. **Firm decision — CQRS in-process buses:** Each bounded context applies the CQRS pattern with in-process command bus and query bus that dispatch to handlers in memory. An in-process event bus handles domain event propagation. These are dispatchers, not infrastructure — no serialization, no network, no broker.

4. **Explicitly excluded:** external messaging infrastructure (Kafka, RabbitMQ, NATS, Redis Pub/Sub, or any broker). If a future requirement demands it, a new ADR will supersede this one for the affected context.

**Agent Execution isolation rule:** A task may reference an agent run (by ID), but `agent-execution` concepts (prompt, tool invocation, output chunk) must never appear in `work-management` domain types. The integration is a cross-context relationship expressed through shared-kernel IDs only.

**Internal module structure:** Each bounded context crate is subdivided into modules. Every module — even if only one exists in the crate — contains its own full set of layers: `domain/`, `application/`, `ports/`, `adapters/`, `projections/`. The module name is chosen by the team based on what groups naturally within the context (aggregate, subdomain, or capability). There is no prescribed rule for grouping. This vertical slice convention ensures that the codebase grows by adding modules, not by growing flat layer directories. The crate never exposes flat layer directories at the `src/` root level.

## Consequences

**Easier:**
- Each context can be understood and tested independently
- No context can accidentally corrupt another's invariants
- Future extraction to separate service is possible without rewriting domain logic

**Harder:**
- Cross-context queries require explicit projections or join services
- Team must respect context boundaries — linter/module visibility rules should enforce this
- All new domain concepts require creating a module subdirectory even if only one layer is initially needed; this prevents shortcuts that lead back to flat structure

See [ADR-001](./ADR-001-general-architecture.md) for the overall modular monolith architecture, and [ADR-004](./ADR-004-data-technology.md) for the shared data technology used by all contexts.
