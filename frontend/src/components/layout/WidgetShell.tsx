import { useCallback } from "react";
import type { WidgetConfig, ParamValue } from "../../gen/open_plx/v1/dashboard_pb.js";
import { WidgetType, ParamValueSchema } from "../../gen/open_plx/v1/dashboard_pb.js";
import { create } from "@bufbuild/protobuf";
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
  onVariableChange?: (name: string, value: ParamValue) => void;
}

/** Convert a JS value to a typed ParamValue proto. */
function toParamValue(value: unknown): ParamValue {
  if (typeof value === "string") {
    return create(ParamValueSchema, { value: { case: "stringValue", value } });
  }
  if (typeof value === "number") {
    if (Number.isInteger(value)) {
      return create(ParamValueSchema, { value: { case: "intValue", value: BigInt(value) } });
    }
    return create(ParamValueSchema, { value: { case: "doubleValue", value } });
  }
  if (typeof value === "boolean") {
    return create(ParamValueSchema, { value: { case: "boolValue", value } });
  }
  return create(ParamValueSchema, { value: { case: "stringValue", value: String(value) } });
}

export function WidgetShell({ dashboardName, config, variableValues, revision, onVariableChange }: WidgetShellProps) {
  const { data, loading, error, permissionDenied } = useWidgetData(
    dashboardName,
    config.id,
    variableValues,
    revision,
  );
  const Component = getWidgetComponent(config.widgetType);

  const hasClickInteractions = config.clickInteractions.length > 0;

  const handleClickInteraction = useCallback(
    (clickedRecord: Record<string, unknown>) => {
      if (!onVariableChange) return;
      for (const ci of config.clickInteractions) {
        const value = clickedRecord[ci.sourceField];
        if (value !== undefined) {
          onVariableChange(ci.targetVariable, toParamValue(value));
        }
      }
    },
    [config.clickInteractions, onVariableChange],
  );

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

  return (
    <Component
      config={config}
      data={data}
      loading={loading}
      error={error}
      onClickInteraction={hasClickInteractions ? handleClickInteraction : undefined}
    />
  );
}
