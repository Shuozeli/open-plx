# Widget Expansion Plan

## Current State

### Implemented Widgets (6)

| Widget Type | Library | Renderer | Status |
|-------------|---------|----------|--------|
| LINE_CHART | G2 | `line` mark | Full (line, smooth, step shapes) |
| BAR_CHART | G2 | `interval` mark | Full (vertical, horizontal, stacked, grouped) |
| PIE_CHART | G2 | `interval` + theta | Full (pie, donut) |
| PIVOT_TABLE | S2 | `PivotSheet` | Full (rows, columns, values, meta, sort) |
| METRIC_CARD | Antd | `Statistic` | Full (value, comparison, sparkline) |
| TEXT | Antd | markdown/plain | Full |

### Proto-defined but Frontend-unimplemented Chart Types (4)

These exist in `widget_spec.proto` as `ChartType` enums and have mapper stubs
in `chartMapper.ts`, but no dedicated widget component or testing:

| Chart Type | G2 Mark | Mapper Status | Notes |
|------------|---------|---------------|-------|
| SCATTER | `point` | Stub | Needs size/color encoding, tooltips |
| HEATMAP | `cell` | Stub | Needs color scale, x/y categorical axes |
| HISTOGRAM | `rect` | Stub | Needs binX transform |
| RADAR | `line` + radar coord | Stub | Needs radar coordinate, area fill |

---

## Proposed New Widget Types

### Tier 1: High Value, Low Effort

These use G2 marks that are close to what we already support. They need
a new `WidgetType` enum + proto spec + mapper logic + widget component.

#### 1. SCATTER_CHART

**G2 mark:** `point` (23 shape variants)
**Use case:** Correlation analysis, distribution visualization, bubble charts.
**Data mapping:** x (numeric), y (numeric), size (optional), color (optional group_by)
**Why:** The most common chart type we're missing. Trivial mapper since G2 `point` is straightforward.

**Proto addition:**
```
WIDGET_TYPE_SCATTER_CHART = 7;
```
Uses existing `ChartSpec` with `CHART_TYPE_SCATTER`. No new proto message needed.

**Effort:** Small. Mapper stub exists, just needs a `ScatterChartWidget.tsx` + registry entry.

#### 2. HEATMAP

**G2 mark:** `cell`
**Use case:** Activity calendars (GitHub-style), correlation matrices, time-of-day patterns.
**Data mapping:** x (category), y (category), color (value)
**Why:** Excellent for the HN demo (stories by weekday x hour).

**Proto addition:**
```
WIDGET_TYPE_HEATMAP = 8;
```
Uses existing `ChartSpec` with `CHART_TYPE_HEATMAP`. Needs color scale config in proto.

**Effort:** Small. Mapper stub exists. Need to add `color` to data mapping and configure continuous color scale.

#### 3. RADAR_CHART

**G2 mark:** `line` (or `area`) + radar coordinate
**Use case:** Multi-dimensional comparison (e.g., comparing companies across metrics).
**Data mapping:** dimensions (categorical), value (numeric), group_by (optional)
**Why:** Common in executive dashboards. G2 has native radar coordinate support.

**Proto addition:**
```
WIDGET_TYPE_RADAR_CHART = 9;
```
Uses existing `ChartSpec` with `CHART_TYPE_RADAR`.

**Effort:** Small. Mapper stub exists. Needs `coordinate: { type: "radar" }` in mapper.

#### 4. TABLE_SHEET (Flat Table)

**S2 type:** `TableSheet`
**Use case:** Simple tabular data without pivot dimensions. Sortable columns.
**Why:** Not every table needs pivot rows/columns. A flat table is often clearer.

**Proto addition:**
```
WIDGET_TYPE_TABLE = 10;

message TableSpec {
  repeated string columns = 1;           // Column fields to display
  repeated FieldMeta meta = 2;           // Column display names + formatters
  repeated TableSortParam sort = 3;      // Default sort
  bool show_series_number = 4;           // Show row numbers
  FrozenConfig frozen = 5;               // Frozen columns
}
```

**Effort:** Medium. New proto message, new mapper (`tableMapper.ts`), new component. But S2 `TableSheet` API is similar to `PivotSheet`.

### Tier 2: Medium Value, Medium Effort

These require new G2 marks or compositions that need more mapper work.

#### 5. GAUGE

**G2 mark:** `gauge` (composite: radial interval + indicator)
**Use case:** KPI progress, SLA compliance, capacity meters.
**Data mapping:** value (0-1 or 0-100), target (optional)
**Why:** Very common in operational dashboards. G2 has a built-in gauge composite.

**Proto addition:**
```
WIDGET_TYPE_GAUGE = 11;

message GaugeSpec {
  string value_field = 1;         // Field containing the gauge value
  double min = 2;                 // Minimum value (default 0)
  double max = 3;                 // Maximum value (default 100)
  string format = 4;              // Display format
  repeated GaugeRange ranges = 5; // Color ranges (e.g., red/yellow/green)
}

message GaugeRange {
  double from = 1;
  double to = 2;
  string color = 3;               // Hex color or semantic name
}
```

**Effort:** Medium. New proto message, new composite mark mapping, new component.

#### 6. TREEMAP

**G2 mark:** `treemap` (composite: hierarchical rect partition)
**Use case:** Hierarchical proportion visualization (disk usage, budget breakdown).
**Data mapping:** name (category), value (numeric), children (nested)
**Why:** Useful for showing part-to-whole relationships with hierarchy.

**Proto addition:**
```
WIDGET_TYPE_TREEMAP = 12;

message TreemapSpec {
  string value_field = 1;         // Numeric field for area size
  string name_field = 2;          // Label field
  string color_field = 3;         // Color grouping field
  bool show_labels = 4;           // Show rect labels
}
```

**Effort:** Medium. The G2 treemap mark handles layout automatically. Data needs to be flat (G2 handles the hierarchy via transforms). Proto + mapper + component.

#### 7. FUNNEL

**G2 mark:** `interval` with funnel/pyramid shape
**Use case:** Conversion funnels, sales pipelines.
**Data mapping:** stage (category), value (numeric)
**Why:** Very common in product/marketing dashboards.

**Proto addition:**
```
WIDGET_TYPE_FUNNEL = 13;

message FunnelSpec {
  string stage_field = 1;         // Stage name field (ordered)
  string value_field = 2;         // Numeric value per stage
  bool show_conversion = 3;       // Show conversion rates between stages
  bool pyramid = 4;               // Use pyramid shape instead of funnel
}
```

**Effort:** Medium. G2 interval supports `shape: "funnel"` and `shape: "pyramid"`. Needs coordinate transform + connector labels for conversion rates.

#### 8. BOX_PLOT

**G2 mark:** `boxplot` (composite: auto-calculates Q1/Q2/Q3/whiskers)
**Use case:** Statistical distribution comparison, outlier detection.
**Data mapping:** x (category), y (numeric values)
**Why:** Essential for data analysis dashboards. G2's `boxplot` mark auto-computes statistics from raw data.

**Proto addition:**
```
WIDGET_TYPE_BOX_PLOT = 14;
```
Uses existing `ChartSpec`. Add `CHART_TYPE_BOX_PLOT = 11` to the enum.

**Effort:** Medium. New chart type enum, mapper support, G2 boxplot mark auto-handles statistics.

### Tier 3: High Value, High Effort

These require significant new proto design or complex G2 compositions.

#### 9. SANKEY

**G2 mark:** `sankey` (composite: polygon nodes + ribbon links)
**Use case:** Flow visualization (traffic sources, budget allocation, user journey).
**Data mapping:** source, target, value (link weights)
**Why:** Powerful for showing flow/transfer relationships. G2 has native support.

**Proto addition:**
```
WIDGET_TYPE_SANKEY = 15;

message SankeySpec {
  string source_field = 1;        // Source node field
  string target_field = 2;        // Target node field
  string value_field = 3;         // Flow value field
  int32 node_width = 4;           // Node rectangle width (default 20)
  int32 node_padding = 5;         // Vertical padding between nodes (default 8)
}
```

**Effort:** High. Link data format differs from standard tabular data. Need to handle source/target/value triples. G2 sankey mark handles layout.

#### 10. WORD_CLOUD

**G2 mark:** `wordcloud` (composite: d3-cloud layout with rotating text)
**Use case:** Topic frequency, tag clouds, text analysis.
**Data mapping:** text (string), value (numeric weight)
**Why:** Engaging for content analysis dashboards. G2 wraps d3-cloud.

**Proto addition:**
```
WIDGET_TYPE_WORD_CLOUD = 16;

message WordCloudSpec {
  string text_field = 1;          // Text/word field
  string value_field = 2;         // Weight/frequency field
  int32 max_words = 3;            // Maximum words to display (default 200)
}
```

**Effort:** Medium-High. d3-cloud layout is CPU-intensive for large datasets. Needs size limits.

#### 11. FORCE_GRAPH

**G2 mark:** `forceGraph` (composite: d3-force nodes + links)
**Use case:** Network visualization, dependency graphs, social graphs.
**Data mapping:** nodes (id, label), links (source, target, value)
**Why:** Specialized but very impactful for the right use cases.

**Proto addition:**
```
WIDGET_TYPE_FORCE_GRAPH = 17;

message ForceGraphSpec {
  string node_id_field = 1;
  string node_label_field = 2;
  string source_field = 3;
  string target_field = 4;
  string link_value_field = 5;    // Optional: link weight
}
```

**Effort:** High. Graph data needs separate node/link tables or a special encoding. Force simulation is interactive (drag nodes). Complex proto design.

#### 12. GEO_MAP

**G2 composition:** `geoView` + `geoPath`
**Use case:** Geographic data visualization (choropleth maps, point maps).
**Data mapping:** region (geo key), value (numeric for color), lat/lon (for points)
**Why:** Essential for location-based analytics. G2 supports d3-geo projections.

**Effort:** Very High. Requires GeoJSON data handling, projection config, and potentially large map tile data. Deferred.

---

## S2 Enhancements (Not New Widget Types)

These improve the existing `PIVOT_TABLE` and add `TABLE` without new widget types:

### A. Conditional Formatting

S2 supports 4 condition types per field:
- **Text conditions:** Font color/size based on value thresholds
- **Background conditions:** Cell background color
- **Icon conditions:** Status icons (arrows, dots) based on value
- **Interval conditions:** In-cell progress bars (mini bar charts)

**Proto addition to PivotTableSpec:**
```
repeated ConditionalFormat conditions = 10;

message ConditionalFormat {
  string field = 1;
  ConditionalFormatType type = 2;    // TEXT, BACKGROUND, ICON, INTERVAL
  repeated Threshold thresholds = 3; // value -> style mapping
}
```

### B. Mini Charts in Cells

S2 data cells can render inline sparklines, bars, and bullet charts:
- **Line sparkline:** Trend indicator within a cell
- **Bar chart:** Mini bar for in-cell comparison
- **Bullet chart:** Progress toward target (actual vs. target vs. range)

These are controlled via the `conditions` config, not a separate widget type.

### C. Totals and Subtotals

Already defined in proto but not yet implemented in the mapper:
```
message TotalsConfig {
  TotalPosition row_position = 1;     // TOP or BOTTOM
  TotalPosition col_position = 2;     // LEFT or RIGHT
  bool show_grand_totals = 3;
  bool show_sub_totals = 4;
  string grand_total_label = 5;       // Default: "Total"
  string sub_total_label = 6;         // Default: "Subtotal"
  Aggregation aggregation = 7;        // SUM, AVG, COUNT, MIN, MAX
}
```

---

## Recommended Implementation Order

### Phase A: Complete Existing Proto Types (4 charts)
Scatter, Heatmap, Histogram, Radar are already in proto. Just need
frontend widget components + mapper enhancements.
**Effort:** 1-2 days

### Phase B: New Tier 1 Widgets (+1 table type)
Table (flat table via S2 TableSheet).
**Effort:** 1 day

### Phase C: Statistical + KPI Widgets (+3)
Gauge, Box Plot, Funnel.
**Effort:** 2-3 days

### Phase D: Hierarchical + Flow Widgets (+2)
Treemap, Sankey.
**Effort:** 2-3 days

### Phase E: Specialized Widgets (+2)
Word Cloud, Force Graph.
**Effort:** 3-4 days

### Phase F: S2 Enhancements
Conditional formatting, mini charts, totals/subtotals.
**Effort:** 2-3 days

---

## Proto Design Considerations

### ChartSpec vs. Dedicated Specs

The current design uses `ChartSpec` for all G2 charts (line, bar, pie, scatter,
heatmap, histogram, radar). This works because they share the same structure:
data_mapping (x/y/color/size), stack_mode, axes, labels, annotations.

New chart types fall into two categories:

**Fits in ChartSpec** (add enum value only):
- Scatter, Heatmap, Histogram, Radar, Box Plot

**Needs dedicated spec** (new proto message):
- Gauge (min/max/ranges, not x/y)
- Treemap (hierarchical, name/value)
- Funnel (stages, conversion rates)
- Sankey (source/target/value links)
- Word Cloud (text/weight, not x/y)
- Force Graph (nodes/links, not tabular)
- Table (columns, not pivot dimensions)

### WidgetSpec Oneof Growth

Current oneof has 4 variants: `chart`, `pivot_table`, `metric_card`, `text`.
With all proposed additions: 11 variants. This is manageable -- proto oneofs
scale well. Each new spec message is independent.

```protobuf
message WidgetSpec {
  oneof spec {
    ChartSpec chart = 1;               // Line, Bar, Pie, Scatter, Heatmap, etc.
    PivotTableSpec pivot_table = 2;    // Pivot table (S2 PivotSheet)
    MetricCardSpec metric_card = 3;    // Single KPI metric
    TextSpec text = 4;                 // Static text/markdown
    TableSpec table = 5;               // Flat table (S2 TableSheet)
    GaugeSpec gauge = 6;               // Radial gauge/meter
    TreemapSpec treemap = 7;           // Hierarchical treemap
    FunnelSpec funnel = 8;             // Conversion funnel
    SankeySpec sankey = 9;             // Flow/Sankey diagram
    WordCloudSpec word_cloud = 10;     // Word cloud
    ForceGraphSpec force_graph = 11;   // Network graph
  }
}
```

---

## G2 Capabilities NOT Proposed as Widgets

These are powerful G2 features but don't map well to a declarative
dashboard config model:

- **Small Multiples** (FacetRect/FacetCircle) -- complex layout, better served by multiple widgets
- **Geographic maps** (GeoView/GeoPath) -- requires GeoJSON, deferred
- **Parallel coordinates** -- niche, complex encoding
- **Circle packing** (Pack) -- overlaps with treemap
- **Partition/Sunburst** -- overlaps with treemap + pie
- **Chord diagram** -- similar to sankey but circular
- **Force-directed tree** -- overlaps with force graph
- **Liquid gauge** -- novelty, gauge covers the use case
- **Beeswarm** -- niche, scatter covers most use cases

These can be reconsidered if specific dashboard use cases emerge.
