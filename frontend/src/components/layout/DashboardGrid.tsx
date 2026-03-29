import type { Dashboard } from "../../gen/open_plx/v1/dashboard_pb.js";
import type { ParamValue } from "../../gen/open_plx/v1/dashboard_pb.js";
import type { VariableValues } from "../../hooks/useVariables.js";
import { isWidgetVisible } from "../../services/evaluateVisibility.js";
import { WidgetErrorBoundary } from "./WidgetErrorBoundary.js";
import { WidgetShell } from "./WidgetShell.js";

interface DashboardGridProps {
  dashboard: Dashboard;
  variableValues?: VariableValues;
  revision?: number;
  onVariableChange?: (name: string, value: ParamValue) => void;
}

/**
 * Renders dashboard widgets in a CSS grid based on server-provided positions.
 * No drag-and-drop -- layout is declarative from config.
 */
export function DashboardGrid({ dashboard, variableValues, revision, onVariableChange }: DashboardGridProps) {
  const grid = dashboard.grid;
  const columns = grid?.columns ?? 24;
  const rowHeight = grid?.rowHeight ?? 40;
  const gap = grid?.gap ?? 8;

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: `repeat(${columns}, 1fr)`,
        gridAutoRows: `${rowHeight}px`,
        gap: `${gap}px`,
      }}
    >
      {dashboard.widgets.filter((widget) =>
        isWidgetVisible(widget.visibleWhen, variableValues)
      ).map((widget) => {
        const pos = widget.position;
        return (
          <div
            key={widget.id}
            style={{
              gridColumn: `${(pos?.x ?? 0) + 1} / span ${pos?.w ?? 6}`,
              gridRow: `${(pos?.y ?? 0) + 1} / span ${pos?.h ?? 4}`,
            }}
          >
            <WidgetErrorBoundary title={widget.title}>
              <WidgetShell
                dashboardName={dashboard.name}
                config={widget}
                variableValues={variableValues}
                revision={revision}
                onVariableChange={onVariableChange}
              />
            </WidgetErrorBoundary>
          </div>
        );
      })}
    </div>
  );
}
