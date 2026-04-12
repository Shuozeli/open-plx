import { Card, Spin, message } from "antd";
import { useEffect, useRef, useState } from "react";
import { create } from "@bufbuild/protobuf";
import type { WidgetProps } from "./WidgetRegistry.js";
import { tableProtoToS2, tableProtoToTestState } from "../../services/mappers/tableMapper.js";
import { S2Table, type ActionInvokeResult } from "./S2Table.js";
import { registerWidget } from "../../services/testRegistry.js";
import { WidgetType, ParamValueSchema } from "../../gen/open_plx/v1/dashboard_pb.js";
import { widgetActionClient } from "../../services/grpc/client.js";

export function TableWidget({ dashboardName, config, data, loading, error, onClickInteraction, onVariableChange }: WidgetProps) {
  const spec = config.spec?.spec.case === "table" ? config.spec.spec.value : null;
  const bodyRef = useRef<HTMLDivElement>(null);
  const [dims, setDims] = useState({ w: 800, h: 300 });

  useEffect(() => {
    if (!bodyRef.current) return;
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setDims({
          w: Math.floor(entry.contentRect.width),
          h: Math.floor(entry.contentRect.height),
        });
      }
    });
    ro.observe(bodyRef.current);
    return () => ro.disconnect();
  }, []);

  useEffect(() => {
    registerWidget({
      widgetId: config.id,
      widgetType: WidgetType[config.widgetType],
      spec: spec ? JSON.parse(JSON.stringify(spec)) : {},
      data,
      g2Spec: null,
      rendered: {
        hasData: data !== null && data.length > 0,
        rowCount: data?.length ?? 0,
        ...(spec && data ? tableProtoToTestState(spec, data) : {}),
      },
      updatedAt: Date.now(),
    });
  }, [config, data, spec]);

  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  if (loading || !data || !spec) {
    return <Card title={config.title} style={{ height: "100%" }}><Spin /></Card>;
  }

  const { dataCfg, options, linkColumn, colSpans, actionColumns } = tableProtoToS2(spec, data, dims.w, dims.h);

  const handleActionInvoke = async (actionId: string, requestBody: string): Promise<ActionInvokeResult> => {
    if (!dashboardName) {
      return { success: false, message: "Dashboard name not available" };
    }
    try {
      const response = await widgetActionClient.invokeAction({
        dashboardName,
        widgetId: config.id,
        actionId,
        requestBody,
      });

      if (response.success) {
        // Handle SET_VARIABLE result handling
        if (response.variableName && response.variableValue && onVariableChange) {
          const paramValue = create(ParamValueSchema, {
            value: { case: "stringValue", value: response.variableValue },
          });
          onVariableChange(response.variableName, paramValue);
        }
        message.success(response.message || "Action completed");
      } else {
        message.error(response.message || "Action failed");
      }
      return {
        success: response.success,
        message: response.message || "Action completed",
        variableName: response.variableName,
        variableValue: response.variableValue,
      };
    } catch (err) {
      console.error("Action invocation failed:", err);
      message.error("Action invocation failed");
      return { success: false, message: "Action invocation failed" };
    }
  };

  return (
    <Card
      title={config.title}
      style={{ height: "100%" }}
      styles={{ body: { height: "calc(100% - 56px)", padding: 0, overflow: "hidden" } }}
    >
      <div ref={bodyRef} style={{ width: "100%", height: "100%" }}>
        {dims.w > 0 && dims.h > 0 && (
          <S2Table
            dataCfg={dataCfg}
            options={options}
            onRowClick={onClickInteraction}
            linkColumnField={linkColumn?.field}
            linkTemplate={linkColumn?.urlTemplate}
            linkNewTab={linkColumn?.newTab}
            colSpans={colSpans}
            actionColumns={actionColumns}
            onActionInvoke={handleActionInvoke}
          />
        )}
      </div>
    </Card>
  );
}
