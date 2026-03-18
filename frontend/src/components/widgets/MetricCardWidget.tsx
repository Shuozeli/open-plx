import { Card, Spin, Statistic } from "antd";
import { ArrowUpOutlined, ArrowDownOutlined } from "@ant-design/icons";
import type { WidgetProps } from "./WidgetRegistry.js";
import { ComparisonDirection } from "../../gen/open_plx/v1/widget_spec_pb.js";

/** Parse format string and format a number value. */
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

  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  if (loading || !data || !spec) {
    return <Card title={config.title} style={{ height: "100%" }}><Spin /></Card>;
  }

  // Use last row for the primary value (e.g., most recent quarter)
  const lastRow = data[data.length - 1];
  const rawValue = lastRow?.[spec.value];
  const format = spec.format || "";
  const displayValue = formatValue(rawValue, format);

  // Comparison
  let prefix: React.ReactNode = null;
  let comparisonText: string | null = null;
  if (spec.comparison && data.length >= 2) {
    const prevRow = data[data.length - 2];
    const currentNum = Number(rawValue);
    const prevNum = Number(prevRow?.[spec.comparison.value] ?? prevRow?.[spec.value]);
    if (!isNaN(currentNum) && !isNaN(prevNum) && prevNum !== 0) {
      const pctChange = ((currentNum - prevNum) / prevNum) * 100;
      const isPositive = pctChange > 0;
      const isGood = spec.comparison.direction === ComparisonDirection.HIGHER_IS_BETTER
        ? isPositive
        : !isPositive;

      prefix = isPositive
        ? <ArrowUpOutlined style={{ color: isGood ? "#3f8600" : "#cf1322" }} />
        : <ArrowDownOutlined style={{ color: isGood ? "#3f8600" : "#cf1322" }} />;
      comparisonText = `${pctChange >= 0 ? "+" : ""}${pctChange.toFixed(1)}% ${spec.comparison.label}`;
    }
  }

  return (
    <Card style={{ height: "100%" }}>
      <Statistic
        title={config.title}
        value={displayValue}
        prefix={prefix}
      />
      {comparisonText && (
        <div style={{ fontSize: 12, color: "#888", marginTop: 4 }}>{comparisonText}</div>
      )}
    </Card>
  );
}
