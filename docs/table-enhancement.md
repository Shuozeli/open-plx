# Dashboard Enhancement Plan

<!-- agent-updated: 2026-04-11T00:00:00Z -->

## Context

The v1 delivery invested heavily in chart widgets (11 chart types, rich declarative spec with axes, scales, transforms, annotations, sort, stack modes). Table support was intentionally minimal: a basic flat table with column selection, pagination, and conditional formatting.

Real-world dashboards have three critical gaps: (1) tables need sort/filter/export/cell rendering, (2) no graph/network visualization, and (3) the backend only speaks Flight SQL while most upstream services are gRPC. This proposal addresses all three across 9 phases.

**Scope A (Phases A–F): Table enhancements** — make flat tables competitive with charts.
**Scope B (Phase G): Graph widget** — new widget type using AntV G6 for network visualization.
**Scope C (Phase H): Row-action columns** — buttons that invoke gRPC calls, adding write capability.
**Scope D (Phase I): gRPC data source adapter** — query any existing gRPC service as a data source.
**Scope E (Phase J): URL variable binding** — link `?ticker=AAPL` to dashboard variables for shareable deep-links.

## Current State

### Implemented for Tables

| Feature | Proto Field | Mapper | S2 Wired |
|---------|-------------|--------|----------|
| Column selection | `TableSpec.columns[]` | Yes | Yes |
| Display names/formatters | `TableSpec.meta[]` | Yes | Yes |
| Pagination | `TableSpec.pagination` | Yes | Yes |
| Row numbers | `TableSpec.showRowNumbers` | Yes | Yes |
| Conditional formatting | `TableSpec.conditions[]` | Yes | Yes |
| Copy interaction | `TableInteraction.enableCopy` | Partial | No |
| Hover highlight | `TableInteraction.enableHoverHighlight` | Partial | No |
| Resize | `TableInteraction.enableResize` | Partial | No |
| Multi-selection | `TableInteraction.enableMultiSelection` | Partial | No |
| Range selection | `TableInteraction.enableRangeSelection` | Partial | No |
| Frozen columns | `PivotTableSpec.frozen` | Yes (pivot only) | N/A |

### Implemented for Pivot Tables (reference)

Pivot tables have richer S2 integration (totals, sub-totals, hierarchy type, frozen rows/cols, series number, pagination, sort params, conditions). Most of these capabilities should be available in flat tables too.

---

## Phase A: Sort & Filter

**Goal**: Users can click column headers to sort, and search/filter table data.

### Proto Changes (`widget_spec.proto`)

```protobuf
// In TableSpec
message TableSpec {
  // ... existing fields ...

  // Table-level view config
  TableViewConfig view = 7;
}

message TableViewConfig {
  bool enable_search = 1;           // Show search box
  string search_placeholder = 2;    // Placeholder text (default: "Search...")
  bool case_sensitive = 3;          // Search is case-sensitive (default: false)
  repeated string search_fields = 4; // Fields to search. If empty, all fields.
}

// In TableColumn
message TableColumn {
  // ... existing fields ...
  bool sortable = 5;                // Enable click-to-sort on this column
  bool filterable = 6;             // Enable column header filter dropdown
  TableFilterConfig filter = 7;   // Default filter for this column
}

message TableFilterConfig {
  TableFilterType type = 1;       // What kind of filter UI to show
  repeated string filter_values = 2; // Pre-selected values (for multi-select)
}

enum TableFilterType {
  TABLE_FILTER_TYPE_UNSPECIFIED = 0;
  TABLE_FILTER_TYPE_LIST = 1;      // Checkbox list in dropdown
  TABLE_FILTER_TYPE_TEXT = 2;      // Text input for contains/equals
  TABLE_FILTER_TYPE_RANGE = 3;    // Min/max number inputs
}
```

### Frontend Changes

- `tableMapper.ts`: Map `view.enableSearch` -> `options.s2Options.search`.
  Map `column.sortable` -> S2's `sort` config. Map `column.filterable` -> S2's `FilterField`.
- `S2Table.tsx`: Expose `onSearch` callback for search state if needed.
- S2 TableSheet supports `sort`, `FilterField`, and `search` natively -- map from proto.

### Exit Criteria

- Dashboard YAML can declare `sortable: true` on a column and S2 renders a sortable header.
- `enableSearch: true` shows the S2 search box.
- `filterable: true` shows column filter dropdown in S2.

---

## Phase B: Column Features

**Goal**: Pin columns left/right, wire resize, hide columns, control column order.

### Proto Changes (`widget_spec.proto`)

```protobuf
// In TableSpec
message TableSpec {
  // ... existing fields ...

  // Columns frozen to the left side (by field name).
  repeated string frozen_cols_left = 8;

  // Columns frozen to the right side (by field name).
  repeated string frozen_cols_right = 9;

  // Default sort: field + direction (applied on initial load).
  TableDefaultSort default_sort = 10;
}

message TableDefaultSort {
  string field = 1;
  SortDirection direction = 2;
}

// In TableColumn
message TableColumn {
  // ... existing fields ...
  bool hidden = 8;                  // Hide this column initially
  int32 order = 9;                 // Display order (lower = earlier). Default: field index.
}

// Add to TableInteraction
message TableInteraction {
  bool enable_copy = 1;
  bool enable_hover_highlight = 2;
  bool enable_resize = 3;
  bool enable_multi_selection = 4;
  bool enable_range_selection = 5;
  bool enable_column_drag = 6;      // Allow drag-to-reorder columns
}
```

### Frontend Changes

- `tableMapper.ts`: Map `frozenColsLeft`/`frozenColsRight` -> S2 `frozenCols`. Map `hidden` -> exclude from `fields.columns` but keep in `meta` for reference. Map `order` -> sort columns by `order` before passing to S2. Wire `enableResize`, `enableColumnDrag` to S2 interaction config.
- `TableWidget.tsx`: If columns are reorderable, handle drag events and update widget state.

### Exit Criteria

- `frozenColsLeft: ["id", "name"]` pins those columns on the left in S2.
- `hidden: true` on a column excludes it from display but it still exists in data.
- Column drag-to-reorder works in the browser.

---

## Phase C: Row Selection & Export

**Goal**: Row selection feeds into cross-widget interactions. CSV/Excel export is available.

### Proto Changes (`widget_spec.proto`)

```protobuf
// In TableSpec
message TableSpec {
  // ... existing fields ...

  // Row selection config
  TableSelectionConfig selection = 11;

  // Data export config
  TableExportConfig export = 12;
}

message TableSelectionConfig {
  // Enable row selection (checkbox column).
  bool enabled = 1;

  // Only one row selectable at a time.
  bool single = 2;

  // Whether selection persists when user navigates to another page.
  bool persistent = 3;
}

message TableExportConfig {
  // Enable CSV export via S2's built-in export.
  bool enable_csv = 1;

  // Enable Excel export via S2's built-in export.
  bool enable_excel = 2;

  // Custom filename template (without extension). Supports {dashboardName}, {widgetId}, {timestamp}.
  string filename_template = 3;
}
```

### Frontend Changes

- `tableMapper.ts`: Map `selection` -> S2 `rowSelection` config. Map `export` -> store export config in widget options (S2's `sheet.exportFile()` is called imperatively).
- `TableWidget.tsx`:
  - Expose selected rows via `onRowSelect` callback (similar to `onClickInteraction`).
  - Add export toolbar button when `export` is configured.
  - Selected rows state feeds into `useVariables` or a dedicated selection store so other widgets can react.
- Cross-widget: selected rows from a table can set a dashboard variable, which filters another widget.

### Exit Criteria

- Selecting rows in a table and clicking a "Filter" button in a linked widget causes the target widget to re-fetch with the selected row's key as a filter param.
- "Export CSV" button downloads the table data as a CSV file.

---

## Phase D: Advanced Cell Rendering

**Goal**: Icon cells (status indicators), bar cells (inline progress bars), link cells (clickable URLs).

### Proto Changes (`widget_spec.proto`)

```protobuf
// In TableColumn
message TableColumn {
  // ... existing fields ...

  // Cell renderer for this column.
  TableCellRenderer renderer = 10;
}

message TableCellRenderer {
  oneof renderer {
    // Text (default): plain text, no special rendering.
    TableCellRendererText text = 1;
    // Icon: show an icon based on cell value.
    TableCellRendererIcon icon = 2;
    // Bar: inline mini horizontal bar chart (0-max per column).
    TableCellRendererBar bar = 3;
    // Link: clickable URL. Opens in new tab.
    TableCellRendererLink link = 4;
    // Progress: shows filled/total as a progress bar (e.g., "75/100").
    TableCellRendererProgress progress = 5;
  }
}

message TableCellRendererText {
  // Alias for the default. No extra config.
}

message TableCellRendererIcon {
  // Map cell value -> icon name.
  // Example: { "active": "check-circle", "inactive": "minus-circle" }
  map<string, string> value_to_icon = 1;
  // Fallback icon when value doesn't match any key.
  string fallback_icon = 2;
}

message TableCellRendererBar {
  // Numeric field for bar width. If not set, uses the cell value itself.
  string value_field = 1;
  // Maximum value for the bar (for normalization). If not set, uses column max.
  double max_value = 2;
  // Show numeric label next to the bar.
  bool show_label = 3;
  // Bar color (hex or theme color name).
  string color = 4;
}

message TableCellRendererLink {
  // URL template. Supports field interpolation: "https://example.com/{id}"
  string url_template = 1;
  // Whether to open in new tab.
  bool new_tab = 2;
}

message TableCellRendererProgress {
  // Field containing the "current" / filled value.
  string value_field = 1;
  // Field containing the "total" value. If not set, max_value is used.
  string total_field = 2;
  // Fixed max if total_field is not set.
  double max_value = 3;
  // Show "current / total" label.
  bool show_label = 4;
}
```

### Frontend Changes

- `tableMapper.ts`: Map each `renderer` variant to S2's `conditions` or custom cell component.
  - `icon` -> S2 `icon` condition with value-to-icon mapping.
  - `bar` -> S2 custom cell with mini SVG/CSS bar.
  - `link` -> S2 text condition with `href` and `newTab` attributes.
  - `progress` -> S2 `interval` condition styled as progress bar.
- S2's cell rendering is done via `conditions` + custom shape. The mapper translates proto to S2's condition schema.
- `S2Table.tsx`: Handle link clicks via `onCellClick` and navigate/open tabs.

### Exit Criteria

- A column declared as `icon` renderer shows check/x icons based on value.
- A column declared as `link` renderer is clickable and opens the URL.
- A column declared as `progress` renderer shows an inline progress bar.

---

## Phase E: Expandable Rows & Cell Spanning

**Goal**: Hierarchical / tree table with expandable rows. Merged cells for grouping.

### Proto Changes (`widget_spec.proto`)

```protobuf
// In TableSpec
message TableSpec {
  // ... existing fields ...

  // Row expansion config.
  TableExpandConfig expandable = 13;

  // Cell spanning config.
  repeated TableColSpan col_spans = 14;
}

message TableExpandConfig {
  // Enable expandable/collapsible rows.
  bool enabled = 1;
  // The field that uniquely identifies a row (for expansion state).
  string row_id_field = 2;
  // Fields that act as hierarchy levels (for tree table layout).
  repeated string hierarchy_fields = 3;
  // Whether to expand all rows by default.
  bool default_expanded = 4;
}

message TableColSpan {
  // Field (column) to apply spanning to.
  string field = 1;
  // Condition: a CEL expression evaluated per row.
  // If true, the cell spans `col_span` columns.
  string condition = 2;
  // How many columns to span.
  int32 col_span = 3;
}
```

### Frontend Changes

- `tableMapper.ts`: Map `expandable` -> S2 tree data configuration. Map `colSpans` -> S2 `colCellContext`.
- S2 `TableSheet` supports hierarchical data via `dataCfg.fields.rowId` and custom expansion state.
- `TableWidget.tsx`: Track expansion state and pass to S2.

### Exit Criteria

- A table with `expandable: { enabled: true, hierarchyFields: ["category", "subcategory"] }` renders as a tree table.
- `colSpans` merges cells where the condition evaluates true.

---

## Phase F: Server-Side Pagination & Virtualization

**Goal**: Large tables use server-side pagination. Frontend only fetches the current page.

### Why

S2 TableSheet is a client-side table. For tables with >10K rows, fetching all data and letting S2 virtualize is workable but not ideal. Server-side pagination keeps payloads small.

### Proto Changes (`widget_spec.proto`)

```protobuf
// In TableSpec
message TableSpec {
  // ... existing fields ...

  // Server-side pagination. When set, frontend requests one page at a time.
  ServerSidePagination server_pagination = 15;
}

message ServerSidePagination {
  // Page size. Frontend sends page number (1-based) in WidgetDataRequest.
  int32 page_size = 1;
  // Whether to show total row count in the UI.
  bool show_total_count = 2;
  // Whether to show page size selector (10, 25, 50, 100).
  bool show_page_size_selector = 3;
}
```

### Backend Changes

- `WidgetDataService.GetWidgetData`: When the widget is a TABLE with `serverPagination` set:
  - Read `page_number` from `WidgetDataRequest.params` (add new param key).
  - Apply `LIMIT page_size OFFSET (page_number - 1) * page_size` to the Flight SQL query.
  - Return only the current page rows.
  - Return total row count in a new `WidgetDataResponse.metadata.total_rows` field (add to proto).
- Proto change: Add `int64 total_rows = 100;` to `WidgetDataResponse` (or a new `TableDataResponse` message if we want type-specific fields).

### Frontend Changes

- `tableMapper.ts`: Map `serverPagination` -> S2 pagination config with `current` from state.
- `TableWidget.tsx`: Track `currentPage` state. On page change, call `useWidgetData` with new `page_number` param and update data.

### Exit Criteria

- Table with `serverPagination: { pageSize: 50 }` shows 50 rows with pagination controls.
- Page navigation (next/prev/page number) triggers a new `GetWidgetData` call with the correct offset.

---

## Phase G: Graph / Network Widget

**Goal**: Render network graphs (nodes + edges) using AntV G6. Enables signal engine dependency visualization, entity graph exploration, and relationship diagrams.

### Background

Three domains need this: `signal_engine.network`, `knowledge_store.entity_graph`, and `knowledge.relationship_explorer`. G6 is the natural choice — it's the AntV graph library (same family as G2 and S2) and has good React integration.

This is a new widget type, not a table enhancement. It requires:
1. New `WidgetSpec` oneof variant
2. New proto message for graph data
3. New G6 mapper
4. New React component
5. Widget registry entry

### Proto Changes (`widget_spec.proto`)

```protobuf
// Add to WidgetType enum in dashboard.proto
WIDGET_TYPE_GRAPH = 18;

// Add to WidgetSpec oneof
message WidgetSpec {
  oneof spec {
    ChartSpec chart = 1;
    // ...
    GraphSpec graph = 11;  // NEW
  }
}

message GraphSpec {
  // Data source for nodes and edges.
  // Expected data shape: rows with source/target/value columns.
  GraphDataMapping data_mapping = 1;

  // Visual layout.
  GraphLayout layout = 2;

  // Node appearance.
  GraphNodeStyle node_style = 3;

  // Edge appearance.
  GraphEdgeStyle edge_style = 4;

  // Interaction config.
  GraphInteraction interaction = 5;
}

message GraphDataMapping {
  // Field containing source node id.
  string source_field = 1;
  // Field containing target node id.
  string target_field = 2;
  // Field containing edge weight/value (optional).
  string value_field = 3;
  // Field containing node label (optional, falls back to source/target id).
  string label_field = 4;
  // Field containing node group/category (for color encoding).
  string group_field = 5;
}

message GraphLayout {
  GraphLayoutType type = 1;
  // For force-directed: number of iterations.
  int32 iterations = 2;
  // For grid/tree: direction (TB, LR, RL, BT).
  string direction = 3;
  // Node separation.
  int32 node_spacing = 4;
  // Edge separation.
  int32 rank_spacing = 5;
}

enum GraphLayoutType {
  GRAPH_LAYOUT_TYPE_UNSPECIFIED = 0;
  GRAPH_LAYOUT_TYPE_FORCE = 1;     // Force-directed (d3-force)
  GRAPH_LAYOUT_TYPE_DAGRE = 2;    // Directed acyclic graph (tb/lr/rl/bt)
  GRAPH_LAYOUT_TYPE_CIRCULAR = 3;  // Circular layout
  GRAPH_LAYOUT_TYPE_GRID = 4;      // Grid layout
  GRAPH_LAYOUT_TYPE_CONCENTRIC = 5; // Concentric circles by centrality
}

message GraphNodeStyle {
  // Node size field (optional, uses value_field if not set).
  string size_field = 1;
  // Fixed node size in pixels.
  double size = 2;
  // Node color field (uses group_field if not set).
  string color_field = 3;
  // Default node color.
  string color = 4;
  // Show node labels.
  bool show_label = 5;
  // Label font size.
  int32 label_font_size = 6;
  // Node shape (circle/square/diamond/triangle).
  string shape = 7;
}

message GraphEdgeStyle {
  // Edge color.
  string color = 1;
  // Edge width field (uses value_field if not set).
  string width_field = 2;
  // Fixed edge width in pixels.
  double width = 3;
  // Show edge labels.
  bool show_label = 4;
  // Edge style (solid/dashed/dotted).
  string style = 5;
  // Show arrow heads for directed edges.
  bool show_arrow = 6;
  // Arrow size.
  double arrow_size = 7;
}

message GraphInteraction {
  // Enable drag nodes.
  bool enable_drag = 1;
  // Enable zoom and pan.
  bool enable_zoom = 2;
  // Enable node click to filter other widgets.
  bool enable_click_select = 3;
  // Enable hover tooltip.
  bool enable_tooltip = 4;
  // Enable edge click.
  bool enable_edge_click = 5;
  // Enable node collapse/expand for hierarchical data.
  bool enable_node_collapse = 6;
}
```

### Frontend Changes

- **`graphMapper.ts`** (NEW): Translates `GraphSpec` proto to G6 configuration.
  - Map `dataMapping` -> G6 node/edge data format (`{ id, label, group }` for nodes, `{ source, target, value }` for edges)
  - Map `layout.type` -> G6 layout plugin (`'force'`, `'dagre'`, `'circular'`, `'grid'`, `'concentric'`)
  - Map `nodeStyle` -> G6 node style config
  - Map `edgeStyle` -> G6 edge style config
  - Map `interaction` -> G6 behavior config
- **`GraphWidget.tsx`** (NEW): React component wrapping `@antv/g6`.
  - Use `useRef` for the canvas container
  - Initialize G6 graph in `useEffect`
  - Handle resize via `ResizeObserver`
  - Pass click/hover events to `onClickInteraction` for cross-widget linking
- **`WidgetRegistry.tsx`**: Add `GRAPH` case mapping to `GraphWidget`.
- **`dashboard.proto`**: Add `WIDGET_TYPE_GRAPH = 18` to `WidgetType` enum.

### Backend Changes

- No backend changes required for a new widget type — the widget type enum just needs to be recognized in `WidgetDataService` (which it already will be via the exhaustive match).

### G6 Integration Notes

G6 is not yet in `vendor/`. Options:
1. Add G6 as a npm dependency: `pnpm add @antv/g6`
2. Add G6 as a git submodule in `vendor/G6` (matching the G2/S2 pattern)

Option 1 is simpler and matches how the project already handles React-compatible libraries. The `vendor/` submodules are reference copies for exploring API — actual runtime dependencies come from npm.

### Exit Criteria

- A dashboard with `widgetType: GRAPH` and `spec.graph { dataMapping { source: "from", target: "to", value: "weight" } }` renders a force-directed graph.
- Clicking a node fires `onClickInteraction` with the node id, enabling cross-widget filtering.

---

## Phase H: Row-Action Columns

**Goal**: Add action buttons to table rows that invoke gRPC calls. This breaks the "frontend is a pure function of config + data" property — the frontend becomes write-capable.

### Why this matters

Most dashboard platforms are read-only. But real ops workflows need actions: "Restart" a service, "Acknowledge" an alert, "Zoom to" a map. Without row actions, every such workflow requires a separate React page. Row actions keep the action in the dashboard context.

### Design Tension

This is architecturally significant. The current model:
```
config (YAML) + data (Arrow) -> React (pure function)
```

Row actions introduce:
```
config (YAML) + data (Arrow) + actions (gRPC methods) -> React (impure)
```

The server must now serve not just layout + data, but also available actions per row. This requires a new proto message and a new service.

### Proto Changes

**New file: `action.proto`**

```protobuf
syntax = "proto3";
package open_plx.v1;

message TableAction {
  string id = 1;              // Unique action id within the widget
  string label = 2;           // Button label
  string icon = 3;             // Optional AntD icon name
  ActionStyle style = 4;       // primary/secondary/danger/link
  string confirm_message = 5;  // If set, show confirmation dialog before invoking
  ActionGrpcCall grpc_call = 6; // The gRPC call to make
}

enum ActionStyle {
  ACTION_STYLE_UNSPECIFIED = 0;
  ACTION_STYLE_PRIMARY = 1;
  ACTION_STYLE_SECONDARY = 2;
  ACTION_STYLE_DANGER = 3;
  ACTION_STYLE_LINK = 4;
}

message ActionGrpcCall {
  // The fully-qualified gRPC method: "package.ServiceName/MethodName"
  string method = 1;
  // Template for the request body. Field paths support variable interpolation:
  // "{row.id}" -> value from the clicked row's column "id"
  // "{row._selected}" -> comma-separated list of selected row keys
  string request_template = 2;
  // Where to put the result: "variable" (set a dashboard variable) or "refresh" (re-fetch widget data)
  ActionResultHandling result_handling = 3;
}

enum ActionResultHandling {
  ACTION_RESULT_HANDLING_UNSPECIFIED = 0;
  // Set the result as a dashboard variable value.
  ACTION_RESULT_HANDLING_SET_VARIABLE = 1;
  // Refresh the widget that owns this action.
  ACTION_RESULT_HANDLING_REFRESH = 2;
  // Show result in a toast notification (for non-state-changing actions).
  ACTION_RESULT_HANDLING_TOAST = 3;
}

// Extend TableColumn to support action type
message TableColumn {
  // ... existing fields ...
  TableAction action = 11;  // NEW: if set, this column renders an action button
}
```

**Add to `data.proto`**: New `WidgetActionService.InvokeAction` RPC:
```protobuf
service WidgetActionService {
  rpc InvokeAction(InvokeActionRequest) returns (InvokeActionResponse);
}

message InvokeActionRequest {
  string dashboard_name = 1;
  string widget_id = 2;
  string action_id = 3;
  // Serialized JSON matching the request_template.
  string request_body = 4;
}

message InvokeActionResponse {
  bool success = 1;
  string message = 2;
  // For SET_VARIABLE result_handling: the variable name and value.
  string variable_name = 3;
  string variable_value = 4;
}
```

### Frontend Changes

- `tableMapper.ts`: Map `column.action` -> column renders as action button (not data cell).
- `S2Table.tsx`: Handle `onCellClick` where the cell is an action column.
  - Show confirmation dialog if `action.confirmMessage` is set.
  - Call `widgetActionClient.invokeAction()`.
  - Handle response: set variable or refresh.
- Add `WidgetActionService` client to `grpc/clients.ts`.

### Backend Changes

- New `WidgetActionService` in `open-plx-server`:
  - Route `InvokeAction` to the appropriate upstream gRPC service.
  - Forward authenticated principal (from auth interceptor).
  - Apply permission check: does the user have permission to invoke this action?
  - Permission check: add `ActionPermission` to `permissions.yaml`.
- `open-plx-config`: Add `actions[]` to `TableSpecYaml`.

### Security Considerations

1. **Action permission**: Users must have explicit `actions` permission on the widget to invoke any action. Without it, the action column is hidden server-side.
2. **Request template validation**: Server must validate that interpolated field paths actually exist in the row schema before forwarding.
3. **Rate limiting**: Actions should be rate-limited to prevent accidental loops (user clicks button -> variable changes -> widget re-fetches -> action column re-renders -> could re-fire).
4. **Audit logging**: Every action invocation should be logged as an event with user, widget, action_id, and result.

### Exit Criteria

- A table with `columns[0].action: { id: "restart", label: "Restart", style: DANGER, confirmMessage: "Are you sure?", method: "ops.Service/RestartInstance" }` renders a red "Restart" button.
- Clicking "Restart" shows a confirmation dialog, then calls `WidgetActionService.InvokeAction`, which forwards to `ops.Service/RestartInstance`.
- After action completes, the table refreshes (or a variable is set, per `resultHandling`).

---

## Phase I: gRPC Data Source Adapter

**Goal**: Query any existing gRPC service as a widget data source, not just Flight SQL servers. Enables open-plx to adopt existing services without a Flight SQL migration.

### Background

open-plx currently only supports Flight SQL data sources. Most upstream services are plain gRPC. Wrapping each service in a Flight SQL adapter is a 20×N lift (N services). The clean path is a `GrpcProxy` data source type that resolves through `ui_proxy`'s existing gRPC-to-columnar conversion.

### Design

New `DataSourceConfig` variant:
```yaml
# In config/data_sources/<name>.yaml
data_source:
  name: "signal-network"
  grpc_proxy:
    # The upstream gRPC service definition (must be in the proto search path).
    service: "signal_engine.NetworkService"
    # The RPC method to call.
    method: "GetGraph"
    # Request template with variable interpolation.
    request_template:
      graph_id: "${graph_id}"
      depth: 3
      filter: "${status}"
    # Response schema: maps RPC response fields to DataColumns.
    # If absent, infer from first response (streaming or unary).
    response_schema:
      columns:
        - field: "node_id"
          type: STRING
        - field: "label"
          type: STRING
        - field: "cpu"
          type: DOUBLE
        - field: "memory"
          type: DOUBLE
```

### Proto Changes (`data_source.proto`)

```protobuf
message DataSourceConfig {
  oneof config {
    FlightSqlConfig flight_sql = 1;
    StaticConfig static = 2;
    GrpcProxyConfig grpc_proxy = 3;  // NEW
  }
}

message GrpcProxyConfig {
  // The fully-qualified gRPC service name (e.g. "signal_engine.NetworkService").
  string service = 1;
  // The RPC method name (e.g. "GetGraph").
  string method = 2;
  // Request template: map of field name to ParamValue.
  // Supports variable_ref interpolation.
  map<string, ParamValue> request_template = 3;
  // Optional: explicit response schema. If absent, infer from first response.
  ResponseSchema response_schema = 4;
}

message ResponseSchema {
  repeated ColumnSchema columns = 1;
}

message ColumnSchema {
  string field = 1;
  DataType type = 2;  // STRING, INT64, DOUBLE, BOOL, TIMESTAMP
}
```

### Backend Changes

**`crates/open-plx-config/`**:
- Add `GrpcProxyConfigYaml` to `model.rs`.
- Add conversion from `GrpcProxyConfigYaml` -> `pb::GrpcProxyConfig` in `convert.rs`.

**`crates/open-plx-server/`**:
- New `GrpcProxyClient` in `widget_data.rs` (or a separate file):
  - Maintains a pool of gRPC channels per service name.
  - Uses `tonic` to make unary calls to the upstream service.
  - Deserializes the response using the `response_schema`.
  - Converts the response fields to Arrow `RecordBatch`.
  - Uses the same `DataColumn` -> proto conversion as Flight SQL path.
- The `WidgetDataService.GetWidgetData` handler adds a branch for `GrpcProxyConfig`:
  ```rust
  match config.config {
    Some(Config::GrpcProxy(cfg)) => self.grpc_proxy_client.fetch(cfg, params).await,
    // ... existing branches
  }
  ```

**Response schema inference** (if `response_schema` is absent):
- Make a single unary call with default values for all template params.
- Inspect the returned protobuf message fields.
- Infer types: `string` -> STRING, `int64` -> INT64, `double` -> DOUBLE, `bool` -> BOOL, `google.protobuf.Timestamp` -> TIMESTAMP.
- Cache the inferred schema in memory (per service+method combination).

### Columnar Conversion

The existing `ui_proxy` already has logic for "given a protobuf message, extract fields to DataColumns". This should be extracted into a shared helper in `open-plx-core` so both `ui_proxy` and `open-plx-server` use it.

### Auth Passthrough

When `GrpcProxyClient` calls an upstream gRPC service:
1. Extract the authenticated principal from the request extensions (set by `AuthInterceptor`).
2. Forward it via gRPC metadata (e.g., `x-auth-principal` header or context).
3. The upstream service is responsible for validating the forwarded principal.

This requires a convention: either `x-auth-principal` metadata or a dedicated `Authorization` header. The `AuthInterceptor` already knows the principal — it just needs to forward it.

### Exit Criteria

- A data source with `grpc_proxy { service: "signal_engine.NetworkService", method: "GetGraph" }` works without any Flight SQL server.
- A widget referencing that data source renders graph data from the gRPC response.
- The upstream gRPC service receives the authenticated principal via metadata.

---

## Phase J: URL Variable Binding

**Goal**: Dashboard variables bind to URL query parameters, enabling deep-links like `/dashboards/stock-detail?ticker=AAPL`.

### Why

Variables currently only persist in React state. Refreshing the page resets them. Users can't share a filtered dashboard view. This is a common UX gap — every dashboard platform eventually needs URL binding.

### Frontend Changes

**`hooks/useVariables.ts`**:
- On mount, read URL search params and initialize variable values from them.
- When a variable changes, update the URL search params via `history.replaceState` (no page reload).
- Use `URLSearchParams` API for parsing and serializing.

**`pages/DashboardPage.tsx`**:
- On mount, call `useVariables().initFromUrl()`.
- After variable changes, sync to URL.

**`hooks/useVariables.ts`** changes:
```typescript
export function useVariables() {
  // ... existing state ...

  // Initialize from URL on mount.
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    for (const [name, value] of params) {
      if (name in variableValues) {
        setVariableValue(name, value);
      }
    }
  }, []);

  // Sync to URL on change.
  useEffect(() => {
    const params = new URLSearchParams();
    for (const [name, value] of Object.entries(variableValues)) {
      params.set(name, String(value));
    }
    const newUrl = `${window.location.pathname}?${params.toString()}`;
    window.history.replaceState(null, '', newUrl);
  }, [variableValues]);
}
```

### URL Parameter Conventions

- Parameter name = variable name (exact match).
- Single-value params: `?ticker=AAPL`
- Multi-select params: `?regions=US&regions=EU` (repeated params)
- Date range params: `?range=2025-01-01,2025-12-31` (comma-separated ISO dates)

### Server-Side (Optional but Recommended)

`GetDashboardRequest` can accept initial variable values:
```protobuf
message GetDashboardRequest {
  string name = 1;
  // Initial variable values from URL params (used for server-side data fetch on first load).
  map<string, ParamValue> initial_variables = 2;
}
```

This enables server-side rendering to start with the correct filtered data, avoiding a flash of wrong data on first paint.

### Exit Criteria

- Navigating to `/dashboards/network?node=server-01` loads the dashboard with `node=server-01` pre-selected.
- Changing a variable updates the URL without a page reload.
- Refreshing the page preserves the variable state from the URL.

---

## Summary: Proto Changes by Phase

### Phase A: Sort & Filter
```
TableSpec.view: TableViewConfig        (NEW)
TableViewConfig.enableSearch: bool      (NEW)
TableViewConfig.searchPlaceholder: string (NEW)
TableViewConfig.caseSensitive: bool    (NEW)
TableViewConfig.searchFields: string[] (NEW)
TableColumn.sortable: bool             (NEW)
TableColumn.filterable: bool           (NEW)
TableColumn.filter: TableFilterConfig  (NEW)
TableFilterConfig.type: TableFilterType (NEW)
TableFilterConfig.filterValues: string[] (NEW)
```

### Phase B: Column Features
```
TableSpec.frozenColsLeft: string[]     (NEW)
TableSpec.frozenColsRight: string[]    (NEW)
TableSpec.defaultSort: TableDefaultSort (NEW)
TableColumn.hidden: bool               (NEW)
TableColumn.order: int32              (NEW)
TableInteraction.enableColumnDrag: bool (NEW)
```

### Phase C: Row Selection & Export
```
TableSpec.selection: TableSelectionConfig (NEW)
TableSelectionConfig.enabled: bool     (NEW)
TableSelectionConfig.single: bool       (NEW)
TableSelectionConfig.persistent: bool    (NEW)
TableSpec.export: TableExportConfig     (NEW)
TableExportConfig.enableCsv: bool       (NEW)
TableExportConfig.enableExcel: bool     (NEW)
TableExportConfig.filenameTemplate: string (NEW)
```

### Phase D: Advanced Cell Rendering
```
TableColumn.renderer: TableCellRenderer (NEW)
TableCellRenderer.renderer: oneof      (NEW)
TableCellRendererText                   (NEW)
TableCellRendererIcon                   (NEW)
TableCellRendererIcon.valueToIcon: map  (NEW)
TableCellRendererIcon.fallbackIcon: string (NEW)
TableCellRendererBar                    (NEW)
TableCellRendererBar.valueField: string  (NEW)
TableCellRendererBar.maxValue: double   (NEW)
TableCellRendererBar.showLabel: bool    (NEW)
TableCellRendererBar.color: string      (NEW)
TableCellRendererLink                   (NEW)
TableCellRendererLink.urlTemplate: string (NEW)
TableCellRendererLink.newTab: bool      (NEW)
TableCellRendererProgress               (NEW)
TableCellRendererProgress.valueField: string (NEW)
TableCellRendererProgress.totalField: string (NEW)
TableCellRendererProgress.maxValue: double  (NEW)
TableCellRendererProgress.showLabel: bool   (NEW)
```

### Phase E: Expandable Rows & Cell Spanning
```
TableSpec.expandable: TableExpandConfig (NEW)
TableExpandConfig.enabled: bool         (NEW)
TableExpandConfig.rowIdField: string     (NEW)
TableExpandConfig.hierarchyFields: string[] (NEW)
TableExpandConfig.defaultExpanded: bool  (NEW)
TableSpec.colSpans: TableColSpan[]      (NEW)
TableColSpan.field: string               (NEW)
TableColSpan.condition: string           (NEW)
TableColSpan.colSpan: int32             (NEW)
```

### Phase F: Server-Side Pagination
```
TableSpec.serverPagination: ServerSidePagination (NEW)
ServerSidePagination.pageSize: int32    (NEW)
ServerSidePagination.showTotalCount: bool (NEW)
ServerSidePagination.showPageSizeSelector: bool (NEW)
// Backend/proto: WidgetDataResponse.totalRows: int64 (NEW)
```

### Phase G: Graph Widget
```
// dashboard.proto
WidgetType.GRAPH = 18                      (NEW enum value)
// widget_spec.proto
WidgetSpec.graph: GraphSpec               (NEW oneof variant)
GraphSpec.dataMapping: GraphDataMapping    (NEW)
GraphDataMapping.sourceField: string       (NEW)
GraphDataMapping.targetField: string       (NEW)
GraphDataMapping.valueField: string        (NEW)
GraphDataMapping.labelField: string        (NEW)
GraphDataMapping.groupField: string        (NEW)
GraphSpec.layout: GraphLayout              (NEW)
GraphLayout.type: GraphLayoutType          (NEW enum)
GraphLayout.iterations: int32             (NEW)
GraphLayout.direction: string             (NEW)
GraphLayout.nodeSpacing: int32            (NEW)
GraphLayout.rankSpacing: int32            (NEW)
GraphSpec.nodeStyle: GraphNodeStyle        (NEW)
GraphNodeStyle.sizeField: string          (NEW)
GraphNodeStyle.size: double               (NEW)
GraphNodeStyle.colorField: string          (NEW)
GraphNodeStyle.color: string               (NEW)
GraphNodeStyle.showLabel: bool             (NEW)
GraphNodeStyle.labelFontSize: int32        (NEW)
GraphNodeStyle.shape: string               (NEW)
GraphSpec.edgeStyle: GraphEdgeStyle        (NEW)
GraphEdgeStyle.color: string               (NEW)
GraphEdgeStyle.widthField: string          (NEW)
GraphEdgeStyle.width: double              (NEW)
GraphEdgeStyle.showLabel: bool             (NEW)
GraphEdgeStyle.style: string               (NEW)
GraphEdgeStyle.showArrow: bool             (NEW)
GraphEdgeStyle.arrowSize: double          (NEW)
GraphSpec.interaction: GraphInteraction    (NEW)
GraphInteraction.enableDrag: bool          (NEW)
GraphInteraction.enableZoom: bool         (NEW)
GraphInteraction.enableClickSelect: bool  (NEW)
GraphInteraction.enableTooltip: bool      (NEW)
GraphInteraction.enableEdgeClick: bool    (NEW)
GraphInteraction.enableNodeCollapse: bool  (NEW)
```

### Phase H: Row-Action Columns
```
// NEW FILE: action.proto
TableAction                              (NEW message)
TableAction.id: string                    (NEW)
TableAction.label: string                (NEW)
TableAction.icon: string                 (NEW)
TableAction.style: ActionStyle            (NEW enum)
TableAction.confirmMessage: string        (NEW)
TableAction.grpcCall: ActionGrpcCall      (NEW)
ActionStyle                              (NEW enum: PRIMARY/SECONDARY/DANGER/LINK)
ActionGrpcCall.method: string             (NEW)
ActionGrpcCall.requestTemplate: string    (NEW)
ActionGrpcCall.resultHandling: ActionResultHandling (NEW enum)
ActionResultHandling                     (NEW enum: SET_VARIABLE/REFRESH/TOAST)
// widget_spec.proto
TableColumn.action: TableAction           (NEW field 11)
// data.proto
WidgetActionService.InvokeAction          (NEW RPC)
InvokeActionRequest                       (NEW)
InvokeActionResponse                      (NEW)
```

### Phase I: gRPC Data Source Adapter
```
// data_source.proto
DataSourceConfig.grpcProxy: GrpcProxyConfig (NEW oneof variant)
GrpcProxyConfig.service: string             (NEW)
GrpcProxyConfig.method: string              (NEW)
GrpcProxyConfig.requestTemplate: map        (NEW)
GrpcProxyConfig.responseSchema: ResponseSchema (NEW)
ResponseSchema.columns: ColumnSchema[]       (NEW)
ColumnSchema.field: string                  (NEW)
ColumnSchema.type: DataType                 (NEW)
```

### Phase J: URL Variable Binding
```
// No proto changes required for Phase J (frontend-only)
```

---

## Interaction with Cross-Widget System

Table selection is a natural source for cross-widget filtering. After Phase C:

- A table with `selection.enabled: true` emits selected row data via `onRowSelect`.
- Dashboard authors can wire this to another widget's `click_interactions`:
  ```yaml
  widgets:
    - id: users-table
      widgetType: TABLE
      spec { table { selection { enabled: true } } }
      data_source: "dataSources/users"
      click_interactions:
        - target_widget: user-detail
          set_variables:
            - name: selected_user_id
              from_field: user_id

    - id: user-detail
      widgetType: METRIC_CARD
      data_source: "dataSources/user-metrics"
      params:
        user_id: "${selected_user_id}"
  ```

---

## Phases Not Proposed

The following were considered but omitted from scope:

- **Column type inference**: Auto-detect string/number/date and apply default formatting. Defer until we have a real-world column type dataset.
- **Column grouping (multi-header)**: S2 supports it but the proto design is complex. Revisit when there is a concrete use case.
- **Server-side sorting/filtering**: Requires query pushdown to Flight SQL. Defer until Phase F is complete and we understand the query builder surface.
- **Keyboard navigation**: Accessibility enhancement. Add after core features are stable.
- **Dashboard embedding (iframe)**: Requires token auth in URL and cross-origin cookie handling. Separate security review needed.
- **Direct DB drivers (non-Flight SQL postgres/MySQL)**: Would duplicate ui_proxy's SQL pool logic. Option 2 (gRPC adapter) is the recommended path instead.
- **Real-time / streaming data**: Arrow Flight supports streaming DoGet. Would need a StreamingGetWidgetData RPC. Defer until there is a concrete use case for live-updating dashboards.
- **Custom widget plugin system**: Explicitly out of scope per the PLX design — keeps the widget set closed and bundle size bounded.

---

## Effort Estimates

| Phase | Effort | Notes |
|-------|--------|-------|
| A: Sort & Filter | Medium | S2 has native support; mapper work mostly wires existing config |
| B: Column Features | Medium | Frozen cols and resize are standard S2 config; order + hidden are trivial |
| C: Row Selection & Export | Medium | Selection is S2 native; export needs toolbar + imperatively call S2 export |
| D: Cell Rendering | Medium-High | Proto is verbose; mapper conditions are complex; S2 custom cell docs are sparse |
| E: Expandable Rows & Span | Medium | S2 tree table + colSpan are well-documented in S2 |
| F: Server Pagination | High | Backend changes to query construction + new proto field in WidgetDataResponse |
| G: Graph Widget (G6) | High | New widget type; G6 integration; no existing pattern in codebase |
| H: Row-Action Columns | High | New proto concept (write capability); new service; security model |
| I: gRPC Data Source | High | New DataSourceConfig variant; shared proto→columnar helper; channel pool |
| J: URL Variable Binding | Low | Frontend-only hook changes; no proto changes |

**Recommended order**: A -> B -> C -> J -> D -> E -> F -> G -> I -> H

Rationale:
- A-F are incremental table improvements
- J (URL binding) is low-effort and high value — do early
- G (Graph) is a new widget type — independent, can run in parallel with A-F
- I (gRPC adapter) enables real deployment — do before H
- H (row actions) is architecturally significant — do last after I is stable
