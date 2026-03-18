/**
 * Test registry: exposes widget state for e2e verification.
 *
 * In dev/test mode, each widget registers its resolved state here.
 * Playwright tests read from window.__OPEN_PLX__ to verify correctness
 * without screenshot comparisons.
 *
 * This module is always imported but only populates the registry
 * when the code calls register(). Production builds can tree-shake
 * if register() calls are removed.
 */

export interface WidgetTestState {
  /** Widget ID from the dashboard config. */
  widgetId: string;
  /** Widget type (e.g., "LINE_CHART", "METRIC_CARD"). */
  widgetType: string;
  /** The proto spec converted to a plain object. */
  spec: Record<string, unknown>;
  /** Row data passed to the chart (or used by the metric card). */
  data: Record<string, unknown>[] | null;
  /** The G2 spec object passed to chart.options() (charts only). */
  g2Spec: Record<string, unknown> | null;
  /** Rendered values (metric card value, comparison, etc.). */
  rendered: Record<string, unknown>;
  /** Timestamp when state was last updated. */
  updatedAt: number;
}

export interface DashboardTestState {
  /** Dashboard name from the config. */
  name: string;
  /** Dashboard title. */
  title: string;
  /** Number of widgets. */
  widgetCount: number;
  /** Grid config. */
  grid: { columns: number; rowHeight: number; gap: number };
  /** Per-widget states keyed by widget ID. */
  widgets: Record<string, WidgetTestState>;
  /** Loading state. */
  loading: boolean;
  /** Error message if any. */
  error: string | null;
}

declare global {
  interface Window {
    __OPEN_PLX__: DashboardTestState;
  }
}

/** Initialize the test registry on the window. */
export function initTestRegistry() {
  window.__OPEN_PLX__ = {
    name: "",
    title: "",
    widgetCount: 0,
    grid: { columns: 0, rowHeight: 0, gap: 0 },
    widgets: {},
    loading: true,
    error: null,
  };
}

/** Register dashboard-level state. */
export function registerDashboard(state: Omit<DashboardTestState, "widgets" | "loading" | "error">) {
  if (!window.__OPEN_PLX__) initTestRegistry();
  window.__OPEN_PLX__.name = state.name;
  window.__OPEN_PLX__.title = state.title;
  window.__OPEN_PLX__.widgetCount = state.widgetCount;
  window.__OPEN_PLX__.grid = state.grid;
  window.__OPEN_PLX__.loading = false;
  window.__OPEN_PLX__.error = null;
}

/** Register loading/error state. */
export function registerDashboardStatus(loading: boolean, error: string | null) {
  if (!window.__OPEN_PLX__) initTestRegistry();
  window.__OPEN_PLX__.loading = loading;
  window.__OPEN_PLX__.error = error;
}

/** Register a widget's test state. */
export function registerWidget(state: WidgetTestState) {
  if (!window.__OPEN_PLX__) initTestRegistry();
  window.__OPEN_PLX__.widgets[state.widgetId] = state;
}
