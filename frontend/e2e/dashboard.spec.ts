import { test, expect } from "@playwright/test";

const BASE_URL = process.env.E2E_BASE_URL ?? "http://10.0.0.183:5199";

test.describe("Demo Dashboard", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    // Wait for the app to mount
    await page.waitForSelector("#app", { timeout: 5_000 });
  });

  test("renders dashboard title and description", async ({ page }) => {
    // Wait for the dashboard to load (title appears after gRPC fetch)
    await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible({
      timeout: 10_000,
    });
    await expect(page.getByText("A test dashboard with static data")).toBeVisible();
  });

  test("renders refresh button", async ({ page }) => {
    await expect(page.getByRole("button", { name: "Refresh" })).toBeVisible({
      timeout: 10_000,
    });
  });

  test("renders all 3 widget cards", async ({ page }) => {
    // Wait for dashboard to load
    await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible({
      timeout: 10_000,
    });

    // Check all widget titles are present
    await expect(page.getByText("Total Revenue")).toBeVisible();
    await expect(page.getByText("Revenue Trend")).toBeVisible();
    await expect(page.getByText("Cost by Quarter")).toBeVisible();
  });

  test("metric card shows formatted revenue value", async ({ page }) => {
    // Wait for data to load -- the metric card should display a currency value
    await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible({
      timeout: 10_000,
    });

    // The last row in static data has revenue = 2,100,000
    // Format is "currency:USD" -> "$2,100,000.00"
    // Wait for the value to appear (data fetch + render)
    await expect(page.getByText(/\$[\d,]+/)).toBeVisible({ timeout: 10_000 });
  });

  test("line chart renders a canvas element", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible({
      timeout: 10_000,
    });

    // G2 renders into a canvas element. Find the Revenue Trend card
    // and check it contains a canvas.
    const trendCard = page.locator(".ant-card", { hasText: "Revenue Trend" });
    await expect(trendCard).toBeVisible({ timeout: 10_000 });

    // G2 chart should produce a canvas element inside the card
    const canvas = trendCard.locator("canvas");
    await expect(canvas).toBeVisible({ timeout: 10_000 });
  });

  test("bar chart renders a canvas element", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible({
      timeout: 10_000,
    });

    const costCard = page.locator(".ant-card", { hasText: "Cost by Quarter" });
    await expect(costCard).toBeVisible({ timeout: 10_000 });

    const canvas = costCard.locator("canvas");
    await expect(canvas).toBeVisible({ timeout: 10_000 });
  });

  test("refresh button re-fetches data without error", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible({
      timeout: 10_000,
    });

    // Click refresh
    await page.getByRole("button", { name: "Refresh" }).click();

    // Dashboard should still be visible (no crash, no error)
    await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible();
    await expect(page.getByText("Total Revenue")).toBeVisible();
  });

  test("no error alerts on the page", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible({
      timeout: 10_000,
    });

    // Wait a bit for all data fetches to settle
    await page.waitForTimeout(3_000);

    // There should be no Antd Alert components with type="error"
    const errorAlerts = page.locator(".ant-alert-error");
    await expect(errorAlerts).toHaveCount(0);
  });
});
