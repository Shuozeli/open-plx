import { Card } from "antd";
import { useEffect } from "react";
import type { WidgetProps } from "./WidgetRegistry.js";
import { registerWidget } from "../../services/testRegistry.js";
import { WidgetType } from "../../gen/open_plx/v1/dashboard_pb.js";

export function TextWidget({ config, data }: WidgetProps) {
  const spec = config.spec?.spec.case === "text" ? config.spec.spec.value : null;
  const content = spec?.content ?? "";
  const format = spec?.format ?? 0;

  useEffect(() => {
    registerWidget({
      widgetId: config.id,
      widgetType: WidgetType[config.widgetType],
      spec: spec ? JSON.parse(JSON.stringify(spec)) : {},
      data,
      g2Spec: null,
      rendered: {
        hasData: true, // text widgets always "have data"
        content,
        format,
      },
      updatedAt: Date.now(),
    });
  }, [config, data, spec, content, format]);

  return (
    <Card title={config.title} style={{ height: "100%" }}>
      <p>{content}</p>
    </Card>
  );
}
