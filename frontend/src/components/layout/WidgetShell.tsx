import type { WidgetConfig } from "../../gen/open_plx/v1/dashboard_pb.js";
import { WidgetType } from "../../gen/open_plx/v1/dashboard_pb.js";
import { useWidgetData } from "../../hooks/useWidgetData.js";
import type { VariableValues } from "../../hooks/useVariables.js";
import { getWidgetComponent } from "../widgets/WidgetRegistry.js";
import { Card, Result } from "antd";
import { LockOutlined } from "@ant-design/icons";

interface WidgetShellProps {
  dashboardName: string;
  config: WidgetConfig;
  variableValues?: VariableValues;
  revision?: number;
}

export function WidgetShell({ dashboardName, config, variableValues, revision }: WidgetShellProps) {
  const { data, loading, error, permissionDenied } = useWidgetData(
    dashboardName,
    config.id,
    variableValues,
    revision,
  );
  const Component = getWidgetComponent(config.widgetType);

  if (permissionDenied) {
    return (
      <Card title={config.title} style={{ height: "100%" }}>
        <Result
          icon={<LockOutlined />}
          title="Access Denied"
          subTitle="You do not have permission to view this data."
          status="403"
        />
      </Card>
    );
  }

  if (!Component) {
    return (
      <Card title={config.title} style={{ height: "100%" }}>
        <span>Unknown widget type: {WidgetType[config.widgetType]}</span>
      </Card>
    );
  }

  return <Component config={config} data={data} loading={loading} error={error} />;
}
