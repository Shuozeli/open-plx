/**
 * Treemap mapper: translates TreemapSpec proto -> G2 v5 treemap spec.
 * Converts flat tabular data into nested {name, value, children} tree.
 */

import type { TreemapSpec } from "../../gen/open_plx/v1/widget_spec_pb.js";
import type { G2Spec } from "./chartMapper.js";

interface TreeNode {
  name: string;
  value?: number;
  children?: TreeNode[];
}

/** Build a tree from flat data by grouping on hierarchy fields in order. */
function buildTree(
  data: Record<string, unknown>[],
  hierarchyFields: string[],
  valueField: string,
): TreeNode {
  if (hierarchyFields.length === 0) {
    return { name: "root", value: data.reduce((s, r) => s + Number(r[valueField] ?? 0), 0) };
  }

  function group(rows: Record<string, unknown>[], depth: number): TreeNode[] {
    if (depth >= hierarchyFields.length) {
      // Leaf level: each row is a leaf
      return rows.map((r) => ({
        name: String(r[hierarchyFields[hierarchyFields.length - 1]] ?? ""),
        value: Number(r[valueField] ?? 0),
      }));
    }

    const field = hierarchyFields[depth];
    const groups = new Map<string, Record<string, unknown>[]>();
    for (const row of rows) {
      const key = String(row[field] ?? "");
      let arr = groups.get(key);
      if (!arr) {
        arr = [];
        groups.set(key, arr);
      }
      arr.push(row);
    }

    return Array.from(groups.entries()).map(([key, groupRows]) => {
      if (depth === hierarchyFields.length - 1) {
        // Last level: sum values
        const value = groupRows.reduce((s, r) => s + Number(r[valueField] ?? 0), 0);
        return { name: key, value };
      }
      return { name: key, children: group(groupRows, depth + 1) };
    });
  }

  return { name: "root", children: group(data, 0) };
}

export function treemapProtoToG2(
  proto: TreemapSpec,
  data: Record<string, unknown>[],
): G2Spec {
  const tree = buildTree(data, proto.hierarchyFields, proto.valueField);

  return {
    type: "treemap",
    data: tree as unknown as Record<string, unknown>[],
    layout: { tile: "treemapSquarify" },
    encode: { value: "value" },
    labels: proto.showLabels ? [{ text: "name", position: "inside", fontSize: 10 }] : [],
    autoFit: true,
  };
}

export function treemapProtoToTestState(
  proto: TreemapSpec,
  data: Record<string, unknown>[],
): Record<string, unknown> {
  return {
    rowCount: data.length,
    hierarchyDepth: proto.hierarchyFields.length,
    valueField: proto.valueField,
  };
}
