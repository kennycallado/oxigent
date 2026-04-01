# ADR-012: Port Naming Conventions

## Status

Accepted

## Context

Every bounded context exposes its capabilities through port traits in `ports/`. These traits
are the only legal coupling point between the application layer and infrastructure adapters.
As the number of contexts grows (currently five, potentially more), inconsistent naming
creates confusion:

- A developer reading `UserRepository` does not know whether it supports writes, reads, or
  both without reading the full trait definition.
- Two contexts naming the same concept differently (`UserStore` vs `UserRepository` vs
  `UserDao`) signals ambiguity that should not exist.
- Without a rule for decomposing a port, traits accumulate methods over time until they
  become impossible to mock efficiently in tests.

The goal is a small set of suffixes that signal intent immediately: what kind of operation
does this trait represent, and in which direction does data flow?

## Options Considered

**Option A — Single `*Repository` trait per aggregate (read + write combined)**
Traditional DDD style. Familiar to most developers. The downside is that read-heavy and
write-heavy operations are always coupled: a use case that only reads must still depend on
a type that knows how to write. In-memory test doubles are harder to keep minimal. Rejected
as the sole pattern; accepted as a convenience alias (see Decision).

**Option B — CQRS-aligned `*CommandStore` / `*QueryStore`**
Matches the CQRS bus terminology already in ADR-002. However, "store" is ambiguous (is it a
cache? a DB? a file?) and "command/query" duplicates the bus vocabulary unnecessarily at the
port level. Rejected.

**Option C — Role-based suffixes: `*Registry`, `*Search`, `*Notifier`, `*Client`**
Each suffix has a single, unambiguous definition. A trait's purpose is declared by its name
before a developer reads a single method. Composed aliases (`*Repository`) are permitted as
opt-in conveniences. Selected.

**Option D — Verb-first naming (`Writer`, `Reader`, `Sender`)**
Common in some ecosystems (Go interfaces). Less readable in Rust where traits are typically
noun-first, and "Reader" / "Writer" collide with `std::io`. Rejected.

## Decision

### Suffix rules

| Suffix      | Direction     | Responsibility                                                       |
| ----------- | ------------- | -------------------------------------------------------------------- |
| `*Registry` | Inbound write | Persisting or removing an aggregate: `save`, `delete`                |
| `*Search`   | Inbound read  | Querying aggregates: accepts `Criteria<F>`, returns collections      |
| `*Finder`   | Inbound read  | Fetching a single aggregate by identity: `find_by_id`                |
| `*Notifier` | Outbound      | Sending a notification to an external channel (email, push, webhook) |
| `*Client`   | Outbound      | Calling an external service (HTTP, gRPC, CLI sidecar)                |

**`*Registry`** — write port. Methods: `save(&self, entity: &T) -> Result<(), AppError>`,
`delete(&self, id: &Id) -> Result<(), AppError>`. Must not contain read methods.
(see [ADR-009](./ADR-009-error-handling.md) for the `AppError` type).

**`*Search`** — read port for collection queries. Accepts `Criteria<F>` as defined in
[ADR-011](./ADR-011-criteria-query-model.md). Returns `Result<Vec<T>, AppError>`.
May include a `count` method that accepts the same `Criteria<F>` and returns `u64`.

**`*Finder`** — read port for single-entity lookup by known identity. Avoids `Criteria`
overhead for the common `find_by_id` case. Returns `Result<Option<T>, AppError>`.
A context may omit `*Finder` and use `*Search` with an `Id` filter if preferred — both are
valid; consistency within a context matters more than uniformity across contexts.

**`*Notifier`** — outbound side-effect port. Methods are fire-and-forget from the
application layer's perspective; errors are returned but callers typically log and continue.
Example: `EmailNotifier::send_welcome(&self, user: &User) -> Result<(), AppError>`.

**`*Client`** — outbound port for external service calls where a response is expected.
Example: `GitProviderClient::fetch_commits(&self, repo: &RepoUrl) -> Result<Vec<Commit>, AppError>`.

### Composed aliases

A composed trait alias is permitted when a use case consistently needs both read and write
access, or when a caller dependency on two ports would be cumbersome:

```rust
/// Convenience alias — do not add methods here.
pub trait UserRepository: UserRegistry + UserFinder + UserSearch {}
```

`UserRepository` has no methods of its own. It is implemented as a blanket impl:

```rust
impl<T> UserRepository for T where T: UserRegistry + UserFinder + UserSearch {}
```

Composed aliases must not add methods. If a method does not fit an existing suffix role,
either the suffix table needs extension (new ADR amendment) or the method belongs in
application logic, not a port.

### File layout

One file per trait inside `ports/`. This refines the `ports/mod.rs` convention from
[ADR-002](./ADR-002-bounded-contexts.md) — `mod.rs` now only contains `pub mod`
declarations, not trait definitions:

```
src/<module>/ports/
  mod.rs                  # pub mod declarations
  <aggregate>_registry.rs # *Registry trait
  <aggregate>_search.rs   # *Search trait
  <aggregate>_finder.rs   # *Finder trait (if used)
  <aggregate>_repository.rs # composed alias (optional)
  password_hasher.rs      # domain service port (no suffix rule — use descriptive name)
```

Traits that are domain services (e.g. `PasswordHasher`, `TokenSigner`) do not require a
suffix from the table above. Their name describes the capability, not a storage role.

A port qualifies as a domain service port (and is exempt from suffix rules) when **both**
conditions hold: (1) its methods do not persist or retrieve aggregates, and (2) it does not
call external services over the network. If either condition is false, it must use a suffix
from the table. When in doubt, use a suffix.

### Naming inside `adapters/`

Adapters implementing these traits follow the pattern `<Technology><Aggregate><Role>`:

```
SurrealDbUserRepository    // implements UserRepository (all three)
InMemoryUserRepository     // test double, implements UserRepository
SmtpEmailNotifier          // implements EmailNotifier
```

## Consequences

**Easier:**

- A developer reading a use case struct's fields knows immediately what each port does from
  its name alone — no need to navigate to the trait definition
- Single-responsibility ports are trivial to mock in tests: a use case that only reads
  receives a `MockUserSearch`, not a full `MockUserRepository`
- Onboarding: new team members learn five suffixes and can read any port in the codebase

**Harder:**

- A use case that needs reads and writes must accept two (or three) port parameters instead
  of one — slightly more verbose constructor injection
- The suffix table is not exhaustive; novel port types (e.g. a streaming port) require an
  ADR amendment before a suffix can be used consistently
- Existing code in other Rust projects often uses `Repository` as a catch-all; contributors
  from those projects must unlearn the habit

See [ADR-002](./ADR-002-bounded-contexts.md) for the vertical slice module structure that
hosts these ports.
See [ADR-011](./ADR-011-criteria-query-model.md) for the `Criteria<F>` type used by
`*Search` traits.
