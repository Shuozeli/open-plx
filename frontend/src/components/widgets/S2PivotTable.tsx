/**
 * S2 pivot table React wrapper. Takes S2 config and renders a PivotSheet.
 * Supports dark mode theming.
 */

import { useEffect, useRef } from "react";
import { PivotSheet } from "@antv/s2";
import type { S2DataConfig, S2Options } from "../../services/mappers/pivotMapper.js";
import { useDarkMode } from "../../hooks/useThemeContext.js";

interface S2PivotTableProps {
  dataCfg: S2DataConfig;
  options: S2Options;
}

export function S2PivotTable({ dataCfg, options }: S2PivotTableProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const sheetRef = useRef<PivotSheet | null>(null);
  const dark = useDarkMode();

  useEffect(() => {
    if (!containerRef.current) return;

    if (sheetRef.current) {
      sheetRef.current.destroy();
      sheetRef.current = null;
    }

    const sheet = new PivotSheet(
      containerRef.current,
      dataCfg as never,
      options as never,
    );

    sheet.setThemeCfg({ name: dark ? "dark" : "default" });
    sheet.render();
    sheetRef.current = sheet;

    return () => {
      if (sheetRef.current) {
        sheetRef.current.destroy();
        sheetRef.current = null;
      }
    };
  }, [dataCfg, options, dark]);

  return <div ref={containerRef} style={{ width: "100%", height: "100%" }} />;
}
