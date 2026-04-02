# Postmortem: Task 7 Login/Logout Route Handlers + Integration Tests

## What was done
- Replaced `backend/crates/api/src/routes/auth.rs` stubs with TDD flow:
  - Added 5 route-level integration tests for login/logout success and failure cases.
  - Verified tests fail first with `unimplemented!()`.
  - Implemented `login` handler with validation (`E_VALIDATION`), authentication, JWT issuance.
  - Implemented `logout` handler with bearer token extraction, JWT validation, deny-list revocation, and repeat-revocation rejection (`E_UNAUTHORIZED`).
- Ran targeted route tests and full `api` crate tests.

## What went well
- TDD cycle was clean: red state was explicit (`not implemented` panics), then green with minimal handler logic.
- Test setup reused existing in-memory identity adapters and shared state to validate deny-list behavior across requests.
- Error mapping worked through existing `From<AppError> for ApiError`, keeping handlers concise.

## What went wrong
- Initial test command was run from the repository root instead of the required worktree path, yielding misleading zero-test output.

## Lessons learned
- Always run cargo commands with explicit worktree path in multi-worktree setups.
- For auth route tests, use shared `Arc<AppState>` when behavior depends on in-memory state continuity (e.g., token revocation).
