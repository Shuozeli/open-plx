/**
 * S2 flat table React wrapper. Uses TableSheet (not PivotSheet).
 * Supports dark mode theming.
 */

import { useEffect, useRef } from "react";
import { TableSheet } from "@antv/s2";
import type { S2TableDataConfig, S2TableOptions } from "../../services/mappers/tableMapper.js";
import { useDarkMode } from "../../hooks/useThemeContext.js";

interface S2TableProps {
  dataCfg: S2TableDataConfig;
  options: S2TableOptions;
}

export function S2Table({ dataCfg, options }: S2TableProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const sheetRef = useRef<TableSheet | null>(null);
  const dark = useDarkMode();

  useEffect(() => {
    if (!containerRef.current) return;

    if (sheetRef.current) {
      sheetRef.current.destroy();
      sheetRef.current = null;
    }

    const sheet = new TableSheet(
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
