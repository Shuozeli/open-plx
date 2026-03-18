import { Card, Spin, Statistic } from "antd";
import { ArrowUpOutlined, ArrowDownOutlined } from "@ant-design/icons";
import { useEffect } from "react";
import type { WidgetProps } from "./WidgetRegistry.js";
import { ComparisonDirection } from "../../gen/open_plx/v1/widget_spec_pb.js";
import { WidgetType } from "../../gen/open_plx/v1/dashboard_pb.js";
import { registerWidget } from "../../services/testRegistry.js";

function formatValue(value: unknown, format: string): string {
  const num = Number(value);
  if (isNaN(num)) return String(value ?? "--");

  if (format.startsWith("currency:")) {
    const currency = format.split(":")[1] ?? "USD";
    return new Intl.NumberFormat("en-US", { style: "currency", currency }).format(num);
  }
  if (format.startsWith("percent")) {
    const precision = parseInt(format.split(":")[1] ?? "1", 10);
    return `${num.toFixed(precision)}%`;
  }
  if (format.startsWith("number:")) {
    const precision = parseInt(format.split(":")[1] ?? "0", 10);
    return num.toLocaleString("en-US", { minimumFractionDigits: precision, maximumFractionDigits: precision });
  }
  if (format === "compact") {
    return new Intl.NumberFormat("en-US", { notation: "compact" }).format(num);
  }
  return String(num);
}

export function MetricCardWidget({ config, data, loading, error }: WidgetProps) {
  const spec = config.spec?.spec.case === "metricCard" ? config.spec.spec.value : null;

  const lastRow = data?.[data.length - 1];
  const rawValue = spec ? lastRow?.[spec.value] : undefined;
  const format = spec?.format ?? "";
  const displayValue = rawValue != null ? formatValue(rawValue, format) : "--";

  // Comparison calculation
  let pctChange: number | null = null;
  let comparisonLabel: string | null = null;
  let isHigherBetter = true;
  if (spec?.comparison && data && data.length >= 2) {
    const prevRow = data[data.length - 2];
    const currentNum = Number(rawValue);
    const prevNum = Number(prevRow?.[spec.comparison.value] ?? prevRow?.[spec.value]);
    if (!isNaN(currentNum) && !isNaN(prevNum) && prevNum !== 0) {
      pctChange = ((currentNum - prevNum) / prevNum) * 100;
      comparisonLabel = spec.comparison.label;
      isHigherBetter = spec.comparison.direction !== ComparisonDirection.LOWER_IS_BETTER;
    }
  }

  // Register state for e2e tests
  useEffect(() => {
    registerWidget({
      widgetId: config.id,
      widgetType: WidgetType[config.widgetType],
      spec: spec ? JSON.parse(JSON.stringify(spec)) : {},
      data,
      g2Spec: null,
      rendered: {
        rawValue: rawValue ?? null,
        displayValue,
        format,
        pctChange,
        comparisonLabel,
        isHigherBetter,
        hasData: data !== null && data.length > 0,
        rowCount: data?.length ?? 0,
      },
      updatedAt: Date.now(),
    });
  }, [config, data, spec, rawValue, displayValue, format, pctChange, comparisonLabel, isHigherBetter]);

  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  if (loading || !data || !spec) {
    return <Card title={config.title} style={{ height: "100%" }}><Spin /></Card>;
  }

  const isPositive = pctChange !== null && pctChange > 0;
  const isGood = isPositive === isHigherBetter;

  let prefix: React.ReactNode = null;
  let comparisonText: string | null = null;
  if (pctChange !== null) {
    prefix = isPositive
      ? <ArrowUpOutlined style={{ color: isGood ? "#3f8600" : "#cf1322" }} />
      : <ArrowDownOutlined style={{ color: isGood ? "#3f8600" : "#cf1322" }} />;
    comparisonText = `${pctChange >= 0 ? "+" : ""}${pctChange.toFixed(1)}% ${comparisonLabel ?? ""}`;
  }

  return (
    <Card style={{ height: "100%" }}>
      <Statistic title={config.title} value={displayValue} prefix={prefix} />
      {comparisonText && (
        <div style={{ fontSize: 12, color: "#888", marginTop: 4 }}>{comparisonText}</div>
      )}
    </Card>
  );
}
