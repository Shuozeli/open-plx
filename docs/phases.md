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

## Phase 0: Foundation

**Goal**: Build pipeline works. Proto generates Rust and TypeScript types.
Dev environment runs locally. CI catches breakage.

### Backend

- [ ] Replace axum scaffolding with tonic + tonic-web
- [ ] Set up `tonic-build` in `build.rs` for all 4 proto files
  - `dashboard.proto` -> Rust types + service trait
  - `data_source.proto` -> Rust types + service trait
  - `data.proto` -> Rust types
  - `widget_spec.proto` -> Rust types
- [ ] Verify generated types compile: `cargo check --workspace`
- [ ] Set up sqlx with PostgreSQL (docker.yuacx.com:5432)
  - Initial migration: `dashboards`, `data_sources`, `event_log` tables
  - All operations wrapped in transactions
- [ ] Skeleton `DashboardService` impl (returns hardcoded data)
- [ ] Skeleton Arrow Flight service impl (returns empty RecordBatches)
- [ ] tonic-web layer wired up for browser clients

### Frontend

- [ ] Set up proto codegen: `buf generate` or `protoc-gen-grpc-web`
  - Generate TypeScript types from all 4 proto files
  - Verify generated types compile: `pnpm tsc --noEmit`
- [ ] gRPC-web client wrapper (`frontend/src/services/grpc/`)
- [ ] Arrow Flight client wrapper (`frontend/src/services/flight/`)
- [ ] `apache-arrow` integration: verify RecordBatch deserialization works
- [ ] Mapper layer stubs (`frontend/src/services/mappers/`)
  - `chartMapper.ts` -- stub that returns empty G2 spec
  - `pivotMapper.ts` -- stub that returns empty S2 config
  - `formatMapper.ts` -- format string parser

### DevOps

- [ ] docker-compose.yml: PostgreSQL (metadata store only)
- [ ] GitHub Actions CI: `cargo check`, `cargo test`, `pnpm tsc`, `pnpm build`
- [ ] `.env.example` with required config (no defaults -- fail if missing)
- [ ] `buf.yaml` + `buf.gen.yaml` for proto generation

### Exit Criteria

- `cargo test --workspace` passes
- `pnpm build` produces a frontend bundle
- gRPC-web client can call skeleton `GetDashboard` from browser
- CI green on push

---

## Phase 1: Static Dashboard

**Goal**: A hardcoded dashboard renders in the browser with static data.
End-to-end vertical slice through every component.

### Backend

- [ ] `DashboardService.GetDashboard` returns a hardcoded dashboard proto
  - 3 metric cards, 1 line chart, 1 bar chart, 1 pivot table
  - Uses `StaticConfig` data sources (no Flight SQL yet)
- [ ] `DashboardService.ListDashboards` returns the hardcoded dashboard
- [ ] Arrow Flight `GetFlightInfo` + `DoGet` for static data sources
  - Build Arrow RecordBatches from `StaticConfig.columns`
  - Stream back via Flight service
- [ ] `DataSourceService` CRUD (backed by PostgreSQL)
  - Create, read, update, delete data sources
  - Store `FlightSqlConfig` / `StaticConfig` as proto bytes in JSONB column
- [ ] Event log: log `dashboard.view` and `data.fetch` events

### Frontend

- [ ] Dashboard page: calls `GetDashboard`, renders grid
  - react-grid-layout with server-provided positions
  - Widget shells with Antd Card + title + Spin
- [ ] Widget data fetching: calls `GetFlightInfo` + `DoGet` per widget
  - Parallel requests for all widgets
  - Arrow RecordBatch deserialization via `apache-arrow`
- [ ] Chart mapper: `chartProtoToG2()` implementation
  - `CHART_TYPE_LINE` -> G2 line spec
  - `CHART_TYPE_BAR` -> G2 interval spec
  - `CHART_TYPE_DONUT` -> G2 interval + theta coordinate
  - `STACK_MODE_STACKED` / `GROUPED` / `PERCENT` -> transforms
  - Axis config, labels, annotations
- [ ] Pivot table mapper: `pivotProtoToS2()` implementation
  - Fields, meta, sort, totals, frozen, pagination -> S2 config
  - Format string parser -> S2 formatter functions
- [ ] Metric card renderer: Antd Statistic + comparison + sparkline
- [ ] Text renderer: markdown rendering
- [ ] Widget type registry: maps WidgetType enum -> React component
- [ ] Error states: per-widget error cards (data fetch failure)
- [ ] Loading states: per-widget Spin while data loads

### Dark Launch Test

Create a test dashboard via `DataSourceService.CreateDataSource` +
database seed script with:
- Static data mimicking AAPL/TSLA/AVGO/AMZN quarterly financials
- 3 metric cards (Revenue, EPS, Market Cap)
- 1 line chart (revenue trend by quarter, 4 series)
- 1 bar chart (revenue comparison, grouped)
- 1 pivot table (company x quarter x metrics)

Verify: dashboard loads, all 6 widgets render with correct data.

### Exit Criteria

- Browser renders a complete dashboard with 6 widgets
- Chart mapper produces correct G2 specs (verified visually)
- Pivot table mapper produces correct S2 config
- Arrow data flows end-to-end (backend static -> Arrow Flight -> frontend)
- No hardcoded data in frontend -- all data comes from backend

---

## Phase 2: Flight SQL Integration

**Goal**: Real data from Flight SQL servers replaces static data.

### Backend

- [ ] Flight SQL client in `open-plx-server`
  - Use `arrow-adbc-rs` (https://github.com/Shuozeli/arrow-adbc-rs.git -- our own lib)
  - Connect to Flight SQL endpoint with auth (bearer, basic, mTLS)
  - Prepare statement, bind typed params, execute, stream RecordBatches
- [ ] `QueryParam` resolution pipeline
  - Map `DataSourceRef.params` (ParamValue) to positional params
  - Type coercion: `ParamValue` -> `QueryParam.param_kind` -> Arrow type
  - Reject mismatches with descriptive errors
  - DATE_RANGE expansion to two positional params
  - STRING_LIST expansion to List<Utf8>
- [ ] Connection pooling for Flight SQL endpoints
- [ ] Query timeout enforcement
- [ ] `DataSourceService.TestDataSource` -- verify connection + schema
- [ ] Event log: log `data.fetch` with query timing, row count

### Testing

- [ ] Set up a DuckDB Flight SQL server for integration tests
  - Load test data (company financials CSV)
  - Run as docker container in CI
- [ ] Integration test: create data source -> create dashboard -> fetch data
- [ ] Test param type coercion for all ParamKind values
- [ ] Test query timeout behavior
- [ ] Test auth methods (bearer token, basic auth, no auth)

### Dark Launch Test

- Replace static data sources with DuckDB Flight SQL data sources
- Same test dashboard (AAPL/TSLA/AVGO/AMZN) now reads from DuckDB
- Verify: identical rendering, data comes from Flight SQL

### Exit Criteria

- Dashboard renders with data from a real Flight SQL server
- All ParamKind coercion paths tested
- Connection errors surface as widget-level error cards (not page crash)
- DuckDB Flight SQL in CI for automated testing

---

## Phase 3: Variables & Filters

**Goal**: Dashboard variables with Antd controls. Cascading. Re-fetch on change.

### Backend

- [ ] Variable resolution in the data pipeline
  - Resolve `${variable_ref}` in `DataSourceRef.params`
  - Substitute resolved ParamValue before Flight SQL binding
- [ ] Options source fetching
  - When a variable has `options_source`, execute the data source query
  - Return options as part of the dashboard response (or separate RPC)

### Frontend

- [ ] Variable bar component (renders above the grid)
  - Layout: horizontal row of Antd controls
  - Maps variable control oneof -> Antd component
- [ ] Variable controls:
  - [ ] TextInputControl -> Antd Input
  - [ ] NumberInputControl -> Antd InputNumber
  - [ ] SelectControl -> Antd Select (static + dynamic options)
  - [ ] MultiSelectControl -> Antd Select mode="multiple"
  - [ ] DatePickerControl -> Antd DatePicker
  - [ ] DateRangeControl -> Antd DatePicker.RangePicker (with presets)
  - [ ] CascaderControl -> Antd Cascader
- [ ] Dependency resolution
  - Build dependency graph from options_source variable_refs
  - Topological sort, reject cycles (frontend validation)
  - Initialize in topo order at Phase 0
- [ ] Cascading behavior
  - When upstream variable changes: re-fetch downstream options
  - Reset downstream value if current value no longer in options
  - Re-fetch widget data for all affected widgets
- [ ] URL state: variable values in URL query params for shareability

### Dark Launch Test

Add to the test dashboard:
- Year select (static options: 2023, 2024, 2025)
- Company multi-select (dynamic options from Flight SQL)
- Date range picker (with "Last Quarter" preset)
- Verify: changing year re-fetches all widgets, multi-select filters data

### Exit Criteria

- All 7 variable control types render and produce correct ParamValue
- Cascading works: Country -> State -> City
- Variable changes trigger widget data refresh
- URL reflects variable state (shareable dashboard links)

---

## Phase 4: Auth

**Goal**: Authentication and group-based authorization.

### Backend

- [ ] `AuthInterceptor` trait in `open-plx-auth`
- [ ] OIDC JWT implementation (verify signature via JWKS)
- [ ] API Key implementation (database lookup)
- [ ] Dev mode implementation (accept all, hardcoded principal)
- [ ] Plugin loading for custom auth
- [ ] tonic interceptor wiring (extracts Principal, injects into request)
- [ ] Database migrations: `groups`, `group_members`, `permissions` tables
- [ ] `AuthService` gRPC implementation
  - Group CRUD + membership management
  - Permission grant/revoke
  - `GetEffectivePermissions` for frontend introspection
- [ ] Permission checks in `DashboardService`
  - `GetDashboard`: role_level >= 10 (viewer) or NOT_FOUND
  - `ListDashboards`: filter by permission
  - `CreateDashboard` / `UpdateDashboard` / `DeleteDashboard`: role_level >= 50 (editor)
- [ ] Permission checks in Arrow Flight service
  - `GetFlightInfo`: role_level >= 1 (reader) on data source or PERMISSION_DENIED
- [ ] `permission_denied_behavior`: SHOW_DENIED vs HIDE
- [ ] Bootstrap CLI: `open-plx admin bootstrap --email admin@example.com`
- [ ] Event log: log `permission.denied`, `permission.granted`, `permission.revoked`

### Frontend

- [ ] Auth flow: redirect to IdP, handle JWT, attach to gRPC metadata
- [ ] Permission-aware UI:
  - Hide edit controls when role < editor
  - Show "Access Denied" card on PERMISSION_DENIED widgets
  - Hide widgets entirely when `PERMISSION_DENIED_BEHAVIOR_HIDE`

### Exit Criteria

- OIDC login flow works end-to-end
- Group-based permissions: assign group to dashboard, all members see it
- Data-level permission denial shows correct widget state
- Bootstrap CLI creates first admin
- CI tests with dev-mode auth

---

## Phase 5: Production Readiness

**Goal**: Error handling, theming, observability, deployment.

### Backend

- [ ] Structured error responses (gRPC status codes + details)
- [ ] Request tracing (OpenTelemetry integration)
- [ ] Health check endpoint
- [ ] Graceful shutdown
- [ ] Configuration validation at startup (fail-fast, no defaults)
- [ ] Event log retention / cleanup
- [ ] Dashboard config versioning (`version` field + `config_history` table)

### Frontend

- [ ] Theme system (light/dark mode, color palettes)
- [ ] Dashboard list/browser page
- [ ] Responsive grid breakpoints
- [ ] Error boundary per widget (widget crash doesn't take down page)
- [ ] User-triggered refresh (re-fetch layout + data)
- [ ] Loading performance: skeleton screens, progressive Arrow streaming

### DevOps

- [ ] Dockerfile: multi-stage build (Rust binary + frontend static assets)
- [ ] docker-compose.yml: full stack (open-plx + postgres + duckdb-flight)
- [ ] Deployment docs
- [ ] Backup/restore for dashboard configs

### Exit Criteria

- Production-deployable Docker image
- Handles errors gracefully (no white screens, no silent failures)
- Observable (traces, event log)
- Theme works (light + dark)
- Refresh button works

---

## Future (Not Scheduled)

- Dashboard YAML/JSON import/export (for version control)
- Visual dashboard editor (drag-and-drop widget placement)
- Cross-widget interactions (click bar -> filter table)
- Widget conditional visibility (`visible_when` expression)
- Per-widget permission_denied_behavior override
- HTTP/3 (QUIC) support when tonic ecosystem matures
- Custom widget plugin system
- Data source result caching with configurable TTL
- Dashboard embedding (iframe with token auth)
