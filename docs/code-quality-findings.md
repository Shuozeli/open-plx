# Code Quality Findings

## Severity Legend
- **HIGH**: Bugs, panics in prod code, silent data loss
- **MEDIUM**: Duplication, missing abstractions, unused code
- **LOW**: Cosmetic, noise, minor style issues

---

## 1. Silent Failures (HIGH)

### 1.1 Unknown `principal_type` silently ignored in permission check -- FIXED
- **Location:** `crates/open-plx-auth/src/lib.rs` (check_permission)
- **Fix applied:** Replaced `_ => {}` catch-all with `other => return Err(Status::internal(...))` that returns an error on unknown principal types.

### 1.2 `parse_date_granularity` error silently swallowed in variable conversion -- FIXED
- **Location:** `crates/open-plx-config/src/convert.rs` (DatePicker and DateRange controls)
- **Fix applied:** Changed `.and_then(|g| parse_date_granularity(g).ok())` to `.map(parse_date_granularity).transpose()?` so errors propagate. `variable_to_proto` and `variable_control_to_proto` now return `Result`.

### 1.3 `text_to_proto` silently defaults unknown text format -- FIXED
- **Location:** `crates/open-plx-config/src/convert.rs` (text_to_proto)
- **Fix applied:** `text_to_proto` now returns `Result` and uses `bail!("unknown text format: '{other}'")` for unrecognized format strings.

### 1.4 `conditional_format_to_proto` silently defaults unknown format types and comparison ops -- FIXED
- **Location:** `crates/open-plx-config/src/convert.rs` (conditional_format_to_proto)
- **Fix applied:** Function now returns `Result`. Both `format_type` and `op` match arms use `bail!` on unknown values instead of defaulting to `Unspecified`.

### 1.5 `total_config_to_proto` silently defaults unknown aggregation -- FIXED
- **Location:** `crates/open-plx-config/src/convert.rs` (total_config_to_proto)
- **Fix applied:** Function now returns `Result` and uses `bail!("unknown aggregation: '{other}'")`. Callers use `.transpose()?` to propagate.

### 1.6 `funnel_to_proto` silently defaults unknown funnel shape -- FIXED
- **Location:** `crates/open-plx-config/src/convert.rs` (funnel_to_proto)
- **Fix applied:** Function now returns `Result` and uses `bail!("unknown funnel shape: '{other}'")`.

### 1.7 `table_to_proto` silently defaults unknown column alignment -- FIXED
- **Location:** `crates/open-plx-config/src/convert.rs` (table_to_proto)
- **Fix applied:** Function now returns `Result`. Column alignment match uses `bail!` on unknown values.

### 1.8 Flight service lacks permission checking -- FIXED
- **Location:** `crates/open-plx-server/src/services/flight.rs` and `crates/open-plx-server/src/main.rs`
- **Fix applied:** Auth interceptor wired to `FlightServiceServer::with_interceptor`. Both `get_flight_info` and `do_get` now extract the principal and call `check_permission` before executing queries.

---

## 2. Stringly-Typed APIs (MEDIUM)

### 2.1 `principal_type` is a raw string instead of an enum -- SKIPPED
- **Reason:** Would require changing the YAML schema and all permission-related deserialization. The fail-fast fix in 1.1 mitigates the silent-failure risk. Enum conversion is a larger refactor best done alongside a schema migration.

### 2.2 `role` field is a raw string instead of an enum -- SKIPPED
- **Reason:** Same as 2.1. `role_to_level` already fails fast on invalid values. Enum conversion is deferred.

### 2.3 `widget_type` field in YAML is a raw string -- SKIPPED
- **Reason:** `parse_widget_type` already fails with `bail!` on unknown values. Converting to a serde enum is a model-layer refactor with no immediate correctness benefit.

### 2.4 `chart_type` field in YAML is a raw string -- SKIPPED
- **Reason:** Same as 2.3. `parse_chart_type` already fails fast.

### 2.5 `arrow_type` field in static columns is a raw string -- SKIPPED
- **Reason:** The type coverage gap between static_data.rs and convert.rs was addressed (see 9.1), but the string-to-enum conversion is deferred. A dedicated `ArrowTypeYaml` enum is a model-layer change.

---

## 3. Duplication (MEDIUM)

### 3.1 `static_column_to_arrow` and `static_column_to_proto` duplicate type-dispatch logic -- SKIPPED
- **Reason:** The type coverage gap was closed (9.1), but the two functions were not unified. Centralizing requires an `ArrowTypeYaml` enum (see 2.5) as a prerequisite.

### 3.2 `FieldMeta` construction is duplicated -- FIXED
- **Location:** `crates/open-plx-config/src/convert.rs`
- **Fix applied:** Extracted `field_meta_to_proto(m: &FieldMetaYaml) -> pb::FieldMeta` helper. Both `pivot_to_proto` and `table_to_proto` now call this shared function.

### 3.3 Integration test connection boilerplate is copy-pasted across backends -- FIXED
- **Location:** `crates/open-plx-server/tests/flight_sql_integration.rs`
- **Fix applied:** Tests were rewritten using ADBC drivers. Each backend module (duckdb, postgres, mysql) has its own `connect()` helper. Shared `collect()` helper extracted at module top.

---

## 4. Missing Abstractions (MEDIUM)

### 4.1 Enum string parsing should use a macro -- SKIPPED
- **Reason:** All 11 `parse_*` functions now correctly fail on unknown values. The macro consolidation is a readability improvement, not a correctness fix. Deferred to a dedicated cleanup pass.

### 4.2 `WidgetSpecYaml` should use a tagged enum instead of 10 Option fields -- SKIPPED
- **Reason:** Structural change to the model layer. The current first-match-wins behavior works correctly for well-formed configs. Deferred.

---

## 5. Placeholder / Stub Code (MEDIUM)

### 5.1 OidcAuth is a dead stub -- FIXED
- **Location:** `crates/open-plx-auth/src/lib.rs`
- **Fix applied:** Server now panics at startup if `provider: oidc` is configured, with a clear message: "OIDC auth is not yet implemented -- use 'dev' or 'api_key' mode". The struct is annotated `#[allow(dead_code)]` and documented as unimplemented.

### 5.2 DataSourceRef params are not converted -- SKIPPED
- **Reason:** The TODO remains in the code. Implementing param conversion requires defining `ParamValue` proto conversion logic. Tracked as a feature gap, not a quality fix.

### 5.3 FlightSql auth config not converted from YAML -- SKIPPED
- **Reason:** The TODO remains. The ADBC rewrite passes credentials via `DatabaseOption::Username`/`Password` at the driver level, so the proto conversion gap is less critical than before. Still a functional gap for `GetDataSource` RPC responses.

---

## 6. Unsafe Patterns (MEDIUM)

### 6.1 `expect()` calls in shutdown_signal -- SKIPPED
- **Reason:** Low priority. Signal handler installation failure is extremely rare. Not addressed in this pass.

### 6.2 `unwrap()` in integration test helper (acceptable) -- NO ACTION NEEDED
- Acceptable in test code.

---

## 7. Missing Permission Checks (MEDIUM)

### 7.1 `list_data_sources` has no permission check -- FIXED
- **Location:** `crates/open-plx-server/src/services/data_source.rs`
- **Fix applied:** Extracts principal via `get_principal`, calls `check_permission` with "reader" role on each data source, and filters the response. Includes structured event logging.

### 7.2 `get_data_source` has no permission check -- FIXED
- **Location:** `crates/open-plx-server/src/services/data_source.rs`
- **Fix applied:** Extracts principal, checks "reader" permission. Returns `not_found` (not `permission_denied`) to avoid information disclosure. Includes event logging.

---

## 8. Noise / Excessive Section Dividers (LOW)

### 8.1 Heavy section divider comments throughout -- SKIPPED
- **Reason:** Cosmetic. Not addressed in this pass.

---

## 9. Incomplete Type Coverage (LOW)

### 9.1 `static_column_to_arrow` supports fewer types than `static_column_to_proto` -- FIXED
- **Location:** `crates/open-plx-config/src/static_data.rs`
- **Fix applied:** Added `boolean`, `date32`, and `timestamp_micros` support to `static_column_to_arrow`. Boolean values use `yaml_value_to_bool`. Date/timestamp values are stored as Utf8 strings in Arrow, consistent with proto conversion.

### 9.2 `UInt64` not handled in `record_batch_to_columns` -- FIXED
- **Location:** `crates/open-plx-server/src/services/widget_data.rs`
- **Fix applied:** Added `DataType::UInt64` to the integer cast arm. Values are cast to Int64 (with a doc comment noting overflow behavior for values > i64::MAX).

---

## 10. Over-Architecture (LOW)

### 10.1 `open-plx-server/src/lib.rs` exists only to re-export one module -- NO ACTION NEEDED
- Standard Rust pattern for testing binary crate internals.

---

## Summary

| Category | Total | FIXED | SKIPPED | N/A |
|---|---|---|---|---|
| Silent Failures (HIGH) | 8 | 8 | 0 | 0 |
| Stringly-Typed APIs (MEDIUM) | 5 | 0 | 5 | 0 |
| Duplication (MEDIUM) | 3 | 2 | 1 | 0 |
| Missing Abstractions (MEDIUM) | 2 | 0 | 2 | 0 |
| Placeholder / Stub Code (MEDIUM) | 3 | 1 | 2 | 0 |
| Unsafe Patterns (MEDIUM) | 2 | 0 | 1 | 1 |
| Missing Permission Checks (MEDIUM) | 2 | 2 | 0 | 0 |
| Noise (LOW) | 1 | 0 | 1 | 0 |
| Incomplete Type Coverage (LOW) | 2 | 2 | 0 | 0 |
| Over-Architecture (LOW) | 1 | 0 | 0 | 1 |
| **Total** | **29** | **15** | **12** | **2** |

### Remaining Priority Items
1. **Stringly-typed enums** (2.1-2.5) -- model-layer refactor to catch invalid YAML at parse time
2. **parse_enum! macro** (4.1) -- reduce boilerplate in convert.rs enum parsers
3. **DataSourceRef params** (5.2) -- functional gap for parameterized queries
4. **FlightSql auth proto conversion** (5.3) -- `GetDataSource` RPC missing auth info
