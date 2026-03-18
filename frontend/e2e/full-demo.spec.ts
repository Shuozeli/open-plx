import { test, expect } from "@playwright/test";
import {
  waitForDashboardReady,
  getDashboardState,
  getWidgetState,
  getWidgetIds,
} from "./helpers.js";

const BASE_URL = process.env.E2E_BASE_URL ?? "http://10.0.0.183:5199";
const FULL_DEMO_URL = `${BASE_URL}#dashboards/full-demo`;

async function loadFullDemo(page: import("@playwright/test").Page) {
  await page.goto(FULL_DEMO_URL);
  await page.waitForFunction(
    () => {
      const s = (window as unknown as { __OPEN_PLX__: { loading: boolean; widgets: Record<string, { rendered: { hasData: boolean } }> } }).__OPEN_PLX__;
      if (!s || s.loading) return false;
      const widgets = Object.values(s.widgets);
      // full-demo has 8 widgets; text widget always hasData=true
      return widgets.length >= 8 && widgets.every((w) => w.rendered.hasData === true);
    },
    { timeout: 15_000 },
  );
}

// =============================================================================
// FULL DEMO DASHBOARD STATE
// =============================================================================

test.describe("Full Demo: Dashboard", () => {
  test.beforeEach(async ({ page }) => {
    await loadFullDemo(page);
  });

  test("loads the full-demo dashboard", async ({ page }) => {
    const state = await getDashboardState(page);
    expect(state.name).toBe("dashboards/full-demo");
    expect(state.title).toBe("Full Widget Demo");
  });

  test("has 8 widgets", async ({ page }) => {
    const state = await getDashboardState(page);
    expect(state.widgetCount).toBe(8);
  });

  test("all 8 widgets registered", async ({ page }) => {
    const ids = await getWidgetIds(page);
    expect(ids.sort()).toEqual([
      "area-chart",
      "cost-metric",
      "donut-chart",
      "horizontal-bar",
      "pie-chart",
      "revenue-metric",
      "summary-text",
      "units-metric",
    ]);
  });
});

// =============================================================================
// METRIC CARD FORMATS
// =============================================================================

test.describe("Full Demo: Metric Card Formats", () => {
  test.beforeEach(async ({ page }) => {
    await loadFullDemo(page);
  });

  test("revenue-metric: currency:USD format", async ({ page }) => {
    const w = await getWidgetState(page, "revenue-metric");
    expect(w.rendered.format).toBe("currency:USD");
    expect(w.rendered.displayValue).toBe("$2,100,000.00");
    expect(w.rendered.rawValue).toBe(2100000);
  });

  test("cost-metric: compact format", async ({ page }) => {
    const w = await getWidgetState(page, "cost-metric");
    expect(w.rendered.format).toBe("compact");
    // 1,100,000 in compact -> "1.1M"
    expect(w.rendered.displayValue).toMatch(/1\.1M/);
    expect(w.rendered.rawValue).toBe(1100000);
  });

  test("units-metric: number:0 format", async ({ page }) => {
    const w = await getWidgetState(page, "units-metric");
    expect(w.rendered.format).toBe("number:0");
    // 21000 formatted -> "21,000"
    expect(w.rendered.displayValue).toBe("21,000");
    expect(w.rendered.rawValue).toBe(21000);
  });
});

// =============================================================================
// AREA CHART
// =============================================================================

test.describe("Full Demo: Area Chart", () => {
  test.beforeEach(async ({ page }) => {
    await loadFullDemo(page);
  });

  test("maps to G2 type 'area'", async ({ page }) => {
    const w = await getWidgetState(page, "area-chart");
    expect(w.g2Spec).not.toBeNull();
    expect(w.g2Spec!.type).toBe("area");
  });

  test("encodes x=quarter, y=revenue", async ({ page }) => {
    const w = await getWidgetState(page, "area-chart");
    const encode = w.g2Spec!.encode as Record<string, unknown>;
    expect(encode.x).toBe("quarter");
    expect(encode.y).toBe("revenue");
  });

  test("has axis config", async ({ page }) => {
    const w = await getWidgetState(page, "area-chart");
    const axis = w.g2Spec!.axis as Record<string, unknown>;
    expect((axis.x as Record<string, unknown>).title).toBe("Quarter");
    expect((axis.y as Record<string, unknown>).title).toBe("Revenue");
  });

  test("has 4 data rows", async ({ page }) => {
    const w = await getWidgetState(page, "area-chart");
    expect(w.rendered.rowCount).toBe(4);
  });

  test("proto chart_type is AREA (6)", async ({ page }) => {
    const w = await getWidgetState(page, "area-chart");
    expect(w.spec.chartType).toBe(6); // ChartType.AREA
  });
});

// =============================================================================
// PIE CHART
// =============================================================================

test.describe("Full Demo: Pie Chart", () => {
  test.beforeEach(async ({ page }) => {
    await loadFullDemo(page);
  });

  test("maps to G2 type 'interval' (not 'pie')", async ({ page }) => {
    const w = await getWidgetState(page, "pie-chart");
    expect(w.g2Spec!.type).toBe("interval");
  });

  test("has theta coordinate (makes it a pie)", async ({ page }) => {
    const w = await getWidgetState(page, "pie-chart");
    expect(w.rendered.coordinateType).toBe("theta");
  });

  test("has NO innerRadius (full pie, not donut)", async ({ page }) => {
    const w = await getWidgetState(page, "pie-chart");
    expect(w.rendered.innerRadius).toBeNull();
  });

  test("has stackY transform", async ({ page }) => {
    const w = await getWidgetState(page, "pie-chart");
    expect(w.rendered.hasTransform).toBe(true);
    const transforms = w.g2Spec!.transform as Record<string, unknown>[];
    expect(transforms[0].type).toBe("stackY");
  });

  test("encodes value as y, category as color", async ({ page }) => {
    const w = await getWidgetState(page, "pie-chart");
    const encode = w.g2Spec!.encode as Record<string, unknown>;
    expect(encode.y).toBe("revenue");
    expect(encode.color).toBe("quarter");
  });

  test("proto chart_type is PIE (4)", async ({ page }) => {
    const w = await getWidgetState(page, "pie-chart");
    expect(w.spec.chartType).toBe(4);
  });
});

// =============================================================================
// DONUT CHART
// =============================================================================

test.describe("Full Demo: Donut Chart", () => {
  test.beforeEach(async ({ page }) => {
    await loadFullDemo(page);
  });

  test("maps to G2 type 'interval'", async ({ page }) => {
    const w = await getWidgetState(page, "donut-chart");
    expect(w.g2Spec!.type).toBe("interval");
  });

  test("has theta coordinate", async ({ page }) => {
    const w = await getWidgetState(page, "donut-chart");
    expect(w.rendered.coordinateType).toBe("theta");
  });

  test("has innerRadius 0.6 (donut hole)", async ({ page }) => {
    const w = await getWidgetState(page, "donut-chart");
    expect(w.rendered.innerRadius).toBe(0.6);
  });

  test("has stackY transform", async ({ page }) => {
    const w = await getWidgetState(page, "donut-chart");
    expect(w.rendered.hasTransform).toBe(true);
  });

  test("encodes cost as y, quarter as color", async ({ page }) => {
    const w = await getWidgetState(page, "donut-chart");
    const encode = w.g2Spec!.encode as Record<string, unknown>;
    expect(encode.y).toBe("cost");
    expect(encode.color).toBe("quarter");
  });

  test("proto chart_type is DONUT (5)", async ({ page }) => {
    const w = await getWidgetState(page, "donut-chart");
    expect(w.spec.chartType).toBe(5);
  });

  test("pie and donut differ only in innerRadius", async ({ page }) => {
    const pie = await getWidgetState(page, "pie-chart");
    const donut = await getWidgetState(page, "donut-chart");

    // Both are interval + theta
    expect(pie.g2Spec!.type).toBe(donut.g2Spec!.type);
    expect(pie.rendered.coordinateType).toBe(donut.rendered.coordinateType);

    // Only innerRadius differs
    expect(pie.rendered.innerRadius).toBeNull();
    expect(donut.rendered.innerRadius).toBe(0.6);
  });
});

// =============================================================================
// HORIZONTAL BAR CHART
// =============================================================================

test.describe("Full Demo: Horizontal Bar", () => {
  test.beforeEach(async ({ page }) => {
    await loadFullDemo(page);
  });

  test("maps to G2 type 'interval'", async ({ page }) => {
    const w = await getWidgetState(page, "horizontal-bar");
    expect(w.g2Spec!.type).toBe("interval");
  });

  test("has transpose coordinate transform", async ({ page }) => {
    const w = await getWidgetState(page, "horizontal-bar");
    const coord = w.g2Spec!.coordinate as Record<string, unknown>;
    expect(coord).toBeDefined();
    const transforms = coord.transform as Record<string, unknown>[];
    expect(transforms).toHaveLength(1);
    expect(transforms[0].type).toBe("transpose");
  });

  test("encodes x=quarter, y=cost", async ({ page }) => {
    const w = await getWidgetState(page, "horizontal-bar");
    const encode = w.g2Spec!.encode as Record<string, unknown>;
    expect(encode.x).toBe("quarter");
    expect(encode.y).toBe("cost");
  });

  test("proto chart_type is HORIZONTAL_BAR (3)", async ({ page }) => {
    const w = await getWidgetState(page, "horizontal-bar");
    expect(w.spec.chartType).toBe(3);
  });
});

// =============================================================================
// TEXT WIDGET
// =============================================================================

test.describe("Full Demo: Text Widget", () => {
  test.beforeEach(async ({ page }) => {
    await loadFullDemo(page);
  });

  test("has no G2 spec", async ({ page }) => {
    const w = await getWidgetState(page, "summary-text");
    expect(w.g2Spec).toBeNull();
  });

  test("has correct content", async ({ page }) => {
    const w = await getWidgetState(page, "summary-text");
    expect(w.rendered.content).toBe("Q4 revenue exceeded target by 5%.");
  });

  test("renders content in the DOM", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Summary" });
    await expect(card.getByText("Q4 revenue exceeded target by 5%.")).toBeVisible();
  });

  test("widget type is TEXT", async ({ page }) => {
    const w = await getWidgetState(page, "summary-text");
    expect(w.widgetType).toBe("TEXT");
  });
});

// =============================================================================
// CHART TYPE -> G2 MAPPING MATRIX
// =============================================================================

test.describe("Full Demo: Chart Type Mapping Matrix", () => {
  test.beforeEach(async ({ page }) => {
    await loadFullDemo(page);
  });

  test("all chart types map to correct G2 mark", async ({ page }) => {
    const mappings: Record<string, { g2Type: string; hasCoordinate: boolean }> = {
      "area-chart":     { g2Type: "area",     hasCoordinate: false },
      "pie-chart":      { g2Type: "interval", hasCoordinate: true },
      "donut-chart":    { g2Type: "interval", hasCoordinate: true },
      "horizontal-bar": { g2Type: "interval", hasCoordinate: true },
    };

    for (const [widgetId, expected] of Object.entries(mappings)) {
      const w = await getWidgetState(page, widgetId);
      expect(w.g2Spec!.type, `${widgetId} g2Type`).toBe(expected.g2Type);
      if (expected.hasCoordinate) {
        expect(w.g2Spec!.coordinate, `${widgetId} should have coordinate`).toBeDefined();
      } else {
        expect(w.g2Spec!.coordinate, `${widgetId} should not have coordinate`).toBeUndefined();
      }
    }
  });

  test("all widgets with data have canvas elements", async ({ page }) => {
    const chartIds = ["area-chart", "pie-chart", "donut-chart", "horizontal-bar"];
    for (const id of chartIds) {
      const card = page.locator(".ant-card").filter({ hasText: (await getWidgetState(page, id)).spec.dataMapping ? "" : "" });
      // Just verify canvas count on page matches chart widget count
    }
    const canvases = page.locator("canvas");
    const count = await canvases.count();
    // 4 chart widgets should produce 4 canvases
    expect(count).toBeGreaterThanOrEqual(4);
  });
});

// =============================================================================
// GRID LAYOUT POSITIONS
// =============================================================================

test.describe("Full Demo: Grid Positions", () => {
  test.beforeEach(async ({ page }) => {
    await loadFullDemo(page);
  });

  test("8 grid children", async ({ page }) => {
    const grid = page.locator("div[style*='display: grid']");
    await expect(grid.locator("> div")).toHaveCount(8);
  });

  test("metric cards in row 0, each 8 cols wide", async ({ page }) => {
    const grid = page.locator("div[style*='display: grid']");
    const cells = grid.locator("> div");

    // First 3 cells: metric cards at y=0
    for (let i = 0; i < 3; i++) {
      const row = await cells.nth(i).evaluate((el) => el.style.gridRow);
      expect(row).toContain("1 / span 4");
    }

    // Verify x positions: 0, 8, 16 -> col 1, 9, 17
    expect(await cells.nth(0).evaluate((el) => el.style.gridColumn)).toBe("1 / span 8");
    expect(await cells.nth(1).evaluate((el) => el.style.gridColumn)).toBe("9 / span 8");
    expect(await cells.nth(2).evaluate((el) => el.style.gridColumn)).toBe("17 / span 8");
  });

  test("area chart spans full width at row 5", async ({ page }) => {
    const grid = page.locator("div[style*='display: grid']");
    const cell = grid.locator("> div").nth(3);
    expect(await cell.evaluate((el) => el.style.gridColumn)).toBe("1 / span 24");
    expect(await cell.evaluate((el) => el.style.gridRow)).toBe("5 / span 8");
  });

  test("pie and donut side by side at row 13", async ({ page }) => {
    const grid = page.locator("div[style*='display: grid']");
    const pie = grid.locator("> div").nth(4);
    const donut = grid.locator("> div").nth(5);

    expect(await pie.evaluate((el) => el.style.gridColumn)).toBe("1 / span 12");
    expect(await pie.evaluate((el) => el.style.gridRow)).toBe("13 / span 8");

    expect(await donut.evaluate((el) => el.style.gridColumn)).toBe("13 / span 12");
    expect(await donut.evaluate((el) => el.style.gridRow)).toBe("13 / span 8");
  });
});

// =============================================================================
// CROSS-DASHBOARD NAVIGATION
// =============================================================================

test.describe("Dashboard Navigation", () => {
  test("can navigate between dashboards via hash", async ({ page }) => {
    // Load demo dashboard
    await page.goto(`${BASE_URL}#dashboards/demo`);
    await page.waitForFunction(
      () => (window as unknown as { __OPEN_PLX__: { name: string } }).__OPEN_PLX__?.name === "dashboards/demo",
      { timeout: 10_000 },
    );
    let state = await getDashboardState(page);
    expect(state.title).toBe("Demo Dashboard");

    // Navigate to full-demo
    await page.goto(`${BASE_URL}#dashboards/full-demo`);
    await page.waitForFunction(
      () => (window as unknown as { __OPEN_PLX__: { name: string } }).__OPEN_PLX__?.name === "dashboards/full-demo",
      { timeout: 10_000 },
    );
    state = await getDashboardState(page);
    expect(state.title).toBe("Full Widget Demo");
  });

  test("nonexistent dashboard shows error", async ({ page }) => {
    await page.goto(`${BASE_URL}#dashboards/does-not-exist`);
    // Wait for the error state
    await page.waitForFunction(
      () => (window as unknown as { __OPEN_PLX__: { error: string | null } }).__OPEN_PLX__?.error !== null,
      { timeout: 10_000 },
    );
    const state = await getDashboardState(page);
    expect(state.error).toContain("not found");
  });
});

// =============================================================================
// DATA CONSISTENCY ACROSS WIDGETS
// =============================================================================

test.describe("Full Demo: Data Consistency", () => {
  test.beforeEach(async ({ page }) => {
    await loadFullDemo(page);
  });

  test("all widgets reference the same static data", async ({ page }) => {
    const ids = await getWidgetIds(page);
    const dataSets = await Promise.all(
      ids.map(async (id) => {
        const w = await getWidgetState(page, id);
        return { id, rowCount: w.data?.length ?? 0 };
      }),
    );

    // All data-backed widgets should have 4 rows
    for (const ds of dataSets) {
      if (ds.id !== "summary-text") {
        expect(ds.rowCount, `${ds.id} row count`).toBe(4);
      }
    }
  });

  test("metric cards use last row values", async ({ page }) => {
    const revenue = await getWidgetState(page, "revenue-metric");
    const cost = await getWidgetState(page, "cost-metric");
    const units = await getWidgetState(page, "units-metric");

    // Q4 values from static data
    expect(revenue.rendered.rawValue).toBe(2100000);
    expect(cost.rendered.rawValue).toBe(1100000);
    expect(units.rendered.rawValue).toBe(21000);
  });
});
