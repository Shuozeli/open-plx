# open-plx Project Guide

Authoritative reference for agents and contributors.

## What is open-plx?

An open-source alternative to Google PLX. Server-driven dashboard platform where:
- Backend declares dashboard layouts declaratively (widget types, positions, data sources)
- Frontend receives layout config via gRPC, renders the UI, then pulls data per-widget via Arrow Flight
- Two-phase permission model: layout visibility vs data access
- All communication is gRPC over HTTP/2. No REST, no HTTP/1.1.

## Tech Stack

| Layer     | Technology                                              |
|-----------|---------------------------------------------------------|
| Backend   | Rust, tonic, tonic-web, arrow-flight, sqlx, tokio       |
| Frontend  | React 18+, Vite, TypeScript, grpc-web, apache-arrow     |
| UI        | Antd (chrome), AntV G2 (charts), AntV S2 (tables)      |
| Database  | PostgreSQL                                              |
| Protocol  | gRPC over HTTP/2 (tonic-web for browser), Arrow Flight  |
| Schema    | Proto-first: types generated from proto files            |

## Project Layout

```
open-plx/
  proto/                 # Protobuf definitions (source of truth)
    open_plx/v1/
      dashboard.proto    # DashboardService, layout types
      data_source.proto  # DataSourceService, data source configs
      data.proto         # Arrow Flight descriptors/metadata
      widget_spec.proto  # WidgetSpec oneof (ChartSpec, PivotTableSpec, etc.)
  crates/
    open-plx-server/     # tonic gRPC server + tonic-web + Arrow Flight
    open-plx-core/       # Domain types (generated from protos + business logic)
    open-plx-store/      # PostgreSQL persistence (sqlx)
    open-plx-auth/       # Permission layer (layout + data)
  frontend/
    src/
      components/        # UI components
        widgets/         # Widget renderers (G2, S2, Antd)
        layout/          # Grid layout system
      hooks/             # React hooks
      services/
        grpc/            # gRPC-web client (generated)
        flight/          # Arrow Flight client
      types/             # Generated TypeScript types
      pages/             # Route pages
  vendor/                # Git submodules
    G2/                  # AntV G2 (branch: v5) -- reference only
    S2/                  # AntV S2 (branch: next) -- reference only
  docs/                  # Documentation
  deploy/                # Docker, k8s configs
```

## Key Documents

- [Architecture](architecture.md) - System diagram, crate map, rendering flow
- [Design](design.md) - Design decisions and rationale
- [Declarative Layout](declarative-layout.md) - Widget spec language (proto -> G2/S2 mapping)
- [Data Format](data-format.md) - Arrow Flight protocol, data source configs
- [Auth](auth.md) - Authentication & authorization design
- [Phases](phases.md) - Implementation plan (phased rollout)
- [Tasks](tasks.md) - Current work items
- [Codelabs](codelabs.md) - Development walkthroughs

## Core Concepts

### Dashboard Layout Config
The central abstraction. A `Dashboard` proto message containing a grid of
`WidgetConfig` entries, each with a type, position, data source reference,
and a typed `WidgetSpec` (oneof: ChartSpec, PivotTableSpec, MetricCardSpec, TextSpec).

### Widget Registry
Frontend maps widget type enums to React components. Adding a new widget
type means: (1) add the type to widget_spec.proto, (2) implement the React
component, (3) register it.

### Two-Phase Permission
Layout permission (can user see this dashboard?) and data permission (can user
access this data source?) are checked independently via gRPC status codes.
See [Architecture](architecture.md).

### Dashboard Variables
Dashboard-level variables (date ranges, filters) that multiple widgets can
reference via `${variable_name}` syntax in their data source params. Enables
cross-widget filtering without per-widget param updates.

## Development

```bash
# Proto generation
buf generate proto/

# Backend
cargo test --workspace
cargo run -p open-plx-server

# Frontend
cd frontend && pnpm install && pnpm dev
```
