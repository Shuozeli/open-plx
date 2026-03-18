/**
 * G2 chart React wrapper. Takes a G2 spec and renders it in a container.
 * Handles lifecycle (mount, update, destroy).
 */

import { useEffect, useRef } from "react";
import { Chart } from "@antv/g2";
import type { G2Spec } from "../../services/mappers/chartMapper.js";

interface G2ChartProps {
  spec: G2Spec;
}

export function G2Chart({ spec }: G2ChartProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<Chart | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // Destroy previous chart instance
    if (chartRef.current) {
      chartRef.current.destroy();
      chartRef.current = null;
    }

    const chart = new Chart({
      container: containerRef.current,
      autoFit: true,
    });

    // Cast to G2's internal spec type. Our G2Spec is a curated subset
    // produced by the mapper layer -- safe to pass through.
    chart.options(spec as Parameters<typeof chart.options>[0]);
    chart.render();
    chartRef.current = chart;

    return () => {
      if (chartRef.current) {
        chartRef.current.destroy();
        chartRef.current = null;
      }
    };
  }, [spec]);

  return <div ref={containerRef} style={{ width: "100%", height: "100%" }} />;
}
