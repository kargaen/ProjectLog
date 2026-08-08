import { expect, test } from "@playwright/test";

import { createQuickPanelShellActions } from "../../src/controllers/quickpanel/createQuickPanelShellActions";

test("restores the saved outer window dimensions without decoration drift", async () => {
  // Authority: architecture/constitution/09-testing-philosophy.md —
  // "Window bounds are restored correctly on reopen."
  const appliedSizes: Array<{ width: number; height: number }> = [];
  const currentWindow = {
    scaleFactor: async () => 1,
    innerSize: async () => ({ width: 780, height: 560 }),
    outerSize: async () => ({ width: 800, height: 600 }),
    setSize: async (size: { width: number; height: number }) => {
      appliedSizes.push(size);
    },
    setPosition: async () => {},
  };

  const actions = createQuickPanelShellActions({
    state: {
      appState: {
        settings: {
          quickpanel_width: 800,
          quickpanel_height: 600,
          quickpanel_x: 100,
          quickpanel_y: 100,
        },
      },
    } as never,
    currentWindow: currentWindow as never,
    quickPanelBridge: {} as never,
    minWindowWidth: 320,
    minWindowHeight: 240,
    setCurrentWindowHeight: () => {},
  });

  await actions.restoreQuickPanelBounds();

  expect(appliedSizes).toHaveLength(1);
  expect(appliedSizes[0]).toMatchObject({ width: 780, height: 560 });
});
