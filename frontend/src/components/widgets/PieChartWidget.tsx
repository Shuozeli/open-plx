import { Card, Spin } from "antd";
import type { WidgetProps } from "./WidgetRegistry.js";

export function PieChartWidget({ config, data, loading, error }: WidgetProps) {
  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  return (
    <Card title={config.title} style={{ height: "100%" }}>
      {loading || !data ? (
        <Spin />
      ) : (
        // TODO(refactor): Wire up G2 chart mapper.
        <pre style={{ fontSize: 11, overflow: "auto", maxHeight: "100%" }}>
          {JSON.stringify(data, null, 2)}
        </pre>
      )}
    </Card>
  );
}
