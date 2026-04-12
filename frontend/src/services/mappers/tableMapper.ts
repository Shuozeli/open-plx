/**
 * Table mapper: translates TableSpec proto -> S2 TableSheet config.
 */

import type { TableSpec } from "../../gen/open_plx/v1/widget_spec_pb.js";
import { SortDirection } from "../../gen/open_plx/v1/widget_spec_pb.js";
import { conditionsProtoToS2 } from "./conditionMapper.js";

/** S2 sort parameter. */
export interface S2SortParam {
  sortFieldId: string;
  sortMethod?: "ASC" | "DESC" | "NONE" | "asc" | "desc" | "none";
}

/** S2 filter parameter. */
export interface S2FilterParam {
  filterKey: string;
  filteredValues?: unknown[];
  customFilter?: (row: Record<string, unknown>) => boolean;
}

/** S2 TableSheet data config. */
export interface S2TableDataConfig {
  data: Record<string, unknown>[];
  fields: { columns: string[]; rowId?: string };
  meta?: { field: string; name?: string; formatter?: (v: unknown) => string }[];
  sortParams?: S2SortParam[];
  filterParams?: S2FilterParam[];
}

/** S2 TableSheet options. */
export interface S2TableOptions {
  width: number;
  height: number;
  showSeriesNumber?: boolean;
  pagination?: {
    pageSize: number;
    current: number;
    showTotalCount?: boolean;
    pageSizeSelector?: number[];
  };
  interaction?: Record<string, unknown>;
  conditions?: unknown;
  // Search config: S2 TableSheet doesn't have built-in search toolbar.
  // Frontend should render a search input and pass search text via data filtering.
  search?: {
    enabled: boolean;
    placeholder?: string;
    caseSensitive?: boolean;
    searchFields?: string[];
  };
  [key: string]: unknown;
}

/** Link column info extracted from renderer config, for click handling. */
export interface LinkColumnInfo {
  field: string;
  urlTemplate: string;
  newTab: boolean;
}

/** ColSpan config extracted from TableSpec, for S2Table to handle. */
export interface ColSpanConfig {
  field: string;
  condition: string;
  colSpan: number;
}

/** Action column info extracted from TableSpec, for row-action buttons. */
export interface ActionColumnInfo {
  field: string;
  action: {
    id: string;
    label: string;
    icon?: string;
    style?: string;
    confirmMessage?: string;
    grpcCall?: {
      method: string;
      requestTemplate: string;
      resultHandling?: number;
    };
  };
}

/**
 * Build a hierarchical tree structure from flat data for S2 tree table.
 */
function buildHierarchy(
  flatData: Record<string, unknown>[],
  hierarchyFields: string[],
  rowIdField?: string,
): Record<string, unknown>[] {
  if (!hierarchyFields.length || !flatData.length) return flatData;

  // Group by the first hierarchy field
  const field = hierarchyFields[0];
  const groups = new Map<unknown, Record<string, unknown>[]>();

  for (const row of flatData) {
    const key = row[field];
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(row);
  }

  // Build tree nodes
  type TreeNode = Record<string, unknown>;
  const result: TreeNode[] = [];
  for (const [key, rows] of groups) {
    const node: TreeNode = {
      [field]: key,
      rowId: rowIdField ? rows[0][rowIdField] : key,
    };

    // Recurse for child levels
    if (hierarchyFields.length > 1) {
      const childField = hierarchyFields[1];
      const childGroups = new Map<unknown, Record<string, unknown>[]>();
      for (const row of rows) {
        const childKey = row[childField];
        if (!childGroups.has(childKey)) childGroups.set(childKey, []);
        childGroups.get(childKey)!.push(row);
      }
      node.children = [] as TreeNode[];
      for (const [childKey, childRows] of childGroups) {
        (node.children as TreeNode[]).push({
          [childField]: childKey,
          rowId: rowIdField ? childRows[0][rowIdField] : childKey,
          children: childRows,
        });
      }
    } else {
      node.children = rows;
    }

    result.push(node);
  }

  return result;
}

/** Convert a TableSpec proto + row data to S2 TableSheet config. */
export function tableProtoToS2(
  proto: TableSpec,
  data: Record<string, unknown>[],
  containerWidth: number,
  containerHeight: number,
): { dataCfg: S2TableDataConfig; options: S2TableOptions; linkColumn?: LinkColumnInfo; colSpans?: ColSpanConfig[]; actionColumns?: ActionColumnInfo[] } {
  // Start with all columns from proto
  let columns = proto.columns;

  // Hidden columns: exclude from display
  const visibleColumns = columns.filter((c) => !c.hidden);

  // Column order: if any column has explicit order, sort by order
  const orderedColumns = [...visibleColumns];
  if (orderedColumns.some((c) => c.order !== 0)) {
    orderedColumns.sort((a, b) => a.order - b.order);
  }

  // Determine column fields: use proto columns in order, else infer from data
  let columnFields: string[];
  if (orderedColumns.length > 0) {
    columnFields = orderedColumns.map((c) => c.field);
  } else if (data.length > 0) {
    columnFields = Object.keys(data[0]);
  } else {
    columnFields = [];
  }

  const dataCfg: S2TableDataConfig = {
    data,
    fields: { columns: columnFields },
  };

  // Field meta (display names + formatters)
  if (proto.meta.length > 0) {
    dataCfg.meta = proto.meta.map((m) => ({
      field: m.field,
      name: m.name || undefined,
      formatter: m.formatter ? buildFormatter(m.formatter) : undefined,
    }));
  }

  const options: S2TableOptions = {
    width: containerWidth,
    height: containerHeight,
    showSeriesNumber: proto.showRowNumbers,
    interaction: proto.interaction
      ? {
          hoverHighlight: proto.interaction.enableHoverHighlight,
          copy: { enable: proto.interaction.enableCopy },
          resize: proto.interaction.enableResize,
          multiSelection: proto.interaction.enableMultiSelection,
          rangeSelection: proto.interaction.enableRangeSelection,
        }
      : {
          hoverHighlight: true,
          copy: { enable: true },
        },
  };

  // Column drag: enable resize.Column for drag-to-reorder
  if (proto.interaction?.enableColumnDrag) {
    options.interaction = options.interaction || {};
    (options.interaction as Record<string, unknown>)["resize"] = {
      ...((options.interaction as Record<string, unknown>)["resize"] as Record<string, unknown>),
      Column: true,
    };
  }

  // Frozen columns (left and right)
  if (proto.frozenColsLeft?.length || proto.frozenColsRight?.length) {
    options.frozenCols = [
      ...(proto.frozenColsLeft || []),
      ...(proto.frozenColsRight || []),
    ];
    options.frozenRowHeader = true;
  }

  // Conditional formatting
  if (proto.conditions.length > 0) {
    options.conditions = conditionsProtoToS2(proto.conditions);
  }

  // Cell renderers: icon, bar, link, progress
  let linkColumn: LinkColumnInfo | undefined;
  for (const col of proto.columns) {
    if (!col.renderer) continue;
    // Skip columns that have an action (action columns are handled separately)
    if (col.action) continue;

    // Initialize conditions object if needed
    if (!options.conditions) {
      options.conditions = {};
    }
    const cond = options.conditions as Record<string, unknown[]>;

    if (col.renderer.renderer.case === "icon" && col.renderer.renderer.value) {
      const iconRenderer = col.renderer.renderer.value;
      const iconMap = iconRenderer.valueToIcon;
      if (!cond["icon"]) cond["icon"] = [];
      cond["icon"].push({
        field: col.field,
        mapping: (value: unknown) => {
          const iconName = iconMap[String(value)] || iconRenderer.fallbackIcon;
          return { fill: "#000000", icon: iconName || "" };
        },
      });
    }

    if (col.renderer.renderer.case === "bar" && col.renderer.renderer.value) {
      const barRenderer = col.renderer.renderer.value;
      const valueField = barRenderer.valueField || col.field;
      // Compute max from data if not specified
      const maxValue = barRenderer.maxValue || Math.max(...data.map((r) => Number(r[valueField]) || 0), 1);
      if (!cond["interval"]) cond["interval"] = [];
      cond["interval"].push({
        field: col.field,
        mapping: (value: unknown) => {
          const numValue = typeof value === "number" ? value : Number(value) || 0;
          const ratio = numValue / maxValue;
          return {
            fill: barRenderer.color || "#1781f2",
            isCompare: true,
            minValue: 0,
            maxValue: 1,
            ratio,
          };
        },
      });
    }

    if (col.renderer.renderer.case === "link" && col.renderer.renderer.value) {
      const linkRenderer = col.renderer.renderer.value;
      if (!cond["text"]) cond["text"] = [];
      cond["text"].push({
        field: col.field,
        mapping: () => ({ fill: "#1890ff" }),
      });
      // Track link column for click handling
      linkColumn = {
        field: col.field,
        urlTemplate: linkRenderer.urlTemplate,
        newTab: linkRenderer.newTab,
      };
    }

    if (col.renderer.renderer.case === "progress" && col.renderer.renderer.value) {
      const progressRenderer = col.renderer.renderer.value;
      const totalField = progressRenderer.totalField;
      // Compute max from data if not specified and no totalField
      const maxValue = progressRenderer.maxValue || 100;
      if (!cond["interval"]) cond["interval"] = [];
      cond["interval"].push({
        field: col.field,
        mapping: (value: unknown, record: Record<string, unknown>) => {
          const current = typeof value === "number" ? value : Number(value) || 0;
          const total = totalField ? (Number(record[totalField]) || 0) : maxValue;
          const ratio = total > 0 ? current / total : 0;
          const fill = ratio > 0.8 ? "#52c41a" : ratio > 0.5 ? "#faad14" : "#ff4d4f";
          return { fill, isCompare: true, minValue: 0, maxValue: 1 };
        },
      });
    }
  }

  if (proto.pagination) {
    options.pagination = {
      pageSize: proto.pagination.pageSize,
      current: 1,
    };
  }

  // Server-side pagination: frontend requests one page at a time from the backend
  if (proto.serverPagination) {
    options.pagination = {
      pageSize: proto.serverPagination.pageSize || 20,
      current: 1,
      showTotalCount: proto.serverPagination.showTotalCount,
      pageSizeSelector: proto.serverPagination.showPageSizeSelector
        ? [10, 25, 50, 100]
        : undefined,
    };
  }

  // Sort params: columns with sortable: true + default sort
  const sortParams: S2SortParam[] = [];
  for (const col of proto.columns) {
    if (col.sortable) {
      sortParams.push({ sortFieldId: col.field });
    }
  }
  // Default sort (applied on initial load)
  if (proto.defaultSort?.field) {
    sortParams.unshift({
      sortFieldId: proto.defaultSort.field,
      sortMethod: proto.defaultSort.direction === SortDirection.DESC ? "DESC" : "ASC",
    });
  }
  if (sortParams.length > 0) {
    dataCfg.sortParams = sortParams;
  }

  // Filter params: columns with filterable: true
  // Note: S2 TableSheet doesn't have a built-in search toolbar like PivotSheet.
  // Search (view.enableSearch) would need to be implemented via custom filter UI.
  const filterParams: S2FilterParam[] = [];
  for (const col of proto.columns) {
    if (col.filterable && col.filter) {
      const filterParam: S2FilterParam = { filterKey: col.field };
      if (col.filter.filterValues && col.filter.filterValues.length > 0) {
        filterParam.filteredValues = col.filter.filterValues;
      }
      // TABLE_FILTER_TYPE_TEXT and TABLE_FILTER_TYPE_RANGE would require customFilter
      // which is not serializable over gRPC - noted as a limitation
      filterParams.push(filterParam);
    }
  }
  if (filterParams.length > 0) {
    dataCfg.filterParams = filterParams;
  }

  // Search config: S2 TableSheet doesn't have built-in search toolbar.
  // The frontend should render a search input and pass search text via filterParams.
  if (proto.view?.enableSearch) {
    options.search = {
      enabled: true,
      placeholder: proto.view.searchPlaceholder || undefined,
      caseSensitive: proto.view.caseSensitive || false,
      searchFields: proto.view.searchFields.length > 0 ? [...proto.view.searchFields] : undefined,
    };
  }

  // Row selection
  if (proto.selection?.enabled) {
    options.rowSelection = {
      strict: true,
      onlySelected: proto.selection.single,
      showSelectedIcon: true,
      persist: proto.selection.persistent,
    };
  }

  // Export config
  if (proto.export && (proto.export.enableCsv || proto.export.enableExcel)) {
    options.export = {
      filename: proto.export.filenameTemplate || "export",
      formats: [
        ...(proto.export.enableCsv ? ["csv"] : []),
        ...(proto.export.enableExcel ? ["xlsx"] : []),
      ] as ("csv" | "xlsx")[],
    };
  }

  // Expandable / tree table
  if (proto.expandable?.enabled) {
    // Set row ID field for expansion tracking
    if (proto.expandable.rowIdField) {
      dataCfg.fields.rowId = proto.expandable.rowIdField;
    }

    // Build hierarchy from flat data using hierarchy_fields
    if (proto.expandable.hierarchyFields?.length && data.length > 0) {
      const hierarchyData = buildHierarchy(
        data,
        [...proto.expandable.hierarchyFields],
        proto.expandable.rowIdField || undefined,
      );
      dataCfg.data = hierarchyData;
    }

    // Set hierarchy collapse state (true = collapsed by default)
    options.hierarchyCollapse = !proto.expandable.defaultExpanded;
  }

  // Action columns: extract row-action button definitions
  const actionColumns: ActionColumnInfo[] = [];
  for (const col of proto.columns) {
    if (col.action) {
      actionColumns.push({
        field: col.field,
        action: {
          id: col.action.id,
          label: col.action.label,
          icon: col.action.icon || undefined,
          style: col.action.style !== 0 ? actionStyleToString(col.action.style) : undefined,
          confirmMessage: col.action.confirmMessage || undefined,
          grpcCall: col.action.grpcCall ? {
            method: col.action.grpcCall.method,
            requestTemplate: col.action.grpcCall.requestTemplate,
            resultHandling: col.action.grpcCall.resultHandling !== 0 ? col.action.grpcCall.resultHandling : undefined,
          } : undefined,
        },
      });
    }
  }

  // Cell spanning: store for S2Table to handle
  const colSpans = proto.colSpans?.length
    ? proto.colSpans.map((cs) => ({
        field: cs.field,
        condition: cs.condition,
        colSpan: cs.colSpan,
      }))
    : undefined;

  return { dataCfg, options, linkColumn, colSpans, actionColumns: actionColumns.length > 0 ? actionColumns : undefined };
}

/** Convert ActionStyle enum to string. */
function actionStyleToString(style: number): string {
  // ActionStyle values: 0=Unspecified, 1=Primary, 2=Secondary, 3=Danger, 4=Link
  switch (style) {
    case 1: return "primary";
    case 2: return "secondary";
    case 3: return "danger";
    case 4: return "link";
    default: return "secondary";
  }
}

/** Build a formatter function from a format string. */
function buildFormatter(fmt: string): (v: unknown) => string {
  if (fmt.startsWith("number:")) {
    const decimals = parseInt(fmt.split(":")[1], 10) || 0;
    return (v: unknown) => {
      if (typeof v === "number") return v.toFixed(decimals);
      return String(v ?? "");
    };
  }
  if (fmt === "compact") {
    return (v: unknown) => {
      if (typeof v !== "number") return String(v ?? "");
      if (Math.abs(v) >= 1_000_000) return `${(v / 1_000_000).toFixed(1)}M`;
      if (Math.abs(v) >= 1_000) return `${(v / 1_000).toFixed(1)}K`;
      return v.toFixed(0);
    };
  }
  if (fmt.startsWith("currency:")) {
    const currency = fmt.split(":")[1] || "USD";
    return (v: unknown) => {
      if (typeof v !== "number") return String(v ?? "");
      return new Intl.NumberFormat("en-US", { style: "currency", currency }).format(v);
    };
  }
  return (v: unknown) => String(v ?? "");
}

/** Extract test state from table spec. */
export function tableProtoToTestState(
  proto: TableSpec,
  data: Record<string, unknown>[],
): Record<string, unknown> {
  return {
    columnCount: proto.columns.length || (data.length > 0 ? Object.keys(data[0]).length : 0),
    rowCount: data.length,
    hasPagination: !!proto.pagination,
    showRowNumbers: proto.showRowNumbers,
  };
}
