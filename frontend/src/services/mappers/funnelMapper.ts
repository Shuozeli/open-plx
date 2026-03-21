/**
 * Funnel mapper: translates FunnelSpec proto -> G2 v5 funnel spec.
 */

import type { FunnelSpec } from "../../gen/open_plx/v1/widget_spec_pb.js";
import type { G2Spec } from "./chartMapper.js";

/** Convert FunnelSpec to G2 spec. */
export function funnelProtoToG2(
  proto: FunnelSpec,
  data: Record<string, unknown>[],
): G2Spec {
  return {
    type: "interval",
    data,
    encode: {
      x: proto.categoryField,
      y: proto.valueField,
      color: proto.categoryField,
      shape: "funnel",
    },
    transform: [{ type: "symmetryY" }],
    coordinate: { transform: [{ type: "transpose" }] },
    scale: { x: { padding: 0 } },
    labels: [{ text: proto.valueField, position: "inside" }],
    axis: { y: false },
    autoFit: true,
  };
}

export function funnelProtoToTestState(
  proto: FunnelSpec,
  data: Record<string, unknown>[],
): Record<string, unknown> {
  return {
    stageCount: data.length,
    categoryField: proto.categoryField,
    valueField: proto.valueField,
  };
}
