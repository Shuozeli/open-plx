import { Card, Spin } from "antd";
import { useEffect } from "react";
import type { WidgetProps } from "./WidgetRegistry.js";
import { chartProtoToG2 } from "../../services/mappers/chartMapper.js";
import { G2Chart } from "./G2Chart.js";
import { registerWidget } from "../../services/testRegistry.js";
import { WidgetType } from "../../gen/open_plx/v1/dashboard_pb.js";

export function PieChartWidget({ config, data, loading, error }: WidgetProps) {
  const chartSpec = config.spec?.spec.case === "chart" ? config.spec.spec.value : null;
  const g2Spec = chartSpec && data ? chartProtoToG2(chartSpec, data) : null;

  useEffect(() => {
    registerWidget({
      widgetId: config.id,
      widgetType: WidgetType[config.widgetType],
      spec: chartSpec ? JSON.parse(JSON.stringify(chartSpec)) : {},
      data,
      g2Spec: g2Spec ? JSON.parse(JSON.stringify(g2Spec)) : null,
      rendered: {
        hasData: data !== null && data.length > 0,
        rowCount: data?.length ?? 0,
        chartType: g2Spec?.type ?? null,
        coordinateType: (g2Spec?.coordinate as Record<string, unknown>)?.type ?? null,
        innerRadius: (g2Spec?.coordinate as Record<string, unknown>)?.innerRadius ?? null,
        hasTransform: Array.isArray(g2Spec?.transform) && (g2Spec.transform as unknown[]).length > 0,
      },
      updatedAt: Date.now(),
    });
  }, [config, data, chartSpec, g2Spec]);

  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  if (loading || !data || !g2Spec) {
    return <Card title={config.title} style={{ height: "100%" }}><Spin /></Card>;
  }

  return (
    <Card title={config.title} style={{ height: "100%" }} styles={{ body: { height: "calc(100% - 56px)", padding: 12 } }}>
      <G2Chart spec={g2Spec} />
    </Card>
  );
}
