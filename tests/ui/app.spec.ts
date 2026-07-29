import { expect, test } from "@playwright/test";
import timesheetWeekCases from "../timesheet-current-week.json" with { type: "json" };
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

function mockInvokedCalls(page: Parameters<typeof test>[0]["page"]) {
  return page.evaluate(() => {
    return ((window as Window & {
      __TAURI_MOCK__?: { invokedCalls?: Array<{ command: string; args: Record<string, unknown> }> };
    }).__TAURI_MOCK__?.invokedCalls ?? []);
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

  test("prioritizes timesheet shortcuts and hides secondary actions in overflow", async ({ page }) => {
    const actions = page.locator(".actions");
    const primaryActions = actions.locator("button.primary");

    await expect(primaryActions).toHaveText(["Yesterday + today", "Full timesheet"]);
    await expect(actions.getByRole("button", { name: "About" })).toHaveCount(0);
    await expect(actions.getByRole("button", { name: "Reset timesheet" })).toHaveCount(0);
    await expect(actions.getByRole("button", { name: "Reset projects" })).toHaveCount(0);
  });

  test("keeps positive action text readable on hover", async ({ page }) => {
    const positiveActions = page.locator(".actions button.primary");

    await expect(positiveActions).toHaveCount(2);
    await expect(positiveActions.first()).toHaveCSS("background-color", "rgb(38, 122, 98)");
    await expect(positiveActions.first()).toHaveCSS("color", "rgb(255, 255, 255)");

    await positiveActions.first().hover();

    await expect(positiveActions.first()).toHaveCSS("background-color", "rgb(33, 107, 86)");
    await expect(positiveActions.first()).toHaveCSS("color", "rgb(255, 255, 255)");
  });

  test("supports keyboard access and destructive grouping in the More menu", async ({ page }) => {
    const more = page.getByRole("button", { name: "⋮ More" });

    await expect(more).toHaveAttribute("aria-haspopup", "menu");
    await expect(more).toHaveAttribute("aria-expanded", "false");
    await more.focus();
    await more.press("ArrowDown");

    const menu = page.getByRole("menu");
    await expect(menu).toBeVisible();
    await expect(more).toHaveAttribute("aria-expanded", "true");
    await expect(menu.getByRole("menuitem")).toHaveText([
      "About",
      "Reset timesheet",
      "Reset projects",
    ]);
    await expect(menu.getByRole("separator")).toHaveCount(1);
    await expect(menu.getByRole("menuitem", { name: "About" })).toBeFocused();

    const aboutColor = await menu.getByRole("menuitem", { name: "About" }).evaluate(
      (element) => getComputedStyle(element).color
    );
    const resetActions = menu.locator(".destructive");
    await expect(resetActions).toHaveCount(2);
    await expect(resetActions.first()).not.toHaveCSS("color", aboutColor);

    await page.keyboard.press("Escape");
    await expect(menu).toHaveCount(0);
    await expect(more).toHaveAttribute("aria-expanded", "false");
    await expect(more).toBeFocused();
  });

  for (const resetCase of [
    {
      action: "Reset timesheet",
      message: "Permanently erase all timesheet data?",
      command: "reset_timesheet",
    },
    {
      action: "Reset projects",
      message: "Permanently erase all saved projects?",
      command: "reset_projects",
    },
  ]) {
    test(`${resetCase.action} requires explicit confirmation`, async ({ page }) => {
      // EPIC-011 §1 Flows 3–4 require a surfaced decision before either reset executes.
      await page.evaluate(() => {
        ((window as Window & { __TAURI_MOCK__?: { dialogResponses?: string[] } }).__TAURI_MOCK__
          ?.dialogResponses ?? []).push("Cancel");
      });
      await page.getByRole("button", { name: "⋮ More" }).click();
      await page.getByRole("menuitem", { name: resetCase.action }).click();
      await expect.poll(async () => (await mockInvokedCommands(page)).filter(
        (command) => command === "plugin:dialog|message"
      )).toHaveLength(1);
      await expect.poll(async () => (await mockInvokedCalls(page)).find(
        ({ command }) => command === "plugin:dialog|message"
      )?.args).toMatchObject({
        message: resetCase.message,
        kind: "warning",
        buttons: { OkCancelCustom: ["Reset", "Cancel"] },
      });
      await expect.poll(async () => (await mockInvokedCommands(page)).filter(
        (command) => command === resetCase.command
      )).toHaveLength(0);

      await page.evaluate(() => {
        ((window as Window & { __TAURI_MOCK__?: { dialogResponses?: string[] } }).__TAURI_MOCK__
          ?.dialogResponses ?? []).push("Reset");
      });
      await page.getByRole("button", { name: "⋮ More" }).click();
      await page.getByRole("menuitem", { name: resetCase.action }).click();
      await expect.poll(async () => (await mockInvokedCommands(page)).filter(
        (command) => command === "plugin:dialog|message"
      )).toHaveLength(2);
      await expect.poll(async () => (await mockInvokedCommands(page)).filter(
        (command) => command === resetCase.command
      )).toHaveLength(1);
    });
  }

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

    await page.getByRole("button", { name: "⋮ More" }).click();
    await page.getByRole("menuitem", { name: "About" }).click();
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

test.describe("QuickPanel grouping", () => {
  const GROUPED_PROJECTS = ["Alpha", "Bravo", "Charlie", "Delta"];

  test.beforeEach(async ({ page }) => {
    await installTauriMocks(page, {
      projects: GROUPED_PROJECTS,
      settings: {
        project_sort_mode: "manual",
        project_manual_order: GROUPED_PROJECTS,
        group_projects_enabled: true,
        // Bravo + Delta in "Work", Charlie in "Personal", Alpha ungrouped.
        project_groups: { Bravo: "Work", Delta: "Work", Charlie: "Personal" },
      },
    });
    await page.goto("/");
    await stepPause(page);
  });

  test("grouped projects render as boxes with indented members and ungrouped rows stay flat", async ({ page }) => {
    const boxes = page.locator(".project-group-box");

    await expect(boxes).toHaveCount(2);
    await expect(boxes.nth(0).locator(".group-header span").last()).toHaveText("Work");
    await expect(boxes.nth(0).locator(".group-chevron")).toBeVisible();
    await expect(boxes.nth(0).locator(".project-row")).toHaveCount(2);
    await expect(boxes.nth(0).locator(".project-row").first()).toHaveClass(/project-row-indented/);
    await expect(boxes.nth(0).locator(".project-button span")).toHaveText(["Bravo", "Delta"]);

    await expect(boxes.nth(1).locator(".group-header span").last()).toHaveText("Personal");
    await expect(boxes.nth(1).locator(".project-row")).toHaveCount(1);
    await expect(boxes.nth(1).locator(".project-row")).toHaveClass(/project-row-indented/);
    await expect(boxes.nth(1).locator(".project-button span")).toHaveText("Charlie");

    const ungrouped = page.locator(".project-list > .project-row", { hasText: "Alpha" });
    await expect(ungrouped).toHaveCount(1);
    await expect(ungrouped).not.toHaveClass(/project-row-indented/);
    await expect(page.getByText("Ungrouped", { exact: true })).toHaveCount(0);
  });
  test("collapse toggle hides and restores group members without changing groups", async ({ page }) => {
    const workBox = page.locator(".project-group-box", { hasText: "Work" });
    const workHeader = workBox.locator(".group-header");

    await expect(workBox.locator(".project-button span")).toHaveText(["Bravo", "Delta"]);
    await expect(workHeader).toHaveAttribute("aria-expanded", "true");

    await workHeader.click();
    await expect(workHeader).toHaveAttribute("aria-expanded", "false");
    await expect(workBox.locator(".project-row")).toHaveCount(0);

    await workHeader.click();
    await expect(workHeader).toHaveAttribute("aria-expanded", "true");
    await expect(workBox.locator(".project-button span")).toHaveText(["Bravo", "Delta"]);
  });

  test("grouping checkbox toggles boxed layout and is locked in manual mode", async ({ page }) => {
    const checkbox = page.getByRole("checkbox", { name: "Group" });

    await expect(checkbox).toBeChecked();
    await expect(checkbox).toBeDisabled();
    await expect(page.locator(".project-group-box")).toHaveCount(2);

    await page.getByRole("button", { name: "A-Z" }).click();
    await expect(checkbox).toBeEnabled();
    await checkbox.uncheck();
    await expect(page.locator(".project-group-box")).toHaveCount(0);

    await checkbox.check();
    await expect(page.locator(".project-group-box")).toHaveCount(2);
  });

  test("manual drag reorders projects only within their current group", async ({ page }) => {
    async function dragProjectTo(project: string, target: string, targetRatio: number) {
      const source = page.locator(".project-row", { has: page.getByRole("button", { name: project }) });
      const targetRow = page.locator(".project-row", { has: page.getByRole("button", { name: target }) });
      const sourceBox = await source.locator(".drag-handle").boundingBox();
      const targetBox = await targetRow.boundingBox();

      expect(sourceBox).not.toBeNull();
      expect(targetBox).not.toBeNull();

      await page.mouse.move(sourceBox!.x + sourceBox!.width / 2, sourceBox!.y + sourceBox!.height / 2);
      await page.mouse.down();
      await page.mouse.move(targetBox!.x + targetBox!.width / 2, targetBox!.y + targetBox!.height * targetRatio);
      await page.mouse.up();
    }

    const labels = page.locator(".project-button span");

    await expect(labels).toHaveText(["Alpha", "Bravo", "Delta", "Charlie"]);

    await dragProjectTo("Bravo", "Delta", 0.75);
    await expect(labels).toHaveText(["Alpha", "Delta", "Bravo", "Charlie"]);

    await dragProjectTo("Bravo", "Charlie", 0.75);
    await expect(labels).toHaveText(["Alpha", "Delta", "Bravo", "Charlie"]);
  });

});

test.describe("QuickPanel grouping checkbox visibility", () => {
  test("hides the grouping checkbox when no projects have groups", async ({ page }) => {
    await installTauriMocks(page, {
      projects: ["Alpha"],
      settings: { project_sort_mode: "alphabetical", project_manual_order: ["Alpha"] },
    });
    await page.goto("/");
    await stepPause(page);

    await expect(page.getByLabel("Group")).toHaveCount(0);
  });
  test("assigning a group from the context menu forces grouped layout on", async ({ page }) => {
    await installTauriMocks(page, {
      projects: ["Alpha", "Bravo"],
      settings: {
        project_sort_mode: "alphabetical",
        project_manual_order: ["Alpha", "Bravo"],
        group_projects_enabled: false,
        project_groups: { Bravo: "Work" },
      },
    });
    await page.goto("/");
    await stepPause(page);

    await expect(page.locator(".project-group-box")).toHaveCount(0);
    await page.locator(".project-row", { hasText: "Alpha" }).click({ button: "right" });
    await page.locator(".context-menu-item", { hasText: "Work" }).click();

    await expect(page.getByRole("checkbox", { name: "Group" })).toBeChecked();
    await expect(page.locator(".project-group-box", { hasText: "Work" })).toBeVisible();
    await expect(page.locator(".project-group-box", { hasText: "Work" }).locator(".project-button span")).toHaveText(["Alpha", "Bravo"]);
  });

  test("empty groups are hidden from the list but remain pickable", async ({ page }) => {
    await installTauriMocks(page, {
      projects: ["Alpha", "Bravo"],
      settings: {
        project_sort_mode: "alphabetical",
        project_manual_order: ["Alpha", "Bravo"],
        group_projects_enabled: true,
        project_groups: { Bravo: "Work" },
      },
    });
    await page.goto("/");
    await stepPause(page);

    await expect(page.locator(".project-group-box", { hasText: "Work" })).toBeVisible();

    await page.locator(".project-row", { hasText: "Bravo" }).click({ button: "right" });
    await page.locator(".context-menu-item", { hasText: "Ungrouped" }).click();

    await expect(page.locator(".project-group-box", { hasText: "Work" })).toHaveCount(0);

    await page.locator(".project-row", { hasText: "Alpha" }).click({ button: "right" });
    await expect(page.locator(".context-menu-item", { hasText: "Work" })).toBeVisible();
  });

});

test.describe("QuickPanel project colors", () => {
  test.beforeEach(async ({ page }) => {
    await installTauriMocks(page, {
      projects: ["Alpha"],
      settings: {
        project_sort_mode: "manual",
        project_manual_order: ["Alpha"],
        project_colors: { Alpha: "#3f8fd1" },
      },
    });
    await page.goto("/");
    await stepPause(page);
  });

  test("assigned color shows as a left accent bar, not a background fill or underline", async ({ page }) => {
    const row = page.locator(".project-row", { hasText: "Alpha" });

    // The true (full-opacity) color renders as a vertical bar at the row's left edge.
    await expect(row.locator(".project-color-accent")).toHaveCSS(
      "background-color",
      "rgb(63, 143, 209)"
    );

    // The bar sits before the text: it is the row's first element.
    await expect(row.locator("> :first-child")).toHaveClass(/project-color-accent/);

    // No underline strip, and no background fill on the title box or the ×/＋ button.
    await expect(row.locator(".project-color-underline")).toHaveCount(0);
    await expect(row.locator(".project-button")).not.toHaveCSS("background-color", "rgb(63, 143, 209)");
    await expect(row.locator(".icon-button")).not.toHaveCSS("background-color", "rgb(63, 143, 209)");
  });

  test("clearing the color removes the accent bar", async ({ page }) => {
    const row = page.locator(".project-row", { hasText: "Alpha" });
    await expect(row.locator(".project-color-accent")).toHaveCount(1);

    // Clear the color through the context menu.
    await row.click({ button: "right" });
    await page.getByTitle("Clear color").click();

    await expect(row.locator(".project-color-accent")).toHaveCount(0);
  });
});

test.describe("QuickPanel context menu", () => {
  const VIEWPORT = { width: 300, height: 320 };

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize(VIEWPORT);
    await installTauriMocks(page, {
      projects: ["Alpha"],
      settings: { project_sort_mode: "manual", project_manual_order: ["Alpha"] },
    });
    await page.goto("/");
    await stepPause(page);
  });

  test("context menu stays fully inside the window when opened near an edge", async ({ page }) => {
    // Right-clicking a row in a window narrower than the menu would clip it without clamping.
    await page.locator(".project-row", { hasText: "Alpha" }).click({ button: "right" });

    const menu = page.locator(".project-context-menu");
    await expect(menu).toBeVisible();

    const box = await menu.boundingBox();
    if (!box) throw new Error("context menu has no bounding box");

    // Flow C: the whole menu is within the viewport on both axes (1px slack for rounding).
    expect(box.x).toBeGreaterThanOrEqual(0);
    expect(box.y).toBeGreaterThanOrEqual(0);
    expect(box.x + box.width).toBeLessThanOrEqual(VIEWPORT.width + 1);
    expect(box.y + box.height).toBeLessThanOrEqual(VIEWPORT.height + 1);
  });
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
    await expect(page.getByText(/Generated at 2026-04-30 (05|07):09,/)).toBeVisible();
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

test("Tauri mocks bootstrap a configured multi-sheet preview", async ({ page }) => {
  const sheet = (name: string) => ({
    name,
    columns: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
    rows: [{
      label: name,
      values: [1, 0, 0, 0, 0, 0, 0],
      total: 1,
      is_comment: false,
      is_total: false,
    }],
  });

  await installTauriMocks(page, {}, {
    currentWindowLabel: "timesheet-preview",
    initialPreviewRequest: { range: "all", format: "full" },
    previewResponse: {
      title: "Configured preview",
      generated_at: "2026-07-21 12:00",
      generated_at_epoch_ms: Date.UTC(2026, 6, 21, 12),
      sheets: [sheet("Earlier"), sheet("Current")],
    },
  });

  await page.goto("/?window=timesheet-preview");

  await expect(page.getByRole("heading", { name: "Configured preview" })).toBeVisible();
  await expect(page.locator(".sheet-tabs button")).toHaveText(["Earlier", "Current"]);
});

test.describe("Full timesheet week selection", () => {
  function isoWeekName(weekOffset: number) {
    const date = new Date();
    date.setDate(date.getDate() + weekOffset * 7);
    const utc = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
    utc.setUTCDate(utc.getUTCDate() + 4 - (utc.getUTCDay() || 7));
    const yearStart = new Date(Date.UTC(utc.getUTCFullYear(), 0, 1));
    const week = Math.ceil(((utc.getTime() - yearStart.getTime()) / 86400000 + 1) / 7);
    return `${utc.getUTCFullYear()}-${week}`;
  }

  function preview(offsets: number[]) {
    return {
      title: "Full timesheet",
      generated_at: "2026-07-21 12:00",
      generated_at_epoch_ms: Date.now(),
      sheets: offsets.map((offset) => ({
        name: isoWeekName(offset),
        columns: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
        rows: [{
          label: `Week ${offset}`,
          values: [1, 0, 0, 0, 0, 0, 0],
          total: 1,
          is_comment: false,
          is_total: false,
        }],
      })),
    };
  }

  async function openPreview(page: Parameters<typeof test>[0]["page"], offsets: number[]) {
    await installTauriMocks(page, {}, {
      currentWindowLabel: "timesheet-preview",
      initialPreviewRequest: { range: "all", format: "full" },
      previewResponse: preview(offsets),
    });
    await page.goto("/?window=timesheet-preview");
  }

  test("opens on the current week", async ({ page }) => {
    await openPreview(page, timesheetWeekCases.with_current_week);
    await expect(page.locator(".sheet-tabs .sort-active")).toHaveText(isoWeekName(0));
  });

  test("falls back to the newest available week", async ({ page }) => {
    await openPreview(page, timesheetWeekCases.without_current_week);
    await expect(page.locator(".sheet-tabs .sort-active")).toHaveText(isoWeekName(-1));
  });

  test("preserves a selected historical week on refresh", async ({ page }) => {
    await openPreview(page, timesheetWeekCases.with_current_week);
    await page.getByRole("button", { name: isoWeekName(-2) }).click();
    await page.getByRole("button", { name: "Update now" }).click();
    await expect(page.locator(".sheet-tabs .sort-active")).toHaveText(isoWeekName(-2));
  });
});
