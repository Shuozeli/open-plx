import { test, expect } from "@playwright/test";
import {
  BASE_URL,
  waitForDashboardReady,
  getDashboardState,
  getWidgetState,
  getWidgetIds,
} from "./helpers.js";

// =============================================================================
// DASHBOARD STATE
// =============================================================================

test.describe("Dashboard State", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboardReady(page);
  });

  test("loads the demo dashboard", async ({ page }) => {
    const state = await getDashboardState(page);
    expect(state.name).toBe("dashboards/demo");
    expect(state.title).toBe("Demo Dashboard");
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  test("has correct grid configuration", async ({ page }) => {
    const state = await getDashboardState(page);
    expect(state.grid.columns).toBe(24);
    expect(state.grid.rowHeight).toBe(40);
    expect(state.grid.gap).toBe(8);
  });

  test("reports 3 widgets", async ({ page }) => {
    const state = await getDashboardState(page);
    expect(state.widgetCount).toBe(3);
  });

  test("all 3 widgets are registered in test registry", async ({ page }) => {
    const ids = await getWidgetIds(page);
    expect(ids.sort()).toEqual(["cost-bar", "revenue-kpi", "revenue-trend"]);
  });
});

// =============================================================================
// METRIC CARD: revenue-kpi
// =============================================================================

test.describe("Widget: revenue-kpi (Metric Card)", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboardReady(page);
  });

  test("has correct widget type", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    expect(w.widgetType).toBe("METRIC_CARD");
  });

  test("has data with 4 rows (Q1-Q4)", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    expect(w.rendered.hasData).toBe(true);
    expect(w.rendered.rowCount).toBe(4);
  });

  test("displays Q4 revenue as raw value 2100000", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    expect(w.rendered.rawValue).toBe(2100000);
  });

  test("formats value as $2,100,000.00", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    expect(w.rendered.displayValue).toBe("$2,100,000.00");
  });

  test("uses currency:USD format", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    expect(w.rendered.format).toBe("currency:USD");
  });

  test("spec has value field = 'revenue'", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    expect(w.spec.value).toBe("revenue");
  });

  test("data contains expected columns", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    expect(w.data).not.toBeNull();
    const firstRow = w.data![0];
    expect(firstRow).toHaveProperty("quarter");
    expect(firstRow).toHaveProperty("revenue");
    expect(firstRow).toHaveProperty("cost");
    expect(firstRow).toHaveProperty("units_sold");
  });

  test("data has correct Q1-Q4 quarter values", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    const quarters = w.data!.map((r) => r.quarter);
    expect(quarters).toEqual(["Q1", "Q2", "Q3", "Q4"]);
  });

  test("data has correct revenue values", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    const revenues = w.data!.map((r) => r.revenue);
    expect(revenues).toEqual([1200000, 1500000, 1800000, 2100000]);
  });
});

// =============================================================================
// LINE CHART: revenue-trend
// =============================================================================

test.describe("Widget: revenue-trend (Line Chart)", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboardReady(page);
  });

  test("has correct widget type", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    expect(w.widgetType).toBe("LINE_CHART");
  });

  test("has data with 4 rows", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    expect(w.rendered.hasData).toBe(true);
    expect(w.rendered.rowCount).toBe(4);
  });

  test("G2 spec type is 'line'", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    expect(w.g2Spec).not.toBeNull();
    expect(w.g2Spec!.type).toBe("line");
  });

  test("G2 spec encodes x='quarter' y='revenue'", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    expect(w.rendered.encodeX).toBe("quarter");
    expect(w.rendered.encodeY).toBe("revenue");
  });

  test("G2 spec has autoFit enabled", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    expect(w.g2Spec!.autoFit).toBe(true);
  });

  test("G2 spec has axis config with titles", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    const axis = w.g2Spec!.axis as Record<string, unknown>;
    expect(axis).toBeDefined();

    const xAxis = axis.x as Record<string, unknown>;
    const yAxis = axis.y as Record<string, unknown>;
    expect(xAxis.title).toBe("Quarter");
    expect(yAxis.title).toBe("Revenue ($)");
  });

  test("G2 spec y-axis has dollar formatter", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    const axis = w.g2Spec!.axis as Record<string, unknown>;
    const yAxis = axis.y as Record<string, unknown>;
    expect(yAxis.labelFormatter).toBe("$~s");
  });

  test("G2 spec data matches the static source", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    const data = w.g2Spec!.data as Record<string, unknown>[];
    expect(data.length).toBe(4);
    expect(data[0]).toEqual({ quarter: "Q1", revenue: 1200000, cost: 800000, units_sold: 12000 });
    expect(data[3]).toEqual({ quarter: "Q4", revenue: 2100000, cost: 1100000, units_sold: 21000 });
  });

  test("proto spec chart_type maps to LINE", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    // ChartType.LINE = 1
    expect(w.spec.chartType).toBe(1);
  });

  test("proto spec data_mapping has correct fields", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    const dm = w.spec.dataMapping as Record<string, unknown>;
    expect(dm.x).toBe("quarter");
    expect(dm.y).toBe("revenue");
  });
});

// =============================================================================
// BAR CHART: cost-bar
// =============================================================================

test.describe("Widget: cost-bar (Bar Chart)", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboardReady(page);
  });

  test("has correct widget type", async ({ page }) => {
    const w = await getWidgetState(page, "cost-bar");
    expect(w.widgetType).toBe("BAR_CHART");
  });

  test("has data with 4 rows", async ({ page }) => {
    const w = await getWidgetState(page, "cost-bar");
    expect(w.rendered.hasData).toBe(true);
    expect(w.rendered.rowCount).toBe(4);
  });

  test("G2 spec type is 'interval'", async ({ page }) => {
    const w = await getWidgetState(page, "cost-bar");
    expect(w.g2Spec).not.toBeNull();
    expect(w.g2Spec!.type).toBe("interval");
  });

  test("G2 spec encodes x='quarter' y='cost'", async ({ page }) => {
    const w = await getWidgetState(page, "cost-bar");
    expect(w.rendered.encodeX).toBe("quarter");
    expect(w.rendered.encodeY).toBe("cost");
  });

  test("G2 spec has y-axis config", async ({ page }) => {
    const w = await getWidgetState(page, "cost-bar");
    const axis = w.g2Spec!.axis as Record<string, unknown>;
    const yAxis = axis.y as Record<string, unknown>;
    expect(yAxis.title).toBe("Cost ($)");
    expect(yAxis.labelFormatter).toBe("$~s");
  });

  test("G2 spec data has correct cost values", async ({ page }) => {
    const w = await getWidgetState(page, "cost-bar");
    const data = w.g2Spec!.data as Record<string, unknown>[];
    const costs = data.map((r) => r.cost);
    expect(costs).toEqual([800000, 900000, 1000000, 1100000]);
  });

  test("proto spec chart_type maps to BAR", async ({ page }) => {
    const w = await getWidgetState(page, "cost-bar");
    // ChartType.BAR = 2
    expect(w.spec.chartType).toBe(2);
  });
});

// =============================================================================
// DATA FLOW INTEGRITY
// =============================================================================

test.describe("Data Flow Integrity", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboardReady(page);
  });

  test("all widgets share the same data source data", async ({ page }) => {
    const kpi = await getWidgetState(page, "revenue-kpi");
    const line = await getWidgetState(page, "revenue-trend");
    const bar = await getWidgetState(page, "cost-bar");

    // All 3 widgets use dataSources/demo-static -> same row count
    expect(kpi.data!.length).toBe(4);
    expect(line.data!.length).toBe(4);
    expect(bar.data!.length).toBe(4);

    // Same data (all reference the same static source)
    expect(kpi.data).toEqual(line.data);
    expect(line.data).toEqual(bar.data);
  });

  test("static data has expected schema (4 columns)", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    const columns = Object.keys(w.data![0]);
    expect(columns.sort()).toEqual(["cost", "quarter", "revenue", "units_sold"]);
  });

  test("revenue values are monotonically increasing", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    const revenues = w.data!.map((r) => r.revenue as number);
    for (let i = 1; i < revenues.length; i++) {
      expect(revenues[i]).toBeGreaterThan(revenues[i - 1]);
    }
  });

  test("cost values are monotonically increasing", async ({ page }) => {
    const w = await getWidgetState(page, "cost-bar");
    const costs = w.data!.map((r) => r.cost as number);
    for (let i = 1; i < costs.length; i++) {
      expect(costs[i]).toBeGreaterThan(costs[i - 1]);
    }
  });
});

// =============================================================================
// G2 SPEC CORRECTNESS
// =============================================================================

test.describe("G2 Spec Correctness", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboardReady(page);
  });

  test("line chart has no transforms (no stacking)", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    // No stack_mode set -> no transforms
    expect(w.g2Spec!.transform).toBeUndefined();
  });

  test("bar chart has no transforms (no stacking)", async ({ page }) => {
    const w = await getWidgetState(page, "cost-bar");
    expect(w.g2Spec!.transform).toBeUndefined();
  });

  test("line chart has no coordinate override (cartesian default)", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-trend");
    expect(w.g2Spec!.coordinate).toBeUndefined();
  });

  test("bar chart has no coordinate override (cartesian default)", async ({ page }) => {
    const w = await getWidgetState(page, "cost-bar");
    expect(w.g2Spec!.coordinate).toBeUndefined();
  });

  test("metric card has no G2 spec (not a chart)", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-kpi");
    expect(w.g2Spec).toBeNull();
  });
});

// =============================================================================
// UI LAYOUT (DOM-based)
// =============================================================================

test.describe("UI Layout", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboardReady(page);
  });

  test("header renders 'open-plx'", async ({ page }) => {
    await expect(page.locator(".ant-layout-header h4")).toHaveText("open-plx");
  });

  test("dashboard title is an h2", async ({ page }) => {
    await expect(page.getByRole("heading", { level: 2, name: "Demo Dashboard" })).toBeVisible();
  });

  test("grid has 3 children", async ({ page }) => {
    const grid = page.locator("div[style*='display: grid']");
    await expect(grid.locator("> div")).toHaveCount(3);
  });

  test("metric card position: col 1/span 8, row 1/span 4", async ({ page }) => {
    const cell = page.locator("div[style*='display: grid'] > div").first();
    expect(await cell.evaluate((el) => el.style.gridColumn)).toBe("1 / span 8");
    expect(await cell.evaluate((el) => el.style.gridRow)).toBe("1 / span 4");
  });

  test("line chart position: col 1/span 16, row 5/span 8", async ({ page }) => {
    const cell = page.locator("div[style*='display: grid'] > div").nth(1);
    expect(await cell.evaluate((el) => el.style.gridColumn)).toBe("1 / span 16");
    expect(await cell.evaluate((el) => el.style.gridRow)).toBe("5 / span 8");
  });

  test("bar chart position: col 17/span 8, row 5/span 8", async ({ page }) => {
    const cell = page.locator("div[style*='display: grid'] > div").nth(2);
    expect(await cell.evaluate((el) => el.style.gridColumn)).toBe("17 / span 8");
    expect(await cell.evaluate((el) => el.style.gridRow)).toBe("5 / span 8");
  });

  test("charts are side by side (same y position)", async ({ page }) => {
    const lineCard = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });
    const barCard = page.locator(".ant-card").filter({ hasText: "Cost by Quarter" });
    const lineBox = await lineCard.boundingBox();
    const barBox = await barCard.boundingBox();
    expect(Math.abs(lineBox!.y - barBox!.y)).toBeLessThan(10);
  });

  test("line chart is ~2x wider than bar chart", async ({ page }) => {
    const lineCard = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });
    const barCard = page.locator(".ant-card").filter({ hasText: "Cost by Quarter" });
    const lineBox = await lineCard.boundingBox();
    const barBox = await barCard.boundingBox();
    expect(lineBox!.width).toBeGreaterThan(barBox!.width * 1.5);
  });

  test("metric card Statistic shows the value", async ({ page }) => {
    const stat = page.locator(".ant-statistic").filter({ hasText: "Total Revenue" });
    await expect(stat.locator(".ant-statistic-content-value")).toContainText("$2,100,000.00");
  });

  test("each chart card has a canvas", async ({ page }) => {
    const lineCard = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });
    const barCard = page.locator(".ant-card").filter({ hasText: "Cost by Quarter" });
    await expect(lineCard.locator("canvas")).toBeVisible();
    await expect(barCard.locator("canvas")).toBeVisible();
  });

  test("no error alerts on page", async ({ page }) => {
    await expect(page.locator(".ant-alert-error")).toHaveCount(0);
  });

  test("refresh button works", async ({ page }) => {
    await page.getByRole("button", { name: "Refresh" }).click();
    // After refresh, state should still be valid
    await waitForDashboardReady(page);
    const state = await getDashboardState(page);
    expect(state.error).toBeNull();
    expect(state.widgetCount).toBe(3);
  });
});

// =============================================================================
// NO ERRORS
// =============================================================================

test.describe("Error Free", () => {
  test("no console errors during load", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));
    await waitForDashboardReady(page);
    const realErrors = errors.filter((e) => !e.includes("deprecated") && !e.includes("Warning:"));
    expect(realErrors).toHaveLength(0);
  });

  test("no failed network requests", async ({ page }) => {
    const failed: string[] = [];
    page.on("response", (r) => { if (r.status() >= 400) failed.push(`${r.status()} ${r.url()}`); });
    await waitForDashboardReady(page);
    expect(failed).toHaveLength(0);
  });

  test("test registry has no error state", async ({ page }) => {
    await waitForDashboardReady(page);
    const state = await getDashboardState(page);
    expect(state.error).toBeNull();
    expect(state.loading).toBe(false);
  });
});
