# Table Enhancement Implementation Review

**Review Date:** 2026-04-11
**Reviewer:** Claude Code Agent
**Scope:** Phases A-J changes across proto, Rust backend, and frontend

## Verification

- `cargo check --workspace` - **PASSES**
- `pnpm tsc --noEmit` - **PASSES**

---

## Overall Assessment

The implementation is largely well-structured with proper proto definitions, good TypeScript typing, and consistent patterns. However, there are several **stub implementations** that need completion before production use, plus a few code quality issues.

---

## Proto Files

### `proto/open_plx/v1/action.proto`

**Status:** Correct and well-defined.

- `WidgetActionService` with `InvokeAction` RPC method
- `InvokeActionRequest` fields 1-4: `dashboard_name`, `widget_id`, `action_id`, `request_body`
- `InvokeActionResponse` fields 1-4: `success`, `message`, `variable_name`, `variable_value`
- `TableAction` fields 1-6: `id`, `label`, `icon`, `style`, `confirm_message`, `grpc_call`
- `ActionGrpcCall` fields 1-3: `method`, `request_template`, `result_handling`
- `ActionResultHandling` enum values 0-4: `UNSPECIFIED`, `SET_VARIABLE`, `REFRESH`, `TOAST`
- No proto numbering conflicts.

### `proto/open_plx/v1/widget_spec.proto`

**Status:** Correct with one notable design limitation.

- `GraphSpec` added at field 11 - field numbering is sequential and correct.
- `GraphLayoutType` enum values 0-5 match frontend usage.
- `GraphDataMapping` fields 1-5: `source_field`, `target_field`, `value_field`, `label_field`, `group_field`
- `GraphLayout` fields 1-5: `type`, `iterations`, `direction`, `node_spacing`, `rank_spacing`
- `GraphNodeStyle` fields 1-6: `size`, `color`, `show_label`, `label_font_size`, `border_color`, `border_width`
- `GraphEdgeStyle` fields 1-5: `color`, `width`, `style`, `show_arrow`, `arrow_size`
- `GraphInteraction` fields 1-5: `enable_drag`, `enable_zoom`, `enable_click_select`, `enable_tooltip`, `enable_edge_click`
- All field numbers unique and sequential within each message.

### `proto/open_plx/v1/data_source.proto`

**Status:** Correct.

- `GrpcProxyConfig` fields 1-5: `service`, `method`, `request_template`, `response_schema`, `endpoint`
- `ResponseSchema` and `ColumnSchema` properly defined.
- `DataType` enum values 0-5 correctly numbered.

---

## Rust Backend

### `crates/open-plx-config/src/model.rs`

**Status:** Correct.

- All YAML structs properly defined with `#[serde]` annotations.
- `GraphSpecYaml` and all sub-structs correctly defined.

### `crates/open-plx-config/src/convert.rs`

**Status:** Mostly correct, with **TODOs requiring attention**.

**Issues:**

1. **Line 66** - `TODO(refactor): Convert typed params` - `data_source.params` is not being converted; HashMap is passed empty.

2. **Line 1069** - `TODO(refactor): Convert auth from YAML` - `auth` field is ignored in FlightSqlConfig conversion.

### `crates/open-plx-server/src/services/widget_action.rs`

**Status:** Stub implementation - not production-ready.

**Issues (Blocking):**

1. **Line 157-164** - `forward_grpc_call` is completely stubbed:
   ```rust
   async fn forward_grpc_call(method: &str, _request_body: &str) -> Result<String, Status> {
       tracing::debug!(method = %method, "upstream gRPC call stubbed -- returning success");
       Ok(r#"{"success": true}"#.to_string())
   }
   ```
   - The actual gRPC call to upstream services is not implemented.
   - Returns a hardcoded success JSON regardless of the method or request body.
   - All row actions will appear to succeed but do nothing.

2. **Lines 130-143** - `extract_json_string` is a naive string parser:
   - Simple string search instead of proper JSON parsing.
   - Does not handle escaped quotes, nested objects, or whitespace variations.
   - Will fail on many valid JSON responses.

### `crates/open-plx-server/src/grpc_proxy_client.rs`

**Status:** Stub implementation - not production-ready.

**Issues (Blocking):**

1. **Lines 34-56** - `fetch` is completely stubbed:
   ```rust
   let _ = (channel, request);
   let schema = self.infer_or_use_schema(&config.response_schema)?;
   let batch = RecordBatch::new_empty(schema);
   Ok(batch)
   ```
   - The actual gRPC call is not implemented.
   - Always returns an empty RecordBatch with the inferred schema.
   - GrpcProxyConfig data sources will always return empty data.

2. **Lines 59-73** - Channel pooling is implemented but never used.

### `crates/open-plx-server/src/state.rs`

**Status:** Mostly correct.

**Issues (Warning):**

1. **Lines 28-32** - `endpoint_resolver` defaults to hardcoded `http://127.0.0.1:50051`:
   ```rust
   let endpoint_resolver = Arc::new(|_service_name: &str| {
       "http://127.0.0.1:50051".to_string()
   });
   ```
   - Not configurable - service discovery is not implemented.
   - Will only work if upstream services are on the same host.

2. **Lines 99-145** - `execute_data_source` builds `GrpcProxyConfig` inline:
   - Data type mapping uses magic numbers (1, 2, 3, 4, 5) instead of enum constants.
   - Could use `DataType::String as i32`, etc.

---

## Frontend

### `frontend/src/services/mappers/tableMapper.ts`

**Status:** Mostly correct.

**Issues (Warning):**

1. **Lines 89-140** - `buildHierarchy` function:
   - Nodes are only created on first occurrence; subsequent occurrences don't update labels.
   - If a node's label is in a different field than the source/target, this works correctly.
   - If the same node appears with different label values in different rows, only the first is used.

2. **Line 581** - `tableSpec.columns` access assumes the proto structure:
   - Code reads `proto.columns` directly, but `columns` is optional in the proto and the generated TS type uses `TableColumn[]` which may not match.

### `frontend/src/components/widgets/S2Table.tsx`

**Status:** Acceptable with minor issues.

**Issues (Minor):**

1. **Lines 63-75** - Export functions use `as any` cast:
   ```typescript
   (sheetRef.current as any).exportFile("csv", exportOptions?.filename || "export");
   ```
   - These S2 export methods are not in the type definitions.
   - Consider creating a typed wrapper or filing a type issue with @antv/s2.

2. **Lines 95-101** - ColSpan handling is noted as incomplete:
   ```typescript
   // TODO(refactor): S2 colSpan API - cell spanning in TableSheet requires custom cell layout
   ```
   - ColSpan is stored but not actually applied to the S2 table.

3. **Lines 114-124** - Action invocation template interpolation:
   ```typescript
   const requestBody = action.grpcCall?.requestTemplate
     .replace(/\{row\.(\w+)\}/g, (_, field) => String(rowData[field] ?? ""))
   ```
   - Simple regex-based interpolation; does not handle edge cases like escaped braces or nested objects.

### `frontend/src/components/widgets/TableWidget.tsx`

**Status:** Correct.

**Issues (Minor):**

1. **Lines 69-72** - SET_VARIABLE result handling is not wired:
   ```typescript
   // TODO(refactor): Set dashboard variable - need to integrate with variable context
   console.log("Action set variable:", response.variableName, "=", response.variableValue);
   ```
   - Action responses that set variables don't actually update dashboard variables.
   - The variable context integration is missing.

### `frontend/src/components/widgets/GraphWidget.tsx`

**Status:** Acceptable with one design limitation.

**Issues (Warning):**

1. **Node label issue** - Labels are extracted from edge data rows, not from a dedicated node table:
   - When a node appears in multiple edges, the label comes from whichever row created the node first.
   - If labelField doesn't exist in the row, falls back to node ID.
   - This is a data model limitation, not necessarily a bug.

### `frontend/src/services/mappers/graphMapper.ts`

**Status:** Mostly correct, with one design limitation.

**Issues (Warning):**

1. **Lines 33-75** - Node label extraction:
   ```typescript
   labelText: spec.nodeStyle?.showLabel
     ? (dm.labelField ? String(row[dm.labelField] ?? sourceId) : sourceId)
   ```
   - The label is taken from the current row being processed.
   - If a node appears multiple times with different label values, the first occurrence wins.

2. **Lines 108-133** - Layout type comparison uses raw numbers:
   ```typescript
   if (layoutType === 2) { // DAGRE
   ```
   - Relies on enum numeric values matching proto definitions.
   - Could use enum constants for better maintainability.

### `frontend/src/hooks/useVariables.ts`

**Status:** Mostly correct.

**Issues (Minor):**

1. **Lines 65-71** - `getDefaultValueByName` has a stale closure risk:
   ```typescript
   const getDefaultValueByName = useCallback(
     (name: string): ParamValue | undefined => {
       const variable = variables.find((v) => v.name === name);
       return variable ? getDefaultValue(variable) : undefined;
     },
     [variables], // This is fine - variables is a prop
   );
   ```
   - Actually correct since `variables` is a prop passed to the hook.

2. **Line 153** - `syncToUrl` effect runs on every value change:
   - This is the intended behavior per the CLAUDE.md.

---

## Stub Implementations Summary

| File | Functionality | Status |
|------|---------------|--------|
| `grpc_proxy_client.rs:34` | gRPC proxy data fetching | **STUB** - returns empty RecordBatch |
| `widget_action.rs:157` | Upstream gRPC call forwarding | **STUB** - returns hardcoded success |
| `widget_action.rs:133` | JSON string extraction | **NAIVE** - simple string search, not proper JSON parsing |
| `tableMapper.ts:95` | S2 ColSpan API | **TODO** - stored but not applied |

---

## Proto Numbering Conflicts

**None found.** All field numbers are unique and sequential within each message.

---

## TODO(refactor) Markers Found

1. `convert.rs:66` - Convert typed params
2. `convert.rs:1069` - Convert auth from YAML
3. `grpc_proxy_client.rs:51` - Implement actual gRPC call forwarding
4. `widget_action.rs:154` - Implement real upstream gRPC forwarding
5. `S2Table.tsx:95` - S2 colSpan API implementation
6. `TableWidget.tsx:70` - Set dashboard variable integration

---

## Severity Classification

### Blocking (must fix before production)
1. `grpc_proxy_client.rs` - GrpcProxyConfig data sources always return empty data
2. `widget_action.rs` - Actions always succeed without doing anything

### Warning (should fix)
1. `convert.rs` - Unconverted params and auth fields
2. `graphMapper.ts` - Node label from first edge row only
3. `state.rs` - Hardcoded gRPC endpoint
4. `TableWidget.tsx` - SET_VARIABLE result not wired

### Nit (minor)
1. `S2Table.tsx` - `as any` casts for export
2. `S2Table.tsx` - Naive JSON string interpolation in action template
3. `graphMapper.ts` - Raw enum value comparisons instead of constants
