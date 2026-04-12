# Table Enhancement Design

<!-- agent-updated: 2026-04-11T00:00:00Z -->

## Context

open-plx v1 invested heavily in chart widgets (11 types, rich declarative spec) while keeping table support minimal — a basic flat table with column selection, pagination, and conditional formatting. This design doc covers extensions across three areas: table enhancements, a graph widget, write-capable row actions, a gRPC data source adapter, and URL variable binding.

## Design Principles

1. **Proto-first**: All types are generated from proto files. No handwritten types.
2. **Semantic vocabulary**: Widget specs use domain terms (`sortable`, `filterable`, `renderer`) not library terms (`sort`, `filter`, `cellRenderer`). The mapper translates to S2/G6 config.
3. **Backend stays thin**: The server resolves data sources and applies permissions. It never dictates rendering details.
4. **Additive only**: All new proto fields are optional with defaults. No breaking changes to existing dashboards.
5. **No callbacks in protos**: Functions are not serializable over gRPC. All behavior is configured declaratively.

---

## Scope A: Table Enhancements (Phases A–F)

### Design Decisions

#### Why S2 for Tables

The project already uses AntV S2 for pivot tables. Using `TableSheet` (flat table) for the plain table widget keeps the library footprint bounded and leverages existing familiarity. The tradeoff: S2 TableSheet has a less mature ecosystem than G2 for charts, and some features (search toolbar) are not built-in.

#### Sort & Filter: Proto vs. Runtime Config

**Option A**: Put sort/filter state in the proto (declarative, server-driven).
**Option B**: Put sort/filter state in React state (client-driven, ephemeral).

Decision: **Option A for defaults, Option B for runtime state**.

- The proto carries *default* sort/filter — applied on initial load from the server.
- Runtime sort/filter (user clicks) stays in React state — S2 handles this natively.
- `view.enableSearch` signals that the frontend should render a search input, but the actual search text is ephemeral client state.

This matches the two-phase rendering model: server provides the initial state, client manages user-driven mutations.

#### Cell Renderers: Oneof vs. Discriminated Union

S2 cell renderers are complex: icon conditions, bar conditions, text conditions. The natural proto representation is a `oneof`, which enforces exhaustive matching in Rust and TypeScript.

```protobuf
message TableCellRenderer {
  oneof renderer {
    TableCellRendererText text = 1;
    TableCellRendererIcon icon = 2;
    TableCellRendererBar bar = 3;
    TableCellRendererLink link = 4;
    TableCellRendererProgress progress = 5;
  }
}
```

This is the same pattern as `WidgetSpec.spec` — proven to work at scale.

#### Filter: Why not `customFilter` Functions

S2 supports `customFilter: (row) => boolean` for programmatic filtering. This cannot be serialized over gRPC. Three options:

| Approach | Pros | Cons |
|----------|------|------|
| `TABLE_FILTER_TYPE_LIST` only | Simple, serializable | Limited to multi-select from pre-defined values |
| CEL expressions in proto | Expressive, serializable | Requires CEL runtime in frontend and backend |
| Pre-defined filter values + `TABLE_FILTER_TYPE_TEXT`/`RANGE` as frontend-only hints | Practical | TEXT/RANGE filters can't be pushed to server |

Decision: Support `LIST` (fully declarative), add `TEXT`/`RANGE` as frontend hints but note they are client-side only. Server-side filter pushdown requires Phase F (server-side pagination) first.

---

## Scope B: Graph Widget (Phase G)

### Why G6, Not G2 or ECharts

G6 (AntV Graph) is the natural choice — same AntV family as G2 and S2, shares the design language, and has a React binding (`@antv/g6`). ECharts is more feature-rich for graphs but is a separate library with different paradigms.

### Graph Data Model

G6 accepts two data formats:
- **Tree data**: `{ id, label, children[] }` — for hierarchical layouts
- **Graph data**: `{ nodes[], edges[] }` — for general network layouts

open-plx uses the graph format (nodes + edges) because it's more general. The mapper transforms flat data (rows with `source`, `target`, `value` columns) into G6 format.

### Layout Strategy

| Layout | Best for | Supports direction |
|--------|----------|-------------------|
| Force-directed | General networks, unknown hierarchy | No |
| Dagre (TB/LR/RL/BT) | DAGs, hierarchies | Yes |
| Circular | Balanced comparisons | No |
| Grid | Uniform data | No |
| Concentric | Centrality visualization | No |

The proto exposes all of these via `GraphLayoutType`. The mapper translates to G6's layout plugins.

### Interaction Model

Graph interaction is different from chart interaction:
- Charts: click on a data point → set variable
- Graphs: click on a node → set variable (same pattern, different data)

The same `onClickInteraction` mechanism applies. `GraphInteraction` configures which interactions are enabled.

---

## Scope C: Row-Action Columns (Phase H)

### Breaking the "Pure Function" Property

The current model:
```
Dashboard (config) + DataColumns (data) -> Widget (React component)
```

Row actions add:
```
Dashboard (config) + DataColumns (data) + Actions (gRPC methods) -> Widget (React component)
```

This is a significant architectural change. The implications:

1. **Backend now has write operations**: `WidgetActionService.InvokeAction` is a write RPC.
2. **Action permission is separate from data permission**: A user can view a table but not invoke "Restart" on it.
3. **State mutation on the server**: Actions change upstream state, not just open-plx state.
4. **The frontend is no longer a pure function of config + data**: The same config + data can render different action buttons depending on user permissions.

### Why a New Service, Not the Existing WidgetDataService

`WidgetDataService` is read-only (arrow flight data fetch). Adding write operations to it would mix concerns. A separate `WidgetActionService` keeps the read/write separation clean and mirrors the AIP pattern of separate resource types.

### Action Invocation Flow

```
User clicks "Restart" button
  -> S2Table detects cell click on action column
  -> TableWidget shows confirmation dialog (if confirmMessage is set)
  -> WidgetActionClient.invokeAction({ widgetId, actionId, requestBody })
  -> WidgetActionService validates action permission
  -> WidgetActionService forwards to upstream gRPC service
  -> Upstream service changes state
  -> WidgetActionService returns response
  -> TableWidget shows success/error toast
  -> Table refreshes (or dashboard variable is updated)
```

### Request Template Interpolation

The `request_template` field uses `{row.fieldName}` interpolation:
```yaml
action:
  id: "restart"
  method: "ops.Service/RestartInstance"
  requestTemplate: "{row.instance_id}"
```

The backend validates that `instance_id` exists in the row schema before forwarding. This prevents arbitrary field injection.

### Security Model

1. **Action permissions**: Separate from layout and data permissions. `permissions.yaml` adds `actions` to the permission check for widgets.
2. **Principal passthrough**: The authenticated principal (from the auth interceptor) is forwarded via gRPC metadata to the upstream service.
3. **Rate limiting**: Actions should be rate-limited. A loop guard: action → variable change → widget re-renders → action column re-renders → re-fire. Mitigation: debounce or explicit "Apply" step.
4. **Audit logging**: Every action invocation logs `action.invoke` with user, widget, action_id, upstream_method, result.

---

## Scope D: gRPC Data Source Adapter (Phase I)

### Why Not Flight SQL for Everything

Flight SQL is the right protocol for data query engines (Dremio, Databricks, DuckDB). But most upstream services are plain gRPC:
- `signal_engine.NetworkService.GetGraph` → returns proto, not Arrow
- `knowledge_store.EntityService.GetEntities` → returns proto, not Arrow

Wrapping each service in a Flight SQL adapter is 20×N work. The gRPC proxy is ~1 shared helper.

### Columnar Conversion: Shared Helper

`ui_proxy` already has logic to convert protobuf messages to Arrow `RecordBatch`. This logic should be extracted to `open-plx-core` so both `ui_proxy` and `open-plx-server` use it.

```
open-plx-core (shared)
  -> proto_columnar.rs: converts any protobuf Message -> RecordBatch

ui_proxy
  -> uses proto_columnar.rs

open-plx-server
  -> GrpcProxyClient
      -> calls upstream gRPC
      -> uses proto_columnar.rs to convert response
      -> returns RecordBatch
```

### Schema: Explicit vs. Inferred

Option A: Require explicit `response_schema` in `GrpcProxyConfig`.
Option B: Infer schema from first response.

Decision: **Option A with Option B as fallback**.

Explicit schema is safer — the dashboard author knows what columns are available at authoring time. Inference is a convenience for quick prototyping.

### Auth Passthrough Convention

When forwarding the authenticated principal to upstream gRPC services, use `x-auth-principal` metadata header. This requires:
1. Upstream services to respect this header (convention, not enforced by open-plx)
2. A note in the `GrpcProxyConfig` docs that upstream services must validate the forwarded principal

This is a security assumption that should be documented clearly.

---

## Scope E: URL Variable Binding (Phase J)

### Why Not a Dedicated API Endpoint

Option A: Dedicated `GetDashboardRequest { variables: {...} }` for initial render with variables.
Option B: Frontend reads URL on mount, no server change.

Decision: **Option B for now, Option A as a future enhancement**.

Option B is simpler (no proto change, no backend change) and covers the main use case (deep-links). Option A is for server-side rendering / SSR which is out of scope.

### URL Encoding Conventions

| Type | Format | Example |
|------|--------|---------|
| Text | `key=value` | `?ticker=AAPL` |
| Multi-select | Repeated params | `?regions=US&regions=EU` |
| Date | ISO 8601 | `?start=2025-01-01&end=2025-12-31` |
| Date range | Comma-separated | `?range=2025-01-01,2025-12-31` |

These conventions are documented but not enforced by the frontend — variables are already typed from the proto.

### History API vs. hash routing

Currently the project uses hash routing (`#dashboards/stock-detail`). `history.replaceState` works the same way for both. No routing change needed.

---

## Cross-Cutting Concerns

### Error Handling

| Phase | Error | User Sees |
|-------|-------|-----------|
| A-F | S2 render error | Widget error boundary with message |
| G | G6 render error | Widget error boundary |
| H | Action invocation fails | Toast with error message |
| H | Upstream service unreachable | Toast: "Action unavailable" |
| I | gRPC call fails | Widget data error (same as Flight SQL failure) |
| J | Invalid URL param | Ignored silently, use default variable value |

### Observability

New event log entries:
- `widget.action.invoke` (Phase H): `{ user, widget_id, action_id, upstream_method, duration_ms, success }`
- `grpc_proxy.fetch` (Phase I): `{ service, method, rows, duration_ms, success }`

### Proto Numbering

All new fields use the next available field numbers per message. No reuse of freed field numbers (proto3 doesn't reuse).

### Backward Compatibility

All additions are `optional` (implicit in proto3). Fields not set default to zero/false/empty. Existing dashboards continue to work without modification.

### Field Number Gaps

Some existing messages have gaps in field numbering (e.g., `TableColumn` fields 1-3, then 5-7). These gaps are from earlier versions and are preserved. New fields always use the next available number.

---

## Comparison with Alternatives

### Table: S2 vs Ag-Grid vs Material Table

| Library | Pros | Cons |
|---------|------|------|
| AntV S2 | Same family as charts, shared config patterns, good React integration | Smaller ecosystem than ag-grid |
| ag-Grid | Enterprise-grade, more features | Different paradigm, larger bundle |
| Material Table | Antd-compatible | Less feature-rich for data apps |

Decision: Stay with S2. The project already uses it for pivot tables. Consistency in the mapper layer is worth more than features we don't currently need.

### Graph: G6 vs Cytoscape.js vs D3 Force

| Library | Pros | Cons |
|---------|------|------|
| AntV G6 | Same family, React binding, good documentation | Less flexible than D3 |
| Cytoscape.js | More layout algorithms | No official React binding |
| D3 Force | Full flexibility | No React binding, lots of boilerplate |

Decision: G6. The AntV family consistency matters more than marginal feature differences.

### Row Actions: WidgetActionService vs. GenericProxy

Option A: `WidgetActionService.InvokeAction` — new service, explicit methods.
Option B: Generic proxy that forwards any gRPC method — simpler but less controlled.

Decision: **Option A**. A generic proxy is a security risk (any method can be called). Explicit action declarations allow permission checks and audit logging.

---

## Out of Scope (for These Phases)

- **Multi-header columns**: S2 supports it but proto design is complex. Revisit when there is a concrete use case.
- **Streaming / real-time data**: Arrow Flight supports streaming. A future `StreamingGetWidgetData` RPC could be added.
- **Dashboard embedding (iframe)**: Requires token auth and cross-origin handling. Separate security review needed.
- **Custom widget plugin system**: Explicitly out of scope per PLX design.
