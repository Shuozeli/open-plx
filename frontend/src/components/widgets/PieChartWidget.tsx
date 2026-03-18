import { Card, Spin } from "antd";
import type { WidgetProps } from "./WidgetRegistry.js";
import { chartProtoToG2 } from "../../services/mappers/chartMapper.js";
import { G2Chart } from "./G2Chart.js";

export function PieChartWidget({ config, data, loading, error }: WidgetProps) {
  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  if (loading || !data) {
    return <Card title={config.title} style={{ height: "100%" }}><Spin /></Card>;
  }

  const chartSpec = config.spec?.spec.case === "chart" ? config.spec.spec.value : null;
  if (!chartSpec) {
    return <Card title={config.title}><span>No chart spec</span></Card>;
  }

  const g2Spec = chartProtoToG2(chartSpec, data);

  return (
    <Card title={config.title} style={{ height: "100%" }} styles={{ body: { height: "calc(100% - 56px)", padding: 12 } }}>
      <G2Chart spec={g2Spec} />
    </Card>
  );
}
