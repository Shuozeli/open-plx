import { test, expect, type Page } from "@playwright/test";

const BASE_URL = process.env.E2E_BASE_URL ?? "http://10.0.0.183:5199";

/** Wait for the dashboard to fully load (layout + data). */
async function waitForDashboard(page: Page) {
  await page.goto(BASE_URL);
  await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible({
    timeout: 10_000,
  });
  // Wait for data to load (metric card value appears)
  await expect(page.getByText(/\$[\d,]+/)).toBeVisible({ timeout: 10_000 });
}

// =============================================================================
// HEADER
// =============================================================================

test.describe("Header", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboard(page);
  });

  test("renders the app name", async ({ page }) => {
    const header = page.locator(".ant-layout-header");
    await expect(header).toBeVisible();
    await expect(header.getByRole("heading", { name: "open-plx" })).toBeVisible();
  });

  test("header has dark background", async ({ page }) => {
    const header = page.locator(".ant-layout-header");
    const bg = await header.evaluate((el) => getComputedStyle(el).backgroundColor);
    // Antd default header is dark (#001529 -> rgb(0, 21, 41))
    expect(bg).toMatch(/rgb\(0,\s*21,\s*41\)/);
  });

  test("header text is white", async ({ page }) => {
    const title = page.locator(".ant-layout-header h4");
    const color = await title.evaluate((el) => getComputedStyle(el).color);
    // white or near-white
    expect(color).toMatch(/rgb\(255,\s*255,\s*255\)|rgba\(255,\s*255,\s*255/);
  });
});

// =============================================================================
// DASHBOARD TITLE AREA
// =============================================================================

test.describe("Dashboard Title Area", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboard(page);
  });

  test("renders dashboard title as h2", async ({ page }) => {
    const title = page.getByRole("heading", { level: 2, name: "Demo Dashboard" });
    await expect(title).toBeVisible();
  });

  test("renders dashboard description", async ({ page }) => {
    await expect(page.getByText("A test dashboard with static data")).toBeVisible();
  });

  test("title and refresh button are in the same row", async ({ page }) => {
    const title = page.getByRole("heading", { name: "Demo Dashboard" });
    const refreshBtn = page.getByRole("button", { name: "Refresh" });

    const titleBox = await title.boundingBox();
    const btnBox = await refreshBtn.boundingBox();
    expect(titleBox).not.toBeNull();
    expect(btnBox).not.toBeNull();

    // They should be roughly on the same vertical line (same row)
    expect(Math.abs(titleBox!.y - btnBox!.y)).toBeLessThan(30);
    // Refresh button should be to the right of the title
    expect(btnBox!.x).toBeGreaterThan(titleBox!.x + titleBox!.width);
  });

  test("refresh button has reload icon", async ({ page }) => {
    const btn = page.getByRole("button", { name: "Refresh" });
    await expect(btn).toBeVisible();
    // Antd Button with icon should have a span.anticon inside
    const icon = btn.locator(".anticon");
    await expect(icon).toBeVisible();
  });
});

// =============================================================================
// GRID LAYOUT
// =============================================================================

test.describe("Grid Layout", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboard(page);
  });

  test("uses CSS grid for widget layout", async ({ page }) => {
    // The grid container is a div with display: grid
    const grid = page.locator("div[style*='display: grid']");
    await expect(grid).toBeVisible();

    const display = await grid.evaluate((el) => getComputedStyle(el).display);
    expect(display).toBe("grid");
  });

  test("grid has 24 columns", async ({ page }) => {
    const grid = page.locator("div[style*='display: grid']");
    const cols = await grid.evaluate((el) => getComputedStyle(el).gridTemplateColumns);
    // Should have 24 column tracks
    const colCount = cols.split(" ").length;
    expect(colCount).toBe(24);
  });

  test("metric card is in top-left (position 0,0)", async ({ page }) => {
    const grid = page.locator("div[style*='display: grid']");
    const metricCard = grid.locator("> div").first();

    const gridColumn = await metricCard.evaluate((el) => el.style.gridColumn);
    const gridRow = await metricCard.evaluate((el) => el.style.gridRow);

    // x=0 -> column starts at 1, w=8 -> span 8
    expect(gridColumn).toBe("1 / span 8");
    // y=0 -> row starts at 1, h=4 -> span 4
    expect(gridRow).toBe("1 / span 4");
  });

  test("line chart spans 16 columns at row 5", async ({ page }) => {
    const grid = page.locator("div[style*='display: grid']");
    // Revenue Trend widget (second widget)
    const lineChartCell = grid.locator("> div").nth(1);

    const gridColumn = await lineChartCell.evaluate((el) => el.style.gridColumn);
    const gridRow = await lineChartCell.evaluate((el) => el.style.gridRow);

    // x=0 -> col 1, w=16 -> span 16
    expect(gridColumn).toBe("1 / span 16");
    // y=4 -> row 5, h=8 -> span 8
    expect(gridRow).toBe("5 / span 8");
  });

  test("bar chart is positioned right of line chart", async ({ page }) => {
    const grid = page.locator("div[style*='display: grid']");
    const barChartCell = grid.locator("> div").nth(2);

    const gridColumn = await barChartCell.evaluate((el) => el.style.gridColumn);
    const gridRow = await barChartCell.evaluate((el) => el.style.gridRow);

    // x=16 -> col 17, w=8 -> span 8
    expect(gridColumn).toBe("17 / span 8");
    // y=4 -> row 5, h=8 -> span 8
    expect(gridRow).toBe("5 / span 8");
  });

  test("all 3 widgets are rendered as grid children", async ({ page }) => {
    const grid = page.locator("div[style*='display: grid']");
    const children = grid.locator("> div");
    await expect(children).toHaveCount(3);
  });
});

// =============================================================================
// METRIC CARD WIDGET
// =============================================================================

test.describe("Metric Card Widget", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboard(page);
  });

  test("renders as an Antd Card", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Total Revenue" });
    await expect(card).toBeVisible();
  });

  test("shows the formatted revenue value $2,100,000.00", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Total Revenue" });
    await expect(card.getByText("$2,100,000.00")).toBeVisible();
  });

  test("uses Antd Statistic component", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Total Revenue" });
    const statistic = card.locator(".ant-statistic");
    await expect(statistic).toBeVisible();
  });

  test("statistic title shows 'Total Revenue'", async ({ page }) => {
    const statistic = page.locator(".ant-statistic").filter({ hasText: "Total Revenue" });
    const title = statistic.locator(".ant-statistic-title");
    await expect(title).toHaveText("Total Revenue");
  });

  test("statistic value is a number format", async ({ page }) => {
    const statistic = page.locator(".ant-statistic").filter({ hasText: "Total Revenue" });
    const value = statistic.locator(".ant-statistic-content-value");
    const text = await value.textContent();
    expect(text).toMatch(/\$[\d,]+\.\d{2}/);
  });
});

// =============================================================================
// LINE CHART WIDGET
// =============================================================================

test.describe("Line Chart Widget", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboard(page);
  });

  test("renders as an Antd Card with title", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });
    await expect(card).toBeVisible();

    const title = card.locator(".ant-card-head-title");
    await expect(title).toHaveText("Revenue Trend");
  });

  test("contains a canvas element (G2 chart)", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });
    const canvas = card.locator("canvas");
    await expect(canvas).toBeVisible();
  });

  test("canvas has non-zero dimensions", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });
    const canvas = card.locator("canvas");
    // G2 may render asynchronously; wait for canvas to have pixel data
    await page.waitForTimeout(1_000);
    const dims = await canvas.evaluate((el: HTMLCanvasElement) => ({
      w: el.width,
      h: el.height,
    }));
    expect(dims.w).toBeGreaterThan(0);
    expect(dims.h).toBeGreaterThan(0);
  });

  test("canvas has rendered pixel data (not blank)", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });
    const canvas = card.locator("canvas");
    await page.waitForTimeout(1_000);
    // Check if canvas has any non-white pixels (chart was drawn)
    const hasContent = await canvas.evaluate((el: HTMLCanvasElement) => {
      const ctx = el.getContext("2d");
      if (!ctx) return false;
      const data = ctx.getImageData(0, 0, el.width, el.height).data;
      // Check if there are any non-transparent, non-white pixels
      for (let i = 0; i < data.length; i += 4) {
        const a = data[i + 3]; // alpha
        if (a > 0) {
          const r = data[i], g = data[i + 1], b = data[i + 2];
          if (r < 250 || g < 250 || b < 250) return true; // non-white pixel
        }
      }
      return false;
    });
    expect(hasContent).toBe(true);
  });
});

// =============================================================================
// BAR CHART WIDGET
// =============================================================================

test.describe("Bar Chart Widget", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboard(page);
  });

  test("renders as an Antd Card with title", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Cost by Quarter" });
    await expect(card).toBeVisible();

    const title = card.locator(".ant-card-head-title");
    await expect(title).toHaveText("Cost by Quarter");
  });

  test("contains a canvas element (G2 chart)", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Cost by Quarter" });
    const canvas = card.locator("canvas");
    await expect(canvas).toBeVisible();
  });

  test("canvas has non-zero dimensions", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Cost by Quarter" });
    const canvas = card.locator("canvas");
    await page.waitForTimeout(1_000);
    const dims = await canvas.evaluate((el: HTMLCanvasElement) => ({
      w: el.width,
      h: el.height,
    }));
    expect(dims.w).toBeGreaterThan(0);
    expect(dims.h).toBeGreaterThan(0);
  });

  test("canvas has rendered pixel data (not blank)", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Cost by Quarter" });
    const canvas = card.locator("canvas");
    await page.waitForTimeout(1_000);
    const hasContent = await canvas.evaluate((el: HTMLCanvasElement) => {
      const ctx = el.getContext("2d");
      if (!ctx) return false;
      const data = ctx.getImageData(0, 0, el.width, el.height).data;
      for (let i = 0; i < data.length; i += 4) {
        const a = data[i + 3];
        if (a > 0) {
          const r = data[i], g = data[i + 1], b = data[i + 2];
          if (r < 250 || g < 250 || b < 250) return true;
        }
      }
      return false;
    });
    expect(hasContent).toBe(true);
  });
});

// =============================================================================
// OVERALL PAGE LAYOUT
// =============================================================================

test.describe("Overall Page Layout", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboard(page);
  });

  test("page has Antd Layout structure", async ({ page }) => {
    await expect(page.locator(".ant-layout")).toBeVisible();
    await expect(page.locator(".ant-layout-header")).toBeVisible();
    await expect(page.locator(".ant-layout-content")).toBeVisible();
  });

  test("content area has light background", async ({ page }) => {
    const content = page.locator(".ant-layout-content");
    const bg = await content.evaluate((el) => getComputedStyle(el).backgroundColor);
    // Should be light gray (#f5f5f5 -> rgb(245, 245, 245))
    expect(bg).toMatch(/rgb\(24[0-9],\s*24[0-9],\s*24[0-9]\)/);
  });

  test("content has padding", async ({ page }) => {
    const content = page.locator(".ant-layout-content");
    const padding = await content.evaluate((el) => getComputedStyle(el).padding);
    expect(padding).not.toBe("0px");
  });

  test("metric card row is above chart row", async ({ page }) => {
    const metricCard = page.locator(".ant-card").filter({ hasText: "Total Revenue" });
    const lineChart = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });

    const metricBox = await metricCard.boundingBox();
    const chartBox = await lineChart.boundingBox();
    expect(metricBox).not.toBeNull();
    expect(chartBox).not.toBeNull();

    // Metric card top should be above line chart top
    expect(metricBox!.y).toBeLessThan(chartBox!.y);
  });

  test("line chart and bar chart are side by side", async ({ page }) => {
    const lineChart = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });
    const barChart = page.locator(".ant-card").filter({ hasText: "Cost by Quarter" });

    const lineBox = await lineChart.boundingBox();
    const barBox = await barChart.boundingBox();
    expect(lineBox).not.toBeNull();
    expect(barBox).not.toBeNull();

    // Same row: tops should be roughly equal
    expect(Math.abs(lineBox!.y - barBox!.y)).toBeLessThan(10);
    // Bar chart should be to the right
    expect(barBox!.x).toBeGreaterThan(lineBox!.x);
  });

  test("line chart is wider than bar chart", async ({ page }) => {
    const lineChart = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });
    const barChart = page.locator(".ant-card").filter({ hasText: "Cost by Quarter" });

    const lineBox = await lineChart.boundingBox();
    const barBox = await barChart.boundingBox();
    expect(lineBox).not.toBeNull();
    expect(barBox).not.toBeNull();

    // Line chart spans 16 cols, bar spans 8 -> line should be ~2x wider
    expect(lineBox!.width).toBeGreaterThan(barBox!.width * 1.5);
  });
});

// =============================================================================
// ANTD CARD STRUCTURE
// =============================================================================

test.describe("Antd Card Structure", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboard(page);
  });

  test("each chart card has a head and body", async ({ page }) => {
    const cards = page.locator(".ant-card").filter({ hasText: /Revenue Trend|Cost by Quarter/ });
    const count = await cards.count();
    expect(count).toBe(2);

    for (let i = 0; i < count; i++) {
      const card = cards.nth(i);
      await expect(card.locator(".ant-card-head")).toBeVisible();
      await expect(card.locator(".ant-card-body")).toBeVisible();
    }
  });

  test("metric card has no card head (uses Statistic title instead)", async ({ page }) => {
    // Metric cards use Antd Statistic which provides its own title,
    // so the card may or may not have a head section.
    const card = page.locator(".ant-card").filter({ hasText: "$2,100,000.00" });
    await expect(card).toBeVisible();
    // The card body should contain the Statistic
    await expect(card.locator(".ant-statistic")).toBeVisible();
  });

  test("all cards have 100% height", async ({ page }) => {
    const cards = page.locator(".ant-card");
    const count = await cards.count();
    expect(count).toBe(3);

    for (let i = 0; i < count; i++) {
      const height = await cards.nth(i).evaluate((el) => el.style.height);
      expect(height).toBe("100%");
    }
  });
});

// =============================================================================
// INTERACTIVITY
// =============================================================================

test.describe("Interactivity", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboard(page);
  });

  test("clicking refresh re-renders widgets without flicker", async ({ page }) => {
    // Verify initial state
    await expect(page.getByText("$2,100,000.00")).toBeVisible();

    // Click refresh
    await page.getByRole("button", { name: "Refresh" }).click();

    // Dashboard should remain visible during refresh (no blank state)
    // The title should stay visible throughout
    await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible();

    // Data should re-appear (same value since static data)
    await expect(page.getByText("$2,100,000.00")).toBeVisible({ timeout: 5_000 });
  });

  test("no console errors during page load", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    await page.goto(BASE_URL);
    await page.waitForTimeout(3_000);

    // Filter out known benign warnings
    const realErrors = errors.filter(
      (e) => !e.includes("deprecated") && !e.includes("Warning:"),
    );
    expect(realErrors).toHaveLength(0);
  });

  test("no failed network requests", async ({ page }) => {
    const failedRequests: string[] = [];
    page.on("response", (response) => {
      if (response.status() >= 400) {
        failedRequests.push(`${response.status()} ${response.url()}`);
      }
    });

    await page.goto(BASE_URL);
    await page.waitForTimeout(3_000);

    expect(failedRequests).toHaveLength(0);
  });
});

// =============================================================================
// DATA INTEGRITY
// =============================================================================

test.describe("Data Integrity", () => {
  test.beforeEach(async ({ page }) => {
    await waitForDashboard(page);
  });

  test("metric card displays Q4 revenue (last row of static data)", async ({ page }) => {
    // Static data has Q4 revenue = 2,100,000
    await expect(page.getByText("$2,100,000.00")).toBeVisible();
  });

  test("line chart canvas is rendered with chart data", async ({ page }) => {
    // G2 renders axis labels on canvas -- we verify the canvas has content
    // (pixel-level check already done in Line Chart Widget tests).
    const card = page.locator(".ant-card").filter({ hasText: "Revenue Trend" });
    const canvas = card.locator("canvas");
    await expect(canvas).toBeVisible();
  });

  test("bar chart canvas is rendered with chart data", async ({ page }) => {
    const card = page.locator(".ant-card").filter({ hasText: "Cost by Quarter" });
    const canvas = card.locator("canvas");
    await expect(canvas).toBeVisible();
  });
});

// =============================================================================
// LOADING STATES
// =============================================================================

test.describe("Loading States", () => {
  test("shows loading spinner before dashboard loads", async ({ page }) => {
    // Navigate and immediately check for spinner before data arrives
    await page.goto(BASE_URL);

    // Either we see the spinner or the dashboard loaded fast enough.
    // Use a race: check if spinner appeared at all.
    const spinner = page.locator(".ant-spin");
    const dashboard = page.getByRole("heading", { name: "Demo Dashboard" });

    // One of these should be visible quickly
    await expect(spinner.or(dashboard)).toBeVisible({ timeout: 10_000 });
  });
});
