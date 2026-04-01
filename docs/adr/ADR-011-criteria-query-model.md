# ADR-011: Criteria Query Model

## Status

Accepted

## Context

Every bounded context needs to expose read operations through its ports. Without a shared
abstraction, each context invents its own filtering API: some traits use method-per-field
(`find_by_email`, `find_by_role`), others use raw strings, others return everything and
filter in the application layer. This leads to:

- Inconsistent port signatures across contexts — harder to reason about and test
- Leaking infrastructure concerns into domain ports (SQL fragments, SurrealQL predicates)
- Duplicated filtering logic between the application layer and the adapter
- An impossible-to-satisfy contract when the adapter is an in-memory stub during tests

The platform will have at least five bounded contexts, each with multiple aggregates that
need queryable ports. A single, reusable query abstraction defined once in `shared-kernel`
removes the duplication and enforces a consistent boundary between application and adapter.

The abstraction must:

1. Be generic enough to work for any aggregate in any context
2. Be type-safe — field names are not strings, they are enum variants
3. Not impose any SQL or SurrealQL vocabulary on the domain layer
4. Be simple enough that in-memory stubs can implement it without a query engine

## Options Considered

**Option A — Method-per-field traits (`find_by_email`, `find_by_role`, …)**
Explicit and easy to implement. Scales poorly: each new filter criterion requires a new
trait method and a new adapter implementation. Cross-field queries (email AND role) are
impossible without combinatorial explosion. Rejected.

**Option B — Free-form string predicates**
A single `find(&self, filter: &str)` where the caller passes a SurrealQL or SQL snippet.
Flexible, but completely breaks the port/adapter boundary — the domain layer now has to
know the query language of the underlying store. Untestable with in-memory stubs without
re-implementing a query parser. Rejected.

**Option C — Generic `Criteria<F>` in `shared-kernel`**
A small, typed data structure that expresses filters as a list of `Filter<F>` triples
(field enum, operator, value string). The field type `F` is a context-local enum (e.g.
`UserField`, `TaskField`) that remains inside the bounded context. The `Criteria` type
itself is infrastructure-agnostic: adapters translate it to SurrealQL, in-memory stubs
iterate over a `Vec` and apply it. Selected.

**Option D — `QuerySpec` trait objects / dynamic dispatch**
Expressing criteria as boxed trait objects allows arbitrary composition but adds complexity
(object safety constraints, lifetime noise, harder debugging). The value over Option C is
marginal for the query patterns expected here. Rejected.

## Decision

Define the following types in `backend/crates/shared-kernel/src/criteria.rs`:

```rust
/// A set of filters and optional pagination to apply to a query.
/// `F` is a context-local field enum (e.g. `UserField`, `TaskField`).
/// All filters in `filters` are AND-conjoined — a result must satisfy every filter
/// to be included. OR conditions and nested predicates are not supported; a future
/// ADR will address them if required.
#[derive(Debug, Clone, PartialEq)]
pub struct Criteria<F> {
    pub filters: Vec<Filter<F>>,
    pub limit:   Option<u64>,
    pub offset:  Option<u64>,
}

/// A single filter triple: (field, operator, value).
/// `value` is always a `String`; adapters are responsible for parsing it into
/// the correct native type (timestamp, integer, etc.) required by `op`.
/// If parsing fails, the adapter returns `AppError` with code `E_INVALID_FILTER_VALUE`.
#[derive(Debug, Clone, PartialEq)]
pub struct Filter<F> {
    pub field: F,
    pub op:    Op,
    pub value: String,
}

/// Comparison operators available to all contexts.
/// The exact matching semantics of each operator are determined by the adapter
/// implementation — the domain layer expresses intent, not algorithm.
/// In particular, `Like` signals a substring/pattern match; whether it is
/// case-sensitive or uses full-text indexing is left to the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Eq,
    NotEq,
    Like,   // substring / pattern match — semantics determined by the adapter
    Gt,
    Lt,
    Gte,
    Lte,
}
```

`PartialEq` is derived on `Criteria<F>` and `Filter<F>` (with `F: PartialEq`) to enable
`assert_eq!` in tests.

`Criteria<F>` is re-exported from `shared-kernel::prelude` so it is available to all
crates with a single `use shared_kernel::prelude::*`.

`Criteria::new()` is the canonical constructor. An empty `Criteria` (no filters, no limit)
means "return all" — adapters are free to cap results for safety. Example usage:

```rust
let criteria = Criteria {
    filters: vec![
        Filter { field: UserField::Email, op: Op::Eq,   value: "a@b.com".into() },
        Filter { field: UserField::Role,  op: Op::NotEq, value: "admin".into() },
    ],
    limit:  Some(20),
    offset: None,
};
```

**Field enums live inside the bounded context, not in `shared-kernel`.** Example:

```rust
// backend/crates/identity-access/src/user/ports/user_search.rs
pub enum UserField {
    Email,
    Role,
    CreatedAt,
}
```

**Port signatures use `Criteria<ConcreteField>`:**

```rust
pub trait UserSearch {
    fn find(&self, criteria: Criteria<UserField>) -> Result<Vec<User>, AppError>;
}
```

**Adapter translation (SurrealDB — not prescribed here, illustrated only):**

```rust
// adapter maps Criteria<UserField> → SurrealQL WHERE clause
// domain layer never sees this translation
// see ADR-004 for SurrealDB as the chosen data technology
```

**In-memory stub (tests):**

The stub applies `Filter<F>` by extracting the relevant field value from the entity as a
string and comparing it against `filter.value`. The caller extracts the field; the filter
struct compares strings:

```rust
impl UserSearch for InMemoryUserRepository {
    fn find(&self, criteria: Criteria<UserField>) -> Result<Vec<User>, AppError> {
        let results = self.store.values()
            .filter(|u| {
                criteria.filters.iter().all(|f| {
                    let field_val: String = match f.field {
                        UserField::Email     => u.email.clone(),
                        UserField::Role      => format!("{:?}", u.role),
                        UserField::CreatedAt => u.created_at.to_string(),
                    };
                    match f.op {
                        Op::Eq    => field_val == f.value,
                        Op::NotEq => field_val != f.value,
                        Op::Like  => field_val.contains(&f.value),
                        Op::Gt    => field_val > f.value,
                        Op::Lt    => field_val < f.value,
                        Op::Gte   => field_val >= f.value,
                        Op::Lte   => field_val <= f.value,
                    }
                })
            })
            .cloned()
            .collect();
        Ok(results)
    }
}
```

The field extraction match is the in-memory stub's responsibility, not `Filter<F>`'s. This
keeps `Filter<F>` generic and infrastructure-agnostic.

**Relationship to the CQRS query bus (ADR-002):**

ADR-002 defines in-process command and query buses. `Criteria<F>` operates at the port
level, one layer below the bus. A query bus message may contain a `Criteria<F>` field
directly, or the query handler may construct a `Criteria<F>` from the message's fields
before calling the `*Search` port. This ADR does not prescribe the query bus message
format — only the port-level contract. The expected read path is:

```
API handler / Tauri command
  → query bus message (contains filter parameters)
      → QueryHandler constructs Criteria<F>
          → *Search port method
              → adapter translates to native query
```

**Typeshare implications:**

`Criteria<F>` and `Filter<F>` are generic types and are not directly Typeshare-derivable.
`Op` may derive `#[derive(Typeshare)]` if an API endpoint needs to accept filter parameters
from the frontend. Field enums that cross the wire (e.g. as part of a REST filter DTO)
should derive `#[derive(Typeshare)]` per context. API endpoints that accept filters define
a serializable DTO (e.g. `FilterDto { field: String, op: Op, value: String }`) that the
application layer maps to `Criteria<F>`. Whether a given context exposes filter parameters
to the frontend is decided per endpoint, not here.

See [ADR-007](./ADR-007-frontend-backend-api-contract.md) for the Typeshare pipeline.

**Sort order:**

`Criteria<F>` does not include a sort field. Deterministic offset-based pagination requires
a consistent sort; adapters must provide a default sort by convention (e.g. creation date
descending). An `OrderBy<F>` field may be added to `Criteria<F>` in a future amendment if
explicit sort control is required.

## Consequences

**Easier:**

- One query abstraction works across all bounded contexts and all adapters
- Adding a new filter criterion only requires adding an enum variant — no new trait methods
- In-memory stubs are straightforward to implement without a query engine
- Application layer is completely decoupled from storage query language
- `PartialEq` on `Criteria<F>` enables assertion-based testing of filter construction

**Harder:**

- Complex queries (joins, aggregations, nested ORs) cannot be expressed in `Criteria<F>`
  without extending `Op` or adding a new abstraction — open question for future ADR
- `Filter::value` is `String`-typed, so adapters must parse values into the correct native
  type (timestamp, number) for comparison operators. Invalid values must return `AppError`
  with code `E_INVALID_FILTER_VALUE`
- `Criteria<F>` has no sort field; adapters impose a default sort order, which must be
  documented per adapter to avoid surprises with paginated results
- Adapter authors must write translation logic from `Criteria<F>` to their query language;
  this is not provided centrally
- Field enums must be kept in sync with the actual fields of the aggregate; stale enum
  variants will compile but produce empty result sets at runtime

See [ADR-002](./ADR-002-bounded-contexts.md) for the bounded context and port/adapter
structure this model operates within.
See [ADR-004](./ADR-004-data-technology.md) for SurrealDB as the primary adapter target.
See [ADR-007](./ADR-007-frontend-backend-api-contract.md) for the Typeshare pipeline that
may propagate `Op` and field enums to TypeScript.
See [ADR-009](./ADR-009-error-handling.md) for the `AppError` type returned by all port
methods, including `E_INVALID_FILTER_VALUE`.
See [ADR-012](./ADR-012-port-naming-conventions.md) for the naming rules that govern the
traits that accept `Criteria<F>`.
