# Table Enhancement Implementation

<!-- agent-updated: 2026-04-11T00:00:00Z -->

## Overview

This guide covers step-by-step implementation for all 10 phases. Each phase lists exact files to modify, proto field additions, and verification steps.

**Prerequisites** for all phases:
```bash
# Proto codegen (run after any proto change)
buf generate proto/

# TypeScript check
cd frontend && pnpm tsc --noEmit

# Rust check
cargo check --workspace
```

---

## Phase A: Sort & Filter

### 1. Proto Changes

**File: `proto/open_plx/v1/widget_spec.proto`**

In `TableSpec`, add field 7:
```protobuf
message TableSpec {
  // ... existing fields (1-6) ...
  TableViewConfig view = 7;  // ADD
}
```

After `TablePagination`, add:
```protobuf
message TableViewConfig {
  bool enable_search = 1;
  string search_placeholder = 2;
  bool case_sensitive = 3;
  repeated string search_fields = 4;
}

message TableFilterConfig {
  TableFilterType type = 1;
  repeated string filter_values = 2;
}

enum TableFilterType {
  TABLE_FILTER_TYPE_UNSPECIFIED = 0;
  TABLE_FILTER_TYPE_LIST = 1;
  TABLE_FILTER_TYPE_TEXT = 2;
  TABLE_FILTER_TYPE_RANGE = 3;
}
```

In `TableColumn`, add fields 5-7:
```protobuf
message TableColumn {
  // ... existing fields (1-3) ...
  bool sortable = 5;              // ADD
  bool filterable = 6;            // ADD
  TableFilterConfig filter = 7;   // ADD
}
```

### 2. Run Codegen

```bash
buf generate proto/
```

Verify: `frontend/src/gen/open_plx/v1/widget_spec_pb.ts` contains `TableViewConfig`, `TableFilterConfig`, `TableFilterType`.

### 3. Rust Config Model

**File: `crates/open-plx-config/src/model.rs`**

In `TableSpecYaml`, add:
```rust
#[derive(Debug, Deserialize)]
pub struct TableSpecYaml {
    // ... existing fields ...
    #[serde(default)]
    pub view: Option<TableViewConfigYaml>,  // ADD
}

#[derive(Debug, Deserialize)]
pub struct TableColumnYaml {
    // ... existing fields ...
    #[serde(default)]
    pub sortable: bool,                         // ADD
    #[serde(default)]
    pub filterable: bool,                       // ADD
    #[serde(default)]
    pub filter: Option<TableFilterConfigYaml>,  // ADD
}

#[derive(Debug, Deserialize)]  // ADD
pub struct TableViewConfigYaml {
    #[serde(default)]
    pub enable_search: bool,
    #[serde(default)]
    pub search_placeholder: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub search_fields: Vec<String>,
}

#[derive(Debug, Deserialize)]  // ADD
pub struct TableFilterConfigYaml {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub filter_values: Vec<String>,
}
```

### 4. Rust Config Conversion

**File: `crates/open-plx-config/src/convert.rs`**

In `table_to_proto()`, add filter conversion after the column loop:
```rust
let filter = c.filter.as_ref().map(|f| -> Result<pb::TableFilterConfig> {
    let filter_type = match f.r#type.as_deref() {
        Some("list") => pb::TableFilterType::List,
        Some("text") => pb::TableFilterType::Text,
        Some("range") => pb::TableFilterType::Range,
        None => pb::TableFilterType::Unspecified,
        Some(other) => bail!("unknown table filter type: '{other}'"),
    };
    Ok(pb::TableFilterConfig {
        r#type: filter_type.into(),
        filter_values: f.filter_values.clone(),
    })
}).transpose()?;
```

Add `sortable`, `filterable`, `filter` to the `TableColumn` construction. Add `view` conversion after the columns loop:
```rust
let view = t.view.as_ref().map(|v| pb::TableViewConfig {
    enable_search: v.enable_search,
    search_placeholder: v.search_placeholder.clone().unwrap_or_default(),
    case_sensitive: v.case_sensitive,
    search_fields: v.search_fields.clone(),
});
```

### 5. Frontend Mapper

**File: `frontend/src/services/mappers/tableMapper.ts`**

Add interfaces:
```typescript
export interface S2SortParam {
  sortFieldId: string;
  sortMethod?: "ASC" | "DESC" | "NONE" | "asc" | "desc" | "none";
}

export interface S2FilterParam {
  filterKey: string;
  filteredValues?: unknown[];
}
```

Add to `S2TableDataConfig`:
```typescript
sortParams?: S2SortParam[];
filterParams?: S2FilterParam[];
```

Add to `S2TableOptions`:
```typescript
search?: {
  enabled: boolean;
  placeholder?: string;
  caseSensitive?: boolean;
  searchFields?: string[];
};
```

In `tableProtoToS2()`, after the column loop:
```typescript
// Sort params
const sortParams: S2SortParam[] = [];
for (const col of proto.columns) {
  if (col.sortable) {
    sortParams.push({ sortFieldId: col.field });
  }
}
if (sortParams.length > 0) {
  dataCfg.sortParams = sortParams;
}

// Filter params
const filterParams: S2FilterParam[] = [];
for (const col of proto.columns) {
  if (col.filterable && col.filter) {
    filterParams.push({
      filterKey: col.field,
      filteredValues: col.filter.filterValues,
    });
  }
}
if (filterParams.length > 0) {
  dataCfg.filterParams = filterParams;
}

// Search
if (proto.view?.enableSearch) {
  options.search = {
    enabled: true,
    placeholder: proto.view.searchPlaceholder || undefined,
    caseSensitive: proto.view.caseSensitive || false,
    searchFields: proto.view.searchFields.length > 0 ? [...proto.view.searchFields] : undefined,
  };
}
```

### 6. Verify

```bash
cd frontend && pnpm tsc --noEmit
cargo check --workspace
```

**Exit criteria**: A YAML dashboard with `spec.table.view.enableSearch: true` and `spec.table.columns[0].sortable: true` renders a searchable, sortable table.

---

## Phase B: Column Features

### 1. Proto Changes

**File: `proto/open_plx/v1/widget_spec.proto`**

In `TableSpec`, add fields 8-10:
```protobuf
message TableSpec {
  // ... existing fields (1-7) ...
  repeated string frozen_cols_left = 8;      // ADD
  repeated string frozen_cols_right = 9;     // ADD
  TableDefaultSort default_sort = 10;         // ADD
}

message TableDefaultSort {
  string field = 1;
  SortDirection direction = 2;
}
```

In `TableColumn`, add fields 8-9:
```protobuf
message TableColumn {
  // ... existing fields (1-7) ...
  bool hidden = 8;       // ADD
  int32 order = 9;      // ADD
}
```

In `TableInteraction`, add field 6:
```protobuf
message TableInteraction {
  // ... existing fields (1-5) ...
  bool enable_column_drag = 6;  // ADD
}
```

### 2. Run Codegen

```bash
buf generate proto/
```

### 3. Rust Config Model

**File: `crates/open-plx-config/src/model.rs`**

In `TableSpecYaml`:
```rust
#[serde(default)]
pub frozen_cols_left: Vec<String>,   // ADD
#[serde(default)]
pub frozen_cols_right: Vec<String>,  // ADD
#[serde(default)]
pub default_sort: Option<TableDefaultSortYaml>,  // ADD
```

In `TableColumnYaml`:
```rust
#[serde(default)]
pub hidden: bool,   // ADD
#[serde(default)]
pub order: i32,    // ADD
```

In `TableInteractionYaml`:
```rust
#[serde(default)]
pub enable_column_drag: bool,  // ADD
```

### 4. Rust Config Conversion

**File: `crates/open-plx-config/src/convert.rs`**

Map the new fields in `table_to_proto()`:
```rust
let frozen_cols_left = t.frozen_cols_left.clone();
let frozen_cols_right = t.frozen_cols_right.clone();
let default_sort = t.default_sort.as_ref().map(|s| pb::TableDefaultSort {
    field: s.field.clone(),
    direction: match s.direction.as_deref() {
        Some("asc") => pb::SortDirection::Asc,
        Some("desc") => pb::SortDirection::Desc,
        _ => pb::SortDirection::Unspecified,
    },
});

let hidden: Vec<_> = proto.columns.iter().enumerate()
    .filter(|(_, c)| c.hidden).map(|(_, c)| c.field.clone()).collect();
let ordered: Vec<_> = proto.columns.iter().enumerate()
    .filter(|(_, c)| c.order != 0).sorted_by_key(|(_, c)| c.order)
    .map(|(_, c)| c.field.clone()).collect();
```

### 5. Frontend Mapper

**File: `frontend/src/services/mappers/tableMapper.ts`**

In `tableProtoToS2()`:
```typescript
// Frozen columns
if (proto.frozenColsLeft?.length || proto.frozenColsRight?.length) {
  options.frozenCols = [
    ...(proto.frozenColsLeft || []),
    ...(proto.frozenColsRight || []),
  ];
  options.frozenRowHeader = true;
}

// Column order
if (proto.columns.some(c => c.order !== 0)) {
  const ordered = [...proto.columns]
    .filter(c => c.order !== 0)
    .sort((a, b) => a.order - b.order);
  const unordered = proto.columns.filter(c => c.order === 0);
  dataCfg.fields.columns = [
    ...ordered.map(c => c.field),
    ...unordered.map(c => c.field),
  ];
}

// Hidden columns: exclude from fields.columns
if (proto.columns.some(c => c.hidden)) {
  const visible = proto.columns.filter(c => !c.hidden);
  dataCfg.fields.columns = visible.map(c => c.field);
}

// Default sort
if (proto.defaultSort) {
  const sortParams = [{ sortFieldId: proto.defaultSort.field, sortMethod: proto.defaultSort.direction === 1 ? 'DESC' : 'ASC' }];
  dataCfg.sortParams = sortParams;
}
```

### 6. Verify

```bash
cd frontend && pnpm tsc --noEmit
cargo check --workspace
```

**Exit criteria**: `frozenColsLeft: ["id", "name"]` pins those columns. `hidden: true` on a column excludes it. `order: 1` reorders columns.

---

## Phase C: Row Selection & Export

### 1. Proto Changes

**File: `proto/open_plx/v1/widget_spec.proto`**

In `TableSpec`, add fields 11-12:
```protobuf
message TableSpec {
  // ... existing fields (1-10) ...
  TableSelectionConfig selection = 11;  // ADD
  TableExportConfig export = 12;         // ADD
}

message TableSelectionConfig {
  bool enabled = 1;
  bool single = 2;
  bool persistent = 3;
}

message TableExportConfig {
  bool enable_csv = 1;
  bool enable_excel = 2;
  string filename_template = 3;
}
```

### 2. Run Codegen

```bash
buf generate proto/
```

### 3. Rust Config Model

**File: `crates/open-plx-config/src/model.rs`**

```rust
#[serde(default)]
pub selection: Option<TableSelectionConfigYaml>,  // ADD
#[serde(default)]
pub export: Option<TableExportConfigYaml>,       // ADD

#[derive(Debug, Deserialize)]
pub struct TableSelectionConfigYaml {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub single: bool,
    #[serde(default)]
    pub persistent: bool,
}

#[derive(Debug, Deserialize)]
pub struct TableExportConfigYaml {
    #[serde(default)]
    pub enable_csv: bool,
    #[serde(default)]
    pub enable_excel: bool,
    #[serde(default)]
    pub filename_template: Option<String>,
}
```

### 4. Rust Config Conversion

**File: `crates/open-plx-config/src/convert.rs`**

```rust
let selection = t.selection.as_ref().map(|s| pb::TableSelectionConfig {
    enabled: s.enabled,
    single: s.single,
    persistent: s.persistent,
});

let export = t.export.as_ref().map(|e| pb::TableExportConfig {
    enable_csv: e.enable_csv,
    enable_excel: e.enable_excel,
    filename_template: e.filename_template.clone().unwrap_or_default(),
});
```

### 5. Frontend Mapper

**File: `frontend/src/services/mappers/tableMapper.ts`**

```typescript
// Selection
if (proto.selection?.enabled) {
  options.rowSelection = {
    strict: true,
    onlySelected: proto.selection.single,
    showSelectedIcon: true,
    persist: proto.selection.persistent,
  };
}

// Export
if (proto.export) {
  options.export = proto.export;
}
```

### 6. Frontend Widget

**File: `frontend/src/components/widgets/TableWidget.tsx`**

Add export toolbar and selection handler:
```typescript
// After Card title, add export buttons if export config is set
{options.export?.enableCsv && (
  <Button size="small" onClick={() => sheetRef.current?.exportFile('csv')}>
    Export CSV
  </Button>
)}
```

Expose selected rows via `onRowSelect` prop (similar to `onClickInteraction`):
```typescript
if (options.rowSelection) {
  sheet.on(S2Event.ROW_SELECTED, (event) => {
    const selected = sheet.getSelectedRows();
    // Pass to parent via onRowSelect callback
  });
}
```

### 7. Verify

```bash
cd frontend && pnpm tsc --noEmit
cargo check --workspace
```

**Exit criteria**: Checkbox column appears. Selecting rows fires `onRowSelect`. "Export CSV" downloads the table.

---

## Phase D: Advanced Cell Rendering

### 1. Proto Changes

**File: `proto/open_plx/v1/widget_spec.proto`**

In `TableColumn`, add field 10:
```protobuf
message TableColumn {
  // ... existing fields (1-9) ...
  TableCellRenderer renderer = 10;  // ADD
}

message TableCellRenderer {  // ADD
  oneof renderer {
    TableCellRendererText text = 1;
    TableCellRendererIcon icon = 2;
    TableCellRendererBar bar = 3;
    TableCellRendererLink link = 4;
    TableCellRendererProgress progress = 5;
  }
}

message TableCellRendererText {}  // ADD

message TableCellRendererIcon {  // ADD
  map<string, string> value_to_icon = 1;
  string fallback_icon = 2;
}

message TableCellRendererBar {  // ADD
  string value_field = 1;
  double max_value = 2;
  bool show_label = 3;
  string color = 4;
}

message TableCellRendererLink {  // ADD
  string url_template = 1;
  bool new_tab = 2;
}

message TableCellRendererProgress {  // ADD
  string value_field = 1;
  string total_field = 2;
  double max_value = 3;
  bool show_label = 4;
}
```

### 2. Run Codegen

```bash
buf generate proto/
```

### 3. Rust Config Model + Conversion

Add `TableCellRendererYaml` and nested types to `model.rs`. In `convert.rs`, map each `oneof` variant using the pattern:
```rust
let renderer = match (r.text.as_ref(), r.icon.as_ref(), ...) {
    (Some(_), _, _, _, _) => Some(pb::TableCellRenderer {
        renderer: Some(pb::table_cell_renderer::Renderer::Text(pb::TableCellRendererText {})),
    }),
    // ...
};
```

### 4. Frontend Mapper

**File: `frontend/src/services/mappers/tableMapper.ts`**

For each column with a `renderer`:
```typescript
// Map icon renderer to S2 conditions
if (col.renderer?.icon) {
  const iconMap = col.renderer.icon.valueToIcon;
  options.conditions = options.conditions || [];
  options.conditions.push({
    field: col.field,
    type: "icon",
    mapping: (value) => ({
      value,
      icon: iconMap[value] || col.renderer.icon.fallbackIcon,
    }),
  });
}

// Map link renderer
if (col.renderer?.link) {
  // S2 text condition with href
  options.conditions = options.conditions || [];
  options.conditions.push({
    field: col.field,
    type: "text",
    mapping: (value) => ({
      text: value,
      href: col.renderer.link.urlTemplate.replace("{row." + col.field + "}", value),
      target: col.renderer.link.newTab ? "_blank" : "_self",
    }),
  });
}
```

### 5. S2Table Link Handling

**File: `frontend/src/components/widgets/S2Table.tsx`**

```typescript
sheet.on(S2Event.DATA_CELL_CLICK, (event) => {
  const cell = event.target;
  const meta = cell.getMeta?.();
  if (meta?.field === linkColumnField) {
    const value = dataCfg.data[meta.rowIndex]?.[meta.field];
    const url = linkTemplate.replace("{row." + meta.field + "}", String(value));
    window.open(url, newTab ? "_blank" : "_self");
  }
});
```

### 6. Verify

```bash
cd frontend && pnpm tsc --noEmit
cargo check --workspace
```

**Exit criteria**: Icon renderer shows icons by value. Link renderer is clickable and opens URL.

---

## Phase E: Expandable Rows & Cell Spanning

### 1. Proto Changes

**File: `proto/open_plx/v1/widget_spec.proto`**

In `TableSpec`, add fields 13-14:
```protobuf
TableExpandConfig expandable = 13;
repeated TableColSpan col_spans = 14;

message TableExpandConfig {  // ADD
  bool enabled = 1;
  string row_id_field = 2;
  repeated string hierarchy_fields = 3;
  bool default_expanded = 4;
}

message TableColSpan {  // ADD
  string field = 1;
  string condition = 2;
  int32 col_span = 3;
}
```

### 2. Run Codegen

```bash
buf generate proto/
```

### 3. Rust Config Model + Conversion

Standard additions to `model.rs` and `convert.rs`.

### 4. Frontend Mapper

**File: `frontend/src/services/mappers/tableMapper.ts`**

```typescript
// Expandable (tree table)
if (proto.expandable?.enabled) {
  dataCfg.fields.rowId = proto.expandable.rowIdField;
  // Build hierarchical data structure from flat rows
  const hierarchyData = buildHierarchy(data, proto.expandable.hierarchyFields);
  dataCfg.data = hierarchyData;
  options.hierarchyCollapse = !proto.expandable.defaultExpanded;
}

// Cell spanning
if (proto.colSpans?.length > 0) {
  options.colCellContext = (rowIndex, colIndex, field, spreadsheet) => {
    for (const span of proto.colSpans) {
      if (span.field === field && evaluateCel(span.condition, data[rowIndex])) {
        return { colSpan: span.colSpan };
      }
    }
  };
}
```

### 5. Verify

```bash
cd frontend && pnpm tsc --noEmit
cargo check --workspace
```

**Exit criteria**: `expandable: { enabled: true, hierarchyFields: ["category"] }` renders a tree table. `colSpans` merges cells.

---

## Phase F: Server-Side Pagination

### 1. Proto Changes

**File: `proto/open_plx/v1/widget_spec.proto`**

In `TableSpec`, add field 15:
```protobuf
ServerSidePagination server_pagination = 15;

message ServerSidePagination {  // ADD
  int32 page_size = 1;
  bool show_total_count = 2;
  bool show_page_size_selector = 3;
}
```

In `data.proto`, add to `WidgetDataResponse`:
```protobuf
message WidgetDataResponse {
  // ... existing fields (1-99) ...
  int64 total_rows = 100;  // ADD
}
```

### 2. Run Codegen

```bash
buf generate proto/
```

### 3. Backend Handler

**File: `crates/open-plx-server/src/widget_data.rs`**

In `get_widget_data()`, after receiving `page_number` from params:
```rust
if let Some(sp) = table_spec.server_pagination {
    let page_size = sp.page_size as i64;
    let page_number = params.get("page_number")
        .and_then(|v| v.int_value)
        .unwrap_or(1) as i64;
    let offset = (page_number - 1) * page_size;
    // Apply LIMIT/OFFSET to Flight SQL query
    let query = format!("{} LIMIT {} OFFSET {}", base_query, page_size, offset);
    let total_rows = count_query(&base_query).await?;  // separate count query
    let batch = execute_flight_sql(&query).await?;
    return Ok(WidgetDataResponse { columns: batch_to_columns(batch), total_rows: Some(total_rows) });
}
```

### 4. Frontend Mapper + Widget

Pass `serverPagination` config through `options`. In `TableWidget`, manage `currentPage` state and refetch on page change.

### 5. Verify

```bash
cd frontend && pnpm tsc --noEmit
cargo check --workspace
cargo test --workspace
```

**Exit criteria**: Pagination controls appear. Navigating pages triggers new `GetWidgetData` calls with correct offset.

---

## Phase G: Graph Widget

### 1. Proto Changes

**File: `proto/open_plx/v1/dashboard.proto`**

Add to `WidgetType` enum:
```protobuf
WIDGET_TYPE_GRAPH = 18;
```

**File: `proto/open_plx/v1/widget_spec.proto`**

In `WidgetSpec` oneof, add:
```protobuf
message WidgetSpec {
  oneof spec {
    // ... existing (1-10) ...
    GraphSpec graph = 11;  // ADD
  }
}
```

Add the full `GraphSpec` message (see `table-enhancement.md` Phase G for full proto text).

### 2. Run Codegen

```bash
buf generate proto/
```

### 3. Rust Config

**File: `crates/open-plx-config/src/model.rs`**

Add `GraphSpecYaml` and all sub-structs.

**File: `crates/open-plx-config/src/convert.rs`**

Add `graph_to_proto()` function and wire into `widget_spec_to_proto()`.

### 4. G6 Dependency

```bash
cd frontend && pnpm add @antv/g6
```

### 5. Graph Mapper

**File: `frontend/src/services/mappers/graphMapper.ts` (NEW)**

```typescript
import type { GraphSpec } from "../../gen/open_plx/v1/widget_spec_pb.js";

export interface G6GraphConfig {
  data: { nodes: G6Node[]; edges: G6Edge[] };
  layout: Record<string, unknown>;
  node: Record<string, unknown>;
  edge: Record<string, unknown>;
  behaviors: string[];
}

export function graphProtoToG6(
  proto: GraphSpec,
  data: Record<string, unknown>[],
): G6GraphConfig {
  // Transform flat data to G6 node/edge format
  const nodes = [...new Set(data.flatMap(row => [row[proto.dataMapping.sourceField], row[proto.dataMapping.targetField]]))]
    .filter(Boolean)
    .map(id => ({ id, label: id }));

  const edges = data
    .filter(row => row[proto.dataMapping.sourceField] && row[proto.dataMapping.targetField])
    .map(row => ({
      source: row[proto.dataMapping.sourceField],
      target: row[proto.dataMapping.targetField],
      value: row[proto.dataMapping.valueField] ?? 1,
    }));

  return {
    data: { nodes, edges },
    layout: { type: proto.layout?.type ?? "force", ... },
    node: { ... },
    edge: { ... },
    behaviors: Object.entries(proto.interaction ?? {})
      .filter(([_, v]) => v === true)
      .map(([k]) => k),
  };
}
```

### 6. Graph Widget

**File: `frontend/src/components/widgets/GraphWidget.tsx` (NEW)**

```typescript
import { useEffect, useRef } from "react";
import { Graph } from "@antv/g6";
import { graphProtoToG6 } from "../../services/mappers/graphMapper.js";

export function GraphWidget({ config, data, onClickInteraction }: WidgetProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<Graph | null>(null);

  useEffect(() => {
    if (!containerRef.current || !data) return;
    const spec = config.spec?.spec.case === "graph" ? config.spec.spec.value : null;
    if (!spec) return;

    const g6Config = graphProtoToG6(spec, data);
    const graph = new Graph({ container: containerRef.current, ...g6Config });
    graph.render();

    graph.on("node:click", (event) => {
      onClickInteraction?.({ widgetId: config.id, field: "node_id", value: event.itemId });
    });

    graphRef.current = graph;
    return () => graph.destroy();
  }, [data, config]);

  return <div ref={containerRef} style={{ width: "100%", height: "100%" }} />;
}
```

### 7. Widget Registry

**File: `frontend/src/components/widgets/WidgetRegistry.tsx`**

```typescript
case WidgetType.GRAPH:
  return <GraphWidget config={config} data={data} loading={loading} error={error} onClickInteraction={onClickInteraction} />;
```

### 8. Verify

```bash
cd frontend && pnpm tsc --noEmit
cargo check --workspace
```

**Exit criteria**: A dashboard with `widgetType: GRAPH` renders a force-directed graph. Clicking a node fires `onClickInteraction`.

---

## Phase H: Row-Action Columns

### 1. Proto Changes

**File: `proto/open_plx/v1/action.proto` (NEW)**

```protobuf
syntax = "proto3";
package open_plx.v1;

import "open_plx/v1/widget_spec.proto";

service WidgetActionService {
  rpc InvokeAction(InvokeActionRequest) returns (InvokeActionResponse);
}

message InvokeActionRequest {
  string dashboard_name = 1;
  string widget_id = 2;
  string action_id = 3;
  string request_body = 4;
}

message InvokeActionResponse {
  bool success = 1;
  string message = 2;
  string variable_name = 3;
  string variable_value = 4;
}

message TableAction {
  string id = 1;
  string label = 2;
  string icon = 3;
  ActionStyle style = 4;
  string confirm_message = 5;
  ActionGrpcCall grpc_call = 6;
}

enum ActionStyle {
  ACTION_STYLE_UNSPECIFIED = 0;
  ACTION_STYLE_PRIMARY = 1;
  ACTION_STYLE_SECONDARY = 2;
  ACTION_STYLE_DANGER = 3;
  ACTION_STYLE_LINK = 4;
}

message ActionGrpcCall {
  string method = 1;
  string request_template = 2;
  ActionResultHandling result_handling = 3;
}

enum ActionResultHandling {
  ACTION_RESULT_HANDLING_UNSPECIFIED = 0;
  ACTION_RESULT_HANDLING_SET_VARIABLE = 1;
  ACTION_RESULT_HANDLING_REFRESH = 2;
  ACTION_RESULT_HANDLING_TOAST = 3;
}
```

In `widget_spec.proto`, add to `TableColumn`:
```protobuf
TableAction action = 11;
```

### 2. Run Codegen

```bash
buf generate proto/
```

### 3. Backend: WidgetActionService

**File: `crates/open-plx-server/src/action.rs` (NEW)**

```rust
use tonic::{Request, Response, Status};
use open_plx_core::pb;

pub struct WidgetActionService {
    auth: AuthInterceptor,
    upstream_pool: Arc<GrpcChannelPool>,
}

#[tonic::async_trait]
impl pb::WidgetActionService for WidgetActionService {
    async fn invoke_action(
        &self,
        request: Request<InvokeActionRequest>,
    ) -> Result<Response<InvokeActionResponse>, Status> {
        let req = request.into_inner();
        // Check action permission
        self.auth.check_action_permission(&req).await?;
        // Forward to upstream gRPC
        let upstream_response = self.upstream_pool.call(&req).await?;
        // Log event
        tracing::info!(event: "action.invoke", action_id = %req.action_id, success = upstream_response.success);
        Ok(Response::new(upstream_response))
    }
}
```

Wire into `server.rs`:
```rust
let action_service = WidgetActionService::new(auth, upstream_pool);
builder.add_service(pb::widget_action_service_server::WidgetActionServiceServer::new(action_service));
```

### 4. Frontend: Action Client

**File: `frontend/src/services/grpc/clients.ts`**

```typescript
export const widgetActionClient = createClient(pb.WidgetActionService, transport);
```

**File: `frontend/src/components/widgets/S2Table.tsx`**

```typescript
sheet.on(S2Event.DATA_CELL_CLICK, async (event) => {
  const cell = event.target;
  const meta = cell.getMeta?.();
  if (meta?.field === actionColumnField) {
    const row = dataCfg.data[meta.rowIndex];
    const action = column.action;

    if (action.confirmMessage && !window.confirm(action.confirmMessage)) {
      return;
    }

    const interpolatedBody = interpolateTemplate(action.grpcCall.requestTemplate, row);
    const response = await widgetActionClient.invokeAction({
      dashboardName,
      widgetId: config.id,
      actionId: action.id,
      requestBody: interpolatedBody,
    });

    if (response.success) {
      if (action.grpcCall.resultHandling === ActionResultHandling.SET_VARIABLE) {
        setVariable(response.variableName, response.variableValue);
      } else if (action.grpcCall.resultHandling === ActionResultHandling.REFRESH) {
        onRefresh();
      }
    }
  }
});
```

### 5. Verify

```bash
cd frontend && pnpm tsc --noEmit
cargo check --workspace
cargo test --workspace
```

**Exit criteria**: A table with an action column renders buttons. Clicking "Restart" shows confirmation, calls `InvokeAction`, forwards to upstream service.

---

## Phase I: gRPC Data Source Adapter

### 1. Proto Changes

**File: `proto/open_plx/v1/data_source.proto`**

In `DataSourceConfig` oneof, add:
```protobuf
message DataSourceConfig {
  oneof config {
    FlightSqlConfig flight_sql = 1;
    StaticConfig static = 2;
    GrpcProxyConfig grpc_proxy = 3;  // ADD
  }
}

message GrpcProxyConfig {  // ADD
  string service = 1;
  string method = 2;
  map<string, ParamValue> request_template = 3;
  ResponseSchema response_schema = 4;
}

message ResponseSchema {  // ADD
  repeated ColumnSchema columns = 1;
}

message ColumnSchema {  // ADD
  string field = 1;
  DataType type = 2;
}
```

### 2. Run Codegen

```bash
buf generate proto/
```

### 3. Shared Columnar Helper

**File: `crates/open-plx-core/src/proto_columnar.rs` (NEW)**

```rust
/// Converts a protobuf Message to an Arrow RecordBatch.
/// Used by both ui_proxy (existing) and open-plx-server (new).
pub fn proto_to_record_batch<M: prost::Message + Default>(
    message: &M,
    schema: &ArrowSchema,
) -> Result<RecordBatch, Box<dyn std::error::Error>> {
    let bytes = message.encode_to_vec();
    let ipc_reader = ipc::reader::StreamReader::try_new(bytes.as_slice())?;
    let batch = ipc_reader.next().ok_or("no batches")??;
    Ok(batch)
}
```

### 4. Backend: GrpcProxyClient

**File: `crates/open-plx-server/src/grpc_proxy_client.rs` (NEW)**

```rust
pub struct GrpcProxyClient {
    channels: Arc<DashMap<String, Channel>>,
    converter: ProtoColumnarConverter,
}

impl GrpcProxyClient {
    pub async fn fetch(
        &self,
        config: &GrpcProxyConfig,
        params: &HashMap<String, ParamValue>,
    ) -> Result<RecordBatch, Box<dyn std::error::Error>> {
        // Interpolate template params
        let request = self.interpolate_template(&config.request_template, params)?;

        // Get or create channel for this service
        let channel = self.channels.entry(config.service.clone())
            .or_insert_with(|| connect_to_service(config.service));

        // Make unary gRPC call
        let response = channel.unary(request, config.method).await?;

        // Convert to RecordBatch
        let schema = self.get_or_infer_schema(config)?;
        self.converter.proto_to_record_batch(&response, &schema)
    }
}
```

### 5. Wire into WidgetDataService

**File: `crates/open-plx-server/src/widget_data.rs`**

```rust
match data_source.config {
    Some(Config::FlightSql(cfg)) => self.flight_sql_client.fetch(cfg, params).await,
    Some(Config::Static(cfg)) => self.static_data.fetch(cfg),
    Some(Config::GrpcProxy(cfg)) => self.grpc_proxy_client.fetch(cfg, params).await,  // ADD
    None => Err(anyhow::anyhow!("no data source config")),
}
```

### 6. Rust Config Model + Conversion

Add `GrpcProxyConfigYaml` to `crates/open-plx-config/src/model.rs` and wire into `data_source_to_proto()`.

### 7. Verify

```bash
cargo check --workspace
cargo test --workspace
```

**Exit criteria**: A data source with `grpcProxy { service: "foo.Bar", method: "GetData" }` fetches from that gRPC service and renders in a widget.

---

## Phase J: URL Variable Binding

### 1. Frontend Hook Changes

**File: `frontend/src/hooks/useVariables.ts`**

Add `initFromUrl()` and `syncToUrl()`:

```typescript
export function useVariables() {
  const [variableValues, setVariableValues] = useState<Record<string, ParamValue>>({});
  const params = new URLSearchParams(window.location.search);

  // Init from URL on mount
  useEffect(() => {
    for (const [name, value] of params) {
      if (name in variableValues) {
        setVariableValues(prev => ({ ...prev, [name]: stringValue(value) }));
      }
    }
  }, []);

  // Sync to URL on change
  useEffect(() => {
    const params = new URLSearchParams();
    for (const [name, value] of Object.entries(variableValues)) {
      if (value !== getDefaultValue(name)) {
        params.set(name, String(value));
      }
    }
    const newUrl = `${window.location.pathname}?${params.toString()}`;
    window.history.replaceState(null, '', newUrl);
  }, [variableValues]);

  return { variableValues, setVariable, initFromUrl, syncToUrl };
}
```

### 2. Dashboard Page

**File: `frontend/src/pages/DashboardPage.tsx`**

```typescript
const { initFromUrl } = useVariables();

useEffect(() => {
  initFromUrl();
}, []);
```

### 3. Verify

```bash
cd frontend && pnpm tsc --noEmit
```

**Exit criteria**: Navigating to `/dashboards/stock-detail?ticker=AAPL` pre-selects `ticker=AAPL`. Changing a variable updates the URL.

---

## Verification Checklist

After each phase, run:
```bash
# TypeScript
cd frontend && pnpm tsc --noEmit

# Rust
cargo check --workspace
cargo test --workspace

# E2e (if Playwright tests exist)
cd frontend && pnpm playwright test
```

Each phase must pass TypeScript and Rust checks before moving to the next phase.
