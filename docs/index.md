# open-plx Project Guide

Authoritative reference for agents and contributors.

## What is open-plx?

An open-source alternative to Google PLX. Server-driven dashboard platform where:
- Backend declares dashboard layouts declaratively (widget types, positions, data sources)
- Frontend receives layout config via gRPC, renders the UI, then pulls data per-widget via WidgetDataService (proto columnar)
- Two-phase permission model: layout visibility vs data access
- All communication is gRPC over HTTP/2. No REST, no HTTP/1.1.

## Tech Stack

| Layer     | Technology                                              |
|-----------|---------------------------------------------------------|
| Backend   | Rust, tonic, tonic-web, arrow-flight, adbc, serde, tokio |
| Frontend  | React 19, Vite 8, TypeScript 5.9, @connectrpc/connect-web |
| UI        | Antd 6 (chrome), AntV G2 5 (charts), AntV S2 2.6 (tables) |
| Config    | YAML files on disk (no database)                        |
| Protocol  | gRPC over HTTP/2 (tonic-web for browser), Arrow Flight SQL via ADBC (backend-to-data-source) |
| Data path | Browser -> WidgetDataService (proto columnar) -> Backend -> Flight SQL via ADBC (Arrow) |
| Schema    | Proto-first: types generated from proto files            |

## Project Layout

```
open-plx/
  proto/                 # Protobuf definitions (source of truth)
    open_plx/v1/
      dashboard.proto    # DashboardService, WidgetType (17 types), DashboardVariable (7 controls)
      data_source.proto  # DataSourceService, FlightSqlConfig, StaticConfig
      data.proto         # WidgetDataService, WidgetDataRequest/Response, DataColumn
      widget_spec.proto  # WidgetSpec oneof (10 variants: chart, pivot_table, metric_card, text, table, gauge, funnel, treemap, sankey, word_cloud)
  crates/
    open-plx-core/       # Generated proto types (tonic-build), gRPC reflection descriptor
    open-plx-config/     # YAML config loader + YAML->proto converter + static data builder
    open-plx-auth/       # Stateless auth (AuthProvider: Dev, ApiKey, OIDC stub) + permissions
    open-plx-cli/        # CLI tool (`plx` binary): dashboard list, export, validate, import
    open-plx-server/     # tonic gRPC server + tonic-web + Arrow Flight + WidgetDataService + ADBC Flight SQL client
  frontend/
    src/
      gen/open_plx/v1/   # Generated TypeScript types (via @bufbuild/protoc-gen-es)
      components/        # UI components
        widgets/         # Widget renderers (G2, S2, Antd): 17 widget types
        layout/          # CSS grid layout, WidgetShell, WidgetErrorBoundary
        variables/       # VariableBar, VariableControl (7 control types)
      hooks/             # useDashboard, useDashboardList, useWidgetData, useVariables, useThemeContext
      services/
        grpc/            # gRPC-web client via @connectrpc/connect-web (2 service clients: dashboardClient, widgetDataClient)
        mappers/         # 9 mappers: chartMapper, pivotMapper, tableMapper, gaugeMapper, funnelMapper, treemapMapper, sankeyMapper, wordCloudMapper, conditionMapper
        evaluateVisibility.ts  # Client-side widget visibility evaluation (9 operators)
        testRegistry.ts  # E2e test helper (window.__OPEN_PLX__ state inspection)
      pages/             # DashboardPage, DashboardListPage
    e2e/                 # Playwright e2e tests (3 spec files)
  config/                # Runtime config
    dashboards/          # Dashboard YAML configs (6 dashboards)
    data_sources/        # Data source YAML configs (34 data sources)
    permissions.yaml     # Groups + permissions
    seed/                # Seed data (DuckDB, PostgreSQL, MySQL init scripts + CSV)
    open-plx.yaml        # Server config
  vendor/                # Git submodules
    G2/                  # AntV G2 (branch: v5) -- reference only
    S2/                  # AntV S2 (branch: next) -- reference only
  tools/                 # Python utility scripts (HN data crawling)
  skills/                # Claude Code skills
  docs/                  # Documentation
  Dockerfile             # Multi-stage build (node:22 + rust:1.85 -> debian:bookworm-slim)
  docker-compose.yml     # DuckDB Flight SQL + PostgreSQL + MySQL (dev/test)
```

## Key Documents

- [Architecture](architecture.md) - System diagram, crate map, rendering flow
- [Design](design.md) - Design decisions and rationale
- [Declarative Layout](declarative-layout.md) - Widget spec language (proto -> G2/S2 mapping)
- [Data Format](data-format.md) - Arrow Flight protocol, data source configs
- [Auth](auth.md) - Authentication & authorization design
- [Phases](phases.md) - Implementation plan (phased rollout)
- [Tasks](tasks.md) - Current work items
- [Widget Expansion](widget-expansion.md) - Widget expansion plan (17 types)
- [Codelabs](codelabs.md) - Development walkthroughs

## Core Concepts

### Dashboard Layout Config
The central abstraction. A `Dashboard` proto message containing a grid of
`WidgetConfig` entries, each with a type, position, data source reference,
and a typed `WidgetSpec` (oneof with 10 variants: ChartSpec, PivotTableSpec,
MetricCardSpec, TextSpec, TableSpec, GaugeSpec, FunnelSpec, TreemapSpec,
SankeySpec, WordCloudSpec).

### Widget Types (17)
LINE_CHART, BAR_CHART, PIE_CHART, SCATTER_CHART, HEATMAP, HISTOGRAM,
RADAR_CHART, BOX_PLOT, PIVOT_TABLE, TABLE, METRIC_CARD, TEXT, GAUGE,
FUNNEL, TREEMAP, SANKEY, WORD_CLOUD.

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
CONFIG_PATH=config/open-plx.yaml cargo run -p open-plx-server

# Frontend
cd frontend && pnpm install && pnpm dev

# Dev servers (Flight SQL + Postgres + MySQL)
docker compose up -d
```
