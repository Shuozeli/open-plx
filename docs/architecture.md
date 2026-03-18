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
- **Rust backend**: Stateless. Reads config from YAML files, proxies Flight SQL to frontend.
- **YAML config**: Dashboards, data sources, permissions, auth. No database.
- **Frontend**: gRPC-web for layout, Arrow Flight for data, mapper layer for G2/S2

## Proto-First Design

Proto files are the source of truth. All types are generated, not handwritten.

```
proto/open_plx/v1/
  dashboard.proto       DashboardService: layout CRUD
  data_source.proto     DataSourceService: admin CRUD, TestDataSource
  data.proto            WidgetDataRequest/Metadata for Arrow Flight descriptors
```

Arrow Flight's own proto (`arrow.flight.protocol.FlightService`) is provided
by the `arrow-flight` crate and not redefined.

## Backend Crate Map

```
crates/
  open-plx-core/        Generated proto types (tonic-build). Pure data, no IO.

  open-plx-config/      YAML config loader. Reads dashboards, data sources,
                         permissions from disk. No database. Depends on core.

  open-plx-auth/         Stateless auth. JWT validation, API key lookup,
                         permission checks against config. Depends on core, config.

  open-plx-server/       tonic gRPC server.
                         - DashboardService (serves config-file dashboards)
                         - DataSourceService (serves config-file data sources)
                         - Arrow Flight proxy (Flight SQL client -> frontend)
                         - tonic-web layer for browser clients
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
  components/
    widgets/             Widget renderers (G2, S2, Antd)
    layout/              Grid layout, dashboard shell
    variables/           Variable controls (Antd Input, Select, DatePicker, etc.)
  hooks/                 useDashboard, useWidgetData, useVariables
  services/
    grpc/                gRPC-web client (generated from protos)
    flight/              Arrow Flight client (GetFlightInfo + DoGet)
    mappers/             Semantic proto -> library config translators
      chartMapper.ts     ChartSpec -> G2 Spec
      pivotMapper.ts     PivotTableSpec -> S2 DataConfig + Options
      formatMapper.ts    Format strings -> formatter functions
  types/                 Generated TypeScript types from protos
  pages/                 Route-level pages
```

## Two-Phase Rendering Flow

```
Phase 1: Layout                          Phase 2: Data (parallel per widget)

Browser                    Server        Browser                    Server
  |                          |             |                          |
  | GetDashboard(name)       |             | GetFlightInfo(           |
  |------------------------->|             |   WidgetDataRequest)     |
  |                          |             |------------------------->|
  | Dashboard{               |             |                          |-- check data perm
  |   grid, widgets[],       |             | FlightInfo{              |
  |   version}               |             |   schema, ticket}        |
  |<-------------------------|             |<-------------------------|
  |                          |             |                          |
  | Render grid +            |             | DoGet(ticket)            |
  | widget shells            |             |------------------------->|
  |                          |             |                          |-- execute query
                                           | stream RecordBatches     |
                                           |<-------------------------|
                                           |                          |
                                           | Render data into widget  |
```

## Data Flow: Arrow End-to-End

```
Flight SQL Server    open-plx Backend       Frontend
                     (Flight SQL client +   (gRPC-web +
                      Flight server)         apache-arrow JS)
      |                     |                     |
      | <-- FlightSQL ---   |                     |
      |     Prepare($query) |                     |
      |                     |                     |
      | --- RecordBatches-> |                     |
      |                     | --- RecordBatches-> |
      |                     |     (Arrow Flight)  |
      |                     |                     |
                                                  | -> G2/S2
```

open-plx does NOT connect to databases directly. It is a **Flight SQL
client** that forwards Arrow RecordBatches to the frontend. Flight SQL
is the only data protocol. The entire pipeline is Arrow-native:

1. **Typed columns**: Int64, Float64, Utf8, Timestamp -- no type guessing
2. **Columnar end-to-end**: No row-to-column conversion anywhere
3. **Streaming**: Large datasets stream as RecordBatches progressively
4. **Zero-copy**: `apache-arrow` JS reads Arrow IPC without deserialization
5. **gRPC-native**: Arrow Flight IS a gRPC service. No protocol mismatch
6. **No SQL injection in open-plx**: Queries execute at the Flight SQL
   server via prepared statements with typed parameter binding
   (`ParamValue` -> `QueryParam.param_kind` coercion)

## Key Design References

- [Design Decisions](design.md) -- rationale for all major choices
- [Declarative Layout Spec](declarative-layout.md) -- widget spec language
- [Data Format Spec](data-format.md) -- Arrow Flight protocol details
- [Auth](auth.md) -- authn/authz design (groups, roles, interceptors)
- Proto definitions: `proto/open_plx/v1/`
