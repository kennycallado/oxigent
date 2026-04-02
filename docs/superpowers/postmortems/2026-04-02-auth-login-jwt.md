# Postmortem: Auth Login/Logout with JWT — Issue #2

**Date:** 2026-04-02
**Issue:** [#2 — POST /v1/auth/login and POST /v1/auth/logout with JWT](https://github.com/kennycallado/oxigent/issues/2)
**PR:** [#26](https://github.com/kennycallado/oxigent/pull/26)
**Branch:** `feat/2-auth-login-jwt`
**Orchestrator:** `@orchestrate`
**Implementer:** `@agent_gpt`
**Reviewer:** `@agent_glm`

---

## What Was Done

Implemented `POST /v1/auth/login` and `POST /v1/auth/logout` endpoints in the `api` crate with full JWT session management, plus a `require_auth` middleware for protected routes. Replaced the `PlainPasswordHasher` stub in `identity-access` with a production-ready `Argon2PasswordHasher`.

### Tasks completed (8 total)

| #   | Task                                  | Notes                                                                                                          |
| --- | ------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| 1   | Workspace deps                        | axum 0.8, jsonwebtoken 9, argon2 0.5, uuid v4; fixed duplicate `tower` in api deps                             |
| 2   | `Argon2PasswordHasher`                | Replaced `PlainPasswordHasher`; UUID-based salt; issue #25 opened for observability follow-up                  |
| 3   | `AuthenticateUser` service            | Application service in `identity-access`; TODO annotation for input validation parity                          |
| 4   | `AppConfig`, `ApiError`, `JwtService` | Orphan-safe `ApiError` newtype; explicit `Role` match for PascalCase JWT claims; `SystemTime` error propagated |
| 5   | `DenyList` + `AppState`               | `Arc<Mutex<HashMap>>` deny-list with prune-on-insert; `AppState::new` wiring                                   |
| 6   | `require_auth` middleware             | Bearer extraction → JWT validate → deny-list check → inject `Claims` extension                                 |
| 7   | `login` + `logout` handlers           | 5 integration tests; all 11 api tests pass                                                                     |
| 8   | Final checks + push                   | clippy clean; 29 tests across workspace pass; spurious postmortem deleted; branch pushed                       |

---

## What Went Well

- **Spec was clear and locked early.** The design doc (`docs/superpowers/specs/2026-04-02-auth-login-jwt-design.md`) resolved ambiguities before implementation began, avoiding rework.
- **Two-stage review caught real issues.** The `@agent_glm` spec reviews caught the `Role::Display` lowercase issue and the `SystemTime::unwrap()` panic risk before they were merged.
- **Subagent-driven development kept context clean.** Fresh context per task meant no confusion between tasks.
- **All 29 workspace tests pass and clippy is clean.** Exit criterion (`login_valid_credentials_returns_200_with_token`) passes.

---

## What Went Wrong

### 1. `Role::Display` produces lowercase — not obvious from the type

`Role` had a `Display` impl yielding `"admin"/"member"/"viewer"`, but the spec required PascalCase JWT claims. The implementer initially used `format!("{:?}", role)` (debug, fragile) before the reviewer caught both approaches and mandated an explicit `match`. This should have been caught in the spec phase.

**Lesson:** When the spec uses string literals for enum variants (e.g., `"Admin"`), explicitly note the serialization strategy in the spec.

### 2. Spurious postmortem created mid-implementation

`@agent_gpt` created `docs/superpowers/postmortems/2026-04-02-task-7-auth-routes.md` during Task 7, mistaking the task for completion of the whole issue. It was deleted in Task 8.

**Lesson:** Implementer prompts should explicitly state "do NOT write a postmortem — that is the orchestrator's responsibility after the PR is merged."

### 3. Argon2 salt approach: `OsRng` vs UUID

The plan specified `OsRng` for salt generation, but `OsRng` required a feature flag (`rand_core/getrandom`) not present in `Cargo.toml`. The implementer switched to `SaltString::encode_b64(Uuid::new_v4().as_bytes())` (122-bit UUIDv4 entropy — still CSPRNG-based). The fix was correct but caused unnecessary churn.

**Lesson:** When specifying crypto primitives, verify feature flags are available or specify the fallback in the plan.

### 4. `access_token` vs `token` false alarm

`@agent_gpt` flagged the response field as `access_token` (concern) but the spec clearly says `token`. Verification against spec was sufficient to resolve this — no code change needed.

**Lesson:** Implementers should cross-check spec before flagging concerns about field names.

---

## Observability Debt

Issue #25 was opened: `hasher.verify` errors are silently mapped to `E_UNAUTHORIZED` with no logging. No logging port exists yet in `identity-access`. This is deferred to a future milestone.

---

## Metrics

- **Tasks:** 8
- **Commits on branch:** 11
- **Tests added:** 11 (api); 3 (identity-access authenticate service)
- **Total workspace tests passing:** 29
- **Clippy warnings:** 0
- **Review cycles:** Task 4 required 2 spec review cycles (Role serialization + SystemTime fix)
