import { Card, Spin } from "antd";
import { useEffect, useRef, useState } from "react";
import { Graph, type IElementEvent } from "@antv/g6";
import type { WidgetProps } from "./WidgetRegistry.js";
import { graphProtoToG6 } from "../../services/mappers/graphMapper.js";
import { registerWidget } from "../../services/testRegistry.js";
import { WidgetType } from "../../gen/open_plx/v1/dashboard_pb.js";

export function GraphWidget({ config, data, loading, error, onClickInteraction }: WidgetProps) {
  const spec = config.spec?.spec.case === "graph" ? config.spec.spec.value : null;
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<Graph | null>(null);
  const [dims, setDims] = useState({ w: 800, h: 400 });

  useEffect(() => {
    if (!containerRef.current) return;
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setDims({
          w: Math.floor(entry.contentRect.width),
          h: Math.floor(entry.contentRect.height),
        });
      }
    });
    ro.observe(containerRef.current);
    return () => ro.disconnect();
  }, []);

  useEffect(() => {
    if (!containerRef.current || !data || !spec) return;

    // Destroy existing graph
    if (graphRef.current) {
      graphRef.current.destroy();
      graphRef.current = null;
    }

    const g6Config = graphProtoToG6(spec, data as Record<string, unknown>[]);
    const graph = new Graph({
      container: containerRef.current,
      width: dims.w,
      height: dims.h,
      data: g6Config.data,
      layout: g6Config.layout,
      node: {
        type: "circle",
        style: g6Config.node,
      },
      edge: {
        type: "line",
        style: g6Config.edge,
      },
      behaviors: g6Config.behaviors,
    });

    graph.render();

    // Handle node click
    if (onClickInteraction) {
      graph.on("node:click", (event: IElementEvent) => {
        const nodeId = (event.target as { id?: string }).id;
        if (nodeId) {
          onClickInteraction({
            widgetId: config.id,
            field: "node_id",
            value: nodeId,
          });
        }
      });

      // Handle edge click
      if (spec.interaction?.enableEdgeClick) {
        graph.on("edge:click", (event: IElementEvent) => {
          const edgeId = (event.target as { id?: string }).id;
          if (edgeId) {
            onClickInteraction({
              widgetId: config.id,
              field: "edge_id",
              value: edgeId,
            });
          }
        });
      }
    }

    graphRef.current = graph;

    return () => {
      if (graphRef.current) {
        graphRef.current.destroy();
        graphRef.current = null;
      }
    };
  }, [data, spec, dims, onClickInteraction, config.id]);

  useEffect(() => {
    registerWidget({
      widgetId: config.id,
      widgetType: WidgetType[config.widgetType],
      spec: spec ? JSON.parse(JSON.stringify(spec)) : {},
      data,
      g2Spec: null,
      rendered: {
        hasData: data !== null && data.length > 0,
        nodeCount: data?.length ?? 0,
      },
      updatedAt: Date.now(),
    });
  }, [config, data, spec]);

  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  if (loading || !data || !spec) {
    return <Card title={config.title} style={{ height: "100%" }}><Spin /></Card>;
  }

  return (
    <Card
      title={config.title}
      style={{ height: "100%" }}
      styles={{ body: { height: "calc(100% - 56px)", padding: 0, overflow: "hidden" } }}
    >
      <div ref={containerRef} style={{ width: "100%", height: "100%" }} />
    </Card>
  );
}
