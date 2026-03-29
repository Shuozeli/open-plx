# Implementation Phases

## Guiding Principles

1. **Dark launch first**: Every phase ends with something runnable against
   a few test dashboards (AAPL, TSLA, AVGO, AMZN style), not a big-bang.
2. **Vertical slices**: Each phase delivers a working end-to-end path
   (proto -> backend -> frontend), not horizontal layers.
3. **Static data first**: Use `StaticConfig` data sources until Phase 2.
   This decouples UI work from Flight SQL integration.
4. **Proto-first**: Generate code from protos before writing business logic.
   Types are never handwritten.
5. **File-based config**: Dashboards, data sources, and permissions are YAML
   files on disk. No database for config storage. Server is stateless.

## Phase Overview

```
Phase 0: Foundation        Proto codegen, build pipeline, dev environment
Phase 1: Static Dashboard  Hardcoded dashboard renders with static data
Phase 2: Flight SQL        Real data from Flight SQL servers
Phase 3: Variables         Dashboard filters and cascading controls
Phase 4: Auth              Authentication and authorization
Phase 5: Polish            Error handling, theming, event log, production readiness
```

---

## Phase 0: Foundation -- COMPLETE

**Goal**: Build pipeline works. Proto generates Rust and TypeScript types.
Dev environment runs locally. CI catches breakage.

### Backend

- [x] tonic + tonic-web server (`crates/open-plx-server/`)
- [x] `tonic-build` in `build.rs` for all 4 proto files
  - `dashboard.proto` -> Rust types + service trait
  - `data_source.proto` -> Rust types + service trait
  - `data.proto` -> Rust types
  - `widget_spec.proto` -> Rust types
- [x] Generated types compile: `cargo check --workspace`
- [x] Config loader (`crates/open-plx-config/`) reads YAML dashboards, data sources, permissions
- [x] Skeleton `DashboardService` impl
- [x] Skeleton `DataSourceService` impl
- [x] Skeleton Arrow Flight service impl
- [x] tonic-web layer wired up for browser clients
- [x] gRPC reflection enabled

### Frontend

- [x] Proto codegen: `protoc-gen-es` generates TypeScript types from all 4 proto files
- [x] Generated types compile: `pnpm tsc --noEmit`
- [x] gRPC-web client wrapper (`frontend/src/services/grpc/`) via @connectrpc/connect
- [x] Mapper layer stubs (`frontend/src/services/mappers/`)

### DevOps

- [x] GitHub Actions CI: `cargo check`, `cargo test`, `cargo clippy`, `pnpm tsc`, `pnpm build`
- [x] `buf.yaml` + `buf.gen.yaml` for proto generation

### Exit Criteria -- MET

- `cargo test --workspace` passes
- `pnpm build` produces a frontend bundle
- gRPC-web client can call skeleton `GetDashboard` from browser
- CI green on push

---

## Phase 1: Static Dashboard -- COMPLETE

**Goal**: A hardcoded dashboard renders in the browser with static data.
End-to-end vertical slice through every component.

### Backend

- [x] `DashboardService.GetDashboard` returns dashboard from YAML config
- [x] `DashboardService.ListDashboards` returns all configured dashboards
- [x] `WidgetDataService.GetWidgetData` resolves static data sources
  - Builds Arrow RecordBatches from `StaticConfig.columns`
  - Converts RecordBatch to proto DataColumns
- [x] `DataSourceService.ListDataSources` reads from config (read-only)
- [x] `DataSourceService.GetDataSource` reads from config (read-only)
- [x] `DataSourceService.ListDataSources` returns actual DataSource protos
- [x] `DataSourceService.GetDataSource` converts config to proto

### Frontend

- [x] Dashboard page: calls `GetDashboard`, renders grid
  - CSS grid with server-provided positions
  - Widget shells with Antd Card + title
- [x] Widget data fetching: `WidgetDataService.GetWidgetData` per widget
  - Parallel requests for all widgets
- [x] Chart mapper: `chartProtoToG2()` implementation
  - `CHART_TYPE_LINE` -> G2 line spec
  - `CHART_TYPE_BAR` -> G2 interval spec
  - `CHART_TYPE_DONUT` -> G2 interval + theta coordinate
  - `CHART_TYPE_AREA` -> G2 area spec
  - `CHART_TYPE_HORIZONTAL_BAR` -> G2 interval + transpose
  - `STACK_MODE_STACKED` / `GROUPED` / `PERCENT` -> transforms
  - Axis config, labels
- [x] Pivot table mapper: `pivotProtoToS2()` implementation
  - Fields, meta, sort, totals -> S2 config
  - Format string parser -> S2 formatter functions
- [x] Metric card renderer: value + format
- [x] Text renderer: plain/markdown
- [x] Widget type registry: maps WidgetType enum -> React component (exhaustive)
- [x] Error states: per-widget error cards (data fetch failure)
- [x] Loading states: per-widget spinner while data loads
- [x] Dashboard list page with navigation
- [x] Hash-based routing (`#dashboards/{name}`)

### Remaining

- [x] DataSourceService: return actual proto objects in List/Get (backend)
- [x] Dark launch test: company financials dashboard with specific layout
  - 3 metric cards (Revenue, EPS, Market Cap) using company-financials data
  - 1 line chart (revenue trend by quarter, 4 series: AAPL/TSLA/AVGO/AMZN)
  - 1 bar chart (revenue comparison, grouped)
  - 1 pivot table (company x quarter x metrics)

### Exit Criteria

- Browser renders a complete dashboard with 6+ widgets
- Chart mapper produces correct G2 specs (verified visually + e2e)
- Pivot table mapper produces correct S2 config
- Static data flows end-to-end (YAML config -> Arrow RecordBatch -> frontend)
- No hardcoded data in frontend -- all data comes from backend
- 96 e2e tests passing (Playwright, state-based)

---

## Phase 2: Flight SQL Integration -- COMPLETE

**Goal**: Real data from Flight SQL servers replaces static data.

### Backend

- [x] Flight SQL client in `open-plx-server`
  - Uses ADBC driver (`adbc` + `adbc-flightsql` crates)
  - Connection pooling per endpoint via `FlightSqlPool`
  - Timeout enforcement via `tokio::time::timeout`
  - Wired into both `WidgetDataService` and `FlightServiceImpl`
- [x] Basic auth for Flight SQL connections (handshake with username/password)
- [x] DuckDB Flight SQL test server via docker-compose with seed data
- [x] 15 integration tests: DuckDB (7), PostgreSQL (4), MySQL (4)
- [ ] `QueryParam` resolution pipeline (positional param binding, type coercion) -- deferred to Future
- [ ] `DataSourceService.TestDataSource` -- deferred to Future

---

## Phase 3: Variables & Filters -- COMPLETE

**Goal**: Dashboard variables with Antd controls. Re-fetch on change.

### Backend

- [x] Typed YAML variable models (DashboardVariableYaml, 7 control types)
- [x] YAML -> proto conversion for all variable types
- [x] Variables served in Dashboard proto (frontend renders controls)
- [x] Frontend-side resolution: variables resolved client-side, concrete values
  sent as `ParamValue` in `WidgetDataRequest.params`
- [ ] Dynamic options source fetching -- deferred to Future
- [ ] Cascading dependency resolution -- deferred to Future

### Frontend

- [x] `useVariables` hook: manages state, initializes from defaults, revision counter
- [x] `VariableBar` component: horizontal row above grid
- [x] `VariableControl` component: exhaustive switch on control oneof
  - [x] TextInputControl -> Antd Input
  - [x] NumberInputControl -> Antd InputNumber
  - [x] SelectControl -> Antd Select
  - [x] MultiSelectControl -> Antd Select mode="multiple"
  - [x] DatePickerControl -> Antd DatePicker
  - [x] DateRangeControl -> Antd DatePicker.RangePicker
  - [x] CascaderControl -> Antd Cascader
- [x] Variable values passed through DashboardGrid -> WidgetShell -> useWidgetData
- [x] Widget re-fetch on variable change via revision counter
- [ ] URL state for variable values -- deferred to Future

---

## Phase 4: Auth -- COMPLETE

**Goal**: Authentication and group-based authorization.

### Backend

- [x] `AuthProvider` trait + `AuthInterceptor` in `open-plx-auth`
- [x] Dev mode: accepts all, hardcoded principal (dev@localhost, admin group)
- [x] API Key: validates `x-api-key` header against config, resolves groups
- [x] OIDC JWT: stub (returns unimplemented, for future implementation)
- [x] tonic interceptor wiring (extracts Principal, injects into request extensions)
- [x] Permission checks in `DashboardService`
  - `GetDashboard`: viewer permission or NOT_FOUND
  - `ListDashboards`: filter by viewer permission
- [x] Permission checks in `WidgetDataService`
  - reader permission on data source or PERMISSION_DENIED
- [x] File-based permissions with wildcard support (`dashboards/*`)
- [x] 5 unit tests (dev auth, group/direct permission, wildcards)

### Frontend

- [x] `useWidgetData` detects `PERMISSION_DENIED` via `ConnectError`
- [x] `WidgetShell` renders "Access Denied" card with lock icon
- [ ] OIDC login flow (IdP redirect, JWT attachment) -- deferred to Future
- [ ] HIDE behavior for permission_denied_behavior -- deferred to Future

---

## Phase 5: Production Readiness -- COMPLETE

**Goal**: Error handling, theming, observability, deployment.

### Backend

- [x] Structured JSON logging via `RUST_LOG_FORMAT=json` (tracing-subscriber)
- [x] Event log: `dashboard.list`, `dashboard.view`, `data.fetch`, `permission.denied`
  with structured fields (user, resource, rows, duration_ms)
- [x] Health check endpoint (grpc.health.v1 via tonic-health)
- [x] Graceful shutdown (SIGTERM + Ctrl+C via tokio::signal)
- [x] Configuration validation at startup (CONFIG_PATH required, no defaults)
- [ ] OpenTelemetry distributed tracing -- deferred to Future
- [ ] Dashboard config versioning -- deferred to Future

### Frontend

- [x] Theme system: dark/light toggle, persisted to localStorage, system preference detection
- [x] Error boundary per widget (`WidgetErrorBoundary` -- crash isolation)
- [x] Refresh button on dashboard page
- [ ] Skeleton screens / progressive loading -- deferred to Future

### DevOps

- [x] Multi-stage Dockerfile (node:22 + rust:1.85 -> debian:bookworm-slim)
- [x] docker-compose.yml with DuckDB Flight SQL server + seed data
- [x] .dockerignore for efficient builds

---

## Future (Not Scheduled)

### Deferred from v1
- QueryParam resolution pipeline (positional param binding, type coercion)
- DataSourceService.TestDataSource (verify connection + schema)
- Dynamic variable options source fetching
- Cascading variable dependency resolution (topological sort)
- URL state for variable values (shareable dashboard links)
- OIDC JWT auth (real IdP integration)
- PERMISSION_DENIED_BEHAVIOR_HIDE (hide widgets entirely)
- OpenTelemetry distributed tracing
- Dashboard config versioning
- Skeleton screens / progressive Arrow streaming
- ADBC driver improvements (parameter binding, auth config conversion)

### New features
- ~~Dashboard YAML import/export CLI tool~~ -- DONE (`plx` binary in `crates/open-plx-cli/`)
- Visual dashboard editor (drag-and-drop widget placement)
- ~~Cross-widget interactions (click bar -> filter table)~~ -- DONE (click_interactions in dashboard config)
- ~~Widget conditional visibility (`visible_when` expression)~~ -- DONE (visible_when with 9 operators)
- Per-widget permission_denied_behavior override
- HTTP/3 (QUIC) support when tonic ecosystem matures
- Custom widget plugin system
- Data source result caching with configurable TTL
- Dashboard embedding (iframe with token auth)
- PostgreSQL metadata store (if config-file approach outgrows its limits)
