# Data Format Specification

## Overview

Widget data is served via **Apache Arrow Flight** over gRPC (HTTP/2). Arrow
provides a typed, columnar, zero-copy binary format that is efficient for
both transport and rendering.

This document specifies how data sources work, how Arrow Flight is used,
and the data contracts between backend and frontend.

## Why Arrow?

| Concern         | JSON                          | Arrow                              |
|-----------------|-------------------------------|-------------------------------------|
| Size            | Verbose, string-encoded       | Compact binary, columnar            |
| Types           | Implicit (number vs string)   | Explicit (Int64, Float64, Utf8, Timestamp, etc.) |
| Parsing         | Full deserialization           | Zero-copy, memory-mapped            |
| Chart libraries | Convert to typed arrays        | Already in columnar form            |
| Streaming       | Newline-delimited JSON        | Arrow IPC streaming (RecordBatch)   |

Arrow's columnar format aligns naturally with chart rendering: each column
becomes a data series, no row-to-column transposition needed.

## Arrow Flight Protocol

### Standard Flow

Arrow Flight defines a gRPC service with these key RPCs:

```
GetFlightInfo(FlightDescriptor) -> FlightInfo    // metadata + schema + ticket
DoGet(Ticket) -> stream FlightData               // Arrow RecordBatches
ListFlights(Criteria) -> stream FlightInfo        // discover available data
```

### open-plx Usage

#### Single Widget Data Fetch

```
1. Client builds FlightDescriptor:
     cmd = WidgetDataRequest {
       dashboard: "dashboards/quarterly-sales"
       widget_id: "revenue-trend"
       params: { "year": "2025" }
     }

2. Client calls GetFlightInfo(descriptor)
     Server:
       - Resolves widget -> data source
       - Checks data permission (returns PERMISSION_DENIED if denied)
       - Prepares query, determines schema
     Returns: FlightInfo {
       schema: Arrow schema (column names + types)
       endpoint: [ FlightEndpoint { ticket, location } ]
       app_metadata: WidgetDataMetadata (row count, truncation, etc.)
     }

3. Client calls DoGet(ticket)
     Server:
       - Executes the data source query
       - Streams results as Arrow RecordBatches
     Returns: stream of FlightData (Arrow IPC format)

4. Client receives RecordBatches, passes to G2/S2 for rendering.
```

#### Batch Discovery (All Widgets in a Dashboard)

```
1. Client calls ListFlights(Criteria { expression: dashboard_id })
     Server returns FlightInfo for each widget the user has data access to.

2. Client calls DoGet per ticket (parallel).
```

This enables the frontend to discover which widgets have data access and
fetch all data in parallel.

## Arrow Schema Conventions

### Column Types

| Data Kind    | Arrow Type               | Example                    |
|-------------|--------------------------|----------------------------|
| Text        | `Utf8`                   | region name, product ID    |
| Integer     | `Int64`                  | count, units sold          |
| Decimal     | `Float64`                | revenue, percentages       |
| Date        | `Date32`                 | calendar date              |
| Timestamp   | `Timestamp(Microsecond)` | event time                 |
| Boolean     | `Boolean`                | flag, status               |

### Schema Metadata

Arrow schemas support key-value metadata. open-plx uses:

```
metadata:
  "open_plx.widget_id": "revenue-trend"
  "open_plx.dashboard": "dashboards/quarterly-sales"
  "open_plx.data_timestamp": "2025-12-15T10:30:00Z"
  "open_plx.total_rows": "15000"
  "open_plx.truncated": "false"
```

### Column Metadata

Per-column metadata for frontend formatting hints:

```
column "revenue":
  metadata:
    "open_plx.format": "currency"
    "open_plx.currency": "USD"
    "open_plx.precision": "0"

column "margin":
  metadata:
    "open_plx.format": "percent"
    "open_plx.precision": "1"
```

These are **hints**, not commands. When a widget's `FieldMeta.formatter`
(in `PivotTableSpec`) or `MetricCardSpec.format` is set, it takes
precedence over Arrow column metadata. Arrow metadata is the fallback
for widgets that don't specify their own formatting.

## Data Source Types

open-plx does NOT connect directly to databases. All data access goes
through **Arrow Flight SQL**. The entire pipeline is Arrow-native.

### Flight SQL

open-plx connects to a Flight SQL server as a client. The server handles
database connections, query validation, and authorization.

```protobuf
config {
  flight_sql {
    endpoint: "grpc+tls://flight.databricks.com:443"
    auth { bearer_token_secret: "secret:databricks-token" }
    query: "SELECT month, region, SUM(revenue) as revenue FROM sales WHERE year = $1 GROUP BY month, region"
    params { name: "year"  position: 1  param_kind: PARAM_KIND_STRING  required: true }
  }
}
```

**Execution:**
1. Connect to Flight SQL endpoint with configured auth.
2. Prepare the statement via `FlightSqlClient::prepare()`.
3. Resolve `ParamValue` from widget (including `${variable_ref}` expansion).
4. Type-coerce `ParamValue` to Arrow type per `QueryParam.param_kind`.
5. Bind typed Arrow values to positional parameters ($1, $2, ...).
6. Execute and receive Arrow RecordBatches directly.
7. Stream back to frontend via open-plx's Arrow Flight service.

**Compatible servers**: Dremio, Databricks, DuckDB, InfluxDB,
Apache Arrow Flight SQL reference implementation, or any custom
Flight SQL server.

### Static

Hardcoded data for testing, reference values, and dark launch validation.
No external connection needed.

```protobuf
config {
  static_data {
    columns { name: "label"   arrow_type: ARROW_TYPE_UTF8   string_values: ["Q1", "Q2", "Q3", "Q4"] }
    columns { name: "target"  arrow_type: ARROW_TYPE_INT64  int_values: [100, 200, 300, 400] }
    columns { name: "actual"  arrow_type: ARROW_TYPE_INT64  int_values: [95, 210, 280, 420] }
  }
}
```

## Frontend Data Consumption

### Current: Proto DataColumns via WidgetDataService

The frontend receives data via `WidgetDataService.GetWidgetData`, which
returns proto `WidgetDataResponse` containing `DataColumn[]` (each column
has one of: string_values, int_values, double_values, bool_values).

The `useWidgetData` hook converts these proto columns to row objects:

```typescript
function columnsToRows(response: WidgetDataResponse): Record<string, unknown>[] {
  const numRows = Number(response.totalRows);
  const rows = new Array(numRows);
  for (let i = 0; i < numRows; i++) rows[i] = {};
  for (const col of response.columns) {
    const values = col.stringValues.length > 0 ? col.stringValues
      : col.doubleValues.length > 0 ? col.doubleValues
      : col.intValues.length > 0 ? col.intValues.map(Number)
      : col.boolValues;
    for (let i = 0; i < numRows; i++) rows[i][col.name] = values[i];
  }
  return rows;
}
```

These row objects are passed to chartMapper (G2) and pivotMapper (S2).

### Future: Direct Arrow Flight from Browser

A future optimization could use the `apache-arrow` JavaScript library
to consume Arrow IPC streams directly from the backend's Arrow Flight
service, avoiding the proto DataColumns intermediate format. This would
enable zero-copy typed arrays for large datasets.

## Permission Denied Response

When a user has layout access but not data access for a widget:

- `GetFlightInfo` returns `PERMISSION_DENIED` (gRPC status code 7).
- The frontend renders the widget shell (title, position) with an
  "Access Denied" state.
- No data is leaked -- the schema is also withheld.

## Caching Strategy

| Layer          | Cache Key                                    | TTL        |
|----------------|----------------------------------------------|------------|
| Server-side    | `(data_source_id, params_hash)`              | Configurable per data source |
| Arrow Flight   | FlightInfo includes `data_timestamp`         | Client compares timestamps   |
| Frontend       | Per-widget, keyed by `(dashboard, widget_id)` | Configurable refresh interval |

The server may return cached Arrow RecordBatches if the data has not changed.
`WidgetDataMetadata.data_timestamp` tells the frontend how fresh the data is.

## Streaming Large Datasets

For large result sets, the server streams multiple RecordBatches:

```
DoGet(ticket) ->
  FlightData { data_header: schema }
  FlightData { data_body: RecordBatch[0..999] }
  FlightData { data_body: RecordBatch[1000..1999] }
  ...
  FlightData { data_body: RecordBatch[N-999..N] }
```

The frontend can begin rendering as soon as the first batch arrives, with
progressive updates as more data streams in.

Default batch size: 1024 rows. Configurable per data source.
