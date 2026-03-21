/**
 * Gauge mapper: translates GaugeSpec proto -> G2 v5 gauge spec.
 */

import type { GaugeSpec } from "../../gen/open_plx/v1/widget_spec_pb.js";
import type { G2Spec } from "./chartMapper.js";

/** Convert GaugeSpec to G2 spec. Uses interval + theta as a semicircular gauge. */
export function gaugeProtoToG2(
  proto: GaugeSpec,
  data: Record<string, unknown>[],
): G2Spec {
  const rawValue = data.length > 0 ? Number(data[0][proto.valueField] ?? 0) : 0;
  const min = proto.min;
  const max = proto.max || 100;
  const normalized = Math.max(0, Math.min(1, (rawValue - min) / (max - min)));

  // Build color thresholds from ranges
  const rangeColors = proto.ranges.length > 0
    ? proto.ranges.map((r) => r.color)
    : ["#30BF78", "#FAAD14", "#F4664A"];

  const rangeDomain = proto.ranges.length > 0
    ? proto.ranges.slice(0, -1).map((r) => (r.to - min) / (max - min))
    : [0.33, 0.66];

  return {
    type: "interval",
    data: [{ value: normalized, rest: 1 - normalized }],
    encode: { y: "value", color: "value" },
    coordinate: {
      type: "radial",
      innerRadius: 0.7,
      startAngle: -Math.PI,
      endAngle: 0,
    },
    scale: {
      color: {
        type: "threshold",
        domain: rangeDomain,
        range: rangeColors,
      },
    },
    style: { stroke: "transparent" },
    autoFit: true,
    // Store raw value for test registry
    _gaugeRawValue: rawValue,
    _gaugeNormalized: normalized,
  };
}

export function gaugeProtoToTestState(
  proto: GaugeSpec,
  data: Record<string, unknown>[],
): Record<string, unknown> {
  const rawValue = data.length > 0 ? Number(data[0][proto.valueField] ?? 0) : 0;
  const max = proto.max || 100;
  return {
    gaugeValue: rawValue,
    gaugePercentage: Math.round((rawValue / max) * 100),
    rangeCount: proto.ranges.length,
  };
}
