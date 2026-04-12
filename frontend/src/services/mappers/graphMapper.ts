/**
 * Graph mapper: translates GraphSpec proto -> G6 configuration.
 */

import type { GraphSpec } from "../../gen/open_plx/v1/widget_spec_pb.js";
import type { GraphData } from "@antv/g6";
import type { LayoutOptions, BehaviorOptions } from "@antv/g6";

export interface G6GraphConfig {
  data: GraphData;
  layout: LayoutOptions;
  node: Record<string, unknown>;
  edge: Record<string, unknown>;
  behaviors: BehaviorOptions;
}

export function graphProtoToG6(
  spec: GraphSpec,
  data: Record<string, unknown>[],
): G6GraphConfig {
  const dm = spec.dataMapping;
  if (!dm) {
    return {
      data: { nodes: [], edges: [] },
      layout: { type: "force" },
      node: {},
      edge: {},
      behaviors: [],
    };
  }

  // Build unique nodes from source/target fields
  const nodes: GraphData["nodes"] = [];
  const seenNodeIds = new Set<string>();
  for (const row of data) {
    const sourceId = String(row[dm.sourceField] ?? "");
    const targetId = String(row[dm.targetField] ?? "");

    if (sourceId && !seenNodeIds.has(sourceId)) {
      seenNodeIds.add(sourceId);
      nodes.push({
        id: sourceId,
        data: {},
        style: {
          size: spec.nodeStyle?.size || 20,
          fill: spec.nodeStyle?.color || "#1781f2",
          stroke: spec.nodeStyle?.borderColor || "#fff",
          lineWidth: spec.nodeStyle?.borderWidth || 0,
          labelText: spec.nodeStyle?.showLabel
            ? (dm.labelField ? String(row[dm.labelField] ?? sourceId) : sourceId)
            : undefined,
          labelFontSize: spec.nodeStyle?.labelFontSize || 12,
          cursor: "pointer",
        },
      });
    }
    if (targetId && !seenNodeIds.has(targetId)) {
      seenNodeIds.add(targetId);
      nodes.push({
        id: targetId,
        data: {},
        style: {
          size: spec.nodeStyle?.size || 20,
          fill: spec.nodeStyle?.color || "#1781f2",
          stroke: spec.nodeStyle?.borderColor || "#fff",
          lineWidth: spec.nodeStyle?.borderWidth || 0,
          labelText: spec.nodeStyle?.showLabel
            ? (dm.labelField ? String(row[dm.labelField] ?? targetId) : targetId)
            : undefined,
          labelFontSize: spec.nodeStyle?.labelFontSize || 12,
          cursor: "pointer",
        },
      });
    }
  }

  // Build edges
  const edges: GraphData["edges"] = [];
  for (const row of data) {
    const sourceId = String(row[dm.sourceField] ?? "");
    const targetId = String(row[dm.targetField] ?? "");
    if (sourceId && targetId) {
      const edgeStyle: Record<string, unknown> = {
        stroke: spec.edgeStyle?.color || "#999",
        lineWidth: spec.edgeStyle?.width || 1,
      };
      if (dm.valueField && row[dm.valueField] !== undefined) {
        const value = Number(row[dm.valueField]);
        edgeStyle["lineWidth"] = Math.max(1, value / 10);
      }
      if (spec.edgeStyle?.style === "dashed") {
        edgeStyle["lineDash"] = [4, 4];
      } else if (spec.edgeStyle?.style === "dotted") {
        edgeStyle["lineDash"] = [1, 2];
      }
      if (spec.edgeStyle?.showArrow) {
        edgeStyle["endArrow"] = { size: spec.edgeStyle?.arrowSize || 5, type: "triangle" };
      }
      edges.push({
        source: sourceId,
        target: targetId,
        style: edgeStyle,
      });
    }
  }

  // Layout config
  const layoutType = spec.layout?.type;
  let g6Layout: LayoutOptions;
  if (layoutType === 2) { // DAGRE
    g6Layout = {
      type: "dagre",
      direction: spec.layout?.direction === "LR" ? "LR" :
                 spec.layout?.direction === "RL" ? "RL" :
                 spec.layout?.direction === "BT" ? "BT" : "TB",
      nodeSpacing: spec.layout?.nodeSpacing || 20,
      rankSpacing: spec.layout?.rankSpacing || 40,
    };
  } else if (layoutType === 3) { // CIRCULAR
    g6Layout = { type: "circular" };
  } else if (layoutType === 4) { // GRID
    g6Layout = { type: "grid" };
  } else if (layoutType === 5) { // CONCENTRIC
    g6Layout = { type: "concentric" };
  } else {
    // FORCE (default)
    g6Layout = {
      type: "force",
      preventOverlap: true,
      nodeSpacing: spec.layout?.nodeSpacing || 20,
      iterations: spec.layout?.iterations || 300,
    };
  }

  // Node style defaults
  const nodeStyle: Record<string, unknown> = {
    size: spec.nodeStyle?.size || 20,
    fill: spec.nodeStyle?.color || "#1781f2",
    stroke: spec.nodeStyle?.borderColor || "#fff",
    lineWidth: spec.nodeStyle?.borderWidth || 0,
    labelFontSize: spec.nodeStyle?.labelFontSize || 12,
    cursor: "pointer",
  };

  // Edge style defaults
  const edgeStyle: Record<string, unknown> = {
    stroke: spec.edgeStyle?.color || "#999",
    lineWidth: spec.edgeStyle?.width || 1,
  };

  // Behaviors (interactions)
  const behaviors: BehaviorOptions = [];
  if (spec.interaction?.enableDrag) {
    behaviors.push("drag-canvas", "drag-node");
  }
  if (spec.interaction?.enableZoom) {
    behaviors.push("zoom-canvas");
  }
  if (spec.interaction?.enableClickSelect) {
    behaviors.push("click-select");
  }
  if (spec.interaction?.enableTooltip) {
    behaviors.push("tooltip");
  }

  return {
    data: {
      nodes,
      edges,
    },
    layout: g6Layout,
    node: nodeStyle,
    edge: edgeStyle,
    behaviors,
  };
}
