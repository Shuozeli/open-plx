# open-plx

Server-driven dashboard platform. Rust backend declares dashboard layouts
declaratively, pushes layout configs to React frontend, frontend pulls data
separately. An open-source alternative to Google PLX.

## Quick Orientation

Read `docs/index.md` for the full project guide, architecture decisions,
and crate/package map. That file is the authoritative reference for agents.

## Git Rules

- Do NOT commit or push unless the user explicitly asks you to
- Do NOT amend commits unless the user explicitly asks you to
- Do NOT force push unless the user explicitly asks you to

## Key Rules

- Two-phase rendering: layout push first, data pull second. This enables separate permission checks for UI visibility vs data access.
- Frontend owns ALL rendering logic. Backend never dictates styles, padding, or component internals.
- Backend is the source of truth for dashboard layout definitions (widget types, grid positions, data source references).
- Use AntV G2 for charts, AntV S2 for pivot/tabular data. Do NOT use raw canvas or custom chart rendering.
- Use Antd for UI chrome (layout, menus, forms). Do NOT mix in other UI component libraries.
- No database. All config is file-based (YAML). Server is stateless.
- Do NOT use `any` in TypeScript. Use `unknown` with explicit type checking if needed.
- Do NOT use default config values. Missing config must fail the server.
- Strictly avoid circular dependencies.

## Code Quality Discipline

Shortcuts during exploration are fine -- getting things working is the
priority. But tech debt must be visible, not silent.

- **Always leave `// TODO(refactor):` comments** when you take a shortcut.
- **Self-review pass:** After getting the feature working and tests passing,
  re-read the diff once. Leave `// TODO(refactor):` markers on anything
  you'd flag in a code review.

## Tech Stack

- **Backend:** Rust (tonic, tonic-web, arrow-flight, serde, tokio)
- **Frontend:** React 19, Vite, TypeScript, Antd, AntV G2, AntV S2, @connectrpc/connect-web
- **Config:** YAML files on disk (dashboards, data sources, permissions). No database.
- **Protocol:** gRPC over HTTP/2 (tonic-web for browser). Arrow Flight SQL between backend and data sources.
- **Data path:** Browser -> WidgetDataService (gRPC, proto columnar) -> Backend -> Flight SQL server (Arrow).
- **Proto-first:** Proto files are the source of truth. Types are generated.

## Build & Test

```bash
# Backend
cargo test --workspace
cargo doc --workspace --no-deps

# Frontend
cd frontend && pnpm install && pnpm dev
pnpm test
pnpm build

# Proto generation (frontend only; backend protos auto-generate via build.rs)
buf generate proto/
```

## CI

After pushing, check GitHub Actions status:
```bash
gh run list --limit 3
gh run view <run-id>
```

## Key Docs

- `docs/index.md` - Full project guide for agents
- `docs/architecture.md` - System architecture, crate map, rendering flow
- `docs/design.md` - Design decisions and rationale
- `docs/declarative-layout.md` - Widget spec language (proto -> G2/S2 mapping)
- `docs/data-format.md` - Arrow Flight protocol, data source configs
- `docs/auth.md` - Authentication & authorization design
- `docs/phases.md` - Implementation plan (phased rollout)
- `docs/tasks.md` - Pending TODOs and deferred items
- `docs/codelabs.md` - Walkthroughs
