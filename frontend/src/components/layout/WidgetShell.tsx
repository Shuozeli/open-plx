import type { WidgetConfig } from "../../gen/open_plx/v1/dashboard_pb.js";
import { WidgetType } from "../../gen/open_plx/v1/dashboard_pb.js";
import { useWidgetData } from "../../hooks/useWidgetData.js";
import { getWidgetComponent } from "../widgets/WidgetRegistry.js";
import { Card } from "antd";

interface WidgetShellProps {
  dashboardName: string;
  config: WidgetConfig;
}

export function WidgetShell({ dashboardName, config }: WidgetShellProps) {
  const { data, loading, error } = useWidgetData(dashboardName, config.id);
  const Component = getWidgetComponent(config.widgetType);

  if (!Component) {
    return (
      <Card title={config.title} style={{ height: "100%" }}>
        <span>Unknown widget type: {WidgetType[config.widgetType]}</span>
      </Card>
    );
  }

  return <Component config={config} data={data} loading={loading} error={error} />;
}
