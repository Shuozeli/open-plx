import { Card, Spin, Statistic } from "antd";
import type { WidgetProps } from "./WidgetRegistry.js";

export function MetricCardWidget({ config, data, loading, error }: WidgetProps) {
  const spec = config.spec?.spec.case === "metricCard" ? config.spec.spec.value : null;

  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  if (loading || !spec) {
    return <Card title={config.title}><Spin /></Card>;
  }

  // Extract value from data (first row, spec.value column)
  const rawValue = data?.[0]?.[spec.value];
  const displayValue = rawValue != null ? String(rawValue) : "--";

  return (
    <Card style={{ height: "100%" }}>
      <Statistic title={config.title} value={displayValue} />
    </Card>
  );
}
