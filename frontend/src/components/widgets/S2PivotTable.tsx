/**
 * S2 pivot table React wrapper. Takes S2 config and renders a PivotSheet.
 * Supports dark mode theming.
 */

import { useEffect, useRef } from "react";
import { PivotSheet, S2Event } from "@antv/s2";
import type { S2DataConfig, S2Options } from "../../services/mappers/pivotMapper.js";
import { useDarkMode } from "../../hooks/useThemeContext.js";

interface S2PivotTableProps {
  dataCfg: S2DataConfig;
  options: S2Options;
  /** Called when the user clicks a data cell row. */
  onRowClick?: (record: Record<string, unknown>) => void;
}

export function S2PivotTable({ dataCfg, options, onRowClick }: S2PivotTableProps) {
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

    if (onRowClick) {
      sheet.on(S2Event.DATA_CELL_CLICK, (event) => {
        const target = event.target as { getMeta?: () => { rowIndex?: number; data?: Record<string, unknown> } };
        const meta = target?.getMeta?.();
        if (meta?.data) {
          onRowClick(meta.data);
        } else if (meta?.rowIndex !== undefined) {
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
