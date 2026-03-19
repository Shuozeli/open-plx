# Architecture

## Overview

open-plx is a server-driven dashboard platform. All communication is
**gRPC over HTTP/2**. No REST, no HTTP/1.1.

- Dashboard layout CRUD: custom gRPC services (`DashboardService`, `DataSourceService`)
- Widget data: Apache Arrow Flight (gRPC-native, columnar binary format)
- Browser access: `tonic-web` layer translates gRPC-web for browser clients
- Future: HTTP/3 (QUIC) when ecosystem matures

## System Diagram

```
+-------------------+                        +-------------------+
| Flight SQL Server |                        |  React Frontend   |
| (Dremio, DuckDB,  |  Arrow RecordBatches   |  (Vite + Antd +   |
|  Databricks, etc.) | <-----------------+   |   gRPC-web +      |
+-------------------+                    |   |   apache-arrow)   |
        ^                               |   +-------------------+
        | Flight SQL                     |           ^
        | (prepare + execute)            |           |
        |                                |    gRPC (HTTP/2)
        |                               |    Layout + Data
        |              +-------------------+
        +------------- |                   | ------------>
                       |   Rust Backend    |
                       |   (tonic +        |
+-------------------+  |    tonic-web +    |
| YAML Config Files |  |    Flight SQL     |
| (dashboards,      |->|    client)        |
|  data sources,    |  |                   |
|  permissions)     |  +-------------------+
+-------------------+
```

- **Flight SQL servers**: Own the data, execute queries (Dremio, Databricks, DuckDB, etc.)
- **Rust backend**: Stateless. Reads config from YAML files. Proxies Flight SQL data as proto columns.
- **YAML config**: Dashboards, data sources, permissions, auth. No database.
- **Frontend**: gRPC-web for layout (DashboardService) and data (WidgetDataService), mapper layer for G2/S2

## Proto-First Design

Proto files are the source of truth. All types are generated, not handwritten.

```
proto/open_plx/v1/
  dashboard.proto       DashboardService: layout CRUD
  data_source.proto     DataSourceService: admin CRUD, TestDataSource
  data.proto            WidgetDataService: browser data access (proto columnar)
                        WidgetDataRequest/Response, Arrow Flight descriptors
  widget_spec.proto     All widget spec types (ChartSpec, PivotTableSpec, etc.)
```

Arrow Flight's own proto (`arrow.flight.protocol.FlightService`) is provided
by the `arrow-flight` crate and not redefined.

## Backend Crate Map

```
crates/
  open-plx-core/        Generated proto types (tonic-build). Pure data, no IO.
                         Exports `pb` module and FILE_DESCRIPTOR_SET for gRPC reflection.

  open-plx-config/      YAML config loader + YAML->proto converter + static data builder.
                         Reads dashboards, data sources, permissions from disk.
                         No database. Depends on core.

  open-plx-auth/         Stateless auth. AuthProvider trait with DevAuth,
                         ApiKeyAuth (config-based), OidcAuth (stub).
                         File-based permission checks with wildcard support.
                         Depends on core, config.

  open-plx-server/       tonic gRPC server with 5 services:
                         - DashboardService (serves config-file dashboards, read-only)
                         - DataSourceService (serves config-file data sources, read-only)
                         - WidgetDataService (resolves data, returns proto DataColumns)
                         - Arrow Flight service (GetFlightInfo + DoGet)
                         - Health check (grpc.health.v1)
                         Plus: tonic-web, gRPC reflection, CORS, graceful shutdown.
                         Flight SQL client pool for upstream data sources.
                         Depends on core, config, auth.
```

Dependency graph (strictly acyclic):
```
server --> auth --> config --> core
  |                             ^
  +-----------------------------+
```

## Frontend Package Map

```
frontend/src/
  gen/open_plx/v1/       Generated TypeScript types from protos (via @bufbuild/protoc-gen-es)
  components/
    widgets/             Widget renderers: G2Chart, S2PivotTable, Line/Bar/Pie/Metric/Text, WidgetRegistry
    layout/              DashboardGrid (CSS grid), WidgetShell, WidgetErrorBoundary
    variables/           VariableBar, VariableControl (7 Antd control types)
  hooks/                 useDashboard, useDashboardList, useWidgetData, useVariables, useThemeContext
  services/
    grpc/                gRPC-web client via @connectrpc/connect-web (3 service clients)
    mappers/             Semantic proto -> library config translators
      chartMapper.ts     ChartSpec -> G2 Spec (10 chart types)
      pivotMapper.ts     PivotTableSpec -> S2 DataConfig + Options + format string parser
    testRegistry.ts      Test helper for widget data state
  pages/                 DashboardPage, DashboardListPage
```

Note: The frontend fetches widget data via `WidgetDataService.GetWidgetData`
(proto columnar format), NOT via Arrow Flight directly. The Arrow Flight
service exists on the backend but is not consumed by the current frontend.

## Two-Phase Rendering Flow

```
Phase 1: Layout                          Phase 2: Data (parallel per widget)

Browser                    Server        Browser                    Server
  |                          |             |                          |
  | GetDashboard(name)       |             | GetWidgetData(           |
  |------------------------->|             |   dashboard, widget_id,  |
  |                          |             |   params)                |
  | Dashboard{               |             |------------------------->|
  |   grid, widgets[],       |             |                          |-- check data perm
  |   variables[], version}  |             |                          |-- resolve data source
  |<-------------------------|             |                          |-- query Flight SQL or static
  |                          |             | WidgetDataResponse{      |
  | Render grid +            |             |   columns[], total_rows} |
  | widget shells +          |             |<-------------------------|
  | variable controls        |             |                          |
                                           | Render data into widget  |
```

Note: The backend also exposes a standard Arrow Flight service
(GetFlightInfo + DoGet) which could be used by non-browser clients.
The current browser frontend uses WidgetDataService for simplicity.

## Data Flow

```
Flight SQL Server    open-plx Backend          Frontend
                     (Flight SQL client +      (gRPC-web +
                      WidgetDataService)        @connectrpc/connect)
      |                     |                        |
      | <-- FlightSQL ---   |                        |
      |     execute($query) |                        |
      |                     |                        |
      | --- RecordBatches-> |                        |
      |     (Arrow IPC)     |                        |
      |                     | --- DataColumns -----> |
      |                     |     (proto columnar,   |
      |                     |      via gRPC-web)     |
      |                     |                        | -> G2/S2
```

open-plx does NOT connect to databases directly. It is a **Flight SQL
client** that queries upstream data sources. The backend converts Arrow
RecordBatches to proto DataColumns for the browser. The Arrow pipeline
is native between the Flight SQL server and the backend:

1. **Typed columns**: Int64, Float64, Utf8, Timestamp -- no type guessing
2. **Arrow-native backend**: Data flows as Arrow RecordBatches from Flight SQL to backend
3. **Proto columnar to browser**: Backend converts Arrow to proto DataColumns (string/int/double/bool arrays)
4. **gRPC-native**: Arrow Flight IS a gRPC service. WidgetDataService is also gRPC.
5. **No SQL injection in open-plx**: Queries execute at the Flight SQL
   server. open-plx sends queries as strings (parameter binding deferred to future).

## Key Design References

- [Design Decisions](design.md) -- rationale for all major choices
- [Declarative Layout Spec](declarative-layout.md) -- widget spec language
- [Data Format Spec](data-format.md) -- Arrow Flight protocol details
- [Auth](auth.md) -- authn/authz design (groups, roles, interceptors)
- Proto definitions: `proto/open_plx/v1/`
