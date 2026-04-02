# Design: POST /v1/auth/login and JWT Session (Issue #2)

**Date:** 2026-04-02
**Issue:** [#2](https://github.com/kennycallado/oxigent/issues/2)
**Milestone:** Fase 1 — Esqueleto funcional
**ADRs:** ADR-007, ADR-009

---

## 1. Scope

Implement authentication in the `api` crate:

- `POST /v1/auth/login` — validates credentials, returns a signed JWT
- `POST /v1/auth/logout` — invalidates the token via an in-memory deny-list
- Auth middleware — validates Bearer tokens for future protected routes
- Replace `PlainPasswordHasher` stub with `Argon2PasswordHasher` in `identity-access`

**Out of scope:** refresh tokens, persistent session storage, role-based access control (those are future milestones).

---

## 2. Architecture

### Crate responsibilities

| Crate              | Responsibility                                                        |
|--------------------|-----------------------------------------------------------------------|
| `identity-access`  | Domain: `User`, `Role`; ports: `PasswordHasher`; app: `AuthenticateUser` |
| `api`              | HTTP transport: Axum router, JWT issuance/validation, deny-list, middleware |
| `shared-kernel`    | `AppError`, `ErrorDetail`                                             |

JWT and session management are **transport/infrastructure concerns** and live in `api`. The `identity-access` crate stays pure domain.

---

## 3. Module Structure

### `identity-access` changes

```
src/user/
  adapters/
    argon2_password_hasher.rs   # NEW: Argon2PasswordHasher implements PasswordHasher
    plain_password_hasher.rs    # REMOVED (replaced)
  application/
    authenticate.rs             # NEW: AuthenticateUser { find_user, verify_password }
```

**`AuthenticateUser`** service:
- Input: `email: String`, `password: String`
- Looks up user by email via `UserSearch` port (filtering by email field)
- Verifies password via `PasswordHasher` port
- Returns `User` on success, `AppError { code: "E_UNAUTHORIZED", ... }` on failure
- Note: always returns `E_UNAUTHORIZED` for both "user not found" and "wrong password" to avoid user enumeration

### `api` crate new structure

```
api/src/
  lib.rs              # build_router(state: AppState) → Router
  config.rs           # AppConfig { jwt_secret, jwt_expiry_secs } — reads from env
  state.rs            # AppState { identity_svc, jwt_svc, deny_list }
  jwt/
    mod.rs            # JwtService: issue(user) → String, validate(token) → Claims
    claims.rs         # Claims { sub, role, jti, exp, iat }
  deny_list.rs        # DenyList: Arc<Mutex<HashSet<String>>> keyed on JTI
  middleware/
    auth.rs           # Axum middleware: extract Bearer, validate, deny-list check
  routes/
    mod.rs
    auth.rs           # POST /v1/auth/login, POST /v1/auth/logout handlers
```

---

## 4. Data Contracts

### `POST /v1/auth/login`

**Request** (JSON body):
```json
{ "email": "alice@example.com", "password": "secret" }
```

**Response 200** (JSON body):
```json
{ "token": "<signed-jwt>" }
```

**Errors:**
- `400 Bad Request` — `E_VALIDATION`: missing `email` or `password`; `details` array carries field-level info (e.g. `[{ "field": "email", "message": "required" }]`)
- `401 Unauthorized` — `E_UNAUTHORIZED`: wrong credentials (same error for unknown email and wrong password — no enumeration)

### `POST /v1/auth/logout`

**Request**: `Authorization: Bearer <token>` header, no body.

**Response 204** No Content.

**Errors:**
- `401 Unauthorized` — `E_UNAUTHORIZED`: token missing, expired, or already revoked

### JWT Claims

```json
{
  "sub": "<user_id>",
  "role": "admin | user",
  "jti": "<uuid-v4>",
  "iat": 1712000000,
  "exp": 1712086400
}
```

Algorithm: HS256. Secret: `JWT_SECRET` env var (minimum 32 bytes recommended). Default expiry: 86400s (24h), overridable via `JWT_EXPIRY_SECS`. No issuer/audience validation in Fase 1 (deferred to production hardening milestone).

### Typeshare types

`LoginRequest` and `LoginResponse` are annotated with `#[derive(Typeshare)]` in the `api` crate and generated into `packages/app-core/src/generated/`.

---

## 5. Auth Middleware

Axum middleware applied to protected route groups (not to `/v1/auth/*`):

1. Extract `Authorization: Bearer <token>` header — return `401 E_UNAUTHORIZED` if absent
2. Validate JWT signature and expiry via `JwtService` — return `401 E_UNAUTHORIZED` on failure
3. Check JTI is not in `DenyList` — return `401 E_UNAUTHORIZED` if revoked
4. Inject `Claims` as Axum extension into request — downstream handlers can extract it

---

## 6. Error Handling (ADR-009)

| Situation                        | Code            | HTTP |
|----------------------------------|-----------------|------|
| Missing email/password field     | `E_VALIDATION`  | 400  |
| Wrong credentials                | `E_UNAUTHORIZED`| 401  |
| Token missing / invalid / expired| `E_UNAUTHORIZED`| 401  |
| Token revoked (in deny-list)     | `E_UNAUTHORIZED`| 401  |
| Argon2 internal failure          | `E_INTERNAL`    | 500  |

`E_INTERNAL` responses log full context server-side and return a generic message to the client (no internal details exposed). `E_VALIDATION` responses include an optional `details` array with field-level info.

---

## 7. New Dependencies

| Crate          | Used in           | Purpose                     |
|----------------|-------------------|-----------------------------|
| `axum`         | `api`             | HTTP framework              |
| `jsonwebtoken` | `api`             | JWT issuance and validation |
| `uuid` (v4)    | `api`             | JTI generation              |
| `argon2`       | `identity-access` | Password hashing            |
| `serde`        | `api`             | Request/response (de)serialization |

All added to the workspace `Cargo.toml` where possible.

---

## 8. Testing

### `identity-access` unit tests

- `Argon2PasswordHasher`: `hash` then `verify` round-trip passes; wrong password returns `false`
- `AuthenticateUser`: happy path returns `User`; wrong password returns `E_UNAUTHORIZED`; unknown user returns `E_UNAUTHORIZED`

### `api` integration tests (axum `TestClient` via `axum-test` crate)

- `POST /v1/auth/login` with valid credentials → 200, response contains `token`, token is valid JWT
- `POST /v1/auth/login` with wrong password → 401 `E_UNAUTHORIZED`
- `POST /v1/auth/login` with missing `password` field → 400 `E_VALIDATION` with `details` array
- `POST /v1/auth/logout` with valid token → 204; subsequent middleware check on protected route → 401
- `POST /v1/auth/logout` without `Authorization` header → 401
- `POST /v1/auth/logout` with already-revoked token → 401 (non-idempotent by design)

---

## 9. Tauri Command Parity (ADR-007)

Per ADR-007, every REST endpoint has an equivalent Tauri command with identical request/response types.

| REST endpoint             | Tauri command          |
|---------------------------|------------------------|
| `POST /v1/auth/login`     | `v1_auth_login`        |
| `POST /v1/auth/logout`    | `v1_auth_logout`       |

Both commands use the same `LoginRequest` / `LoginResponse` Typeshare types. The desktop adapter calls these Tauri commands instead of HTTP. The `api` crate exposes handler functions that both the Axum router and the Tauri command layer call directly (no HTTP hop).

---

## 10. Deny-List Constraints

The in-memory deny-list is a deliberate Fase 1 simplification. Known limitations:

- **Single-process scope:** tokens revoked in one process instance are not revoked in others (no cross-instance coordination).
- **Non-persistent:** the deny-list is cleared on process restart; revoked tokens become valid again.
- **Unbounded growth:** JTI entries accumulate. A background cleanup task removes entries whose `exp` timestamp has passed (run on every insert, or on a timer). This keeps memory bounded without requiring external infrastructure.

The `DenyList` interface is designed to be replaced with a Redis-backed adapter in a later milestone without changing callers.

---

## 11. Exit Criterion

`POST /v1/auth/login` returns a valid JWT token for an existing user with correct credentials.
