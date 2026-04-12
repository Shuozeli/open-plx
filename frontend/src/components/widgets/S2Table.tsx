/**
 * S2 flat table React wrapper. Uses TableSheet (not PivotSheet).
 * Supports dark mode theming.
 */

import { useEffect, useRef } from "react";
import { S2Event, TableSheet } from "@antv/s2";
import type { S2TableDataConfig, S2TableOptions } from "../../services/mappers/tableMapper.js";
import { useDarkMode } from "../../hooks/useThemeContext.js";

/** Action column definition for row-action buttons. */
export interface ActionColumn {
  /** The field name of this action column. */
  field: string;
  /** The action configuration. */
  action: {
    id: string;
    label: string;
    icon?: string;
    style?: string;
    confirmMessage?: string;
    grpcCall?: {
      method: string;
      requestTemplate: string;
      resultHandling?: number;
    };
  };
}

export interface ActionInvokeResult {
  success: boolean;
  message: string;
  variableName?: string;
  variableValue?: string;
}

interface S2TableProps {
  dataCfg: S2TableDataConfig;
  options: S2TableOptions;
  /** Called when the user clicks a data row. */
  onRowClick?: (record: Record<string, unknown>) => void;
  /** Link column field for click-to-navigate. */
  linkColumnField?: string;
  /** URL template for link navigation. */
  linkTemplate?: string;
  /** Whether to open link in new tab. */
  linkNewTab?: boolean;
  /** Cell spanning config from TableSpec.colSpans. */
  colSpans?: { field: string; condition: string; colSpan: number }[];
  /** Action column definitions for row-action buttons. */
  actionColumns?: ActionColumn[];
  /** Called when an action button is clicked. */
  onActionInvoke?: (actionId: string, requestBody: string) => Promise<ActionInvokeResult>;
}

export function S2Table({ dataCfg, options, onRowClick, linkColumnField, linkTemplate, linkNewTab, colSpans, actionColumns, onActionInvoke }: S2TableProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const sheetRef = useRef<TableSheet | null>(null);
  const dark = useDarkMode();

  const exportOptions = options.export as { filename?: string; formats?: string[] } | undefined;

  const handleExportCsv = () => {
    if (sheetRef.current) {
      // @ts-ignore: exportFile not in S2 public types
      sheetRef.current.exportFile("csv", exportOptions?.filename || "export");
    }
  };

  const handleExportExcel = () => {
    if (sheetRef.current) {
      // @ts-ignore: exportFile not in S2 public types
      sheetRef.current.exportFile("xlsx", exportOptions?.filename || "export");
    }
  };

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

    // Cell spanning: apply colSpan config
    // TODO(refactor): S2 colSpan API - cell spanning in TableSheet requires custom cell layout
    // or pre-computed colSpan data. The colSpans array is stored on the sheet instance for
    // custom cell renderers to reference. Full implementation requires S2 customCell callback.
    if (colSpans?.length) {
      // @ts-ignore: __colSpans is internal S2 TableSheet property
      sheet.__colSpans = colSpans;
    }

    if (onRowClick || linkColumnField || (actionColumns?.length && onActionInvoke)) {
      sheet.on(S2Event.DATA_CELL_CLICK, (event) => {
        const target = event.target as { getMeta?: () => { rowIndex?: number; field?: string } };
        const meta = target?.getMeta?.();
        if (meta?.rowIndex !== undefined) {
          const rowData = dataCfg.data[meta.rowIndex];
          if (rowData) {
            // Check if this is an action column click
            if (actionColumns?.length && onActionInvoke) {
              const actionCol = actionColumns.find(ac => ac.field === meta.field);
              if (actionCol) {
                const action = actionCol.action;
                // Show confirmation dialog if configured
                if (action.confirmMessage && !window.confirm(action.confirmMessage)) {
                  return;
                }
                // Build request body from template
                const requestBody = action.grpcCall?.requestTemplate
                  .replace(/\{row\.(\w+)\}/g, (_, field) => String(rowData[field] ?? ""))
                  ?? "{}";
                onActionInvoke(action.id, requestBody);
                return;
              }
            }
            // Check if this is a link column click
            if (linkColumnField && meta.field === linkColumnField) {
              const value = rowData[linkColumnField];
              let url = (linkTemplate || "{value}").replace(/\{value\}/g, String(value ?? ""));
              // Also support {row.field} interpolation
              url = url.replace(/\{row\.(\w+)\}/g, (_, field) => String(rowData[field] ?? ""));
              window.open(url, linkNewTab ? "_blank" : "_self");
              return;
            }
            if (onRowClick) {
              onRowClick(rowData as Record<string, unknown>);
            }
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
  }, [dataCfg, options, dark, onRowClick, colSpans, actionColumns, onActionInvoke]);

  return (
    <div style={{ display: "flex", flexDirection: "column", width: "100%", height: "100%" }}>
      {exportOptions && (
        <div
          style={{
            padding: "4px 8px",
            borderBottom: "1px solid #f0f0f0",
            display: "flex",
            gap: 8,
            flexShrink: 0,
          }}
        >
          {exportOptions.formats?.includes("csv") && (
            <button onClick={handleExportCsv} type="button">
              Export CSV
            </button>
          )}
          {exportOptions.formats?.includes("xlsx") && (
            <button onClick={handleExportExcel} type="button">
              Export Excel
            </button>
          )}
        </div>
      )}
      <div
        ref={containerRef}
        style={{
          flex: 1,
          cursor: onRowClick ? "pointer" : undefined,
        }}
      />
    </div>
  );
}
