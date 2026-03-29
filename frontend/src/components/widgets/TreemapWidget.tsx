import { Card, Spin } from "antd";
import { useEffect } from "react";
import type { WidgetProps } from "./WidgetRegistry.js";
import { treemapProtoToG2, treemapProtoToTestState } from "../../services/mappers/treemapMapper.js";
import { G2Chart } from "./G2Chart.js";
import { registerWidget } from "../../services/testRegistry.js";
import { WidgetType } from "../../gen/open_plx/v1/dashboard_pb.js";

export function TreemapWidget({ config, data, loading, error, onClickInteraction }: WidgetProps) {
  const spec = config.spec?.spec.case === "treemap" ? config.spec.spec.value : null;
  const g2Spec = spec && data ? treemapProtoToG2(spec, data) : null;

  useEffect(() => {
    registerWidget({
      widgetId: config.id,
      widgetType: WidgetType[config.widgetType],
      spec: spec ? JSON.parse(JSON.stringify(spec)) : {},
      data,
      g2Spec: g2Spec ? JSON.parse(JSON.stringify(g2Spec)) : null,
      rendered: {
        hasData: data !== null && data.length > 0,
        rowCount: data?.length ?? 0,
        ...(spec && data ? treemapProtoToTestState(spec, data) : {}),
      },
      updatedAt: Date.now(),
    });
  }, [config, data, spec, g2Spec]);

  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  if (loading || !data || !g2Spec) {
    return <Card title={config.title} style={{ height: "100%" }}><Spin /></Card>;
  }

  return (
    <Card title={config.title} style={{ height: "100%" }} styles={{ body: { height: "calc(100% - 56px)", padding: 12 } }}>
      <G2Chart spec={g2Spec} onElementClick={onClickInteraction} />
    </Card>
  );
}
