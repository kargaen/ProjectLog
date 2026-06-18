import type { Window } from "@tauri-apps/api/window";

import { createLogger } from "../../lib/logger";
import type { QuickPanelMode, SortMode } from "../../models/types";
import type { QuickPanelState } from "../quickpanel/quickPanelTypes";

const log = createLogger("quickpanel.settings");

type CreateSettingsActionsControllerArgs = {
  state: QuickPanelState;
  currentWindow: Window;
  persistUiSettings: () => Promise<void>;
  applyQuickpanelModeLayout: () => Promise<void>;
  queueSettingsSave: () => void;
};

export function createSettingsActionsController(
  args: CreateSettingsActionsControllerArgs
) {
  const {
    state,
    currentWindow,
    persistUiSettings,
    applyQuickpanelModeLayout,
    queueSettingsSave,
  } = args;

  async function toggleAlwaysOnTop() {
    state.alwaysOnTop = !state.alwaysOnTop;
    await currentWindow.setAlwaysOnTop(state.alwaysOnTop).catch(() => {});
    await currentWindow.setSkipTaskbar(state.alwaysOnTop).catch(() => {});
    await persistUiSettings();
    log.info("toggleAlwaysOnTop", {
      alwaysOnTop: state.alwaysOnTop,
    });
  }

  async function toggleOpenOnStart() {
    state.openOnStart = !state.openOnStart;
    await persistUiSettings();
    log.info("toggleOpenOnStart", {
      openOnStart: state.openOnStart,
    });
  }

  async function setQuickPanelMode(nextMode: QuickPanelMode) {
    if (state.quickPanelMode === nextMode) {
      return;
    }

    state.quickPanelMode = nextMode;
    await persistUiSettings();
    await applyQuickpanelModeLayout();
    log.info("setQuickPanelMode", {
      quickPanelMode: nextMode,
    });
  }

  async function toggleQuickPanelMode() {
    await setQuickPanelMode(
      state.quickPanelMode === "compact" ? "normal" : "compact"
    );
  }

  function setSortMode(nextMode: SortMode) {
    state.sortMode = nextMode;
    queueSettingsSave();
  }

  function setQuickPanelOpacity(nextOpacity: number) {
    state.quickPanelOpacity = nextOpacity;
    queueSettingsSave();
  }

  return {
    toggleAlwaysOnTop,
    toggleOpenOnStart,
    setQuickPanelMode,
    toggleQuickPanelMode,
    setSortMode,
    setQuickPanelOpacity,
  };
}
