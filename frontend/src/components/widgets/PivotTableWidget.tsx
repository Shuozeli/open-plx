import { Card, Spin } from "antd";
import type { WidgetProps } from "./WidgetRegistry.js";

export function PivotTableWidget({ config, data, loading, error }: WidgetProps) {
  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  return (
    <Card title={config.title} style={{ height: "100%" }}>
      {loading || !data ? (
        <Spin />
      ) : (
        // TODO(refactor): Wire up S2 pivot table mapper.
        <pre style={{ fontSize: 11, overflow: "auto", maxHeight: "100%" }}>
          {JSON.stringify(data.slice(0, 10), null, 2)}
        </pre>
      )}
    </Card>
  );
}
