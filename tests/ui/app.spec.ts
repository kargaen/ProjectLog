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
});
