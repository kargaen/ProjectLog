import { expect, test } from "@playwright/test";
import { installTauriMocks } from "./helpers/tauri";

test.describe("QuickPanel UI", () => {
  test.beforeEach(async ({ page }) => {
    await installTauriMocks(page);
    await page.goto("/");
  });

  test("keeps Recent sort selected after project activation", async ({ page }) => {
    await page.getByRole("button", { name: "Recent" }).click();
    await expect(page.getByRole("button", { name: "Recent" })).toHaveClass(/sort-active/);

    await page.getByRole("button", { name: /Jot/ }).click();

    await expect(page.getByRole("button", { name: "Recent" })).toHaveClass(/sort-active/);
  });

  test("moves the latest clicked project to the top in Recent mode", async ({ page }) => {
    await page.getByRole("button", { name: "Recent" }).click();

    await page.getByRole("button", { name: /Sous Chef/ }).click();
    await expect(page.locator(".project-button span").first()).toHaveText("Sous Chef");

    await page.getByRole("button", { name: /Test 5/ }).click();
    await expect(page.locator(".project-button span").first()).toHaveText("Test 5");
  });

  test("shows the clicked project as active", async ({ page }) => {
    await page.getByRole("button", { name: /Sous Chef/ }).click();

    await expect(page.locator(".project-row.active .project-button span")).toHaveText("Sous Chef");
    await expect(page.getByText("Active")).toBeVisible();
    await expect(page.getByText("Sous Chef").first()).toBeVisible();
  });

  test("sorts projects alphabetically in A-Z mode", async ({ page }) => {
    await page.getByRole("button", { name: "A-Z" }).click();

    await expect(page.locator(".project-button span").nth(0)).toHaveText("Jot");
    await expect(page.locator(".project-button span").nth(1)).toHaveText("Sous Chef");
    await expect(page.locator(".project-button span").nth(2)).toHaveText("Test 5");
  });

  test("persists compact mode toggle in the UI", async ({ page }) => {
    const compactToggle = page.getByRole("button", { name: "Compact mode" });

    await compactToggle.click();

    await expect(compactToggle).toHaveClass(/toggle-on/);
    await expect(page.getByRole("button", { name: "Recent" })).toHaveCount(0);
  });

  test("renders the dedicated timesheet preview window with generated status", async ({ page }) => {
    await installTauriMocks(page, undefined, {
      currentWindowLabel: "timesheet-preview",
      initialPreviewRequest: { range: "all", format: "full" },
    });
    await page.goto("/");

    await expect(page.getByRole("heading", { name: "Full timesheet" })).toBeVisible();
    await expect(page.getByText(/Generated at 2026-04-30 07:09,/)).toBeVisible();
    await expect(page.getByRole("button", { name: "Update now" })).toBeVisible();
  });

  test("applies banding and crosshair highlighting in the timesheet preview", async ({ page }) => {
    await installTauriMocks(page, undefined, {
      currentWindowLabel: "timesheet-preview",
      initialPreviewRequest: { range: "all", format: "full" },
      previewResponse: {
        title: "Full timesheet",
        generated_at: "2026-04-30 07:09",
        generated_at_epoch_ms: Date.UTC(2026, 3, 30, 5, 9, 0),
        sheets: [
          {
            name: "2026-18",
            columns: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
            rows: [
              {
                label: "Alpha",
                values: [1, 0, 0, 0, 0, 0, 0],
                total: 1,
                is_comment: false,
                is_total: false,
              },
              {
                label: "  - Prep",
                values: [1, 0, 0, 0, 0, 0, 0],
                total: 1,
                is_comment: true,
                is_total: false,
              },
              {
                label: "Beta",
                values: [0, 2, 0, 0, 0, 0, 0],
                total: 2,
                is_comment: false,
                is_total: false,
              },
              {
                label: "Total",
                values: [1, 2, 0, 0, 0, 0, 0],
                total: 3,
                is_comment: false,
                is_total: true,
              },
            ],
          },
        ],
      },
    });
    await page.goto("/");

    await expect(page.locator("tbody tr").nth(2)).toHaveClass(/banded-row/);

    const targetCell = page.locator("tbody tr").first().locator("td").nth(2);
    await targetCell.hover();

    await expect(page.locator("tbody tr").first()).toHaveClass(/crosshair-row/);
    await expect(page.locator("thead th").nth(2)).toHaveClass(/crosshair-column/);
    await expect(targetCell).toHaveClass(/crosshair-column/);
  });
});
