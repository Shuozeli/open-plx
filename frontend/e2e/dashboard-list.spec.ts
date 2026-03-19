import { test, expect } from "@playwright/test";

const BASE_URL = process.env.E2E_BASE_URL ?? "http://10.0.0.183:5199";

test.describe("Dashboard List Page", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to root (no hash) -> list page
    await page.goto(BASE_URL);
  });

  test("shows 'Dashboards' title", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Dashboards" })).toBeVisible({
      timeout: 10_000,
    });
  });

  test("lists both dashboards", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Dashboards" })).toBeVisible({
      timeout: 10_000,
    });
    await expect(page.getByText("Demo Dashboard")).toBeVisible();
    await expect(page.getByText("Full Widget Demo")).toBeVisible();
  });

  test("shows widget count per dashboard", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Dashboards" })).toBeVisible({
      timeout: 10_000,
    });
    await expect(page.getByText("3 widgets")).toBeVisible();
    await expect(page.getByText("9 widgets")).toBeVisible();
  });

  test("clicking a dashboard card navigates to it", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Dashboards" })).toBeVisible({
      timeout: 10_000,
    });

    await page.getByText("Demo Dashboard").click();

    // Should navigate to the dashboard
    await expect(page.getByRole("heading", { name: "Demo Dashboard", level: 2 })).toBeVisible({
      timeout: 10_000,
    });
    expect(page.url()).toContain("#dashboards/demo");
  });

  test("clicking open-plx header navigates back to list", async ({ page }) => {
    // First go to a dashboard
    await page.goto(`${BASE_URL}#dashboards/demo`);
    await expect(page.getByRole("heading", { name: "Demo Dashboard" })).toBeVisible({
      timeout: 10_000,
    });

    // Click the header link
    await page.locator(".ant-layout-header a").click();

    // Should show list page
    await expect(page.getByRole("heading", { name: "Dashboards" })).toBeVisible({
      timeout: 10_000,
    });
  });
});
