/**
 * Chart mapper: translates semantic ChartSpec proto -> G2 v5 spec.
 *
 * This is the decoupling layer between our proto schema and G2.
 * If G2 is ever replaced, only this file changes.
 */

import { ChartType, StackMode, LineShape } from "../../gen/open_plx/v1/widget_spec_pb.js";
import { ScaleType } from "../../gen/open_plx/v1/widget_spec_pb.js";
import type { ChartSpec, AxisConfig } from "../../gen/open_plx/v1/widget_spec_pb.js";

/** G2 spec object passed to chart.options(). */
export interface G2Spec {
  type: string;
  data: Record<string, unknown>[];
  encode?: Record<string, unknown>;
  transform?: Record<string, unknown>[];
  scale?: Record<string, unknown>;
  axis?: Record<string, unknown>;
  coordinate?: Record<string, unknown>;
  labels?: Record<string, unknown>[];
  style?: Record<string, unknown>;
  autoFit?: boolean;
  [key: string]: unknown;
}

/** Convert a semantic ChartSpec proto to a G2 v5 spec. */
export function chartProtoToG2(
  proto: ChartSpec,
  data: Record<string, unknown>[],
): G2Spec {
  const spec: G2Spec = {
    type: mapChartType(proto.chartType),
    data,
    autoFit: true,
  };

  // Encode: map data columns to visual channels
  const dm = proto.dataMapping;
  if (dm) {
    const encode: Record<string, unknown> = {};
    if (dm.x) encode.x = dm.x;
    if (dm.y) encode.y = dm.y;
    if (dm.groupBy) encode.color = dm.groupBy;
    if (dm.size) encode.size = dm.size;
    // Pie/donut: value -> y, category -> color
    if (dm.value) encode.y = dm.value;
    if (dm.category) encode.color = dm.category;
    spec.encode = encode;
  }

  // Line shape
  if (proto.lineShape !== LineShape.UNSPECIFIED) {
    if (!spec.encode) spec.encode = {};
    spec.encode.shape = mapLineShape(proto.lineShape);
  }

  // Stack mode -> transform
  const transforms = mapStackMode(proto.stackMode);
  // Pie/donut require stackY to work with theta coordinate
  if (proto.chartType === ChartType.PIE || proto.chartType === ChartType.DONUT) {
    if (!transforms.some((t) => t.type === "stackY")) {
      transforms.push({ type: "stackY" });
    }
  }
  if (transforms.length > 0) {
    spec.transform = transforms;
  }

  // Coordinate (pie/donut/radar)
  const coord = mapCoordinate(proto);
  if (coord) {
    spec.coordinate = coord;
  }

  // Axes
  const axis: Record<string, unknown> = {};
  if (proto.xAxis) {
    axis.x = mapAxis(proto.xAxis);
  }
  if (proto.yAxis) {
    axis.y = mapAxis(proto.yAxis);
  }
  if (Object.keys(axis).length > 0) {
    spec.axis = axis;
  }

  // Scale overrides
  const scale: Record<string, unknown> = {};
  if (proto.xAxis?.scaleType) {
    scale.x = { type: mapScaleType(proto.xAxis.scaleType) };
  }
  if (proto.yAxis?.scaleType) {
    scale.y = { type: mapScaleType(proto.yAxis.scaleType) };
  }
  if (Object.keys(scale).length > 0) {
    spec.scale = scale;
  }

  // Labels
  if (proto.labels.length > 0) {
    spec.labels = proto.labels.map((l) => {
      const label: Record<string, unknown> = {};
      if (l.field) label.text = l.field;
      if (l.position) label.position = mapLabelPosition(l.position);
      if (l.connector) label.connector = true;
      return label;
    });
  }

  return spec;
}

function mapChartType(ct: ChartType): string {
  switch (ct) {
    case ChartType.LINE: return "line";
    case ChartType.BAR: return "interval";
    case ChartType.HORIZONTAL_BAR: return "interval";
    case ChartType.PIE: return "interval";
    case ChartType.DONUT: return "interval";
    case ChartType.AREA: return "area";
    case ChartType.SCATTER: return "point";
    case ChartType.HEATMAP: return "cell";
    case ChartType.HISTOGRAM: return "rect";
    case ChartType.RADAR: return "line";
    default: return "interval";
  }
}

function mapStackMode(sm: StackMode): Record<string, unknown>[] {
  switch (sm) {
    case StackMode.STACKED: return [{ type: "stackY" }];
    case StackMode.GROUPED: return [{ type: "dodgeX" }];
    case StackMode.PERCENT: return [{ type: "normalizeY" }];
    default: return [];
  }
}

function mapCoordinate(proto: ChartSpec): Record<string, unknown> | null {
  if (proto.chartType === ChartType.PIE) {
    return { type: "theta" };
  }
  if (proto.chartType === ChartType.DONUT) {
    return {
      type: "theta",
      innerRadius: proto.coordinate?.innerRadius ?? 0.6,
    };
  }
  if (proto.chartType === ChartType.HORIZONTAL_BAR) {
    return { transform: [{ type: "transpose" }] };
  }
  if (proto.chartType === ChartType.RADAR) {
    return { type: "radar" };
  }
  return null;
}

function mapAxis(axis: AxisConfig): Record<string, unknown> | false {
  if (axis.hidden) return false;
  const result: Record<string, unknown> = {};
  if (axis.title) result.title = axis.title;
  if (axis.labelFormat) result.labelFormatter = axis.labelFormat;
  if (axis.tickCount > 0) result.tickCount = axis.tickCount;
  return result;
}

function mapScaleType(st: ScaleType): string | undefined {
  switch (st) {
    case ScaleType.LINEAR: return "linear";
    case ScaleType.LOG: return "log";
    case ScaleType.TIME: return "time";
    case ScaleType.BAND: return "band";
    case ScaleType.ORDINAL: return "ordinal";
    default: return undefined;
  }
}

function mapLineShape(ls: LineShape): string {
  switch (ls) {
    case LineShape.SMOOTH: return "smooth";
    case LineShape.STEP: return "vh";
    case LineShape.STEP_BEFORE: return "hv";
    case LineShape.STEP_MIDDLE: return "hvh";
    default: return "line";
  }
}

function mapLabelPosition(pos: number): string {
  // LabelPosition enum values
  switch (pos) {
    case 1: return "top";
    case 2: return "bottom";
    case 3: return "left";
    case 4: return "right";
    case 5: return "inside";
    case 6: return "outside";
    default: return "top";
  }
}
