import { expect, test } from "@playwright/test";
import { installTauriMocks } from "./helpers/tauri";

const MANUAL_PROJECT = "Jot";
const SECONDARY_PROJECT = "Sous Chef";
const RECENT_PROJECT = "Test 5";
const EDGE_PROJECT = "Ægir Analytics";
const LONG_NAME_PROJECT = "UX – Very Long Project Name For Overflow Checks";
const DEBUG_STEP_DELAY_MS = Number(process.env.PW_STEP_DELAY_MS ?? "0");

function mockInvokedCommands(page: Parameters<typeof test>[0]["page"]) {
  return page.evaluate(() => {
    return ((window as Window & { __TAURI_MOCK__?: { invokedCommands?: string[] } }).__TAURI_MOCK__
      ?.invokedCommands ?? []) as string[];
  });
}

async function stepPause(page: Parameters<typeof test>[0]["page"]) {
  if (DEBUG_STEP_DELAY_MS > 0) {
    await page.waitForTimeout(DEBUG_STEP_DELAY_MS);
  }
}

test.describe("QuickPanel UI", () => {
  test.beforeEach(async ({ page }) => {
    await installTauriMocks(page);
    await page.goto("/");
    await stepPause(page);
  });

  test("renders the QuickPanel shell", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "ProjectLog QuickPanel" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Hide" })).toBeVisible();
  });

  test("manual mode supports select-select-deselect workflow", async ({ page }) => {
    await page.getByRole("button", { name: MANUAL_PROJECT }).click();
    await stepPause(page);
    await expect(page.locator(".project-row.active .project-button span")).toHaveText(MANUAL_PROJECT);

    await page.getByRole("button", { name: SECONDARY_PROJECT }).click();
    await stepPause(page);
    await expect(page.locator(".project-row.active .project-button span")).toHaveText(SECONDARY_PROJECT);

    await page.getByRole("button", { name: SECONDARY_PROJECT }).click();
    await stepPause(page);
    await expect(page.locator(".project-row.active")).toHaveCount(0);
    await expect(page.getByText("No active project")).toBeVisible();
  });

  test("A-Z mode keeps alphabetical order during select-select-deselect workflow", async ({ page }) => {
    await page.getByRole("button", { name: "A-Z" }).click();
    await stepPause(page);

    await expect(page.locator(".project-button span").nth(0)).toHaveText("_Internal Tools");
    await expect(page.locator(".project-button span").nth(1)).toHaveText("Ægir Analytics");
    await expect(page.locator(".project-button span").nth(2)).toHaveText("Alpha Project");

    await page.getByRole("button", { name: EDGE_PROJECT }).click();
    await stepPause(page);
    await page.getByRole("button", { name: LONG_NAME_PROJECT }).click();
    await stepPause(page);

    await expect(page.locator(".project-row.active .project-button span")).toHaveText(LONG_NAME_PROJECT);

    await page.getByRole("button", { name: LONG_NAME_PROJECT }).click();
    await stepPause(page);
    await expect(page.locator(".project-row.active")).toHaveCount(0);

    await expect(page.locator(".project-button span").nth(0)).toHaveText("_Internal Tools");
    await expect(page.locator(".project-button span").nth(1)).toHaveText("Ægir Analytics");
    await expect(page.locator(".project-button span").nth(2)).toHaveText("Alpha Project");
  });

  test("Recent mode reorders by latest selection and supports deselect", async ({ page }) => {
    await page.getByRole("button", { name: "Recent" }).click();
    await stepPause(page);

    await page.getByRole("button", { name: SECONDARY_PROJECT }).click();
    await stepPause(page);
    await expect(page.locator(".project-button span").first()).toHaveText(SECONDARY_PROJECT);

    await page.getByRole("button", { name: RECENT_PROJECT }).click();
    await stepPause(page);
    await expect(page.locator(".project-button span").first()).toHaveText(RECENT_PROJECT);
    await expect(page.locator(".project-row.active .project-button span")).toHaveText(RECENT_PROJECT);

    await page.getByRole("button", { name: RECENT_PROJECT }).click();
    await stepPause(page);
    await expect(page.locator(".project-row.active")).toHaveCount(0);
    await expect(page.locator(".project-button span").first()).toHaveText(RECENT_PROJECT);
  });

  test("core QuickPanel actions respond visibly", async ({ page }) => {
    const compactToggle = page.getByRole("button", { name: "Compact mode" });

    await page.getByRole("button", { name: MANUAL_PROJECT }).click();
    await stepPause(page);
    await page.getByPlaceholder("Comment").fill("Investigate close button");
    await stepPause(page);
    await page.getByRole("button", { name: "Save" }).click();
    await stepPause(page);
    await expect(page.getByText("Investigate close button")).toBeVisible();

    await page.getByRole("button", { name: "Clear" }).click();
    await stepPause(page);
    await expect(page.getByText("Investigate close button")).toHaveCount(0);

    await page.getByPlaceholder("Add project").fill("Alpha Project");
    await stepPause(page);
    await page.getByRole("button", { name: "Add" }).click();
    await stepPause(page);
    await expect(page.getByRole("button", { name: /Alpha Project/ })).toBeVisible();

    await page.getByPlaceholder("Quick project").fill("Adhoc Work");
    await stepPause(page);
    await page.getByRole("button", { name: "Track" }).click();
    await stepPause(page);
    await expect(page.locator(".project-row.active .project-button span")).toHaveText("Adhoc Work");

    await compactToggle.click();
    await stepPause(page);
    await expect(compactToggle).toHaveClass(/toggle-on/);
    await expect(page.getByRole("button", { name: "Recent" })).toHaveCount(0);

    await compactToggle.click();
    await stepPause(page);
    await expect(page.getByRole("button", { name: "Recent" })).toBeVisible();

    await page.getByRole("button", { name: "About" }).click();
    await stepPause(page);
    await expect(page.getByRole("heading", { name: "About ProjectLog" })).toBeVisible();
    await page.getByRole("button", { name: "Close", exact: true }).click();
    await stepPause(page);
    await expect(page.getByRole("heading", { name: "About ProjectLog" })).toHaveCount(0);

    await page.getByRole("button", { name: "Hide" }).click();
    await stepPause(page);

    const invokedCommands = await mockInvokedCommands(page);
    expect(invokedCommands).toContain("plugin:window|hide");
    expect(invokedCommands).toContain("plugin:window|set_size_constraints");
  });

  // Deferred: tray menu behavior needs desktop/native coverage rather than browser Playwright.
  // Deferred: external opener actions such as log file, release notes, mail, and portfolio are
  // best covered in a dedicated native integration layer because the browser suite can only assert
  // transport calls, not the OS-level result.
});

test.describe("Timesheet preview window", () => {
  test.beforeEach(async ({ page }) => {
    await installTauriMocks(page);
  });

  test("full timesheet preview renders expected table and generated timestamp", async ({ page }) => {
    await page.goto(
      "/?window=timesheet-preview&mockWindowLabel=timesheet-preview&mockPreviewRange=all&mockPreviewFormat=full"
    );
    await stepPause(page);

    await expect(page.getByRole("heading", { name: "Full timesheet" })).toBeVisible();
    await expect(page.getByText(/Generated at 2026-04-30 07:09,/)).toBeVisible();
    await expect(page.getByRole("button", { name: "Update now" })).toBeVisible();
    await expect(page.locator("tbody tr").nth(0).locator("td").nth(0)).toHaveText("Alpha");
    await expect(page.locator("tbody tr").nth(0).locator("td").nth(1)).toHaveText("1.00");
    await expect(page.locator("tbody tr").nth(0).locator("td").nth(2)).toHaveText("-");
    await expect(page.locator("tbody tr").nth(1).locator("td").nth(0)).toHaveText("Total");
    await expect(page.locator("tbody tr").nth(1).locator("td").last()).toHaveText("1.00");
  });

  test("yesterday plus today preview renders expected reduced output", async ({ page }) => {
    await page.goto(
      "/?window=timesheet-preview&mockWindowLabel=timesheet-preview&mockPreviewRange=today&mockPreviewFormat=recent"
    );
    await stepPause(page);

    await expect(page.getByRole("heading", { name: "Yesterday + today overview" })).toBeVisible();
    await expect(page.locator("tbody tr").nth(0).locator("td").nth(0)).toHaveText("Alpha");
    await expect(page.locator("tbody tr").nth(1).locator("td").nth(0)).toHaveText("Total");
  });

  test("preview actions work for refresh, rounding, export and close", async ({ page }) => {
    await page.goto(
      "/?window=timesheet-preview&mockWindowLabel=timesheet-preview&mockPreviewRange=all&mockPreviewFormat=full&mockPreviewScenario=banding"
    );
    await stepPause(page);

    await page.getByRole("button", { name: "Update now" }).click();
    await stepPause(page);
    await expect(page.getByRole("button", { name: "Update now" })).toBeVisible();

    const roundingToggle = page.getByRole("button", { name: "Round to 0.5h" });
    await roundingToggle.click();
    await stepPause(page);
    await expect(roundingToggle).toHaveClass(/toggle-on/);

    await page.getByRole("button", { name: "Export to Excel" }).click();
    await stepPause(page);

    await page.getByRole("button", { name: "Close" }).click();
    await stepPause(page);

    const invokedCommands = await mockInvokedCommands(page);

    expect(invokedCommands).toContain("preview_timesheet");
    expect(invokedCommands).toContain("set_timesheet_rounding_enabled");
    expect(invokedCommands).toContain("generate_timesheet_export");
    expect(invokedCommands).toContain("hide_timesheet_preview_window");
  });

  // Deferred: opening the preview from QuickPanel as a true separate native window should be covered
  // later with a desktop E2E layer. The current browser suite can validate the preview surface and
  // the open command path, but not an actual spawned Tauri window.
});
