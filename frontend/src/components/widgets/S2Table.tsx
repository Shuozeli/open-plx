/**
 * S2 flat table React wrapper. Uses TableSheet (not PivotSheet).
 * Supports dark mode theming.
 */

import { useEffect, useRef } from "react";
import { S2Event, TableSheet } from "@antv/s2";
import type { S2TableDataConfig, S2TableOptions } from "../../services/mappers/tableMapper.js";
import { useDarkMode } from "../../hooks/useThemeContext.js";

interface S2TableProps {
  dataCfg: S2TableDataConfig;
  options: S2TableOptions;
  /** Called when the user clicks a data row. */
  onRowClick?: (record: Record<string, unknown>) => void;
}

export function S2Table({ dataCfg, options, onRowClick }: S2TableProps) {
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

    if (onRowClick) {
      sheet.on(S2Event.DATA_CELL_CLICK, (event) => {
        const target = event.target as { getMeta?: () => { rowIndex?: number } };
        const meta = target?.getMeta?.();
        if (meta?.rowIndex !== undefined) {
          const rowData = dataCfg.data[meta.rowIndex];
          if (rowData) {
            onRowClick(rowData as Record<string, unknown>);
          }
        }
      });
    }

    sheetRef.current = sheet;

    return () => {
      if (sheetRef.current) {
        sheetRef.current.destroy();
        sheetRef.current = null;
      }
    };
  }, [dataCfg, options, dark, onRowClick]);

  return (
    <div
      ref={containerRef}
      style={{
        width: "100%",
        height: "100%",
        cursor: onRowClick ? "pointer" : undefined,
      }}
    />
  );
}
