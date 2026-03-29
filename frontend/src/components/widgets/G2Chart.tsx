/**
 * G2 chart React wrapper. Takes a G2 spec and renders it in a container.
 * Handles lifecycle (mount, update, destroy) and dark mode theming.
 */

import { useEffect, useRef } from "react";
import { Chart } from "@antv/g2";
import type { G2Spec } from "../../services/mappers/chartMapper.js";
import { useDarkMode } from "../../hooks/useThemeContext.js";

interface G2ChartProps {
  spec: G2Spec;
  /** Called when the user clicks a chart element (bar, point, slice, etc.). */
  onElementClick?: (record: Record<string, unknown>) => void;
}

export function G2Chart({ spec, onElementClick }: G2ChartProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<Chart | null>(null);
  const dark = useDarkMode();

  useEffect(() => {
    if (!containerRef.current) return;

    // Destroy previous chart instance
    if (chartRef.current) {
      chartRef.current.destroy();
      chartRef.current = null;
    }

    const themedSpec = {
      ...spec,
      theme: dark ? "classicDark" : "classic",
    };

    const chart = new Chart({
      container: containerRef.current,
      autoFit: true,
    });

    chart.options(themedSpec as Parameters<typeof chart.options>[0]);
    chart.render();

    if (onElementClick) {
      chart.on("element:click", (event: Record<string, unknown>) => {
        const data = event.data as Record<string, unknown> | undefined;
        if (data) {
          // G2 v5: event.data is the datum bound to the clicked mark.
          // For grouped/stacked marks, event.data.data may contain the row.
          const record = (data.data as Record<string, unknown>) ?? data;
          onElementClick(record);
        }
      });
    }

    chartRef.current = chart;

    return () => {
      if (chartRef.current) {
        chartRef.current.destroy();
        chartRef.current = null;
      }
    };
  }, [spec, dark, onElementClick]);

  return (
    <div
      ref={containerRef}
      style={{
        width: "100%",
        height: "100%",
        cursor: onElementClick ? "pointer" : undefined,
      }}
    />
  );
}
