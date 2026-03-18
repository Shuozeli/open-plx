import type { ComponentType } from "react";
import type { WidgetConfig } from "../../gen/open_plx/v1/dashboard_pb.js";
import { WidgetType } from "../../gen/open_plx/v1/dashboard_pb.js";
import { LineChartWidget } from "./LineChartWidget.js";
import { BarChartWidget } from "./BarChartWidget.js";
import { PieChartWidget } from "./PieChartWidget.js";
import { PivotTableWidget } from "./PivotTableWidget.js";
import { MetricCardWidget } from "./MetricCardWidget.js";
import { TextWidget } from "./TextWidget.js";

/** Props passed to every widget renderer. */
export interface WidgetProps {
  config: WidgetConfig;
  data: Record<string, unknown>[] | null;
  loading: boolean;
  error: string | null;
}

/** Registry mapping WidgetType enum -> React component. */
const WIDGET_REGISTRY: Partial<Record<WidgetType, ComponentType<WidgetProps>>> = {
  [WidgetType.LINE_CHART]: LineChartWidget,
  [WidgetType.BAR_CHART]: BarChartWidget,
  [WidgetType.PIE_CHART]: PieChartWidget,
  [WidgetType.PIVOT_TABLE]: PivotTableWidget,
  [WidgetType.METRIC_CARD]: MetricCardWidget,
  [WidgetType.TEXT]: TextWidget,
};

export function getWidgetComponent(
  widgetType: WidgetType,
): ComponentType<WidgetProps> | null {
  return WIDGET_REGISTRY[widgetType] ?? null;
}
