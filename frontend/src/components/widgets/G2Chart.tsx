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
}

export function G2Chart({ spec }: G2ChartProps) {
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
    chartRef.current = chart;

    return () => {
      if (chartRef.current) {
        chartRef.current.destroy();
        chartRef.current = null;
      }
    };
  }, [spec, dark]);

  return <div ref={containerRef} style={{ width: "100%", height: "100%" }} />;
}
