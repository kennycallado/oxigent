# ADR-003: Technology Stack

## Status
Accepted

## Context
The project needs a backend language, a frontend framework, a desktop shell, and a UI component library. Data technology and dev environment choices are addressed in [ADR-004](./ADR-004-data-technology.md) and [ADR-008](./ADR-008-monorepo-dev-environment.md) respectively.

## Options Considered

**Backend language:**
- *Go:* good concurrency, simpler than Rust, but weaker type system and no Tauri integration story. Rejected.
- *Rust:* strong type system enforces domain invariants at compile time, native Tauri integration, excellent async with Tokio, embedded SurrealDB runs in-process. Selected.

**Frontend framework:**
- *React/Vue/Angular:* component-per-framework, not portable between shells without wrappers. Rejected.
- *Lit + Web Components:* web standards, works identically in browser and Tauri webview, composable with any UI library that exports web components. Selected.

**Desktop shell:**
- *Electron:* ships a full Chromium + Node.js; binary size ~150MB baseline, high memory usage. Rejected.
- *Tauri 2:* uses the OS webview, Rust backend, ~5–10MB baseline, strong security model with per-window capabilities. Selected.

**UI component library:**
- *Custom design system:* high effort, inconsistent results early on. Rejected for now.
- *Siemens iX:* enterprise-grade, exports Web Components, has Lit integration, theming via CSS variables, actively maintained. Selected.

**Frontend state:**
- *Redux/Zustand/Pinia:* framework-coupled or too heavyweight for our use. Rejected.
- *Nanostores:* framework-agnostic atomic stores, <1KB, works with Lit. Selected for UI state only — business logic stays in application services.

## Decision

| Layer | Technology | Justification |
|---|---|---|
| Backend | Rust (edition 2024) + Tokio | Type safety for domain invariants, Tauri integration, async performance |
| Desktop shell | Tauri 2 | Security capabilities model, OS webview, Rust backend |
| Frontend core | Lit + TypeScript | Web Components standard, shared across shells |
| UI components | Siemens iX | Enterprise Web Components, Lit integration, theming |
| UI state | Nanostores | Framework-agnostic, atomic, minimal |

**Tauri 2 usage constraints** (see [ADR-006](./ADR-006-frontend-architecture.md) for full frontend architecture):
- Capabilities defined per window/webview — no blanket permissions
- Agent CLI execution via declared `externalBin` sidecars with explicit allowlists
- High-frequency streaming via Tauri channels, not Tauri events
- No business logic in Tauri command handlers — they delegate to Rust application services

## Consequences

**Easier:**
- Single language (Rust) for all backend logic including desktop bridge
- UI components work in both shells with no adaptation
- Tauri capabilities model enforces security boundaries at the framework level

**Harder:**
- Rust learning curve for contributors unfamiliar with ownership/borrowing
- Siemens iX theming requires understanding CSS custom properties and iX token system
- Tauri 2 capabilities require explicit configuration for each privileged operation
