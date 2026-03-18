import { Card, Spin } from "antd";
import type { WidgetProps } from "./WidgetRegistry.js";

export function LineChartWidget({ config, data, loading, error }: WidgetProps) {
  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  return (
    <Card title={config.title} style={{ height: "100%" }}>
      {loading || !data ? (
        <Spin />
      ) : (
        // TODO(refactor): Wire up G2 chart mapper. For now show data summary.
        <pre style={{ fontSize: 11, overflow: "auto", maxHeight: "100%" }}>
          {JSON.stringify(data.slice(0, 5), null, 2)}
          {data.length > 5 && `\n... ${data.length - 5} more rows`}
        </pre>
      )}
    </Card>
  );
}
