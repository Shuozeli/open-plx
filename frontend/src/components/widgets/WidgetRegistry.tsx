import type { ComponentType } from "react";
import type { WidgetConfig, ParamValue } from "../../gen/open_plx/v1/dashboard_pb.js";
import { WidgetType } from "../../gen/open_plx/v1/dashboard_pb.js";
import { LineChartWidget } from "./LineChartWidget.js";
import { BarChartWidget } from "./BarChartWidget.js";
import { PieChartWidget } from "./PieChartWidget.js";
import { PivotTableWidget } from "./PivotTableWidget.js";
import { MetricCardWidget } from "./MetricCardWidget.js";
import { TextWidget } from "./TextWidget.js";
import { ScatterChartWidget } from "./ScatterChartWidget.js";
import { HeatmapWidget } from "./HeatmapWidget.js";
import { HistogramWidget } from "./HistogramWidget.js";
import { RadarChartWidget } from "./RadarChartWidget.js";
import { TableWidget } from "./TableWidget.js";
import { GaugeWidget } from "./GaugeWidget.js";
import { FunnelWidget } from "./FunnelWidget.js";
import { BoxPlotWidget } from "./BoxPlotWidget.js";
import { TreemapWidget } from "./TreemapWidget.js";
import { SankeyWidget } from "./SankeyWidget.js";
import { WordCloudWidget } from "./WordCloudWidget.js";
import { GraphWidget } from "./GraphWidget.js";

/** Props passed to every widget renderer. */
export interface WidgetProps {
  /** The dashboard containing this widget. */
  dashboardName?: string;
  config: WidgetConfig;
  data: Record<string, unknown>[] | null;
  loading: boolean;
  error: string | null;
  /** Called when the user clicks a data element (bar, point, row, etc.).
   *  Passes the full data record of the clicked element. */
  onClickInteraction?: (record: Record<string, unknown>) => void;
  /** Called when an action result wants to set a dashboard variable. */
  onVariableChange?: (name: string, value: ParamValue) => void;
}

/** Registry mapping WidgetType enum -> React component. */
const WIDGET_REGISTRY: Partial<Record<WidgetType, ComponentType<WidgetProps>>> = {
  [WidgetType.LINE_CHART]: LineChartWidget,
  [WidgetType.BAR_CHART]: BarChartWidget,
  [WidgetType.PIE_CHART]: PieChartWidget,
  [WidgetType.PIVOT_TABLE]: PivotTableWidget,
  [WidgetType.METRIC_CARD]: MetricCardWidget,
  [WidgetType.TEXT]: TextWidget,
  [WidgetType.SCATTER_CHART]: ScatterChartWidget,
  [WidgetType.HEATMAP]: HeatmapWidget,
  [WidgetType.HISTOGRAM]: HistogramWidget,
  [WidgetType.RADAR_CHART]: RadarChartWidget,
  [WidgetType.TABLE]: TableWidget,
  [WidgetType.GAUGE]: GaugeWidget,
  [WidgetType.FUNNEL]: FunnelWidget,
  [WidgetType.BOX_PLOT]: BoxPlotWidget,
  [WidgetType.TREEMAP]: TreemapWidget,
  [WidgetType.SANKEY]: SankeyWidget,
  [WidgetType.WORD_CLOUD]: WordCloudWidget,
  [WidgetType.GRAPH]: GraphWidget,
};

export function getWidgetComponent(
  widgetType: WidgetType,
): ComponentType<WidgetProps> | null {
  return WIDGET_REGISTRY[widgetType] ?? null;
}
