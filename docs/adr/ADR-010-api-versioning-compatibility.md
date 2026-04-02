# ADR-010: API Versioning and Desktop Compatibility

## Status

Accepted

## Context

The desktop app ships as a native binary via Tauri. Users may not update immediately. The
backend API evolves. Without a formal compatibility strategy:

- A backend deployment can silently break an older desktop app mid-session.
- There is no clear rule for what constitutes a "breaking" vs "non-breaking" change.
- There is no mechanism to force a desktop update when backward-compatibility is abandoned.

The web client always runs the latest version (served fresh), so the compat problem is
primarily desktop ↔ backend.

## Options Considered

**Option A — No versioning; always latest**
Simple to operate. Unacceptable for a shipped desktop binary: any breaking API change
breaks all unupdated clients immediately. Rejected.

**Option B — Indefinite backward compatibility**
Never remove or rename anything. Avoids forced updates. Leads to permanent API cruft and
makes schema evolution impossible. Rejected.

**Option C — Versioned API with deprecation window and startup handshake**
URL-versioned REST API (`/v1/`, `/v2/`). A handshake endpoint lets the backend declare the
minimum client version it still supports. Desktop checks at startup and blocks the UI if
the client is too old, triggering the Tauri updater. Old API versions are maintained for a
defined deprecation window after a new version ships. Selected.

**Option D — Feature flags instead of versioning**
Allows gradual rollout but does not solve the "old client on new backend" problem; still
needs a compatibility check. Adds complexity without replacing versioning. Rejected.

## Decision

### API versioning scheme

REST endpoints are prefixed with a major version segment: `/v1/<resource>`.
Tauri commands follow the same convention: command names are prefixed `v1_<command>`.

When a new major version ships (`/v2/`), `/v1/` continues to be served for the deprecation
window defined below.

### Breaking vs non-breaking changes

| Change type                                  | Classification |
| -------------------------------------------- | -------------- |
| Add optional response field                  | Non-breaking   |
| Add new endpoint / Tauri command             | Non-breaking   |
| Remove response field                        | **Breaking**   |
| Rename response field                        | **Breaking**   |
| Change field type (e.g. `string` → `number`) | **Breaking**   |
| Remove endpoint / Tauri command              | **Breaking**   |
| Change error code constant (see ADR-009)     | **Breaking**   |
| Add required request field                   | **Breaking**   |

Non-breaking changes may be deployed without a version bump.
Breaking changes require a new version (`/vN+1/`) and must not be deployed to `/vN/`.

### Deprecation window

After `/v2/` ships, `/v1/` is maintained for **6 months**. After that window, `/v1/`
returns `410 Gone` for all requests. The `GET /v1/info` handshake endpoint is the last
endpoint to be retired and continues to respond (with an upgrade-required payload) until
the window closes.

### Handshake endpoint

```
GET /v1/info
```

Response (success):

```json
{
  "api_version": "1",
  "min_client_version": "1.2.0",
  "latest_version": "1.5.0"
}
```

- `api_version` — the major version of this endpoint group.
- `min_client_version` — the minimum desktop app version that this backend instance still
  fully supports. Follows semver.
- `latest_version` — the latest available desktop app version (for display only).

Response when backend no longer supports the calling client:

```json
{
  "api_version": "2",
  "min_client_version": "2.0.0",
  "latest_version": "2.1.0",
  "upgrade_required": true,
  "upgrade_message": "This version is no longer supported. Please update the app."
}
```

HTTP status: `200 OK` in both cases. The client checks `upgrade_required`, not the status
code, to avoid confusion with network errors.

### Desktop startup check

The desktop app performs the following at startup, before rendering any UI:

```
app starts
  → GET /v1/info (or /v<current>/info)
      ┌── network error / timeout
      │     → show "Cannot reach server" banner; allow offline mode if ADR-005 applies
      │
      ├── upgrade_required == true
      │     → block UI with "Update required" modal
      │         → trigger Tauri updater API (tauri-plugin-updater)
      │             → if update available: install + restart
      │             → if no update found: show manual download link
      │
      └── upgrade_required absent or false
            → proceed to normal startup
```

The check is non-blocking only when the app is in offline mode and the last-known compat
check (cached locally) passed within the last 7 days.

### Compatibility check flow

```
Desktop app startup
  │
  ├── [online] GET /v1/info
  │     │
  │     ├── upgrade_required == true
  │     │     → block UI → Tauri updater → install/restart or show download link
  │     │
  │     └── upgrade_required == false
  │           → cache result with timestamp → proceed
  │
  └── [offline] check local cache
        ├── cache age ≤ 7 days AND last result was "not required"
        │     → proceed (offline mode)
        └── cache age > 7 days OR cache missing
              → show warning banner but allow offline mode
```

## Consequences

**Easier:**

- Backend can make breaking changes without silently breaking shipped desktop clients
- Deprecation window gives users a clear, time-bounded upgrade path
- Startup handshake prevents users from running incompatible combinations silently

**Harder:**

- Every breaking change requires shipping both the old and new API version simultaneously
  during the deprecation window (doubles maintenance surface temporarily)
- Backend must track `min_client_version` and update it deliberately when dropping old compat
- Desktop app must implement the startup handshake and offline-cache logic before any
  breaking API change is deployed

See [ADR-007](./ADR-007-frontend-backend-api-contract.md) for the REST/Tauri transport
contract that this versioning scheme applies to.
See [ADR-009](./ADR-009-error-handling.md) for error codes, which are versioned under this
policy (renaming an error code is a breaking change).
See [ADR-005](./ADR-005-offline-sync-strategy.md) for the offline mode this handshake
integrates with.
