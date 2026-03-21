# Widget Expansion: 6 to 17 Widget Types

## Context

open-plx currently supports 6 widget types (line, bar, pie, pivot table, metric card, text). The AntV G2 v5 library supports 34 mark types, 9 coordinate systems, and 28 interaction types. S2 v2 offers TableSheet, ChartSheet, and rich analysis features. This plan adds 11 new widget types and S2 enhancements across 6 phases to make open-plx a comprehensive dashboard platform.

## Implementation Phases

### Phase A: Complete 4 Existing Chart Types
**Scatter, Heatmap, Histogram, Radar** -- proto ChartType enums already exist, mapper stubs exist. Need: WidgetType enum entries, widget components, mapper enhancements, test data.

### Phase B: Flat Table
**Table** -- S2 `TableSheet` for simple tabular data. Need: new proto `TableSpec`, `tableMapper.ts`, `S2Table.tsx`, `TableWidget.tsx`.

### Phase C: Statistical/KPI
**Gauge, Funnel, Box Plot** -- G2 composites. Gauge and Funnel need new proto specs. Box Plot fits in ChartSpec.

### Phase D: Hierarchical/Flow
**Treemap, Sankey** -- G2 composites with non-tabular data transforms. Need: new proto specs, mappers that convert flat data to hierarchical/link structures.

### Phase E: Specialized
**Word Cloud** -- G2 wordcloud composite. Need: new proto WordCloudSpec.

### Phase F: S2 Enhancements
**Conditional formatting, totals/subtotals, mini charts** -- enhance existing pivot/table, no new widget types.

---

## Per-Phase Details

### Phase A: Scatter, Heatmap, Histogram, Radar

**Proto changes:**
- `dashboard.proto`: Add `WIDGET_TYPE_SCATTER_CHART=7`, `WIDGET_TYPE_HEATMAP=8`, `WIDGET_TYPE_HISTOGRAM=9`, `WIDGET_TYPE_RADAR_CHART=10`
- No widget_spec.proto changes (use existing ChartSpec + ChartType enums)

**Rust config:**
- `convert.rs`: Add 4 entries to `parse_widget_type()`

**chartMapper.ts enhancements:**
- Scatter: add `size` encoding + size scale `range: [4, 20]`
- Heatmap: map `group_by` -> `color` encoding + sequential palette
- Histogram: add `binX` transform, remove y encoding (auto-counted)
- Radar: add `coordinate: { type: "radar" }` + axis grid config

**New files (4 widgets, 4 data sources):**
- `ScatterChartWidget.tsx`, `HeatmapWidget.tsx`, `HistogramWidget.tsx`, `RadarChartWidget.tsx`
- `config/data_sources/{scatter,heatmap,histogram,radar}-demo.yaml`

**G2 specs produced:**
```
Scatter:   { type: "point", encode: { x, y, size, color }, scale: { size: { range: [4,20] } } }
Heatmap:   { type: "cell", encode: { x, y, color }, scale: { color: { palette: "ylGnBu" } } }
Histogram: { type: "rect", encode: { x }, transform: [{ type: "binX", y: "count" }] }
Radar:     { type: "line", encode: { x, y, color }, coordinate: { type: "radar" }, axis: { x: { grid: true } } }
```

**E2e tests:** 4 new describe blocks verifying chart type, encodings, coordinate, transforms.

### Phase B: Table

**Proto changes:**
- `dashboard.proto`: Add `WIDGET_TYPE_TABLE=11`
- `widget_spec.proto`: Add `TableSpec` message (columns, pagination, sorting, frozen cols) + `TableColumn`, `TablePagination` messages. Add `TableSpec table = 5` to WidgetSpec oneof.

**Rust config:**
- `model.rs`: Add `TableSpecYaml`, `TableColumnYaml`, `TablePaginationYaml`
- `convert.rs`: Add `table_to_proto()`, update `widget_spec_to_proto()`

**New files:**
- `frontend/src/services/mappers/tableMapper.ts` -- proto -> S2 TableSheet config
- `frontend/src/components/widgets/S2Table.tsx` -- S2 TableSheet wrapper (like S2PivotTable but uses `TableSheet`)
- `frontend/src/components/widgets/TableWidget.tsx` -- extracts `spec.case === "table"`, calls mapper, renders

**S2 config produced:**
```
dataCfg: { data: [...], fields: { columns: ["col1", "col2"] }, meta: [...] }
options:  { width, height, pagination: { pageSize: 20 }, interaction: { enableCopy: true } }
```

### Phase C: Gauge, Funnel, Box Plot

**Gauge proto:** `GaugeSpec` (value_field, min, max, format, ranges[]) + `GaugeRange` (from, to, color). `WIDGET_TYPE_GAUGE=12`.

**Gauge G2 spec:**
```
{ type: "gauge", data: { value: 0.75 }, scale: { color: { range: ["#30BF78","#FAAD14","#F4664A"] } } }
```
Fallback if gauge mark unavailable: semicircular interval with theta coordinate.

**Funnel proto:** `FunnelSpec` (category_field, value_field, show_conversion_rate, shape). `WIDGET_TYPE_FUNNEL=13`.

**Funnel G2 spec:**
```
{ type: "interval", encode: { x: "stage", y: "count", shape: "funnel" }, transform: [{ type: "symmetryY" }], coordinate: { transform: [{ type: "transpose" }] } }
```

**Box Plot:** Add `CHART_TYPE_BOX_PLOT=11` to ChartType enum. `WIDGET_TYPE_BOX_PLOT=14`. Uses ChartSpec -- no new proto message. chartMapper maps to `"boxplot"` mark.

**Box Plot G2 spec:**
```
{ type: "boxplot", encode: { x: "category", y: "value" } }
```

### Phase D: Treemap, Sankey

**Treemap proto:** `TreemapSpec` (value_field, hierarchy_fields[], color_field, show_labels). `WIDGET_TYPE_TREEMAP=15`.

**Treemap mapper challenge:** Converts flat tabular data to nested `{name, children[]}` tree. Mapper groups by hierarchy_fields in order.

**Treemap G2 spec:**
```
{ type: "treemap", data: { value: { name: "root", children: [...] } }, encode: { value: "value" }, labels: [{ text: "name" }] }
```

**Sankey proto:** `SankeySpec` (source_field, target_field, value_field). `WIDGET_TYPE_SANKEY=16`.

**Sankey mapper challenge:** Transforms flat source/target/value rows into `{ links, nodes }` structure.

**Sankey G2 spec:**
```
{ type: "sankey", data: { value: { links: [...], nodes: [...] } }, layout: { nodeWidth: 0.008, nodePadding: 0.03 } }
```

### Phase E: Word Cloud

**Proto:** `WordCloudSpec` (text_field, weight_field, color_field, max_words, font_size_range[]). `WIDGET_TYPE_WORD_CLOUD=17`.

**G2 spec:**
```
{ type: "wordCloud", data: [...], encode: { color: "text" }, layout: { fontSize: [12, 60] } }
```

### Phase F: S2 Enhancements

**Conditional formatting:** Add `ConditionalFormat` message to widget_spec.proto. Add to both `PivotTableSpec` and `TableSpec`. Mapper translates to S2 `conditions` option with 4 types: text color, background color, icon, data bar.

**Totals/subtotals:** Wire existing `TotalsConfig` from proto (currently `totals: None` in convert.rs) to S2 `totals` option in pivotMapper.

**Mini charts:** Add `MiniChart` message to `FieldMeta`. Mapper generates S2 custom cell renderer with sparkline/bar/bullet config.

---

## File Change Matrix

| Phase | Proto Files | Rust Files | Frontend Files | Data Sources | Tests |
|-------|------------|------------|----------------|--------------|-------|
| A | 1 modified | 1 modified | 5 created, 1 modified | 4 created | ~16 tests |
| B | 2 modified | 2 modified | 3 created, 1 modified | 1 created | ~8 tests |
| C | 2 modified | 2 modified | 4 created, 2 modified | 3 created | ~12 tests |
| D | 2 modified | 2 modified | 4 created, 1 modified | 2 created | ~8 tests |
| E | 2 modified | 2 modified | 2 created, 1 modified | 1 created | ~4 tests |
| F | 1 modified | 2 modified | 2 modified | 0 | ~6 tests |

**Total: ~18 new files, ~10 modified files, ~11 data sources, ~54 new e2e tests**

---

## Execution Order (Per Phase)

1. Proto changes -> `buf generate proto/` + `cargo build -p open-plx-core`
2. Rust model.rs + convert.rs -> `cargo test && cargo clippy`
3. Static data source YAML
4. Frontend mapper (new or modified)
5. Widget component + S2 wrapper if needed
6. WidgetRegistry entry
7. Dashboard YAML (add widgets to full-demo)
8. `pnpm tsc && pnpm build`
9. E2e test blocks
10. `pnpm exec playwright test` (requires running server)

---

## Verification Plan

**Per phase:**
```bash
buf generate proto/                      # Regenerate TS proto types
cargo test --workspace                   # Rust tests (8+ existing)
cargo clippy --workspace -- -D warnings  # Lint
cd frontend && pnpm tsc --noEmit         # TS type check
pnpm build                               # Production build
```

**E2e (requires docker compose up + cargo run):**
```bash
docker compose up -d                             # Flight SQL + seed data
CONFIG_PATH=config/open-plx.yaml cargo run &     # Backend
cd frontend && pnpm exec playwright test         # All e2e tests
```

**Dark launch verification (per widget):**
1. Open full-demo dashboard in browser
2. Verify widget renders with correct data
3. Toggle dark mode -- verify G2 theme applies
4. Check browser console for errors
5. Verify e2e test for the widget passes

---

## HN Demo Dashboard Expansion

After all phases, expand `config/dashboards/hackernews.yaml` with new widget types:
- **Heatmap:** Stories by weekday x hour (requires new hn-by-hour data source)
- **Radar:** Compare HN story type metrics (stories, avg points, avg comments per type)
- **Table:** Top 50 stories flat table (sortable by points, comments)
- **Funnel:** HN engagement funnel (stories -> with comments -> with 10+ comments -> with 100+ comments)
- **Word Cloud:** Most frequent domains or title words
- **Treemap:** Stories by domain, sized by total points
