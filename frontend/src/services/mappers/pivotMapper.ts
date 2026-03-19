/**
 * Pivot table mapper: translates PivotTableSpec proto -> S2 config.
 *
 * Decoupling layer between our proto schema and AntV S2.
 * If S2 is ever replaced, only this file changes.
 */

import type { PivotTableSpec, FieldMeta } from "../../gen/open_plx/v1/widget_spec_pb.js";

export interface S2DataConfig {
  data: Record<string, unknown>[];
  fields: {
    rows: string[];
    columns: string[];
    values: string[];
    valueInCols: boolean;
  };
  meta?: S2Meta[];
  sortParams?: S2SortParam[];
}

export interface S2Meta {
  field: string;
  name?: string;
  formatter?: (value: unknown) => string;
}

export interface S2SortParam {
  sortFieldId: string;
  sortMethod?: "ASC" | "DESC";
}

export interface S2Options {
  width: number;
  height: number;
  hierarchyType?: "grid" | "tree";
  frozen?: {
    rowHeader?: boolean;
    rowCount?: number;
    colCount?: number;
  };
  seriesNumber?: {
    enable: boolean;
    text?: string;
  };
  interaction?: {
    hoverHighlight?: boolean;
    enableCopy?: boolean;
  };
}

/** Build a formatter function from a format string. */
function buildFormatter(formatStr: string): ((value: unknown) => string) | undefined {
  if (!formatStr) return undefined;

  if (formatStr.startsWith("currency:")) {
    const currency = formatStr.split(":")[1] ?? "USD";
    return (value: unknown) => {
      const num = Number(value);
      if (isNaN(num)) return String(value);
      return new Intl.NumberFormat("en-US", { style: "currency", currency }).format(num);
    };
  }

  if (formatStr.startsWith("percent")) {
    const precision = parseInt(formatStr.split(":")[1] ?? "1", 10);
    return (value: unknown) => {
      const num = Number(value);
      if (isNaN(num)) return String(value);
      return `${num.toFixed(precision)}%`;
    };
  }

  if (formatStr.startsWith("number:")) {
    const precision = parseInt(formatStr.split(":")[1] ?? "0", 10);
    return (value: unknown) => {
      const num = Number(value);
      if (isNaN(num)) return String(value);
      return num.toLocaleString("en-US", {
        minimumFractionDigits: precision,
        maximumFractionDigits: precision,
      });
    };
  }

  if (formatStr === "compact") {
    return (value: unknown) => {
      const num = Number(value);
      if (isNaN(num)) return String(value);
      return new Intl.NumberFormat("en-US", { notation: "compact" }).format(num);
    };
  }

  return undefined;
}

/** Convert a PivotTableSpec proto to S2 dataCfg + options. */
export function pivotProtoToS2(
  proto: PivotTableSpec,
  data: Record<string, unknown>[],
  containerWidth: number,
  containerHeight: number,
): { dataCfg: S2DataConfig; options: S2Options } {
  const fields = proto.fields;

  const dataCfg: S2DataConfig = {
    data,
    fields: {
      rows: fields ? [...fields.rows] : [],
      columns: fields ? [...fields.columns] : [],
      values: fields ? [...fields.values] : [],
      valueInCols: fields?.valueInCols ?? true,
    },
  };

  // Meta: field aliases and formatters
  if (proto.meta.length > 0) {
    dataCfg.meta = proto.meta.map((m: FieldMeta) => {
      const meta: S2Meta = { field: m.field };
      if (m.name) meta.name = m.name;
      if (m.formatter) meta.formatter = buildFormatter(m.formatter);
      return meta;
    });
  }

  // Sort params
  if (proto.sort.length > 0) {
    dataCfg.sortParams = proto.sort.map((s) => ({
      sortFieldId: s.sortFieldId,
      sortMethod: s.sortDirection === 1 ? "ASC" as const : "DESC" as const,
    }));
  }

  const options: S2Options = {
    width: containerWidth,
    height: containerHeight,
    interaction: {
      hoverHighlight: true,
      enableCopy: true,
    },
  };

  // Hierarchy type
  if (proto.hierarchyType === 2) {
    options.hierarchyType = "tree";
  }

  // Frozen
  if (proto.frozen) {
    options.frozen = {};
    if (proto.frozen.rowHeader) options.frozen.rowHeader = true;
    if (proto.frozen.rowCount > 0) options.frozen.rowCount = proto.frozen.rowCount;
    if (proto.frozen.colCount > 0) options.frozen.colCount = proto.frozen.colCount;
  }

  // Series number
  if (proto.seriesNumber) {
    options.seriesNumber = {
      enable: proto.seriesNumber.enable,
      text: proto.seriesNumber.text || "#",
    };
  }

  return { dataCfg, options };
}

/** Export the test-friendly config for the registry (no functions). */
export function pivotProtoToTestState(
  proto: PivotTableSpec,
  data: Record<string, unknown>[],
): Record<string, unknown> {
  return {
    rows: proto.fields ? [...proto.fields.rows] : [],
    columns: proto.fields ? [...proto.fields.columns] : [],
    values: proto.fields ? [...proto.fields.values] : [],
    valueInCols: proto.fields?.valueInCols ?? true,
    metaCount: proto.meta.length,
    metaFields: proto.meta.map((m) => ({
      field: m.field,
      name: m.name,
      formatter: m.formatter,
    })),
    sortCount: proto.sort.length,
    rowCount: data.length,
    columnNames: data.length > 0 ? Object.keys(data[0]) : [],
  };
}
