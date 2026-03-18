# Declarative Layout Specification

## Overview

A dashboard is a declarative document describing **what** to render and
**where**. The backend stores and serves these documents. The frontend
interprets them using a **mapper layer** that translates the semantic
proto spec to the rendering library (currently G2 for charts, S2 for
pivot tables, Antd for UI controls).

**Key design decision**: The widget `spec` is a **typed proto message**
using **semantic vocabulary** -- not G2/S2 API terms. This decouples the
proto schema from the rendering library:

- `CHART_TYPE_LINE` not `MARK_TYPE_LINE`
- `data_mapping.group_by` not `encode.color`
- `STACK_MODE_STACKED` not `TRANSFORM_TYPE_STACK_Y`

The proto definitions are the source of truth:
- `proto/open_plx/v1/widget_spec.proto` -- all widget spec types
- `proto/open_plx/v1/dashboard.proto` -- Dashboard, WidgetConfig, DashboardVariable

## Document Structure

```
Dashboard (proto: Dashboard)
  |- name: "dashboards/{id}"
  |- title, description
  |- version (optimistic concurrency)
  |- GridConfig { columns, row_height, gap }
  |- variables: DashboardVariable[]   (shared filters/inputs)
  |- permission_denied_behavior
  |- WidgetConfig[]
       |- id (unique within dashboard)
       |- widget_type (enum)
       |- title
       |- GridPosition { x, y, w, h }
       |- DataSourceRef { data_source, params: map<string, ParamValue> }
       |- spec (WidgetSpec oneof: ChartSpec | PivotTableSpec | MetricCardSpec | TextSpec)
```

## Grid System

Column-based coordinate system (matches Antd's 24-column grid):

- `columns`: Number of equal-width columns (default: 24)
- `row_height`: Height of one row unit in pixels (default: 40)
- `gap`: Spacing between widgets in pixels (default: 8)

Widget positions:
- `x`: column start (0-indexed)
- `y`: row start (0-indexed)
- `w`: column span
- `h`: row span

---

## Dashboard Variables

Variables are shared parameters rendered as Antd input controls at the
top of the dashboard. Widgets reference them in `data_source.params`
via `variable_ref: "${variable_name}"`.

### Rendering Order (Three Phases)

```
Phase 0: Variable Initialization (topological order)
  1. Build dependency graph: variable A depends on variable B if
     A's options_source.params contains variable_ref("${B}")
  2. Topological sort. Reject if cycles detected (server validates on save).
  3. Initialize variables in topo order:
     a. Render control with default value
     b. If options_source exists: resolve any variable_ref params from
        already-initialized upstream variables, fetch options
     c. Set value (default or first option)
  4. All variables initialized before widget data is fetched.

Phase 1: Layout Fetch
  - GetDashboard -> render grid + widget shells + variable controls

Phase 2: Data Fetch (parallel per widget)
  - Resolve ${variable_ref} in each widget's params
  - Type-coerce ParamValue -> QueryParam.param_kind
  - Fetch data via Arrow Flight
```

### Cascading Variables

Variables can depend on other variables via `options_source.params`
containing `variable_ref` values. This enables patterns like
Country -> State -> City:

```protobuf
variables {
  name: "country"  label: "Country"
  default_value { string_value: "US" }
  select {
    options_source { data_source: "dataSources/countries" }
    value_field: "code"  label_field: "name"
  }
}

variables {
  name: "state"  label: "State"
  default_value { string_value: "CA" }
  select {
    options_source {
      data_source: "dataSources/states"
      params { key: "country"  value { variable_ref: "${country}" } }
    }
    value_field: "code"  label_field: "name"
  }
}

variables {
  name: "city"  label: "City"
  default_value { string_value: "SF" }
  select {
    options_source {
      data_source: "dataSources/cities"
      params {
        key: "country"  value { variable_ref: "${country}" }
        key: "state"  value { variable_ref: "${state}" }
      }
    }
    value_field: "code"  label_field: "name"
  }
}
```

**Dependency resolution:**
```
country (no deps) -> state (depends on country) -> city (depends on country, state)
```

The dependency graph must be a DAG (no cycles). The server validates
acyclicity on dashboard create/update. The frontend initializes variables
in topological order and re-fetches downstream options when an upstream
variable changes.

**When an upstream variable changes:**
1. Downstream variables' options are re-fetched with the new upstream value
2. If the current downstream value is no longer in the new options,
   reset to the first option (or default_value if it's still valid)
3. Widget data for all affected widgets is re-fetched

**Constraint**: Cycles are rejected at validation time (server-side).
Example of a rejected config: A depends on B, B depends on A.

### Control Types -> Antd Components

| Control | Antd Component | ParamValue Output |
|---------|---------------|-------------------|
| `text_input` | `Input` | `string_value` |
| `number_input` | `InputNumber` | `int_value` or `double_value` |
| `select` | `Select` | `string_value` |
| `multi_select` | `Select mode="multiple"` | `string_list` |
| `date_picker` | `DatePicker` | `string_value` (ISO 8601) |
| `date_range` | `DatePicker.RangePicker` | `date_range { start, end }` |
| `cascader` | `Cascader` | `string_value` (leaf value) |

---

## Chart Widgets (Semantic Spec)

Chart widgets use `ChartSpec` -- a semantic description of the chart.
The frontend's **mapper layer** translates to G2 v5 config.

### Semantic -> G2 Translation (in frontend mapper)

```
Proto (semantic)                    G2 Config (generated by mapper)
────────────────                    ─────────────────────────────────
CHART_TYPE_LINE                 ->  { type: "line" }
CHART_TYPE_BAR                  ->  { type: "interval" }
CHART_TYPE_HORIZONTAL_BAR       ->  { type: "interval", coordinate: { transform: [{ type: "transpose" }] } }
CHART_TYPE_PIE                  ->  { type: "interval", coordinate: { type: "theta" } }
CHART_TYPE_DONUT                ->  { type: "interval", coordinate: { type: "theta", innerRadius: 0.6 } }
CHART_TYPE_AREA                 ->  { type: "area" }
CHART_TYPE_SCATTER              ->  { type: "point" }
CHART_TYPE_HEATMAP              ->  { type: "cell" }
CHART_TYPE_HISTOGRAM            ->  { type: "rect", transform: [{ type: "binX" }] }
CHART_TYPE_RADAR                ->  { type: "line", coordinate: { type: "radar" } }

data_mapping.x: "date"          ->  encode: { x: "date" }
data_mapping.y: "revenue"       ->  encode: { y: "revenue" }
data_mapping.group_by: "region" ->  encode: { color: "region" }
data_mapping.size: "count"      ->  encode: { size: "count" }
data_mapping.value: "amount"    ->  encode: { y: "amount" }        (pie/donut)
data_mapping.category: "type"   ->  encode: { color: "type" }      (pie/donut)

STACK_MODE_STACKED              ->  transform: [{ type: "stackY" }]
STACK_MODE_GROUPED              ->  transform: [{ type: "dodgeX" }]
STACK_MODE_PERCENT              ->  transform: [{ type: "normalizeY" }]
```

### ChartSpec Properties

| Field | Type | Purpose |
|-------|------|---------|
| `chart_type` | `ChartType` enum | What kind of chart |
| `data_mapping` | `ChartDataMapping` | Which columns map to which roles |
| `stack_mode` | `StackMode` enum | How series are stacked/grouped |
| `x_axis` | `AxisConfig` | X-axis title, format, scale, position |
| `y_axis` | `AxisConfig` | Y-axis title, format, scale, position |
| `coordinate` | `CoordinateConfig` | Inner/outer radius, angles (pie/donut) |
| `labels` | `LabelConfig[]` | Data labels |
| `annotations` | `Annotation[]` | Reference lines and bands |
| `scales` | `map<string, ScaleConfig>` | Scale overrides per channel |
| `transforms` | `DataTransform[]` | Binning, sampling, aggregation |
| `sort` | `SortConfig` | Sort data before rendering |
| `layers` | `ChartSpec[]` | Composite/layered charts |

### Per-Chart-Type Examples

#### LINE_CHART

```protobuf
spec { chart {
  chart_type: CHART_TYPE_LINE
  data_mapping { x: "date"  y: "revenue"  group_by: "region" }
  x_axis { title: "Date"  scale_type: SCALE_TYPE_TIME }
  y_axis { title: "Revenue ($)"  label_format: "$~s" }
  sort { field: "date"  direction: SORT_DIRECTION_ASC }
  annotations { type: ANNOTATION_TYPE_LINE_Y  value: 1000000  label: "Target" }
}}
```

#### BAR_CHART (grouped)

```protobuf
spec { chart {
  chart_type: CHART_TYPE_BAR
  data_mapping { x: "category"  y: "value"  group_by: "quarter" }
  stack_mode: STACK_MODE_GROUPED
  x_axis { title: "Category" }
  y_axis { title: "Value"  label_format: "~s" }
}}
```

#### BAR_CHART (stacked)

```protobuf
spec { chart {
  chart_type: CHART_TYPE_BAR
  data_mapping { x: "category"  y: "value"  group_by: "quarter" }
  stack_mode: STACK_MODE_STACKED
}}
```

#### PIE_CHART / DONUT

```protobuf
spec { chart {
  chart_type: CHART_TYPE_DONUT
  data_mapping { value: "revenue"  category: "region" }
  labels { field: "region"  position: LABEL_POSITION_OUTSIDE  connector: true }
}}
```

#### SCATTER (bubble)

```protobuf
spec { chart {
  chart_type: CHART_TYPE_SCATTER
  data_mapping { x: "height"  y: "weight"  group_by: "gender"  size: "age" }
  scales { key: "size"  value { range_min: 4  range_max: 20 } }
}}
```

#### AREA (stacked)

```protobuf
spec { chart {
  chart_type: CHART_TYPE_AREA
  data_mapping { x: "date"  y: "value"  group_by: "series" }
  stack_mode: STACK_MODE_STACKED
  x_axis { scale_type: SCALE_TYPE_TIME }
}}
```

#### HEATMAP

```protobuf
spec { chart {
  chart_type: CHART_TYPE_HEATMAP
  data_mapping { x: "weekday"  y: "hour"  group_by: "count" }
  scales { key: "color"  value { type: SCALE_TYPE_SEQUENTIAL  palette: "reds" } }
}}
```

---

## Pivot Table (S2 Mapping)

Pivot table widgets use `PivotTableSpec`. The frontend mapper translates
to S2's `dataCfg` + `options` and injects Arrow data.

### Mapper Translation

```
Proto (PivotTableSpec)               S2 Config
──────────────────────               ─────────
fields.rows                      ->  dataCfg.fields.rows
fields.columns                   ->  dataCfg.fields.columns
fields.values                    ->  dataCfg.fields.values
fields.value_in_cols             ->  dataCfg.fields.valueInCols
meta[]                           ->  dataCfg.meta[] (formatter string -> function)
sort[]                           ->  dataCfg.sortParams[]
totals                           ->  options.totals
hierarchy_type                   ->  options.hierarchyType
frozen                           ->  options.frozen
pagination                       ->  options.pagination
series_number                    ->  options.seriesNumber
```

### Formatter Strings

| Format String | Example Output | Description |
|---------------|----------------|-------------|
| `"currency:USD"` | `$1,234.56` | Currency with symbol |
| `"currency:EUR"` | `EUR 1,234.56` | Euro currency |
| `"percent"` | `45.2%` | Percentage |
| `"percent:0"` | `45%` | Percentage, 0 decimals |
| `"number:2"` | `1,234.56` | Number with 2 decimal places |
| `"number:0"` | `1,235` | Integer with grouping |
| `"compact"` | `1.2K`, `3.4M` | SI-prefix compact |

### Example

```protobuf
spec { pivot_table {
  fields { rows: ["region", "product"]  columns: ["quarter"]  values: ["revenue", "units_sold"]  value_in_cols: true }
  meta { field: "revenue"    name: "Revenue"    formatter: "currency:USD" }
  meta { field: "units_sold" name: "Units Sold"  formatter: "number:0" }
  sort { sort_field_id: "revenue"  sort_direction: SORT_DIRECTION_DESC }
  totals {
    row { show_grand_totals: true  show_sub_totals: true  sub_totals_dimensions: ["region"]  aggregation: AGGREGATION_SUM }
    col { show_grand_totals: true  aggregation: AGGREGATION_SUM }
  }
  hierarchy_type: PIVOT_HIERARCHY_TYPE_GRID
  frozen { row_header: true }
}}
```

---

## Metric Card

```protobuf
spec { metric_card {
  value: "total_revenue"
  format: "currency:USD"
  comparison { value: "prev_quarter_revenue"  label: "vs Q3"  direction: COMPARISON_DIRECTION_HIGHER_IS_BETTER }
  sparkline { type: SPARKLINE_TYPE_AREA  x: "date"  y: "daily_revenue" }
}}
```

## Text

```protobuf
spec { text {
  content: "## Q4 Summary\n\nRevenue targets exceeded by 12%."
  format: TEXT_FORMAT_MARKDOWN
}}
```

---

## Frontend Mapper Layer

The mapper lives in `frontend/src/services/mappers/` and provides a
clean boundary between the proto spec and the rendering library:

```typescript
// Chart mapper: semantic proto + Arrow data -> G2 spec
function chartProtoToG2(proto: ChartSpec, batch: RecordBatch): G2Spec {
  // Extract typed arrays from Arrow (zero-copy views, not row objects)
  const data = arrowToG2Data(batch);
  const g2: G2Spec = { data };

  // Map chart type to G2 mark + coordinate
  switch (proto.chartType) {
    case ChartType.CHART_TYPE_LINE:
      g2.type = 'line';
      break;
    case ChartType.CHART_TYPE_BAR:
      g2.type = 'interval';
      break;
    case ChartType.CHART_TYPE_DONUT:
      g2.type = 'interval';
      g2.coordinate = { type: 'theta', innerRadius: proto.coordinate?.innerRadius ?? 0.6 };
      break;
    // ...
  }

  // Map data_mapping to G2 encode
  g2.encode = {};
  if (proto.dataMapping.x) g2.encode.x = proto.dataMapping.x;
  if (proto.dataMapping.y) g2.encode.y = proto.dataMapping.y;
  if (proto.dataMapping.groupBy) g2.encode.color = proto.dataMapping.groupBy;
  if (proto.dataMapping.size) g2.encode.size = proto.dataMapping.size;

  // Map stack mode to G2 transform
  switch (proto.stackMode) {
    case StackMode.STACK_MODE_STACKED:
      g2.transform = [{ type: 'stackY' }];
      break;
    case StackMode.STACK_MODE_GROUPED:
      g2.transform = [{ type: 'dodgeX' }];
      break;
    case StackMode.STACK_MODE_PERCENT:
      g2.transform = [{ type: 'normalizeY' }];
      break;
  }

  // Frontend adds: autoFit, theme, style, tooltip, legend, animate
  return g2;
}

// Pivot table mapper: semantic proto + Arrow data -> S2 config
function pivotProtoToS2(proto: PivotTableSpec, batch: RecordBatch): S2Config {
  const data = arrowToG2Data(batch); // same efficient row materialization
  // ... translate proto fields to S2 dataCfg + options
}
```

If G2 is ever replaced with another library, only the mapper changes.
The proto schema, backend, and widget components remain untouched.

---

## Validation Rules

1. Widget `id` must be unique within a dashboard.
2. Widget positions must not overlap (server validates on create/update).
3. Positions must fit within the grid (`x + w <= columns`).
4. `data_source` required for all types except `TEXT`.
5. `WidgetSpec.spec` oneof must match `widget_type`:
   - `LINE_CHART` -> `chart` with `chart_type: CHART_TYPE_LINE`
   - `BAR_CHART` -> `chart` with `chart_type: CHART_TYPE_BAR`
   - `PIE_CHART` -> `chart` with `chart_type: CHART_TYPE_PIE` or `CHART_TYPE_DONUT`
   - `PIVOT_TABLE` -> `pivot_table`
   - `METRIC_CARD` -> `metric_card`
   - `TEXT` -> `text`
6. `ParamValue` type must match `FlightSqlParam.param_kind` (enforced at query time).
7. Variable dependency graph must be acyclic (no cycles in `options_source` variable refs).
8. `version` must match current version on update (optimistic concurrency).

## What's NOT in the Proto Spec

Intentionally excluded (frontend-owned):

- **Visual styling**: Colors, fonts, opacity, animations (theme system)
- **Interaction config**: Hover, click, brush selection, tooltips
- **Container dimensions**: Derived from grid position at render time
- **Data**: Comes via Arrow Flight, not embedded in the layout config
- **Functions/callbacks**: Proto messages are pure data
- **Library-specific terms**: No G2 mark types, encode channels, or transform names

---

## Complete Dashboard Example

```protobuf
name: "dashboards/quarterly-sales"
title: "Q4 2025 Sales Dashboard"
description: "Executive view of quarterly sales metrics"
version: 1
grid { columns: 24  row_height: 40  gap: 8 }
permission_denied_behavior: PERMISSION_DENIED_BEHAVIOR_SHOW_DENIED

# --- Variables (rendered as Antd controls at dashboard top) ---

variables {
  name: "selected_year"  label: "Year"
  default_value { string_value: "2025" }
  select {
    options { value: "2024"  label: "2024" }
    options { value: "2025"  label: "2025" }
  }
}

variables {
  name: "selected_regions"  label: "Regions"
  default_value { string_list { values: ["US", "EU", "APAC"] } }
  multi_select {
    options_source { data_source: "dataSources/region-list" }
    value_field: "region_code"
    label_field: "region_name"
  }
}

variables {
  name: "date_range"  label: "Date Range"
  default_value { date_range { start: "2025-01-01"  end: "2025-12-31" } }
  date_range {
    granularity: DATE_GRANULARITY_MONTH
    presets { label: "This Year"  start: "2025-01-01"  end: "2025-12-31" }
    presets { label: "Last 90 Days"  start: "-90d"  end: "now" }
  }
}

# --- Row 0: KPI cards ---

widgets {
  id: "revenue-kpi"
  widget_type: WIDGET_TYPE_METRIC_CARD
  title: "Total Revenue"
  position { x: 0  y: 0  w: 8  h: 4 }
  data_source {
    data_source: "dataSources/sales-flight"
    params {
      key: "year"  value { variable_ref: "${selected_year}" }
      key: "regions"  value { variable_ref: "${selected_regions}" }
    }
  }
  spec { metric_card {
    value: "total_revenue"  format: "currency:USD"
    comparison { value: "prev_quarter_revenue"  label: "vs Q3"  direction: COMPARISON_DIRECTION_HIGHER_IS_BETTER }
  }}
}

# --- Row 4: Charts ---

widgets {
  id: "revenue-trend"
  widget_type: WIDGET_TYPE_LINE_CHART
  title: "Revenue Trend"
  position { x: 0  y: 4  w: 16  h: 8 }
  data_source {
    data_source: "dataSources/sales-flight"
    params {
      key: "year"  value { variable_ref: "${selected_year}" }
      key: "date_range"  value { variable_ref: "${date_range}" }
    }
  }
  spec { chart {
    chart_type: CHART_TYPE_LINE
    data_mapping { x: "month"  y: "revenue"  group_by: "region" }
    x_axis { title: "Month"  scale_type: SCALE_TYPE_TIME }
    y_axis { title: "Revenue ($)"  label_format: "$~s" }
    annotations { type: ANNOTATION_TYPE_LINE_Y  value: 1000000  label: "Target" }
  }}
}

widgets {
  id: "region-breakdown"
  widget_type: WIDGET_TYPE_PIE_CHART
  title: "Revenue by Region"
  position { x: 16  y: 4  w: 8  h: 8 }
  data_source {
    data_source: "dataSources/sales-flight"
    params { key: "year"  value { variable_ref: "${selected_year}" } }
  }
  spec { chart {
    chart_type: CHART_TYPE_DONUT
    data_mapping { value: "revenue"  category: "region" }
    labels { field: "region"  position: LABEL_POSITION_OUTSIDE  connector: true }
  }}
}

# --- Row 12: Pivot table ---

widgets {
  id: "sales-table"
  widget_type: WIDGET_TYPE_PIVOT_TABLE
  title: "Sales Breakdown"
  position { x: 0  y: 12  w: 24  h: 10 }
  data_source {
    data_source: "dataSources/sales-flight"
    params { key: "year"  value { variable_ref: "${selected_year}" } }
  }
  spec { pivot_table {
    fields { rows: ["region", "product"]  columns: ["quarter"]  values: ["revenue", "units_sold"]  value_in_cols: true }
    meta { field: "revenue"    name: "Revenue"    formatter: "currency:USD" }
    meta { field: "units_sold" name: "Units Sold"  formatter: "number:0" }
    sort { sort_field_id: "revenue"  sort_direction: SORT_DIRECTION_DESC }
    totals {
      row { show_grand_totals: true  show_sub_totals: true  sub_totals_dimensions: ["region"]  aggregation: AGGREGATION_SUM }
      col { show_grand_totals: true  aggregation: AGGREGATION_SUM }
    }
    hierarchy_type: PIVOT_HIERARCHY_TYPE_GRID
    frozen { row_header: true }
  }}
}
```
