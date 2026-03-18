/**
 * E2E test helpers for reading widget state from the test registry.
 *
 * Usage: const state = await getWidgetState(page, "revenue-kpi");
 *        expect(state.rendered.displayValue).toBe("$2,100,000.00");
 */

import type { Page } from "@playwright/test";

export const BASE_URL = process.env.E2E_BASE_URL ?? "http://10.0.0.183:5199";

export interface WidgetTestState {
  widgetId: string;
  widgetType: string;
  spec: Record<string, unknown>;
  data: Record<string, unknown>[] | null;
  g2Spec: Record<string, unknown> | null;
  rendered: Record<string, unknown>;
  updatedAt: number;
}

export interface DashboardTestState {
  name: string;
  title: string;
  widgetCount: number;
  grid: { columns: number; rowHeight: number; gap: number };
  widgets: Record<string, WidgetTestState>;
  loading: boolean;
  error: string | null;
}

/** Wait for the dashboard to fully load and all widget data to arrive. */
export async function waitForDashboardReady(page: Page, expectedWidgets: number = 3) {
  await page.goto(BASE_URL);

  // Wait until all widgets are registered AND have data loaded
  await page.waitForFunction(
    (count) => {
      const state = (window as unknown as { __OPEN_PLX__: DashboardTestState }).__OPEN_PLX__;
      if (!state || state.loading) return false;
      const widgets = Object.values(state.widgets);
      if (widgets.length < count) return false;
      // Wait for all widgets to have data (rendered.hasData === true)
      return widgets.every((w) => w.rendered.hasData === true);
    },
    expectedWidgets,
    { timeout: 15_000 },
  );
}

/** Read the full dashboard state from the test registry. */
export async function getDashboardState(page: Page): Promise<DashboardTestState> {
  return page.evaluate(() => {
    return JSON.parse(JSON.stringify(
      (window as unknown as { __OPEN_PLX__: unknown }).__OPEN_PLX__,
    ));
  });
}

/** Read a single widget's state. */
export async function getWidgetState(page: Page, widgetId: string): Promise<WidgetTestState> {
  return page.evaluate((id) => {
    const state = (window as unknown as { __OPEN_PLX__: DashboardTestState }).__OPEN_PLX__;
    const widget = state.widgets[id];
    if (!widget) throw new Error(`Widget "${id}" not found in test registry`);
    return JSON.parse(JSON.stringify(widget));
  }, widgetId);
}

/** Get all widget IDs from the test registry. */
export async function getWidgetIds(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const state = (window as unknown as { __OPEN_PLX__: DashboardTestState }).__OPEN_PLX__;
    return Object.keys(state.widgets);
  });
}
