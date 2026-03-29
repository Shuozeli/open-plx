/**
 * Word Cloud mapper: translates WordCloudSpec proto -> G2 v5 wordCloud spec.
 */

import type { WordCloudSpec } from "../../gen/open_plx/v1/widget_spec_pb.js";
import type { G2Spec } from "./chartMapper.js";

export function wordCloudProtoToG2(
  proto: WordCloudSpec,
  data: Record<string, unknown>[],
): G2Spec {
  // Limit data if maxWords is set
  const limited = proto.maxWords > 0 ? data.slice(0, proto.maxWords) : data;

  const minFont = proto.fontSizeRange.length >= 1 ? proto.fontSizeRange[0] : 12;
  const maxFont = proto.fontSizeRange.length >= 2 ? proto.fontSizeRange[1] : 60;

  return {
    type: "wordCloud",
    data: limited,
    encode: {
      text: proto.textField,
      size: proto.weightField,
      color: proto.textField,
    },
    layout: {
      spiral: "rectangular",
      fontSize: [minFont, maxFont],
    },
    autoFit: true,
  };
}

export function wordCloudProtoToTestState(
  proto: WordCloudSpec,
  data: Record<string, unknown>[],
): Record<string, unknown> {
  return {
    wordCount: proto.maxWords > 0 ? Math.min(data.length, proto.maxWords) : data.length,
    textField: proto.textField,
    weightField: proto.weightField,
  };
}
