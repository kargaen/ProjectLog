import { invoke } from "@tauri-apps/api/core";

import type {
  ProjectState,
  QuickPanelMode,
  SortMode,
} from "../../models/types";

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

export function createQuickPanelBridge() {
  return {
    getState() {
      return invoke<ProjectState>("get_state");
    },
    saveUiSettings(input: SaveUiSettingsInput) {
      return invoke("save_ui_settings", input);
    },
    selectProject(project: string) {
      return invoke("select_project", { project });
    },
    addProject(value: string) {
      return invoke("add_project", { value });
    },
    quickProject(value: string) {
      return invoke("quick_project", { value });
    },
    setComment(value: string) {
      return invoke("set_comment", { value });
    },
    removeProject(project: string) {
      return invoke("remove_project", { project });
    },
    saveQuickpanelBounds(input: SaveQuickpanelBoundsInput) {
      return invoke("save_quickpanel_bounds", input);
    },
    setUpdateAvailable(available: boolean) {
      return invoke("set_update_available", { available });
    },
    submitInput(mode: string, value: string) {
      return invoke("submit_input", { mode, value });
    },
    resetProjects() {
      return invoke("reset_projects");
    },
    openLogFile() {
      return invoke("open_log_file");
    },
    openDiagnosticLog() {
      return invoke("open_diagnostic_log");
    },
    openFeedback() {
      return invoke("open_feedback");
    },
    openProjectHomepage() {
      return invoke("open_project_homepage");
    },
    openPortfolio() {
      return invoke("open_portfolio");
    },
    openReleaseNotes() {
      return invoke("open_release_notes");
    },
  };
}

export type QuickPanelBridge = ReturnType<typeof createQuickPanelBridge>;
