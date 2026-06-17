import { invoke } from "@tauri-apps/api/core";

import type { QuickPanelMode, SortMode } from "../../models/types";

export type SaveUiSettingsInput = {
  alwaysOnTop: boolean;
  openOnStart: boolean;
  quickpanelOpacity: number;
  projectSortMode: SortMode;
  quickpanelMode: QuickPanelMode;
  projectManualOrder: string[];
  projectRecentUsage: Record<string, number>;
  timesheetRoundingEnabled: boolean;
};

export type SaveQuickpanelBoundsInput = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export function createSettingsBridge() {
  return {
    saveUiSettings(input: SaveUiSettingsInput) {
      return invoke("save_ui_settings", input);
    },
    saveQuickpanelBounds(input: SaveQuickpanelBoundsInput) {
      return invoke("save_quickpanel_bounds", input);
    },
  };
}

export type SettingsBridge = ReturnType<typeof createSettingsBridge>;
