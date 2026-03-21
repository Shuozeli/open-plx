/**
 * Conditional formatting mapper: translates proto ConditionalFormat rules
 * into S2 condition mapping functions at runtime.
 *
 * Proto thresholds are declarative (op + value + color).
 * S2 conditions require runtime functions: mapping(fieldValue, data, cell) => result.
 * This mapper bridges the gap by generating closures from the declarative rules.
 */

import {
  ConditionalFormatType,
  ComparisonOp,
} from "../../gen/open_plx/v1/widget_spec_pb.js";
import type { ConditionalFormat } from "../../gen/open_plx/v1/widget_spec_pb.js";

/** S2 conditions config object. */
export interface S2Conditions {
  text?: Array<{ field: string; mapping: (val: number | string) => { fill?: string; fontSize?: number } | undefined }>;
  background?: Array<{ field: string; mapping: (val: number | string) => { fill: string; intelligentReverseTextColor?: boolean } | undefined }>;
  interval?: Array<{ field: string; mapping: (val: number | string) => { fill?: string; isCompare?: boolean } | undefined }>;
  icon?: Array<{ field: string; position?: "left" | "right"; mapping: (val: number | string) => { fill: string; icon: string } | undefined }>;
}

/** Convert proto ConditionalFormat[] to S2 Conditions. */
export function conditionsProtoToS2(formats: ConditionalFormat[]): S2Conditions {
  const conditions: S2Conditions = {};

  for (const fmt of formats) {
    switch (fmt.type) {
      case ConditionalFormatType.TEXT: {
        if (!conditions.text) conditions.text = [];
        conditions.text.push({
          field: fmt.field,
          mapping: buildTextMapper(fmt),
        });
        break;
      }
      case ConditionalFormatType.BACKGROUND: {
        if (!conditions.background) conditions.background = [];
        conditions.background.push({
          field: fmt.field,
          mapping: buildBackgroundMapper(fmt),
        });
        break;
      }
      case ConditionalFormatType.ICON: {
        if (!conditions.icon) conditions.icon = [];
        conditions.icon.push({
          field: fmt.field,
          mapping: buildIconMapper(fmt),
        });
        break;
      }
      case ConditionalFormatType.INTERVAL: {
        if (!conditions.interval) conditions.interval = [];
        conditions.interval.push({
          field: fmt.field,
          mapping: buildIntervalMapper(fmt),
        });
        break;
      }
    }
  }

  return conditions;
}

/** Check if a numeric value matches a threshold's comparison operator. */
function matches(val: number, op: ComparisonOp, threshold: number, thresholdEnd?: number): boolean {
  switch (op) {
    case ComparisonOp.GT: return val > threshold;
    case ComparisonOp.GTE: return val >= threshold;
    case ComparisonOp.LT: return val < threshold;
    case ComparisonOp.LTE: return val <= threshold;
    case ComparisonOp.EQ: return val === threshold;
    case ComparisonOp.NEQ: return val !== threshold;
    case ComparisonOp.BETWEEN: return val >= threshold && val <= (thresholdEnd ?? threshold);
    default: return false;
  }
}

function buildTextMapper(fmt: ConditionalFormat) {
  return (val: number | string) => {
    const num = Number(val);
    if (isNaN(num)) return undefined;
    for (const t of fmt.thresholds) {
      if (matches(num, t.op, t.value, t.valueEnd ?? undefined)) {
        return { fill: t.color || undefined };
      }
    }
    return undefined;
  };
}

function buildBackgroundMapper(fmt: ConditionalFormat) {
  return (val: number | string) => {
    const num = Number(val);
    if (isNaN(num)) return undefined;
    for (const t of fmt.thresholds) {
      if (matches(num, t.op, t.value, t.valueEnd ?? undefined)) {
        return { fill: t.color, intelligentReverseTextColor: true };
      }
    }
    return undefined;
  };
}

function buildIconMapper(fmt: ConditionalFormat) {
  return (val: number | string) => {
    const num = Number(val);
    if (isNaN(num)) return undefined;
    for (const t of fmt.thresholds) {
      if (matches(num, t.op, t.value, t.valueEnd ?? undefined)) {
        return { fill: t.color, icon: t.icon || "CellUp" };
      }
    }
    return undefined;
  };
}

function buildIntervalMapper(fmt: ConditionalFormat) {
  return (val: number | string) => {
    const num = Number(val);
    if (isNaN(num)) return undefined;
    const color = fmt.thresholds[0]?.color || "#80BFFF";
    return {
      fill: color,
      isCompare: true,
      minValue: fmt.intervalMin ?? 0,
      maxValue: fmt.intervalMax ?? 100,
    };
  };
}
