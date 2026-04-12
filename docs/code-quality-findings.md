# Code Quality Findings

## Severity Legend
- **HIGH**: Bugs, panics in prod code, silent data loss, correctness issues
- **MEDIUM**: Duplication, missing abstractions, unused code, type safety gaps
- **LOW**: Cosmetic, noise, minor style issues

---

## Previous Review Passes (1 & 2)

All findings from previous review passes are retained below for audit trail.
Items marked FIXED were resolved in commits `e21a817` and `d24003d`.

### Pass 1 -- Silent Failures (HIGH) -- All FIXED
- 1.1 Unknown `principal_type` silently ignored in permission check
- 1.2 `parse_date_granularity` error silently swallowed
- 1.3 `text_to_proto` silently defaults unknown text format
- 1.4 `conditional_format_to_proto` silently defaults
- 1.5 `total_config_to_proto` silently defaults unknown aggregation
- 1.6 `funnel_to_proto` silently defaults unknown funnel shape
- 1.7 `table_to_proto` silently defaults unknown column alignment
- 1.8 Flight service lacks permission checking

### Pass 2 -- Duplicate Resolution & Formatting -- All FIXED
- 11.1 `get_flight_info` calls `resolve_data_source_name` twice
- 11.2 `do_get` calls `resolve_data_source_name` twice
- 12.1 Multiple files fail `cargo fmt --check`

---

## Review Pass 3 (Current) -- Full Codebase Audit

### 1. Duplication (MEDIUM-HIGH)

#### 1.1 Identical G2 Chart Widget Components (11 files)
- **Location:** `frontend/src/components/widgets/LineChartWidget.tsx`, `BarChartWidget.tsx`, `ScatterChartWidget.tsx`, `HeatmapWidget.tsx`, `HistogramWidget.tsx`, `RadarChartWidget.tsx`, `BoxPlotWidget.tsx`, `FunnelWidget.tsx`, `TreemapWidget.tsx`, `SankeyWidget.tsx`, `WordCloudWidget.tsx`
- **Problem:** These 11 widget components follow the exact same structural pattern: extract spec from proto, call a mapper to produce a G2Spec, register test state via `registerWidget()`, render `Card` + `Spin` + `G2Chart`. The only differences are (a) which proto `spec.case` string they check, (b) which mapper function they call, and (c) the test registry `rendered` payload shape. `LineChartWidget` and `BarChartWidget` are **character-for-character identical** except for the function name. Most others differ only in the `registerWidget` rendered fields.
- **Fix:** Extract a single `G2ChartWidgetBase` component parameterized by: spec case string, mapper function, and an optional test-state extractor function. The 11 individual files become one-liner re-exports. This eliminates ~400 lines of copy-paste.

#### 1.2 Duplicated `buildFormatter` Function (3 locations)
- **Location:** `frontend/src/services/mappers/pivotMapper.ts:54-96` (`buildFormatter`)
- **Also at:** `frontend/src/services/mappers/tableMapper.ts:91-115` (`buildFormatter`)
- **Also at:** `frontend/src/components/widgets/MetricCardWidget.tsx:9-29` (`formatValue`)
- **Problem:** Three independent implementations of the same "format string -> display string" logic (currency, percent, number, compact). They use slightly different signatures but identical core logic. If a new format is added, it must be updated in three places.
- **Fix:** Extract a shared `formatValue(value: unknown, formatString: string): string` helper into `frontend/src/services/formatHelper.ts`. pivotMapper and tableMapper call it inside their formatter closures; MetricCardWidget calls it directly.

#### 1.3 Duplicated YAML File Iteration Pattern (6+ locations, Rust)
- **Location:** `crates/open-plx-config/src/loader.rs:40-67` (`load_dashboards`)
- **Also at:** `crates/open-plx-config/src/loader.rs:69-96` (`load_data_sources`)
- **Also at:** `crates/open-plx-cli/src/validate.rs:32-45`, `validate.rs:49-65`
- **Also at:** `crates/open-plx-cli/src/import.rs:30-47`, `import.rs:51-68`
- **Also at:** `crates/open-plx-cli/src/export.rs:112-126`
- **Problem:** The pattern "iterate directory, filter for .yaml/.yml, read file, parse as T" is repeated 6+ times across crates. The `extension().is_some_and(|ext| ext == "yaml" || ext == "yml")` filter appears verbatim in 5 locations.
- **Fix:** Add a generic helper to `open-plx-config`: `fn load_yaml_dir<T: DeserializeOwned>(dir: &Path) -> Result<HashMap<String, T>>` and an `fn is_yaml_file(path: &Path) -> bool` predicate. All call sites use the shared helpers.

#### 1.4 Duplicated Permission Check + Deny Log Pattern (3 locations)
- **Location:** `crates/open-plx-server/src/services/flight.rs:61-71` (`get_flight_info`)
- **Also at:** `crates/open-plx-server/src/services/flight.rs:119-129` (`do_get`)
- **Also at:** `crates/open-plx-server/src/services/widget_data.rs:148-158` (`get_widget_data`)
- **Problem:** The three-step sequence (resolve data source name, check permission, log denial with identical event/fields) is copy-pasted verbatim across three methods.
- **Fix:** Extract a helper: `fn check_data_access(principal: &Principal, state: &AppState, req: &WidgetDataRequest) -> Result<String, Status>` that returns the data source name on success.

#### 1.5 Duplicated ResizeObserver Setup (2 locations, Frontend)
- **Location:** `frontend/src/components/widgets/PivotTableWidget.tsx:14-27`
- **Also at:** `frontend/src/components/widgets/TableWidget.tsx:14-27`
- **Problem:** Identical `ResizeObserver` setup/teardown logic with identical state shape `{ w: 800, h: 300 }`.
- **Fix:** Extract a `useContainerDimensions()` custom hook returning `[ref, { w, h }]`.

---

### 2. Dead Code / Unused Exports (MEDIUM)

#### 2.1 Unused `dataSourceClient` Export
- **Location:** `frontend/src/services/grpc/client.ts:8`
- **Problem:** `dataSourceClient` is created and exported but never imported by any other file in the frontend. It is dead code.
- **Fix:** Remove the export and the `DataSourceService` import. Re-add when a feature needs it.

#### 2.2 Unused Workspace Dependencies
- **Location:** `Cargo.toml:37` (`thiserror = "2"`), `Cargo.toml:41` (`tower = "0.5"`)
- **Problem:** `thiserror` and `tower` are declared in `[workspace.dependencies]` but no crate in the workspace has `thiserror.workspace = true` or `tower.workspace = true` in its `Cargo.toml`.
- **Fix:** Remove both entries from workspace dependencies.

#### 2.3 `#[allow(dead_code)]` Suppressing Real Warning
- **Location:** `crates/open-plx-auth/src/lib.rs:90`
- **Problem:** `#[allow(dead_code)]` on `OidcAuth` suppresses the warning that its underscore-prefixed fields (`_issuer`, `_audience`, `_jwks_uri`) are never read. The struct's `authenticate()` method always returns `Err(Status::unimplemented(...))` and is unreachable because construction panics first (line 152). The `allow` masks future accidental dead code additions.
- **Fix:** Remove the fields entirely (the stub does not need them) and remove the `#[allow(dead_code)]`. Keep the struct skeleton and `authenticate()` stub with a `// TODO(phase2)` comment.

---

### 3. Stringly-Typed APIs / Missing Type Safety (MEDIUM)

#### 3.1 String-typed Enum Fields in YAML Models
- **Location:** `crates/open-plx-config/src/model.rs:76` (`widget_type: String`)
- **Also at:** `model.rs:140` (`chart_type: String`), `model.rs:644` (`principal_type: String`), `model.rs:403` (`format_type: String`), `model.rs:92` (`operator: String`)
- **Problem:** These fields are strings that must be one of a fixed set of values. Invalid values pass serde deserialization and are only caught during proto conversion at runtime. A typo in YAML silently passes the parse step.
- **Fix:** Use `#[serde(rename = "...")]` enum variants. Serde rejects unknown values at parse time, giving earlier and clearer errors. (Carried forward from pass 1, still not addressed.)

#### 3.2 Magic Numbers in Pivot/Table Mapper
- **Location:** `frontend/src/services/mappers/pivotMapper.ts:166-167` (`s.sortDirection === 1 ? "ASC" : "DESC"`)
- **Also at:** `frontend/src/services/mappers/pivotMapper.ts:180` (`proto.hierarchyType === 2`)
- **Problem:** Numeric proto enum values are compared as raw integers without using the generated enum constants. If proto values change, these silently produce wrong results.
- **Fix:** Import the generated enums (e.g., `SortDirection`, `PivotHierarchyType`) and compare against named constants.

---

### 4. Correctness Issues (HIGH)

#### 4.1 `useVariables` Does Not Re-Initialize When Dashboard Changes
- **Location:** `frontend/src/hooks/useVariables.ts:40`
- **Problem:** `useState(defaults)` only uses `defaults` as the initial state. When the user navigates to a different dashboard (new `variables` array), `useMemo` recomputes `defaults`, but `useState` ignores the updated initial value -- the previous dashboard's variable values persist. No `useEffect` resets the state.
- **Fix:** Add a `useEffect` that calls `setValues(defaults)` and `setRevision(0)` when `defaults` changes (using a stable identity check), or use a key on the parent component to force remount on dashboard change.

#### 4.2 `useWidgetData` Omits `variableValues` from Dependency Array
- **Location:** `frontend/src/hooks/useWidgetData.ts:88`
- **Problem:** The `useCallback` dependency array is `[dashboardName, widgetId, revision]`. The `variableValues` parameter is captured in the closure (line 74: `params: variableValues ?? {}`) but is not in the dependency array. The hook relies on `revision` incrementing whenever variables change, which is a fragile implicit contract. If any code path changes a variable without incrementing revision, stale params are sent.
- **Fix:** Either add `variableValues` to the dependency array (with a stable serialization for comparison), or add a comment documenting that `revision` MUST change whenever `variableValues` changes.

#### 4.3 `get_flight_info` Executes Full Query Just for Schema
- **Location:** `crates/open-plx-server/src/services/flight.rs:79`
- **Problem:** `get_flight_info` executes the full data source query (`execute_data_source`) to obtain schema and row count. For Flight SQL data sources with expensive queries, the query runs twice: once for `get_flight_info` and once for the subsequent `do_get`. This doubles query load on the upstream database.
- **Fix:** Cache the query result with a short TTL, implement a schema-only query path, or document that clients should skip `get_flight_info` and call `do_get` directly.

#### 4.4 Word Cloud Missing `text`/`size` Encoding
- **Location:** `frontend/src/services/mappers/wordCloudMapper.ts:20-23`
- **Problem:** The G2 word cloud spec only encodes `color: proto.textField` but does not set `text` or `size` (weight) encodings. G2's `wordCloud` mark needs `text` to know which field contains word text and `size` (or `value`) for word weight. Without these, the chart may render incorrectly or not render words at all.
- **Fix:** Add `text: proto.textField` and `size: proto.weightField` to the `encode` object.

---

### 5. Unsafe Patterns (MEDIUM)

#### 5.1 `unwrap()` on `file_name()` in Non-Test CLI Code
- **Location:** `crates/open-plx-cli/src/import.rs:35`, `import.rs:57`
- **Also at:** `crates/open-plx-cli/src/export.rs:62`, `export.rs:74`
- **Problem:** `Path::file_name()` returns `None` for paths ending in `..` or root paths. While unlikely for files found by `read_dir()`, `unwrap()` in CLI code produces an unhelpful panic backtrace instead of a user-friendly error message.
- **Fix:** Replace with `.ok_or_else(|| anyhow!("path has no file name: {}", path.display()))?`.

#### 5.2 Fragile YAML Name Matching via Substring Search
- **Location:** `crates/open-plx-cli/src/export.rs:112-126` (`find_yaml_by_name`)
- **Problem:** This function finds a YAML file by searching raw file content for `name: {name}` or `name: "{name}"`. This is fragile: it can match inside comments, multi-line strings, or fields like `display_name: dashboards/foo`. It also fails for names containing YAML special characters.
- **Fix:** Parse each YAML file and extract the `name` field value, rather than doing substring matching on raw content. The `ConfigLoader` already has the parsed `HashMap<String, DashboardFile>` -- expose the source file path alongside the parsed data.

---

### 6. Placeholder / Stub Code (MEDIUM)

#### 6.1 OidcAuth Stub That Panics at Startup
- **Location:** `crates/open-plx-auth/src/lib.rs:91-118` (struct + impl)
- **Also at:** `crates/open-plx-auth/src/lib.rs:147-156` (`from_config` panic)
- **Problem:** If a user configures `provider: oidc` in their config YAML, the server panics at startup (line 152) rather than returning a structured error. The `OidcAuth::authenticate()` method body is unreachable dead code because construction always panics first.
- **Fix:** Either remove `OidcAuth` entirely until implemented, or replace the `panic!` with a structured error return from `from_config` (change return type to `Result`). The current `authenticate()` body is dead code behind an unreachable path.

#### 6.2 Validate Errors Cloned Unnecessarily
- **Location:** `crates/open-plx-cli/src/validate.rs:117` (`errors: errors.clone()`)
- **Problem:** The `errors` Vec is cloned into the `ValidateOutput` struct, then the original is only used for printing. Since `errors` is consumed by the print loop and then dropped, the clone is wasteful.
- **Fix:** Move `errors` into the struct (remove `.clone()`), and iterate over `output.errors` for the subsequent print loop.

---

### 7. Noise / Production Overhead (LOW)

#### 7.1 Test Registry Code Ships to Production
- **Location:** `frontend/src/services/testRegistry.ts` (entire file)
- **Also at:** Every widget component imports and calls `registerWidget()` unconditionally.
- **Problem:** The test registry writes to `window.__OPEN_PLX__` in every environment, not just test/dev. Every widget component has a `useEffect` that serializes state via `JSON.parse(JSON.stringify(...))` on every render -- unnecessary overhead in production.
- **Fix:** Guard all `registerWidget()` calls behind `if (import.meta.env.DEV)` or use a no-op stub in production builds. The code comment says "Production builds can tree-shake if register() calls are removed" but no build config actually strips them.

#### 7.2 Excessive Section Divider Comments
- **Location:** `crates/open-plx-auth/src/lib.rs:21-23`, `84-86`, `120-122`, `171-173`, `228-230`
- **Also at:** `crates/open-plx-config/src/model.rs:5-7`, `36-37`, `462-463`, `575-576`, `625-626`
- **Problem:** Long `// ====...====` divider lines that add no information beyond what module structure provides. They inflate line counts and make diffs noisier.
- **Fix:** Low priority / cosmetic. Replace with doc comments where explanation is needed; remove purely decorative dividers.

---

### 8. Typos (LOW)

#### 8.1 `hasPageination` Typo
- **Location:** `frontend/src/services/mappers/tableMapper.ts:125`
- **Problem:** `hasPageination` should be `hasPagination`. Any e2e test referencing this field must use the misspelled name.
- **Fix:** Rename to `hasPagination`. Update any e2e tests that reference the field.

---

## Summary

| Category | New (Pass 3) | Carried Forward | Total Open |
|---|---|---|---|
| Duplication (MEDIUM-HIGH) | 5 | 1 (static_column dispatch) | 6 |
| Dead Code / Unused (MEDIUM) | 3 | 0 | 3 |
| Stringly-Typed APIs (MEDIUM) | 2 | 5 (from pass 1) | 7 |
| Correctness Issues (HIGH) | 4 | 0 | 4 |
| Unsafe Patterns (MEDIUM) | 2 | 0 | 2 |
| Placeholder / Stub (MEDIUM) | 2 | 2 (params, auth conv) | 4 |
| Noise / Overhead (LOW) | 2 | 0 | 2 |
| Typos (LOW) | 1 | 0 | 1 |
| **Total** | **21** | **8** | **29** |

### Priority Fix Order

1. **Correctness (HIGH):** 4.1 (useVariables re-init), 4.4 (word cloud encoding) -- these are likely bugs
2. **Correctness (HIGH):** 4.2 (useWidgetData deps), 4.3 (double query in get_flight_info)
3. **Duplication (HIGH impact):** 1.1 (11 identical chart widgets) -- largest single source of code debt
4. **Duplication (MEDIUM):** 1.2 (buildFormatter x3), 1.3 (YAML iteration x6), 1.4 (permission check x3)
5. **Type Safety (MEDIUM):** 3.1 (stringly-typed YAML enums), 3.2 (magic numbers)
6. **Dead Code (MEDIUM):** 2.1 (unused client), 2.2 (unused workspace deps), 2.3 (allow dead_code)
7. **Unsafe (MEDIUM):** 5.1 (unwrap in CLI), 5.2 (fragile name matching)
8. **Low priority:** Stubs, noise, typos

---

## Review Pass 4 (2026-04-12) -- Pipeline Code Review

### 1. Silent Failures / Swallowed Errors (HIGH)

#### 1.1 Silent YAML serialization fallback
- **Location:** `crates/open-plx-config/src/convert.rs:1153`
- **Problem:** When YAML serialization of sequence/mapping values fails, `unwrap_or_default()` silently returns an empty string. A YAML config error becomes an invisible empty string instead of failing fast.
- **Fix:** Return a `Result` or at minimum log a warning when falling back to empty string.

#### 1.2 Server startup panics on signal handler failure
- **Location:** `crates/open-plx-server/src/main.rs:133, 139`
- **Problem:** `.expect()` calls on signal handler installation will panic the entire server if it fails (e.g., in restricted environments).
- **Fix:** Replace with graceful error handling that logs and continues, or gracefully shuts down.

#### 1.3 Chained unwrap in test code
- **Location:** `crates/open-plx-config/src/loader.rs:183, 195, 199`
- **Problem:** Inside tests, chained `.unwrap()` calls will panic on malformed test fixtures rather than giving a clear error.
- **Fix:** Use `expect()` with descriptive messages, or add a test helper that validates fixtures upfront.

---

### 2. Stub Implementations That Return Empty Data (CRITICAL)

#### 2.1 gRPC proxy returns empty Struct (Critical - silent data loss)
- **Location:** `crates/open-plx-server/src/grpc_proxy_client.rs:220-238` (`call_grpc`)
- **Problem:** The gRPC proxy client returns an empty `Struct` (no fields). Widgets using gRPC proxy silently receive empty RecordBatches with no indication the call was stubbed. Caller cannot distinguish "stubbed" from "genuinely empty data."
- **Fix:** Return an error with clear message indicating the gRPC forwarding is not yet implemented, instead of silent empty struct.

#### 2.2 Widget action stub returns fake success (Critical - silent failure)
- **Location:** `crates/open-plx-server/src/services/widget_action.rs:146-162` (`forward_grpc_call`)
- **Problem:** Stub returns `{"success": true}` regardless of method or request body. Actions silently do nothing in production.
- **Fix:** Return an error indicating the feature is not implemented, or implement actual gRPC forwarding.

---

### 3. Dead Code (HIGH)

#### 3.1 OidcAuth struct completely dead
- **Location:** `crates/open-plx-auth/src/lib.rs:90`
- **Problem:** `OidcAuth` struct annotated `#[allow(dead_code)]` with `authenticate` always returning `Status::unimplemented`. Never instantiated anywhere.
- **Fix:** Either implement OIDC properly or remove the dead code entirely. The TODO at line 109 confirms it was planned but never finished.

#### 3.2 Redundant shadowing import
- **Location:** `crates/open-plx-config/src/convert.rs:1086`
- **Problem:** `use std::collections::HashMap;` inside `grpc_proxy_to_proto` shadows the module-level import unnecessarily.
- **Fix:** Remove the redundant local `use` statement.

#### 3.3 Flight SQL params field explicitly ignored
- **Location:** `crates/open-plx-server/src/flight_sql_client.rs:43-44`
- **Problem:** `params: _` is matched but never used. Query parameters cannot be bound from `DataSourceRef.params`.
- **Fix:** Implement parameter binding per the TODO at line 68, or document why it's not yet supported.

---

### 4. Unsafe Patterns (MEDIUM)

#### 4.1 S2 `as any` casts accessing internal APIs
- **Location:** `frontend/src/components/widgets/S2Table.tsx:66, 73, 100`
- **Problem:** Three `as any` casts to access S2 internal APIs. Project rules prohibit `any` types.
- **Fix:** Create typed wrapper functions for S2 internals, or file a type issue with @antv/s2 to expose these APIs properly.

---

### 5. Missing Abstractions / Code Duplication (MEDIUM)

#### 5.1 Repeated permission check logging blocks (4 copies)
- **Location:**
  - `crates/open-plx-server/src/services/widget_data.rs:149-157`
  - `crates/open-plx-server/src/services/flight.rs:62-70` and `122-131`
  - `crates/open-plx-server/src/services/widget_action.rs:73-81`
- **Problem:** Identical permission-denied logging pattern copy-pasted 4 times.
- **Fix:** Extract to a shared helper: `permission_denied(principal: &Principal, resource: &str, required_role: &str) -> Status`.

#### 5.2 `extract_basic_auth` silently ignores non-basic auth
- **Location:** `crates/open-plx-server/src/flight_sql_client.rs:223-225`
- **Problem:** When user configures `bearer_token_secret` or `mtls` auth, `extract_basic_auth` returns `None` and code proceeds without credentials. No warning that auth type was silently ignored.
- **Fix:** Add validation that returns an error if non-basic auth is configured, or log a warning.

#### 5.3 Repetitive yaml_value conversion functions
- **Location:** `crates/open-plx-config/src/static_data.rs:31-90`
- **Problem:** Four nearly identical functions (`yaml_value_to_string`, `_to_i64`, `_to_f64`, `_to_bool`) follow the same pattern. Could be a single generic function.
- **Fix:** Consolidate into a generic `yaml_value_to<T>` function using `TryFrom` trait.

#### 5.4 Repeated frontend catch blocks
- **Location:**
  - `frontend/src/hooks/useDashboard.ts:24`
  - `frontend/src/hooks/useDashboardList.ts:23`
  - `frontend/src/hooks/useWidgetData.ts:78`
  - `frontend/src/components/widgets/TableWidget.tsx:86`
- **Problem:** Identical `catch (err)` pattern duplicated 4 times.
- **Fix:** Extract to a shared utility: `function parseError(err: unknown): string`.

---

### 6. Wildcard Import (LOW)

#### 6.1 `use crate::model::*` in convert.rs
- **Location:** `crates/open-plx-config/src/convert.rs:6`
- **Problem:** Wildcard import makes it unclear what's being used. The model module has many types (50+).
- **Fix:** Replace with explicit imports for only the types actually used.

---

### 7. Performance Issue (MEDIUM)

#### 7.1 Double query execution for schema-only queries
- **Location:** `crates/open-plx-server/src/services/flight.rs:79-82`
- **Problem:** `get_flight_info` executes the full query just to get schema, then `do_get` executes it again. Doubles load for expensive queries.
- **Fix:** Implement schema-only query path per the TODO comment.

---

## Summary (Pass 4)

| Severity | Count | Issues |
|----------|-------|--------|
| **Critical** | 2 | gRPC stub returning empty struct, YAML silent fallback |
| **High** | 3 | OIDC dead code, gRPC forwarding stubs, action stub |
| **Medium** | 6 | Service discovery, params not bound, double query, auth silent ignore, signal handler panic, test unwrap |
| **Low** | 5 | Duplicate logging, duplicate catch, wildcard import, shadowing import, repetitive yaml functions |

**Total Pass 4: 18 issues**
