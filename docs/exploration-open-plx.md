# Exploration: open-plx via ast-agent

Date: 2026-03-20

## Method

Used only `ast-agent` JSON operations (list, skeleton, read) to understand
the open-plx codebase. Never read any source file directly.

## Stats

- 19 Rust files in the project
- ~300 lines of AST output consumed (skeletons + targeted reads)
- Estimated actual codebase size: 3000+ lines
- Reduction: ~10x

## Operations Used

| Operation | Count | Purpose |
|-----------|-------|---------|
| list | 19 | Inventory all files, find where to drill in |
| skeleton | 3 | Understand file structure (main.rs, state.rs, model.rs) |
| read | ~10 | Read specific structs and functions |

## What Worked Well

1. **list as the entry point.** Item counts immediately show which files are
   interesting. `model.rs` (38 items) and `convert.rs` (33 items) stand out
   as the data model. `flight.rs` (11 methods) is clearly the main service.

2. **skeleton for file understanding.** The state.rs skeleton (20 lines) tells
   you everything about AppState: its fields, its constructor, and the key
   method (resolve_widget_data). No need to read the implementation.

3. **read for specific items.** Reading DashboardFile, WidgetConfigYaml, and
   DataSourceConfigYaml gave the complete data model in ~40 lines.

4. **Structured list output.** Field counts, method counts, and stmt counts
   help prioritize. A function with 13 stmts is worth reading; one with 1 stmt
   is trivial.

## What Didn't Work Well

1. **Trait impl display bug.** `impl AuthProvider for AuthProvider for DevAuth`
   instead of `impl AuthProvider for DevAuth`. The trait name is duplicated in
   the display.

2. **No cross-file navigation.** After seeing `self.state.resolve_widget_data()`
   in flight.rs, I had to manually figure out that `state` lives in state.rs.
   A "go to definition" or "find type" operation would help.

3. **No use-statement listing.** The `list` operation skips `use` statements.
   These are critical for understanding imports and dependencies between crates.
   The skeleton shows them but list doesn't.

4. **No module-level overview.** I had to manually `find *.rs` first. The tool
   should have a `list-files` or `project` operation that shows the file tree
   with per-file summaries.

5. **No param type resolution.** The list output shows `params: ["&self"]` but
   for non-self params it shows the raw type as a string. For message types
   like `Request<Ticket>`, you can't tell what `Ticket` is without reading
   the import or the proto file.

## Suggested Improvements

### Priority 1: Project-level operation
```json
{"op": "project", "dir": "/path/to/project"}
```
Returns a tree of files with per-file item summaries. Eliminates the need
to `find *.rs` and list each file separately.

### Priority 2: Include use statements in list
Add `use` items to the list output so the agent can see dependencies
without reading the skeleton.

### Priority 3: Cross-file "find type"
```json
{"op": "find", "dir": "/path/to/project", "name": "AppState"}
```
Search all files for a type/function by name. Returns the file and address.

### Priority 4: Fix trait impl display
`impl Trait for Type` should display correctly, not `impl Trait for Trait for Type`.

## Architecture Derived

```
open-plx (4 crates):

  open-plx-config
    model.rs      -- 38 types: dashboards, widgets, data sources, permissions
    loader.rs     -- ConfigLoader reads YAML files
    convert.rs    -- YAML -> proto conversion (33 functions)
    static_data.rs -- static data source -> Arrow RecordBatch

  open-plx-core
    build.rs      -- protoc compilation
    lib.rs        -- re-exports generated proto types

  open-plx-auth
    lib.rs        -- Principal, AuthProvider trait, 3 providers (Dev/ApiKey/OIDC),
                     AuthInterceptor, permission checking

  open-plx-server
    main.rs       -- tonic server setup, gRPC-web, CORS
    state.rs      -- AppState (dashboards + data_sources + flight_sql_pool)
    services/
      dashboard.rs    -- DashboardService (6 RPCs)
      data_source.rs  -- DataSourceService (7 RPCs)
      flight.rs       -- Arrow Flight (do_get is the key path)
      widget_data.rs  -- WidgetDataService (get_widget_data)
      health.rs       -- health check

Data flow:
  YAML -> ConfigLoader -> AppState
  Client -> gRPC -> AuthInterceptor -> Service -> AppState.resolve_widget_data()
         -> FlightSQL or static -> Arrow RecordBatch -> Flight stream -> Client
```
