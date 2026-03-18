import type { Dashboard } from "../../gen/open_plx/v1/dashboard_pb.js";
import { WidgetShell } from "./WidgetShell.js";

interface DashboardGridProps {
  dashboard: Dashboard;
}

/**
 * Renders dashboard widgets in a CSS grid based on server-provided positions.
 * No drag-and-drop -- layout is declarative from config.
 */
export function DashboardGrid({ dashboard }: DashboardGridProps) {
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
      {dashboard.widgets.map((widget) => {
        const pos = widget.position;
        return (
          <div
            key={widget.id}
            style={{
              gridColumn: `${(pos?.x ?? 0) + 1} / span ${pos?.w ?? 6}`,
              gridRow: `${(pos?.y ?? 0) + 1} / span ${pos?.h ?? 4}`,
            }}
          >
            <WidgetShell dashboardName={dashboard.name} config={widget} />
          </div>
        );
      })}
    </div>
  );
}
