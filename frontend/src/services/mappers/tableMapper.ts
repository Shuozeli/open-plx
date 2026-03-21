/**
 * Table mapper: translates TableSpec proto -> S2 TableSheet config.
 */

import type { TableSpec } from "../../gen/open_plx/v1/widget_spec_pb.js";
import { conditionsProtoToS2 } from "./conditionMapper.js";

/** S2 TableSheet data config. */
export interface S2TableDataConfig {
  data: Record<string, unknown>[];
  fields: { columns: string[] };
  meta?: { field: string; name?: string; formatter?: (v: unknown) => string }[];
}

/** S2 TableSheet options. */
export interface S2TableOptions {
  width: number;
  height: number;
  showSeriesNumber?: boolean;
  pagination?: { pageSize: number; current: number };
  interaction?: Record<string, unknown>;
  conditions?: unknown;
  [key: string]: unknown;
}

/** Convert a TableSpec proto + row data to S2 TableSheet config. */
export function tableProtoToS2(
  proto: TableSpec,
  data: Record<string, unknown>[],
  containerWidth: number,
  containerHeight: number,
): { dataCfg: S2TableDataConfig; options: S2TableOptions } {
  // Determine columns: use proto columns if specified, else infer from data
  let columnFields: string[];
  if (proto.columns.length > 0) {
    columnFields = proto.columns.map((c) => c.field);
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

  // Conditional formatting
  if (proto.conditions.length > 0) {
    options.conditions = conditionsProtoToS2(proto.conditions);
  }

  if (proto.pagination) {
    options.pagination = {
      pageSize: proto.pagination.pageSize,
      current: 1,
    };
  }

  return { dataCfg, options };
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
    hasPageination: !!proto.pagination,
    showRowNumbers: proto.showRowNumbers,
  };
}
