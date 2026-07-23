import { expect, test } from "@playwright/test";
import { createQuickPanelDialogActions } from "../../src/controllers/quickpanel/createQuickPanelDialogActions";
import { NEW_GROUP_DIALOG_PREFIX } from "../../src/controllers/projects/createProjectContextMenuController";
import type { QuickPanelState } from "../../src/controllers/quickpanel/quickPanelTypes";
import type { QuickPanelBridge } from "../../src/services/bridge/quickPanelBridge";

// Authority: description/13-project-list-color-grouping.md — assigning a project
// to a new group via the dialog must enable grouping and persist the group.
// Regression: submitDialog referenced an undefined `enableGrouping`, so pressing
// Enter or OK on the new-group dialog threw before the group was ever saved.
test.describe("createQuickPanelDialogActions — new group dialog (description/13)", () => {
  test("submitDialog assigns the project to the new group and closes", async () => {
    const state = {
      dialogOpen: true,
      dialogMode: `${NEW_GROUP_DIALOG_PREFIX}Mango`,
      dialogValue: "Work",
    } as QuickPanelState;

    const groupCalls: Array<[string, string | null]> = [];
    let forced = false;

    const actions = createQuickPanelDialogActions({
      state,
      quickPanelBridge: {
        setProjectGroup: (project: string, group: string | null) => {
          groupCalls.push([project, group]);
        },
      } as unknown as QuickPanelBridge,
      loadState: async () => {},
      forceGroupProjectsEnabled: () => {
        forced = true;
      },
    });

    await actions.submitDialog();

    expect(groupCalls).toEqual([["Mango", "Work"]]);
    expect(forced).toBe(true);
    expect(state.dialogOpen).toBe(false);
  });
});
