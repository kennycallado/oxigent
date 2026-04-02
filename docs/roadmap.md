# Oxigent — Roadmap

> Fuente canónica del plan de desarrollo. Se actualiza al cerrar/crear issues.
> Última actualización: 2026-04-02

## Visión

Plataforma de gestión de tareas con integración de agentes IA. Monorepo Rust (DDD/CQRS) + Lit/Siemens iX + Tauri 2 + SurrealDB.

- **Repo:** [`kennycallado/oxigent`](https://github.com/kennycallado/oxigent)
- **Board:** [Oxigent Roadmap](https://github.com/users/kennycallado/projects/2)
- **ADRs:** `docs/adr/ADR-001` a `ADR-012`

---

## Estado global

| Métrica         | Valor                |
| --------------- | -------------------- |
| Issues totales  | 22                   |
| Cerrados        | 2 (#1, #25)          |
| Abiertos        | 20                   |
| PRs merged      | 3 (#21, #23, #24)    |
| PRs descartados | 1 (#22, Copilot WIP) |
| Fase activa     | **Fase 1**           |
| Progreso Fase 1 | 1 / 7 (14 %)         |

---

## Fase 1 — Esqueleto funcional

|                    |                                                                     |
| ------------------ | ------------------------------------------------------------------- |
| **Milestone**      | Fase 1 — Esqueleto funcional                                        |
| **Objetivo**       | App que arranca, autentica y muestra tareas reales en web y desktop |
| **Fecha objetivo** | 2026-05-30                                                          |
| **Progreso**       | 1 / 7 cerrados                                                      |
| **Descripción**    | Backend auth + tasks, frontend login + lista, desktop handshake     |

### Issues

| #   | Estado   | Issue                                                                   | Labels   | Depende de |
| --- | -------- | ----------------------------------------------------------------------- | -------- | ---------- |
| 1   | **DONE** | Scaffold crate identity-access: módulo user/ con dominio y puertos      | backend  | —          |
| 2   | OPEN     | Endpoint POST /v1/auth/login y sesión JWT                               | backend  | #1         |
| 3   | OPEN     | Scaffold crate work-management: módulo task/ con CreateTask y ListTasks | backend  | —          |
| 4   | OPEN     | Endpoint GET /v1/info — handshake de compatibilidad (ADR-010)           | backend  | —          |
| 5   | OPEN     | Pantalla de login en web — conectada al backend real                    | frontend | #2         |
| 6   | OPEN     | Vista lista de tareas en web — datos reales desde backend               | frontend | #3         |
| 7   | OPEN     | Desktop: startup handshake + Tauri commands para auth y tareas          | desktop  | #2, #3     |

### Orden sugerido

```
#2 + #3 + #4  (paralelizables, backend)
        |
       #5 + #6  (frontend, depende de #2 y #3)
        |
       #7       (desktop, depende de #2 y #3)
```

- **#2** tiene spec + plan listos en `docs/superpowers/specs/` y `docs/superpowers/plans/`.
- **#4** es independiente — se intentó vía Copilot (PR #22, cerrado sin merger). Pendiente reintentar.

---

## Fase 2 — Gestión de tareas completa

|                    |                                                              |
| ------------------ | ------------------------------------------------------------ |
| **Milestone**      | Fase 2 — Gestión de tareas completa                          |
| **Objetivo**       | Feature set comparable a un Jira mínimo, con offline support |
| **Fecha objetivo** | 2026-08-15                                                   |
| **Progreso**       | 0 / 6 cerrados                                               |

### Issues

| #   | Estado | Issue                                                                  | Labels         |
| --- | ------ | ---------------------------------------------------------------------- | -------------- |
| 8   | OPEN   | Task: editar, eliminar, estados, etiquetas, prioridad, asignado        | backend        |
| 9   | OPEN   | Scaffold crate planning: módulo board/ con Board y Sprint              | backend        |
| 10  | OPEN   | Offline support: outbox pattern para sync desktop → servidor (ADR-005) | backend, infra |
| 11  | OPEN   | CI: Typeshare freshness check (ADR-007)                                | infra          |
| 12  | OPEN   | Vista Kanban board con drag & drop                                     | frontend       |
| 13  | OPEN   | Pipeline CI/CD básico: build, test, lint                               | infra          |

### Orden sugerido

```
#8 (CRUD tareas)  →  #9 (planning)  →  #12 (Kanban UI)
#11 + #13 (CI, paralelizables desde Fase 1)
#10  (offline, puede empezar cuando #8 esté listo)
```

- **#13** conviene priorizarlo temprano — actualmente no hay CI.
- **#10** es el issue más complejo de la fase (outbox pattern, ADR-005).

---

## Fase 3 — Integración de agentes

|                    |                                                                |
| ------------------ | -------------------------------------------------------------- |
| **Milestone**      | Fase 3 — Integración de agentes                                |
| **Objetivo**       | Ejecutar Claude Code / Codex desde desktop, vinculado a tareas |
| **Fecha objetivo** | 2026-10-31                                                     |
| **Progreso**       | 0 / 5 cerrados                                                 |

### Issues

| #   | Estado | Issue                                                                          | Labels   |
| --- | ------ | ------------------------------------------------------------------------------ | -------- |
| 14  | OPEN   | Scaffold crate agent-execution: módulos run/ y session/                        | backend  |
| 15  | OPEN   | TauriAgentExecutor: lanzar sidecar CLI y streamear output                      | desktop  |
| 16  | OPEN   | Panel de agente en desktop: output en tiempo real y aprobación de herramientas | frontend |
| 17  | OPEN   | Vincular AgentRun a Task: cross-context reference por ID                       | backend  |
| 27  | OPEN   | Observability: wire tracing port into application services + structured logging | backend  |

### Orden sugerido

```
#14 (scaffold)  →  #15 (sidecar)  →  #16 (panel UI)
                              #17 (cross-context, paralelo a #16)
                              #27 (observability, paralelo a #16)
```

- **#27** depende de #14 (necesita el crate agent-execution scaffolded).
- **#25** (cerrado) — era prematuro. Reemplazado por #27.

---

## Fase 4 — Integraciones externas

|                    |                                          |
| ------------------ | ---------------------------------------- |
| **Milestone**      | Fase 4 — Integraciones externas          |
| **Objetivo**       | Git sync, webhooks, Jira import/export   |
| **Fecha objetivo** | Backlog abierto (sin fecha comprometida) |
| **Progreso**       | 0 / 3 cerrados                           |

### Issues

| #   | Estado | Issue                                                  | Labels  |
| --- | ------ | ------------------------------------------------------ | ------- |
| 18  | OPEN   | Git sync: vincular commits y PRs a tareas              | backend |
| 19  | OPEN   | Jira import: importar proyectos y tareas desde Jira    | backend |
| 20  | OPEN   | Webhooks: notificaciones salientes a sistemas externos | backend |

- Los tres son independientes entre sí.
- Orden de prioridad sugerido: #18 → #20 → #19 (Git sync primero por valor para developers).

---

## Labels

| Label      | Ámbito                             |
| ---------- | ---------------------------------- |
| `fase-1`   | Fase 1: Esqueleto funcional        |
| `fase-2`   | Fase 2: Gestión de tareas completa |
| `fase-3`   | Fase 3: Integración de agentes     |
| `fase-4`   | Fase 4: Integraciones externas     |
| `backend`  | Rust backend                       |
| `frontend` | TypeScript / Lit frontend          |
| `desktop`  | Tauri desktop shell                |
| `infra`    | CI, monorepo, tooling              |

---

## Board fields

El project board [Oxigent Roadmap](https://github.com/users/kennycallado/projects/2) usa estos campos:

| Campo         | Valores                               | Uso                      |
| ------------- | ------------------------------------- | ------------------------ |
| **Status**    | Todo → In Progress → In Review → Done | Ciclo de vida del issue. **Done es manual** — mover tras confirmar merge |
| **Priority**  | Critical / High / Medium / None       | Priorización             |
| **Blocked**   | Yes / No                              | Dependencias bloqueantes |
| **Agent**     | Libre (e.g. `@agent_gpt`)             | Subagente asignado       |
| **Milestone** | Fase 1–4                              | Agrupación por fase      |
