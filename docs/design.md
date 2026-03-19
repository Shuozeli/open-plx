# open-plx Design Document

## 1. Problem Statement

Google PLX is an internal dashboard platform that lets teams define data
visualizations declaratively. It handles layout, data fetching, and
rendering through a server-driven model. There is no good open-source
equivalent that provides:

- Declarative dashboard definitions (not drag-and-drop builders)
- Server-driven layout with separate data fetching
- Fine-grained permission control (layout vs data access)
- High-quality chart and table rendering

open-plx aims to fill this gap.

## 2. Goals and Non-Goals

### Goals

- Declarative dashboard configs: define dashboards as structured data, not code
- Two-phase rendering: layout push, then per-widget data pull
- Separate permission layers for layout visibility and data access
- Rich widget library: charts (G2), pivot tables (S2), metrics, text
- Data source abstraction: widgets reference data sources, backend resolves them
- Single deployment artifact (Rust binary + static frontend assets)

### Non-Goals (for now)

- Drag-and-drop dashboard editor (admin tool, not end-user builder)
- Real-time collaborative editing
- Custom widget development by end users (plugin system)
- Mobile-native clients (web-only initially)
- Multi-tenancy (single-tenant deployment first)

## 3. Protocol: gRPC over HTTP/2

All communication uses **gRPC over HTTP/2**. No REST, no HTTP/1.1.

- **Backend**: `tonic` with `tonic-web` layer (for browser clients)
- **Frontend**: `@connectrpc/connect-web` client (types via `@bufbuild/protoc-gen-es`)
- **Data transport**: Apache Arrow Flight (gRPC-native) for widget data
- **Layout/admin**: Custom gRPC services for dashboard and data source CRUD
- **Future**: HTTP/3 (QUIC) when `tonic` ecosystem supports it

Proto files are the source of truth. Rust and TypeScript types are generated.

```
proto/open_plx/v1/
  dashboard.proto       -- DashboardService (layout CRUD + watch)
  data_source.proto     -- DataSourceService (admin CRUD)
  data.proto            -- Arrow Flight descriptor/metadata messages
```

See also:
- [Declarative Layout Spec](declarative-layout.md) -- full widget spec language
- [Data Format Spec](data-format.md) -- Arrow Flight protocol details

## 4. Core Abstraction: The Dashboard Config

The central concept is a **Dashboard** proto message -- a declarative
description of what a dashboard looks like and where its data comes from.
This is what the backend stores and serves via `DashboardService`.

```
Dashboard
  |-- name (resource ID: "dashboards/{id}")
  |-- title, description
  |-- version (optimistic concurrency)
  |-- GridConfig { columns, row_height, gap }
  |-- WidgetConfig[]
        |-- id, widget_type, title
        |-- GridPosition { x, y, w, h }
        |-- DataSourceRef { data_source, params }
        |-- spec (WidgetSpec oneof: ChartSpec | PivotTableSpec | MetricCardSpec | TextSpec)
```

### 4.1 Widget Types

| Type           | Renderer | Description                        |
|----------------|----------|------------------------------------|
| `LINE_CHART`   | G2       | Time series, trends                |
| `BAR_CHART`    | G2       | Comparisons, distributions         |
| `PIE_CHART`    | G2       | Proportions                        |
| `PIVOT_TABLE`  | S2       | Tabular data with pivoting         |
| `METRIC_CARD`  | Antd     | Single KPI with optional sparkline |
| `TEXT`         | Antd     | Static markdown/text               |

### 4.2 WidgetSpec Design

**Decision: Semantic spec with frontend mapper layer.**

The `spec` field uses semantic vocabulary (chart_type, data_mapping,
stack_mode, not G2/S2 API terms). The frontend's mapper layer translates
to the rendering library's native config. See Section 13.

Server controls (via spec): chart type, data-to-visual mapping, stacking,
axes, annotations, table structure.

Frontend controls (via theme + mapper): colors, fonts, animations,
tooltips, responsive behavior, library-specific config.

See [declarative-layout.md](declarative-layout.md) for full spec examples
per widget type.

### 4.3 Data Format: Arrow

Widget data is served via **Arrow Flight** (gRPC-native). All data sources
return typed, columnar Arrow RecordBatches.

See [data-format.md](data-format.md) for the full Arrow protocol spec.

## 5. Two-Phase Rendering Protocol

### Phase 1: Layout Fetch (gRPC)

```
Client                          Server
  |                                |
  |  GetDashboard(name)            |
  |------------------------------->|
  |                                | -- check layout permission
  |  Dashboard (proto)             |
  |<-------------------------------|
  |                                |
  |  Render grid + widget shells   |
  |  (titles, positions, loading)  |
```

### Phase 2: Data Fetch (Arrow Flight, per widget, parallel)

```
Client                          Server
  |                                |
  |  GetFlightInfo(                |
  |    WidgetDataRequest{          |
  |      dashboard, widget_id })   |
  |------------------------------->|
  |                                | -- check data permission
  |  FlightInfo (schema + ticket)  |
  |<-------------------------------|
  |                                |
  |  DoGet(ticket)                 |
  |------------------------------->|
  |                                | -- execute query
  |  stream Arrow RecordBatches    |
  |<-------------------------------|
  |                                |
  |  Render data into widget       |
```

If data permission is denied, `GetFlightInfo` returns `PERMISSION_DENIED`
(gRPC status 7). The widget shell stays visible with an "Access Denied"
state. No data or schema is leaked.

### Why Separate Phases?

| Concern              | Separate                          | Combined                     |
|----------------------|-----------------------------------|------------------------------|
| Permission granularity | Per-widget data checks           | All-or-nothing per dashboard |
| Initial render speed | Layout renders immediately        | Blocked by slowest data source |
| Cacheability         | Layout: long TTL, Data: short TTL | Single mixed TTL             |
| Independent refresh  | Per-widget intervals              | Full dashboard refresh       |
| Complexity           | More round trips, loading states  | Simpler protocol             |

We accept the complexity trade-off for the permission and caching benefits.

## 6. Data Source Abstraction

**open-plx does NOT connect directly to databases.** All data access goes
through **Arrow Flight SQL**. open-plx is a Flight SQL client only.

```
DataSource
  |-- name ("dataSources/{id}")
  |-- display_name, description
  |-- config (oneof):
        |-- FlightSqlConfig { endpoint, auth, query, params: QueryParam[] }
        |-- StaticConfig { columns[] }
```

### Why Flight SQL Only?

- **No SQL injection surface in open-plx**: The Flight SQL server handles
  query parsing and execution. open-plx only sends parameterized prepared
  statements via Flight SQL's standard parameter binding.
- **No database credentials in open-plx**: Auth is per-Flight-endpoint
  (bearer token, mTLS, basic auth), not per-database.
- **Arrow-native end-to-end**: Data flows as Arrow RecordBatches from the
  Flight SQL server through open-plx to the frontend. No row-to-column
  conversion anywhere.
- **Standard protocol**: Any Flight SQL server works (Dremio, Databricks,
  DuckDB, InfluxDB, or a custom bridge).
- **Our own client lib**: Uses `arrow-adbc-rs`
  (https://github.com/Shuozeli/arrow-adbc-rs.git) for the Flight SQL
  client. We own this library and can extend it as needed.
- **Typed parameters**: `QueryParam` defines the name, position, and
  `ParamKind` for each parameter. `ParamValue` from the widget is
  type-coerced and validated before Flight SQL binding.

See `proto/open_plx/v1/data_source.proto` for the full typed schema.
See [data-format.md](data-format.md) for the data flow details.

Multiple widgets can share a data source with different params. The backend
can optimize by batching identical queries across widgets.

## 7. Dashboard Variables

Dashboards support named variables that widgets reference in their
`data_source.params` via `${variable_name}` syntax. This enables:

- **Shared filters**: A date range picker that affects all widgets
- **Cross-widget parameters**: Select a region in one widget, filter others
- **Dynamic defaults**: Variables have default values; the frontend renders
  appropriate controls (dropdowns, date pickers, text inputs)

```protobuf
// In Dashboard message:
variables {
  name: "selected_region"
  label: "Region"
  default_value { string_value: "ALL" }
  select {
    options { value: "ALL"  label: "All Regions" }
    options { value: "US"   label: "United States" }
    options { value: "EU"   label: "Europe" }
  }
}

// In WidgetConfig.data_source:
data_source {
  data_source: "dataSources/sales-flight"
  params { key: "region"  value { variable_ref: "${selected_region}" } }
}
```

Variable controls map to Antd components (Input, InputNumber, Select,
DatePicker, DatePicker.RangePicker, Cascader). Select and MultiSelect
controls can load options from a data source query.

### Rendering Order

```
Phase 0: Variable Initialization (topological order)
  - Build dependency graph from options_source variable_ref references
  - Topological sort (reject cycles at validation time)
  - Initialize in order: render control, fetch options, set default
  - Cascading supported: Country -> State -> City

Phase 1: Layout Fetch (gRPC)
  - GetDashboard -> grid, widgets, variables

Phase 2: Data Fetch (Arrow Flight, parallel per widget)
  - Resolve ${variable_ref} -> ParamValue
  - Type-coerce ParamValue -> QueryParam.param_kind
  - Bind and execute via Flight SQL
```

When an upstream variable changes, downstream variables re-fetch their
options (topological order), then all affected widgets re-fetch data.

When a variable changes, the frontend re-fetches data for all widgets that
reference it. The layout does not change -- only data is refreshed.

### Variable-to-Param Pipeline

When the backend resolves a widget's data source params:

```
1. Widget's DataSourceRef.params:
     { "year": { variable_ref: "${selected_year}" },
       "regions": { variable_ref: "${selected_regions}" },
       "date_range": { variable_ref: "${report_dates}" } }

2. Resolve variable references (from frontend-provided current values):
     selected_year -> ParamValue { string_value: "2025" }
     selected_regions -> ParamValue { string_list: { values: ["US", "EU"] } }
     report_dates -> ParamValue { date_range: { start: "2025-01-01", end: "2025-12-31" } }

3. Match to FlightSqlParam declarations (by name):
     "year"       -> position: 1, param_kind: PARAM_KIND_STRING
     "regions"    -> position: 2, param_kind: PARAM_KIND_STRING_LIST
     "date_range" -> position: 3, param_kind: PARAM_KIND_DATE_RANGE

4. Type coercion + binding:
     $1 = Utf8("2025")
     $2 = List<Utf8>(["US", "EU"])
     $3 = Date32(2025-01-01), $4 = Date32(2025-12-31)   // range expands to 2 positions

5. Validation: if ParamValue type doesn't match ParamKind, reject with error.
     e.g., string_value for PARAM_KIND_INT -> error, not silent cast.
```

### Multi-Value Control Expansion

| Control Type | ParamValue Type | Binding |
|-------------|----------------|---------|
| TextInput | `string_value` | Single `Utf8` param |
| NumberInput | `int_value` or `double_value` | Single `Int64`/`Float64` param |
| Select | `string_value` | Single `Utf8` param |
| MultiSelect | `string_list` | `List<Utf8>` for IN-clause |
| DatePicker | `string_value` (ISO 8601) | Single `Date32` param |
| DateRange | `date_range` | Two `Date32` params at position and position+1 |
| Cascader | `string_value` (leaf value) | Single `Utf8` param |

**v1 scope**: Variables are defined in the proto schema and rendered by the
frontend. Cross-widget interactions (click bar A -> filter table B) are
deferred to v2.

## 8. Permission Model

Two independent permission layers:

### Layer 1: Layout Permission

- Checked when: `GetDashboard` / `ListDashboards`
- Grants: visibility of dashboard structure, widget titles, positions
- Denies with: `NOT_FOUND` (gRPC status 5 -- dashboard is invisible)

### Layer 2: Data Permission

- Checked when: `GetFlightInfo` for a widget's data
- Grants: access to the underlying data source
- Denies with: `PERMISSION_DENIED` (gRPC status 7 -- widget visible, data restricted)

### Permission Denied Behavior

Configurable per dashboard via `permission_denied_behavior`:

- `SHOW_DENIED` (default): Widget shell visible with "Access Denied" message.
  User knows the widget exists but cannot see data. Good for transparency.
  Layout is consistent across all users.
- `HIDE`: Widget hidden entirely. User doesn't know it exists. Good for
  sensitive dashboards where even the existence of data is restricted.

**HIDE creates layout gaps.** When widgets are hidden, the grid has holes
where hidden widgets would be. The frontend does NOT re-flow the grid
(re-flowing would make layouts unpredictable across permission levels).
This is a known UX trade-off: `HIDE` is best used when entire rows of
widgets are hidden (e.g., all widgets at y=4 require the same permission),
not for individual widgets in a mixed row. This is documented as an open
question -- a future option could be per-widget `permission_denied_behavior`
override instead of dashboard-level.

### Permission Storage & Auth

Permissions use a groups + roles model (not direct user-resource mapping).
Authentication is pluggable via external identity providers (OIDC, reverse
proxy, API key).

See [auth.md](auth.md) for the full authn/authz design including:
- Group-based permission model with role inheritance
- `AuthProvider` trait for pluggable identity providers
- Permission resolution algorithm
- File-based permission config (YAML)
- Future: `AuthService` gRPC API for managing groups and permissions

## 9. Auth

See [auth.md](auth.md) for the complete design. Current implementation
is file-based (groups and permissions defined in YAML). The auth.md doc
includes a future database schema design for when the project outgrows
file-based config.

## 10. gRPC Services

### DashboardService

```protobuf
service DashboardService {
  rpc ListDashboards(...)  returns (ListDashboardsResponse);
  rpc GetDashboard(...)    returns (Dashboard);
  rpc CreateDashboard(...) returns (Dashboard);
  rpc UpdateDashboard(...) returns (Dashboard);    // field mask
  rpc DeleteDashboard(...) returns (DeleteDashboardResponse);
}
```

### DataSourceService

```protobuf
service DataSourceService {
  rpc ListDataSources(...)  returns (ListDataSourcesResponse);
  rpc GetDataSource(...)    returns (DataSource);
  rpc CreateDataSource(...) returns (DataSource);
  rpc UpdateDataSource(...) returns (DataSource);
  rpc DeleteDataSource(...) returns (DeleteDataSourceResponse);
  rpc TestDataSource(...)   returns (TestDataSourceResponse);
}
```

### Arrow Flight (widget data)

Standard Arrow Flight gRPC service. open-plx encodes `WidgetDataRequest`
in the `FlightDescriptor.cmd` field. See `proto/open_plx/v1/data.proto`.

Full proto definitions: `proto/open_plx/v1/`

## 11. Frontend Architecture

### Rendering Pipeline

```
1. Fetch Dashboard           -- DashboardService.GetDashboard (gRPC-web)
2. Render grid skeleton      -- CSS grid (server-provided positions)
3. For each widget:
   a. Resolve WidgetComponent -- lookup in WIDGET_REGISTRY by type
   b. Render shell (title, card, loading spinner)
   c. GetWidgetData            -- WidgetDataService (gRPC-web)
   d. Convert proto DataColumns -> row objects
   e. Merge server spec + theme defaults
   f. Render chart/table/card -- G2 / S2 / Antd
4. User-triggered refresh re-fetches layout + data on demand
```

### Widget Registry

Static registry mapping widget type enums to React components. Adding a
new widget type requires a frontend code change. Intentionally not dynamic
to keep it type-safe.

### Theme System

Frontend owns all visual presentation: color palettes, typography, spacing,
animations, responsive breakpoints. The server never sends styling.

## 12. Backend Architecture

### Crate Structure

```
crates/
  open-plx-core/       -- Generated proto types (tonic-build)
                          Pure data, no IO, no dependencies beyond serde/prost
  open-plx-config/     -- YAML config loader + YAML->proto converter
                          Reads dashboards, data sources, permissions from disk
  open-plx-auth/       -- Stateless auth (AuthProvider trait: Dev, ApiKey, OIDC stub)
                          File-based permission checks with wildcard support
  open-plx-server/     -- tonic gRPC server + tonic-web + Arrow Flight + WidgetDataService
                          Flight SQL client pool for upstream data sources
```

Dependency graph (no cycles):
```
server --> auth --> config --> core
  |                             ^
  +-----------------------------+
```

### Event Log

Event logging uses structured tracing (not a database table). Events are
emitted as structured log lines with fields like `event`, `user`,
`resource`, `rows`, `duration_ms`. When `RUST_LOG_FORMAT=json`, these
become queryable JSON log entries.

Logged events:
- `dashboard.list` -- user listed dashboards (count)
- `dashboard.view` -- user viewed a dashboard
- `data.fetch` -- widget data fetched (data_source, rows, duration_ms)
- `permission.denied` -- access denied (resource, required_role)

## 13. Library Decoupling

### Semantic Spec, Not Library Config

The `ChartSpec` proto uses **semantic vocabulary** (chart_type, data_mapping,
stack_mode) instead of G2-specific terms (mark_type, encode, transform).
The frontend has a **mapper layer** that translates the semantic spec to
the rendering library's native config:

```
Proto (semantic)              Mapper               Library (G2/S2)
────────────────              ──────               ───────────────
CHART_TYPE_LINE           ->  chartMapper()    ->  { type: "line", encode: {...} }
CHART_TYPE_BAR            ->  chartMapper()    ->  { type: "interval", encode: {...} }
CHART_TYPE_DONUT          ->  chartMapper()    ->  { type: "interval", coordinate: { type: "theta", innerRadius: 0.6 } }
STACK_MODE_STACKED        ->  chartMapper()    ->  transform: [{ type: "stackY" }]
PivotTableSpec            ->  pivotMapper()    ->  { dataCfg: {...}, options: {...} }
```

Benefits:
- **Library upgrades**: Changing G2 v5 -> v6 only changes the mapper, not the proto
- **Library swap**: Switching to ECharts/Vega-Lite means writing a new mapper
- **Domain language**: Dashboard authors think in "line chart with grouping"
  not "interval mark with dodgeX transform"

The mapper lives in `frontend/src/services/mappers/`.

## 14. Open Questions

### Resolved

- **Dashboard variables**: Added. `DashboardVariable` in proto, widgets
  reference via `${variable_name}` syntax. (Section 7)
- **Permission denied behavior**: Configurable per dashboard:
  `SHOW_DENIED` or `HIDE`. (Section 8)
- **Auth provider**: Pluggable `AuthInterceptor` trait. OIDC JWT for
  production, dev-mode bypass for local. (Section 9)
- **DataSource config typing**: Typed `oneof` per source type, not
  `Struct`. SQL injection mitigated via prepared statements. (Section 6)

### Open

1. **Widget conditional visibility**: Should a widget config support
   `visibleWhen: { variable: "region", value: "US" }`?
   Now that we have variables, this is expressible. Defer to v2.

2. **Dashboard authoring**: How do admins create configs?
   (a) raw JSON/YAML, (b) admin gRPC API + CLI tool, (c) visual editor.
   Propose (b) for v1.

3. **Data source error handling**: Error message vs last cached value?
   Propose error message for v1.

4. **Config versioning**: Propose yes -- `version` counter +
   `config_history` table for rollback/audit.

5. ~~G2/S2 decoupling~~: Resolved. Proto now uses semantic vocabulary.
   Frontend mapper layer handles translation. (Section 13)

6. **HIDE layout gaps**: When `PERMISSION_DENIED_BEHAVIOR_HIDE` removes
   widgets, the grid has holes. No re-flow (would break layout consistency).
   Consider per-widget override or row-level permission grouping.

7. ~~Cascading variable dependencies~~: Resolved. Variables can reference
   other variables in options_source.params. Dependency graph must be
   acyclic (validated on save). Frontend resolves in topological order.
   (Section 7, declarative-layout.md)
