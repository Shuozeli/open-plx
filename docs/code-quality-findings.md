# Code Quality Findings

## Severity Legend
- **HIGH**: Bugs, panics in prod code, silent data loss
- **MEDIUM**: Duplication, missing abstractions, unused code
- **LOW**: Cosmetic, noise, minor style issues

---

## Review Pass 1 (Previous)

All findings from the first review pass are documented below. Items marked FIXED
were resolved in commit `e21a817`.

## 1. Silent Failures (HIGH)

### 1.1 Unknown `principal_type` silently ignored in permission check -- FIXED (pass 1)
### 1.2 `parse_date_granularity` error silently swallowed -- FIXED (pass 1)
### 1.3 `text_to_proto` silently defaults unknown text format -- FIXED (pass 1)
### 1.4 `conditional_format_to_proto` silently defaults -- FIXED (pass 1)
### 1.5 `total_config_to_proto` silently defaults unknown aggregation -- FIXED (pass 1)
### 1.6 `funnel_to_proto` silently defaults unknown funnel shape -- FIXED (pass 1)
### 1.7 `table_to_proto` silently defaults unknown column alignment -- FIXED (pass 1)
### 1.8 Flight service lacks permission checking -- FIXED (pass 1)

---

## 2. Stringly-Typed APIs (MEDIUM) -- SKIPPED (pass 1)

### 2.1-2.5 Various string-typed enums in YAML model
- `principal_type`, `role`, `widget_type`, `chart_type`, `arrow_type`
- All have fail-fast parsers. Enum conversion deferred as a model-layer refactor.

---

## 3. Duplication (MEDIUM)

### 3.1 `static_column_to_arrow` and `static_column_to_proto` duplicate type-dispatch -- SKIPPED (pass 1)
### 3.2 `FieldMeta` construction duplicated -- FIXED (pass 1)
### 3.3 Integration test connection boilerplate copy-pasted -- FIXED (pass 1)

---

## 4. Missing Abstractions (MEDIUM) -- SKIPPED (pass 1)

### 4.1 Enum string parsing should use a macro
### 4.2 `WidgetSpecYaml` should use a tagged enum

---

## 5. Placeholder / Stub Code (MEDIUM)

### 5.1 OidcAuth is a dead stub -- FIXED (pass 1)
### 5.2 DataSourceRef params not converted -- SKIPPED (pass 1)
### 5.3 FlightSql auth config not converted from YAML -- SKIPPED (pass 1)

---

## 6-10. Previous categories -- See pass 1 notes above

---

## Review Pass 2 (Current)

### 11. Duplicate Data Source Resolution (MEDIUM)

#### 11.1 `get_flight_info` calls `resolve_data_source_name` twice -- FIXED (pass 2)
- **Location:** `crates/open-plx-server/src/services/flight.rs`, lines 60 and 80
- **Problem:** `resolve_data_source_name` is called once for permission checking, then
  called again inside the block that fetches data. The second call is redundant since
  `ds_name` is already in scope.
- **Fix:** Reuse the `ds_name` variable from the first call.

#### 11.2 `do_get` calls `resolve_data_source_name` twice -- FIXED (pass 2)
- **Location:** `crates/open-plx-server/src/services/flight.rs`, lines 121 and 141
- **Problem:** Same pattern as 11.1.
- **Fix:** Reuse the `ds_name` variable from the first call.

### 12. Formatting (LOW)

#### 12.1 Multiple files fail `cargo fmt --check` -- FIXED (pass 2)
- **Location:** `convert.rs`, `flight_sql_client.rs`, `flight.rs`, `flight_sql_integration.rs`
- **Fix:** Run `cargo fmt`.

---

## Summary

| Category | Total | FIXED | SKIPPED | N/A |
|---|---|---|---|---|
| Silent Failures (HIGH) | 8 | 8 | 0 | 0 |
| Stringly-Typed APIs (MEDIUM) | 5 | 0 | 5 | 0 |
| Duplication (MEDIUM) | 5 | 4 | 1 | 0 |
| Missing Abstractions (MEDIUM) | 2 | 0 | 2 | 0 |
| Placeholder / Stub Code (MEDIUM) | 3 | 1 | 2 | 0 |
| Unsafe Patterns (MEDIUM) | 2 | 0 | 1 | 1 |
| Missing Permission Checks (MEDIUM) | 2 | 2 | 0 | 0 |
| Noise (LOW) | 1 | 0 | 1 | 0 |
| Incomplete Type Coverage (LOW) | 2 | 2 | 0 | 0 |
| Over-Architecture (LOW) | 1 | 0 | 0 | 1 |
| Formatting (LOW) | 1 | 1 | 0 | 0 |
| **Total** | **32** | **18** | **12** | **2** |

### Remaining Priority Items
1. **Stringly-typed enums** (2.1-2.5) -- model-layer refactor to catch invalid YAML at parse time
2. **parse_enum! macro** (4.1) -- reduce boilerplate in convert.rs enum parsers
3. **DataSourceRef params** (5.2) -- functional gap for parameterized queries
4. **FlightSql auth proto conversion** (5.3) -- `GetDataSource` RPC missing auth info
