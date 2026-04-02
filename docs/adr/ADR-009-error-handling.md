# ADR-009: Error Handling Strategy

## Status

Accepted

## Context

The system spans Rust backend, TypeScript frontend (Lit components), two transports (REST
and Tauri commands), and two runtimes (web, desktop). Errors must:

1. Carry enough information for the frontend to display a localised, user-facing message.
2. Be structured enough for the frontend to branch on error type without string-matching.
3. Not leak internal stack traces or DB query details to web clients.
4. Map predictably to HTTP status codes so API consumers (including tests) can rely on them.

Without a documented convention, each handler invents its own shape, making frontend error
handling inconsistent and i18n impossible.

## Options Considered

**Option A — Plain HTTP status codes, no body**
Simplest to produce. Useless for the frontend: no way to distinguish two different 422s
without parsing the body. Rejected.

**Option B — Free-form JSON `{ "error": "message string" }`**
Easy to produce but hard to consume. String-matching is fragile across locales. No
structured branching for the frontend. Rejected.

**Option C — Typed error codes with structured JSON body**
A `code` field (short string constant, e.g. `"E_TASK_NOT_FOUND"`) enables frontend
branching and i18n key lookup. A `message` field carries a developer-readable English
description. An optional `details` array carries field-level validation info. Selected.

**Option D — GraphQL-style partial success with `errors` array**
Overkill for a REST/Tauri API that returns discrete responses. Adds complexity with no
benefit for this use case. Rejected.

## Decision

### Error wire format

All error responses (REST and Tauri command errors) use this JSON shape:

```json
{
  "code": "E_TASK_NOT_FOUND",
  "message": "Task with id 'abc-123' does not exist.",
  "details": [{ "field": "task_id", "issue": "not_found" }]
}
```

- `code` — required. A `SCREAMING_SNAKE_CASE` string constant prefixed `E_`. Stable across
  releases; changing a code is a breaking change (see [ADR-010](./ADR-010-api-versioning-compatibility.md)).
- `message` — required. English developer-readable description. Not shown to end users.
- `details` — optional array of `{ field, issue }` objects for field-level validation errors.

### Rust type

Defined in `backend/crates/shared-kernel/src/errors.rs`:

```rust
#[derive(Debug, Serialize, Typeshare)]
pub struct AppError {
    pub code:    String,        // e.g. "E_TASK_NOT_FOUND"
    pub message: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Serialize, Typeshare)]
pub struct ErrorDetail {
    pub field: String,
    pub issue: String,
}
```

`AppError` is annotated with `#[derive(Typeshare)]` so the TypeScript types are generated
automatically into `packages/app-core/src/generated/` (see [ADR-007](./ADR-007-frontend-backend-api-contract.md)).

### TypeScript type (generated)

```typescript
// packages/app-core/src/generated/errors.ts  — do not edit manually
export interface AppError {
  code: string;
  message: string;
  details: ErrorDetail[];
}

export interface ErrorDetail {
  field: string;
  issue: string;
}
```

Frontend feature code receives a typed `AppError` (or discriminated union of known codes)
and maps `code` to an i18n key for display.

### HTTP status mapping

| Situation                                     | HTTP Status               |
| --------------------------------------------- | ------------------------- |
| Input fails schema validation (missing field) | 400 Bad Request           |
| Domain validation fails (business rule)       | 422 Unprocessable Entity  |
| Resource not found                            | 404 Not Found             |
| Caller not authenticated                      | 401 Unauthorized          |
| Caller lacks permission                       | 403 Forbidden             |
| Unexpected server error                       | 500 Internal Server Error |

500 responses never include internal details in `message`; they log full context server-side
and return a generic `E_INTERNAL` code to the client.

### Tauri commands

Tauri command handlers return `Result<T, AppError>`. On the TypeScript side, the Tauri
`invoke()` call rejects with a serialised `AppError` on `Err`. Frontend platform-desktop
adapters deserialise this into the same `AppError` type used for REST errors — one error
handler per feature works for both transports.

### Error data flow

```
Rust domain / application layer raises AppError
  │
  ├── REST path:
  │     api crate maps AppError → HTTP response (status + JSON body)
  │       → platform-web adapter deserialises → AppError
  │           → feature service receives typed AppError
  │               → maps code to i18n key → Lit component displays message
  │
  └── Tauri path:
        Tauri command returns Err(AppError)
          → invoke() rejects with serialised AppError
              → platform-desktop adapter deserialises → AppError
                  → feature service receives typed AppError
                      → maps code to i18n key → Lit component displays message
```

## Consequences

**Easier:**

- Frontend can branch on `code` string without string-matching `message`
- i18n: `code` maps 1:1 to an i18n key; no translation logic in backend
- One `AppError` type works for both REST and Tauri transports
- 500 errors never leak internals to web clients

**Harder:**

- Error codes must be treated as part of the API contract — renaming a code is a breaking
  change subject to the deprecation policy in [ADR-010](./ADR-010-api-versioning-compatibility.md)
- Every new domain error requires adding a code constant to `shared-kernel` and a
  corresponding i18n key to the frontend translation files
- Typeshare codegen must be re-run when `AppError` or `ErrorDetail` fields change (CI
  enforces this — see [ADR-007](./ADR-007-frontend-backend-api-contract.md))

See [ADR-007](./ADR-007-frontend-backend-api-contract.md) for the type-sharing pipeline
that propagates `AppError` to TypeScript.
See [ADR-003](./ADR-003-technology-stack.md) for the Tauri command transport.
