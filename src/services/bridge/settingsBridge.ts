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
  projectColors: Record<string, string>;
  projectGroups: Record<string, string>;
  groupProjectsEnabled: boolean;
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
    setTimesheetRoundingEnabled(enabled: boolean) {
      return invoke("set_timesheet_rounding_enabled", { enabled });
    },
    setUiFontScale(scale: number) {
      return invoke("set_ui_font_scale", { scale });
    },
    setProjectColor(project: string, color: string | null) {
      return invoke("set_project_color", { project, color });
    },
    setProjectGroup(project: string, group: string | null) {
      return invoke("set_project_group", { project, group });
    },
  };
}

export type SettingsBridge = ReturnType<typeof createSettingsBridge>;
