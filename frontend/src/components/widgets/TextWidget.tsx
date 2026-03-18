import { Card } from "antd";
import type { WidgetProps } from "./WidgetRegistry.js";

export function TextWidget({ config }: WidgetProps) {
  const spec = config.spec?.spec.case === "text" ? config.spec.spec.value : null;
  const content = spec?.content ?? "";

  return (
    <Card title={config.title} style={{ height: "100%" }}>
      <p>{content}</p>
    </Card>
  );
}
