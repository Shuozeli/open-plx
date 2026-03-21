/**
 * Sankey mapper: translates SankeySpec proto -> G2 v5 sankey spec.
 * Transforms flat source/target/value rows into links+nodes structure.
 */

import type { SankeySpec } from "../../gen/open_plx/v1/widget_spec_pb.js";
import type { G2Spec } from "./chartMapper.js";

interface SankeyLink {
  source: string;
  target: string;
  value: number;
}

export function sankeyProtoToG2(
  proto: SankeySpec,
  data: Record<string, unknown>[],
): G2Spec {
  const links: SankeyLink[] = data.map((row) => ({
    source: String(row[proto.sourceField] ?? ""),
    target: String(row[proto.targetField] ?? ""),
    value: Number(row[proto.valueField] ?? 0),
  }));

  // Extract unique node names from links
  const nodeSet = new Set<string>();
  for (const link of links) {
    nodeSet.add(link.source);
    nodeSet.add(link.target);
  }

  return {
    type: "sankey",
    data: { links, nodes: Array.from(nodeSet).map((name) => ({ name })) } as unknown as Record<string, unknown>[],
    layout: { nodeWidth: 0.008, nodePadding: 0.03 },
    style: { labelFontSize: 10 },
    autoFit: true,
  };
}

export function sankeyProtoToTestState(
  proto: SankeySpec,
  data: Record<string, unknown>[],
): Record<string, unknown> {
  const nodeSet = new Set<string>();
  for (const row of data) {
    nodeSet.add(String(row[proto.sourceField] ?? ""));
    nodeSet.add(String(row[proto.targetField] ?? ""));
  }
  return {
    linkCount: data.length,
    nodeCount: nodeSet.size,
    sourceField: proto.sourceField,
    targetField: proto.targetField,
  };
}
